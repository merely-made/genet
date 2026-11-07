//! Test-only JSONL host for the headed browser interoperability runner.
//! Uses the real per-document runtime/factory, with no signaling network service.

use murm_rtc::resolve::{DnsConfig, DnsLimits, DnsResolver, MdnsConfig};
use std::io::{self, BufRead, Read, Write};
use std::net::{IpAddr, SocketAddr};
use std::rc::Rc;
use std::sync::Arc;
use std::sync::mpsc;
use std::time::Duration;

use genet_webrtc::{LocalAddressPolicy, NativeWebRtcFactory, Settings};
use script_engine_api::ScriptEngine;
use script_runtime_api::Runtime;
use serde_json::{Value, json};

#[path = "interop_host/trace.rs"]
mod trace;

struct Page;
impl script_runtime_api::ScriptResourceLoader for Page {
    fn load(&self, url: &str) -> Option<String> {
        (url == "https://rtc-interop.invalid/two.html")
            .then(|| "<html><body>RTC navigation target</body></html>".into())
    }
}

fn runtime<E: ScriptEngine>(
    bind: IpAddr,
    dns_server: Option<SocketAddr>,
) -> Result<(Runtime<E>, Arc<DnsResolver>), String> {
    let mut rt = Runtime::<E>::new().map_err(|_| "runtime initialization failed")?;
    let limits = DnsLimits {
        timeout: Duration::from_secs(5),
        ..DnsLimits::default()
    };
    let config = dns_server.map_or_else(
        || DnsConfig::system(limits),
        |server| DnsConfig {
            servers: vec![server],
            limits,
        },
    );
    let mut resolver = DnsResolver::new(config).map_err(|_| "DNS configuration failed")?;
    if let IpAddr::V4(ip) = bind {
        let mut mdns = MdnsConfig::for_interface(ip).map_err(|_| "mDNS interface unavailable")?;
        mdns.timeout = Duration::from_secs(5);
        resolver = resolver
            .with_mdns(mdns)
            .map_err(|_| "mDNS configuration failed")?;
    }
    let resolver = Arc::new(resolver);
    rt.set_webrtc_factory(Rc::new(
        NativeWebRtcFactory::with_resolver(
            Settings {
                bind,
                local_address_policy: if std::env::var("GENET_RTC_LOCAL_MDNS").as_deref() == Ok("1")
                {
                    LocalAddressPolicy::Mdns
                } else {
                    LocalAddressPolicy::Expose
                },
                gathering_timeout: Duration::from_secs(5),
                allocation_refresh_interval_cap: Some(Duration::from_secs(1)),
                mdns_timeout: Duration::from_secs(5),
                ..Settings::default()
            },
            resolver.clone(),
        )
        .map_err(|_| "factory initialization failed")?,
    ));
    rt.set_base_url("https://rtc-interop.invalid/one.html")
        .map_err(|_| "base URL failed")?;
    rt.set_script_resource_loader(Box::new(Page));
    rt.parse_document_interleaved(
        "<html><body>RTC native test peer</body></html>",
        &script_runtime_api::NoScriptLoader,
    );
    Ok((rt, resolver))
}

