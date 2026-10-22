/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! An in-memory RFC 6265bis cookie jar.
//!
//! Parses `Set-Cookie` via the `cookie` crate, then applies the standard
//! client-side storage + retrieval rules: domain-match (host-only vs `Domain`),
//! path-match (with the request's default-path), `Secure` (https-only), and
//! expiry (`Max-Age` over `Expires`). Cookies replace any prior entry with the
//! same (name, domain, path); `cookies_for` returns matches sorted longest-path
//! first, then oldest-first (the spec's serialization order).
//!
//! The default jar enforces public-suffix, `__Secure-`, and `__Host-` storage
//! restrictions. Fetch supplies the request's SameSite context at retrieval.

use std::net::IpAddr;
use std::sync::Mutex;

use cookie::{Cookie, SameSite};
use time::OffsetDateTime;
use url::{Host, Url};

use crate::context::{CookieRecord, CookieStore, SameSiteContext};

#[derive(Clone)]
struct StoredCookie {
    name: String,
    value: String,
    /// Lowercased; the host for a host-only cookie, else the `Domain` value.
    domain: String,
    host_only: bool,
    path: String,
    secure: bool,
    http_only: bool,
    same_site: Option<SameSite>,
    /// `None` = session cookie (kept for the jar's lifetime).
    expiry: Option<OffsetDateTime>,
    created: OffsetDateTime,
}

/// In-memory cookie jar. Cheap to share; all access is behind a `Mutex`.
#[derive(Default)]
pub struct InMemoryCookieJar {
    cookies: Mutex<Vec<StoredCookie>>,
}

impl InMemoryCookieJar {
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of live (unexpired) stored cookies — inspection aid for tests.
    pub fn len(&self) -> usize {
        let now = OffsetDateTime::now_utc();
        self.cookies
            .lock()
            .map(|c| c.iter().filter(|c| !expired(c, now)).count())
            .unwrap_or(0)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Every live (unexpired) stored cookie, as structured records — for persisting
    /// the whole jar to a durable backing (the native session store). Unlike
    /// [`records_for`](CookieStore::records_for) this is not URL-scoped; it is the
    /// full jar snapshot.
    pub fn all_records(&self) -> Vec<CookieRecord> {
        let now = OffsetDateTime::now_utc();
        let Ok(jar) = self.cookies.lock() else {
            return Vec::new();
        };
        jar.iter()
            .filter(|c| !expired(c, now))
            .map(|c| CookieRecord {
                name: c.name.clone(),
                value: c.value.clone(),
                domain: c.domain.clone(),
                host_only: c.host_only,
                path: c.path.clone(),
                secure: c.secure,
                http_only: c.http_only,
                same_site: c.same_site,
                expires: c.expiry.map(|e| e.unix_timestamp() as f64),
            })
            .collect()
    }

    /// Restore records into the jar (from a durable backing at startup). Each record
    /// replaces any existing cookie with the same (name, domain, path) identity, the
    /// same rule as a fresh `Set-Cookie`; already-expired records are dropped. The
    /// `created` time is reset to now (it only breaks serialization-order ties).
    pub fn load_records(&self, records: impl IntoIterator<Item = CookieRecord>) {
        let now = OffsetDateTime::now_utc();
        let Ok(mut jar) = self.cookies.lock() else {
            return;
        };
        for r in records {
            let expiry = r
                .expires
                .and_then(|s| OffsetDateTime::from_unix_timestamp(s as i64).ok());
            if expiry.is_some_and(|e| e <= now) {
                continue;
            }
            jar.retain(|c| !(c.name == r.name && c.domain == r.domain && c.path == r.path));
            jar.push(StoredCookie {
                name: r.name,
                value: r.value,
                domain: r.domain,
                host_only: r.host_only,
                path: r.path,
                secure: r.secure,
                http_only: r.http_only,
                same_site: r.same_site,
                expiry,
                created: now,
            });
        }
    }

    /// The stored cookies that match a request to `url` under `ctx` (domain / path /
    /// `Secure` / `SameSite` gating), sorted longest-path then oldest-first (the
    /// spec's serialization order). Expired cookies are pruned as a side effect.
    /// Shared by [`cookies_for`](CookieStore::cookies_for) (which renders `name=value`)
    /// and [`records_for`](CookieStore::records_for) (which keeps the attributes).
    fn matching(&self, url: &Url, ctx: SameSiteContext) -> Vec<StoredCookie> {
        let host = url.host_str().unwrap_or_default().to_ascii_lowercase();
        let req_path = url.path();
        let secure_ctx = url.scheme() == "https";
        let now = OffsetDateTime::now_utc();

        let Ok(mut jar) = self.cookies.lock() else {
            return Vec::new();
        };
        jar.retain(|c| !expired(c, now));

        let mut hits: Vec<StoredCookie> = jar
            .iter()
            .filter(|c| {
                let domain_ok = if c.host_only {
                    host == c.domain
                } else {
                    !is_public_suffix(&c.domain) && domain_matches(&host, &c.domain)
                };
                // SameSite gating. Unset defaults to Lax (modern browser default).
                let samesite_ok = match c.same_site.unwrap_or(SameSite::Lax) {
                    SameSite::Strict => ctx.same_site,
                    SameSite::Lax => ctx.same_site || ctx.top_level_navigation,
                    SameSite::None => true,
                };
                domain_ok
                    && valid_stored_prefix(c)
                    && path_matches(req_path, &c.path)
                    && (!c.secure || secure_ctx)
                    && samesite_ok
            })
            .cloned()
            .collect();

        // Longer paths first; ties broken by creation order (oldest first).
        hits.sort_by(|a, b| {
            b.path
                .len()
                .cmp(&a.path.len())
                .then(a.created.cmp(&b.created))
        });
        hits
    }
}

impl CookieStore for InMemoryCookieJar {
    fn cookies_for(&self, url: &Url, ctx: SameSiteContext) -> Vec<String> {
        self.matching(url, ctx)
            .iter()
            .map(|c| format!("{}={}", c.name, c.value))
            .collect()
    }

    fn records_for(&self, url: &Url, ctx: SameSiteContext) -> Vec<CookieRecord> {
        self.matching(url, ctx)
            .into_iter()
            .map(|c| CookieRecord {
                name: c.name,
                value: c.value,
                domain: c.domain,
                host_only: c.host_only,
                path: c.path,
                secure: c.secure,
                http_only: c.http_only,
                same_site: c.same_site,
                expires: c.expiry.map(|e| e.unix_timestamp() as f64),
            })
            .collect()
    }

    fn set_cookie(&self, url: &Url, set_cookie_header: &str) {
        let Ok(parsed) = Cookie::parse(set_cookie_header.to_owned()) else {
            return;
        };
        let Some(host) = url.host_str() else {
            return;
        };
        let host = host.to_ascii_lowercase();

        let secure = parsed.secure().unwrap_or(false);
        if secure && url.scheme() != "https" {
            return;
        }

        let (domain, host_only) = match parsed.domain() {
            Some(d) => {
                // Cookie Domain is lowercased literal text, not a URL host to
                // percent-decode or IDNA-convert. `cookie` removes one leading
                // dot; reject any further host-parser normalization.
                if !d.is_ascii() {
                    return;
                }
                let d = d.to_ascii_lowercase();
                let Ok(canonical) = Host::parse(&d) else {
                    return;
                };
                if canonical.to_string() != d {
                    return;
                }
                // Reject a Domain the request host doesn't domain-match.
                if !domain_matches(&host, &d) {
                    return;
                }
                if is_public_suffix(&d) {
                    if d != host {
                        return;
                    }
                    (host.clone(), true)
                } else {
                    (d, false)
                }
            },
            None => (host, true),
        };
        let path = parsed
            .path()
            .map(str::to_owned)
            .unwrap_or_else(|| default_path(url));
        let name = parsed.name();
        if has_cookie_prefix(name, "__Secure-") && !secure {
            return;
        }
        if has_cookie_prefix(name, "__Host-")
            && (!secure || !host_only || parsed.domain().is_some() || parsed.path() != Some("/"))
        {
            return;
        }
        if name.is_empty()
            && (has_cookie_prefix(parsed.value(), "__Secure-")
                || has_cookie_prefix(parsed.value(), "__Host-"))
        {
            return;
        }
        let now = OffsetDateTime::now_utc();
        let expiry = compute_expiry(&parsed, now);

        let stored = StoredCookie {
            name: parsed.name().to_owned(),
            value: parsed.value().to_owned(),
            domain,
            host_only,
            path,
            secure,
            http_only: parsed.http_only().unwrap_or(false),
            same_site: parsed.same_site(),
            expiry,
            created: now,
        };

        let Ok(mut jar) = self.cookies.lock() else {
            return;
        };
        // A new cookie replaces any prior one with the same identity tuple.
        jar.retain(|c| {
            !(c.name == stored.name && c.domain == stored.domain && c.path == stored.path)
        });
        // An already-expired Set-Cookie is a deletion (replace step above) — don't re-add.
        if expiry.is_some_and(|e| e <= now) {
            return;
        }
        jar.push(stored);
    }
}

fn expired(c: &StoredCookie, now: OffsetDateTime) -> bool {
    c.expiry.is_some_and(|e| e <= now)
}

fn is_public_suffix(domain: &str) -> bool {
    psl::suffix_str(domain)
        .is_some_and(|suffix| domain.trim_end_matches('.').eq_ignore_ascii_case(suffix))
}

fn has_cookie_prefix(name: &str, prefix: &str) -> bool {
    name.get(..prefix.len())
        .is_some_and(|start| start.eq_ignore_ascii_case(prefix))
}

fn valid_stored_prefix(c: &StoredCookie) -> bool {
    if c.name.is_empty()
        && (has_cookie_prefix(&c.value, "__Secure-") || has_cookie_prefix(&c.value, "__Host-"))
    {
        return false;
    }
    if has_cookie_prefix(&c.name, "__Secure-") && !c.secure {
        return false;
    }
    if has_cookie_prefix(&c.name, "__Host-") && (!c.secure || !c.host_only || c.path != "/") {
        return false;
    }
    true
}

/// Max-Age takes precedence over Expires (RFC 6265bis §5.3).
fn compute_expiry(c: &Cookie, now: OffsetDateTime) -> Option<OffsetDateTime> {
    if let Some(max_age) = c.max_age() {
        return Some(now.saturating_add(max_age));
    }
    c.expires_datetime()
}

/// RFC 6265bis domain-match: equal, or `host` is a sub-domain of `domain` and
/// `host` is not an IP literal.
fn domain_matches(host: &str, domain: &str) -> bool {
    if host == domain {
        return true;
    }
    if host.len() <= domain.len() || !host.ends_with(domain) {
        return false;
    }
    let boundary = host.len() - domain.len() - 1;
    host.as_bytes()[boundary] == b'.' && host.parse::<IpAddr>().is_err()
}

/// RFC 6265bis path-match.
fn path_matches(req_path: &str, cookie_path: &str) -> bool {
    if req_path == cookie_path {
        return true;
    }
    if !req_path.starts_with(cookie_path) {
        return false;
    }
    cookie_path.ends_with('/') || req_path.as_bytes().get(cookie_path.len()) == Some(&b'/')
}

/// RFC 6265bis default-path: the request path up to (not including) the last `/`,
/// or `/` if there's no non-leading slash.
fn default_path(url: &Url) -> String {
    let path = url.path();
    if !path.starts_with('/') {
        return "/".to_owned();
    }
    match path.rfind('/') {
        Some(0) | None => "/".to_owned(),
        Some(i) => path[..i].to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn url(s: &str) -> Url {
        s.parse().unwrap()
    }

    /// Same-site context — the default for these matching tests.
    fn ss() -> SameSiteContext {
        SameSiteContext::same_site()
    }

    #[test]
    fn stores_and_returns_a_host_only_cookie() {
        let jar = InMemoryCookieJar::new();
        jar.set_cookie(&url("https://example.org/"), "id=abc");
        assert_eq!(
            jar.cookies_for(&url("https://example.org/"), ss()),
            vec!["id=abc"]
        );
        // Host-only must not leak to a sub-domain.
        assert!(
            jar.cookies_for(&url("https://sub.example.org/"), ss())
                .is_empty()
        );
    }

    #[test]
    fn domain_cookie_reaches_subdomains() {
        let jar = InMemoryCookieJar::new();
        jar.set_cookie(&url("https://example.org/"), "id=abc; Domain=example.org");
        assert_eq!(
            jar.cookies_for(&url("https://api.example.org/"), ss()),
            vec!["id=abc"]
        );
    }

    #[test]
    fn public_suffix_domain_cannot_reach_other_registrants() {
        let jar = InMemoryCookieJar::new();
        let alice = url("https://alice.github.io/");
        jar.set_cookie(&alice, "shared=1; Domain=github.io");
        jar.set_cookie(
            &url("https://alice.github.io./"),
            "dotted=1; Domain=github.io.",
        );
        jar.set_cookie(&url("https://a.example.co.uk/"), "uk=1; Domain=co.uk");
        jar.set_cookie(&url("https://example.org/"), "tld=1; Domain=org");
        assert!(jar.is_empty());

        // A public-suffix host can still set a cookie for itself, but only
        // with host-only scope, even when it supplied a Domain attribute.
        jar.set_cookie(&url("https://github.io/"), "own=1; Domain=github.io");
        assert_eq!(
            jar.cookies_for(&url("https://github.io/"), ss()),
            vec!["own=1"]
        );
        assert!(jar.cookies_for(&alice, ss()).is_empty());
        assert!(jar.all_records()[0].host_only);

        jar.set_cookie(&alice, "local=1; Domain=alice.github.io");
        assert_eq!(
            jar.cookies_for(&url("https://sub.alice.github.io/"), ss()),
            vec!["local=1"]
        );
    }

    #[test]
    fn canonical_domain_and_loaded_public_suffix_records() {
        let jar = InMemoryCookieJar::new();
        let source = url("https://%65xample.org/");
        jar.set_cookie(&source, "id=1; Domain=EXAMPLE.ORG");
        assert_eq!(
            jar.cookies_for(&url("https://sub.example.org/"), ss()),
            vec!["id=1"]
        );
        jar.set_cookie(&source, "dot=1; Domain=.example.org");
        assert_eq!(
            jar.cookies_for(&url("https://sub.example.org/"), ss())
                .len(),
            2
        );

        let mut old = jar
            .all_records()
            .into_iter()
            .find(|c| c.name == "id")
            .unwrap();
        old.domain = "github.io".into();
        old.host_only = false;
        let restored = InMemoryCookieJar::new();
        restored.load_records([old]);
        assert_eq!(
            restored.all_records().len(),
            1,
            "keep raw persisted records"
        );
        assert!(
            restored
                .cookies_for(&url("https://alice.github.io/"), ss())
                .is_empty()
        );
    }

    #[test]
    fn domain_attribute_is_not_url_decoded_or_idna_converted() {
        let jar = InMemoryCookieJar::new();
        let source = url("https://sub.example.org/");
        jar.set_cookie(&source, "dot=1; Domain=example%2eorg");
        jar.set_cookie(&source, "letter=1; Domain=%65xample.org");
        jar.set_cookie(&source, "invalid=1; Domain=exa/mple.org");
        jar.set_cookie(
            &url("https://sub.xn--bcher-kva.example/"),
            "unicode=1; Domain=bücher.example",
        );
        assert!(jar.is_empty());
    }

    #[test]
    fn explicit_ip_domain_remains_limited_to_that_host() {
        let jar = InMemoryCookieJar::new();
        jar.set_cookie(&url("https://127.0.0.1/"), "v4=1; Domain=127.0.0.1");
        jar.set_cookie(&url("https://[::1]/"), "v6=1; Domain=[::1]");
        assert_eq!(
            jar.cookies_for(&url("https://127.0.0.1/"), ss()),
            vec!["v4=1"]
        );
        assert_eq!(jar.cookies_for(&url("https://[::1]/"), ss()), vec!["v6=1"]);
        assert!(jar.cookies_for(&url("https://127.0.0.2/"), ss()).is_empty());
    }

    #[test]
    fn secure_and_host_prefixes_restrict_admission() {
        let jar = InMemoryCookieJar::new();
        let https = url("https://example.org/sub/page");
        let http = url("http://example.org/sub/page");
        jar.set_cookie(&https, "__Secure-missing=1");
        jar.set_cookie(&http, "__Secure-insecure=1; Secure");
        jar.set_cookie(&https, "__secure-valid=1; Secure");

        jar.set_cookie(&https, "__Host-missing-secure=1; Path=/");
        jar.set_cookie(&https, "__Host-default-path=1; Secure");
        jar.set_cookie(&https, "__Host-wrong-path=1; Secure; Path=/sub");
        jar.set_cookie(
            &https,
            "__Host-domain=1; Secure; Path=/; Domain=example.org",
        );
        jar.set_cookie(&https, "__HOST-valid=1; Secure; Path=/");
        jar.set_cookie(&https, "=__Host-spoof; Secure; Path=/");
        jar.set_cookie(&https, "=__secure-spoof; Secure; Path=/");

        let cookies = jar.cookies_for(&https, ss());
        assert_eq!(cookies.len(), 2);
        assert!(cookies.contains(&"__secure-valid=1".to_owned()));
        assert!(cookies.contains(&"__HOST-valid=1".to_owned()));
        assert!(jar.cookies_for(&http, ss()).is_empty());
        assert!(
            jar.all_records()
                .iter()
                .find(|c| c.name == "__HOST-valid")
                .unwrap()
                .host_only
        );
    }

    #[test]
    fn loaded_prefix_records_keep_raw_data_but_only_valid_ones_are_sent() {
        let source = InMemoryCookieJar::new();
        let u = url("https://example.org/sub/page");
        source.set_cookie(&u, "__Host-valid=1; Secure; Path=/");
        source.set_cookie(&u, "__Secure-valid=1; Secure; Path=/");
        let valid = source.all_records();
        let mut bad_secure = valid
            .iter()
            .find(|c| c.name == "__Secure-valid")
            .unwrap()
            .clone();
        bad_secure.name = "__Secure-bad".into();
        bad_secure.secure = false;
        let mut bad_domain = valid
            .iter()
            .find(|c| c.name == "__Host-valid")
            .unwrap()
            .clone();
        bad_domain.name = "__Host-domain".into();
        bad_domain.host_only = false;
        let mut bad_path = bad_domain.clone();
        bad_path.name = "__Host-path".into();
        bad_path.host_only = true;
        bad_path.path = "/sub".into();

        let restored = InMemoryCookieJar::new();
        restored.load_records(valid.into_iter().chain([bad_secure, bad_domain, bad_path]));
        assert_eq!(restored.all_records().len(), 5);
        let sent = restored.cookies_for(&u, ss());
        assert_eq!(sent.len(), 2);
        assert!(sent.contains(&"__Host-valid=1".to_owned()));
        assert!(sent.contains(&"__Secure-valid=1".to_owned()));
    }

    #[test]
    fn secure_cookie_not_sent_over_http() {
        let jar = InMemoryCookieJar::new();
        jar.set_cookie(&url("https://example.org/"), "id=abc; Secure");
        assert!(
            jar.cookies_for(&url("http://example.org/"), ss())
                .is_empty()
        );
        assert_eq!(
            jar.cookies_for(&url("https://example.org/"), ss()),
            vec!["id=abc"]
        );
    }

    #[test]
    fn path_scopes_the_cookie() {
        let jar = InMemoryCookieJar::new();
        jar.set_cookie(&url("https://example.org/app/"), "id=abc; Path=/app");
        assert_eq!(
            jar.cookies_for(&url("https://example.org/app/x"), ss()),
            vec!["id=abc"]
        );
        assert!(
            jar.cookies_for(&url("https://example.org/other"), ss())
                .is_empty()
        );
    }

    #[test]
    fn max_age_zero_deletes() {
        let jar = InMemoryCookieJar::new();
        jar.set_cookie(&url("https://example.org/"), "id=abc");
        assert_eq!(jar.len(), 1);
        jar.set_cookie(&url("https://example.org/"), "id=abc; Max-Age=0");
        assert!(
            jar.cookies_for(&url("https://example.org/"), ss())
                .is_empty()
        );
        assert_eq!(jar.len(), 0);
    }

    #[test]
    fn longer_paths_sort_first() {
        let jar = InMemoryCookieJar::new();
        jar.set_cookie(&url("https://example.org/"), "a=1; Path=/");
        jar.set_cookie(&url("https://example.org/app/"), "b=2; Path=/app");
        assert_eq!(
            jar.cookies_for(&url("https://example.org/app/page"), ss()),
            vec!["b=2", "a=1"],
        );
    }

    #[test]
    fn all_records_then_load_records_round_trips() {
        let jar = InMemoryCookieJar::new();
        jar.set_cookie(
            &url("https://example.org/app/"),
            "sid=abc; Path=/app; Secure; HttpOnly",
        );
        jar.set_cookie(
            &url("https://example.org/"),
            "pref=dark; Domain=example.org",
        );
        let saved = jar.all_records();
        assert_eq!(saved.len(), 2);

        // A fresh jar restored from the snapshot serves the same cookies.
        let restored = InMemoryCookieJar::new();
        restored.load_records(saved);
        assert_eq!(
            restored.cookies_for(&url("https://example.org/app/x"), ss()),
            vec!["sid=abc", "pref=dark"], // longer path first
        );
        let r = &restored
            .records_for(&url("https://example.org/app/x"), ss())
            .into_iter()
            .find(|r| r.name == "sid")
            .unwrap();
        assert!(r.secure && r.http_only); // attributes survived the round trip
    }

    #[test]
    fn load_records_drops_expired() {
        let jar = InMemoryCookieJar::new();
        let expired = CookieRecord {
            name: "old".into(),
            value: "1".into(),
            domain: "example.org".into(),
            host_only: true,
            path: "/".into(),
            secure: false,
            http_only: false,
            same_site: None,
            expires: Some(1.0), // 1970, long past
        };
        jar.load_records([expired]);
        assert!(jar.is_empty());
    }

    #[test]
    fn records_for_preserves_attributes() {
        let jar = InMemoryCookieJar::new();
        jar.set_cookie(
            &url("https://example.org/app/"),
            "sid=abc; Path=/app; Secure; HttpOnly; SameSite=Strict",
        );
        let records = jar.records_for(&url("https://example.org/app/x"), ss());
        assert_eq!(records.len(), 1);
        let r = &records[0];
        assert_eq!((r.name.as_str(), r.value.as_str()), ("sid", "abc"));
        assert_eq!(r.path, "/app");
        assert!(r.secure && r.http_only && r.host_only);
        assert_eq!(r.same_site, Some(SameSite::Strict));
        assert_eq!(r.domain, "example.org");
    }

    #[test]
    fn samesite_gates_cross_site_requests() {
        let cross = SameSiteContext {
            same_site: false,
            top_level_navigation: false,
        };
        let cross_nav = SameSiteContext {
            same_site: false,
            top_level_navigation: true,
        };

        let jar = InMemoryCookieJar::new();
        jar.set_cookie(&url("https://example.org/"), "strict=1; SameSite=Strict");
        jar.set_cookie(&url("https://example.org/"), "lax=1; SameSite=Lax");
        jar.set_cookie(
            &url("https://example.org/"),
            "none=1; SameSite=None; Secure",
        );

        let u = url("https://example.org/");
        // Same-site: all three.
        assert_eq!(jar.cookies_for(&u, ss()).len(), 3);
        // Cross-site, not a navigation: only SameSite=None.
        assert_eq!(jar.cookies_for(&u, cross), vec!["none=1"]);
        // Cross-site top-level navigation: Lax and None (not Strict).
        let nav = jar.cookies_for(&u, cross_nav);
        assert!(nav.contains(&"lax=1".to_string()) && nav.contains(&"none=1".to_string()));
        assert!(!nav.contains(&"strict=1".to_string()));
    }
}