fn run<E: ScriptEngine>(bind: IpAddr, dns_server: Option<SocketAddr>) -> Result<(), String> {
    let (mut rt, resolver) = runtime::<E>(bind, dns_server)?;
    let (sender, receiver) = mpsc::sync_channel(1);
    std::thread::spawn(move || {
        let stdin = io::stdin();
        let mut input = stdin.lock();
        loop {
            let mut line = Vec::new();
            // Bound the control channel before allocating a full input line.
            match input.by_ref().take(262_145).read_until(b'\n', &mut line) {
                Ok(0) | Err(_) => break,
                Ok(_) if line.len() > 262_144 => break,
                Ok(_) => {
                    if sender.send(line).is_err() {
                        break;
                    }
                },
            }
        }
    });
    let mut stdout = io::stdout().lock();
    writeln!(stdout, "{{\"ready\":true}}").map_err(|_| "ready write failed")?;
    stdout.flush().map_err(|_| "ready flush failed")?;
    loop {
        rt.pump_webrtc(8);
        let now = rt.clock_elapsed_ms();
        rt.run_timers(8, now).map_err(|_| "timer dispatch failed")?;
        rt.run_microtasks();
        let line = match receiver.recv_timeout(Duration::from_millis(1)) {
            Ok(line) => line,
            Err(mpsc::RecvTimeoutError::Timeout) => continue,
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        };
        let command: Value = serde_json::from_slice(&line).map_err(|_| "invalid control JSON")?;
        let result = match command["op"].as_str() {
            Some("eval") => {
                let source = command["source"].as_str().ok_or("source required")?;
                match rt.eval(source) {
                    Ok(value) => rt
                        .value_to_string(&value)
                        .map(Value::String)
                        .map_err(|_| "result serialization failed".to_owned()),
                    Err(error) => Err(rt.describe_error(&error)),
                }
            },
            Some("metrics") => {
                let d = resolver.mdns_diagnostics();
                Ok(
                    json!({"pending":rt.pending_webrtc(),"dnsPending":resolver.pending(),"mdnsStarted":resolver.mdns_counts().0,"mdnsResolved":resolver.mdns_counts().1,"ice":trace::snapshot(),
                    "mdnsDiagnostics":{"timeouts":d.timeouts,"ioErrors":d.io_errors,"invalidAnswers":d.invalid_answers,"multicastReceived":d.multicast_received,"sourceRefusals":d.source_refusals,"messageRefusals":d.message_refusals}}),
                )
            },
            Some("navigate") => {
                let stale = rt
                    .eval(
                        r#"(function(Ctor,peer,channels){return function(){
                    const result=[peer.signalingState,...channels.map(c=>c.readyState)];
                    for(const f of [()=>new Ctor(),()=>peer.createDataChannel('stale')]){
                        try{f();result.push('accepted');}catch(e){result.push(e.name);}
                    }return JSON.stringify(result);
                };})(RTCPeerConnection,__interop.pc,__interop.channels)"#,
                    )
                    .map_err(|_| "stale-handle capture failed")?;
                rt.eval("location.href='https://rtc-interop.invalid/two.html'")
                    .map_err(|_| "navigation request failed")?;
                rt.run_event_loop(64)
                    .map_err(|_| "navigation dispatch failed")?;
                let value = rt
                    .engine_mut()
                    .call_function(&stale, &stale, &[])
                    .map_err(|_| "stale-handle check failed")?;
                let text = rt
                    .value_to_string(&value)
                    .map_err(|_| "stale result failed")?;
                Ok(
                    json!({"pending":rt.pending_webrtc(),"stale":serde_json::from_str::<Value>(&text).map_err(|_|"stale JSON failed")?}),
                )
            },
            Some("quit") => break,
            _ => Err("unknown control operation".into()),
        };
        let response = match result {
            Ok(value) => json!({"id":command["id"],"ok":true,"value":value}),
            // Fixed fixture scripts only; never include source/SDP in diagnostics.
            Err(_) => {
                json!({"id":command["id"],"ok":false,"error":"native script operation failed"})
            },
        };
        writeln!(stdout, "{response}").map_err(|_| "control write failed")?;
        stdout.flush().map_err(|_| "control flush failed")?;
    }
    Ok(())
}

fn main() {
    trace::install();
    // Match the substantial stack available to the engine's integration-test
    // threads rather than Windows' small executable-entry stack.
    let result = std::thread::Builder::new()
        .name("rtc-interop-runtime".into())
        .stack_size(8 * 1024 * 1024)
        .spawn(fixture)
        .expect("fixture thread")
        .join()
        .unwrap_or_else(|_| Err("fixture thread panicked".into()));
    if let Err(error) = result {
        eprintln!("RTC fixture: {error}");
        std::process::exit(1);
    }
}

fn fixture() -> Result<(), String> {
    let args: Vec<String> = std::env::args().collect();
    let bind = args
        .get(2)
        .ok_or("usage: interop_host boa|nova BIND_IP [DNS_IP:PORT]")?
        .parse()
        .map_err(|_| "invalid bind IP")?;
    let dns_server = args
        .get(3)
        .map(|s| s.parse().map_err(|_| "invalid DNS endpoint"))
        .transpose()?;
    match args.get(1).map(String::as_str) {
        Some("boa") => run::<script_engine_boa::BoaEngine>(bind, dns_server),
        #[cfg(target_pointer_width = "64")]
        Some("nova") => run::<script_engine_nova::NovaEngine>(bind, dns_server),
        _ => Err("unknown engine".into()),
    }
}
