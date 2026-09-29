use std::{fmt, str::FromStr};

use super::ParseError;
use cssparser::{Parser, ParserInput, ToCss, Token};

/// Bounded text-only CSS content grammar. Unsupported items invalidate the
/// declaration instead of silently producing a different string.
#[derive(Clone, Debug, PartialEq)]
pub enum Content {
    Normal,
    None,
    Items(Vec<ContentItem>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum ContentItem {
    Text(String),
    Attribute(String),
    /// Decimal `counter()` or `counters()`; the latter carries its separator.
    Counter {
        name: String,
        separator: Option<String>,
    },
}

impl Content {
    /// Resolve without a counter environment (every absent counter starts at
    /// zero). A document with counter state uses `resolve_with_counters`.
    pub fn resolve(&self, attribute: impl FnMut(&str) -> Option<String>) -> Option<String> {
        self.resolve_with_counters(attribute, |_, _| "0".into())
    }

    pub fn has_counters(&self) -> bool {
        matches!(self, Self::Items(items) if items.iter().any(|item| matches!(item, ContentItem::Counter { .. })))
    }

    pub fn resolve_with_counters(
        &self,
        mut attribute: impl FnMut(&str) -> Option<String>,
        mut counter: impl FnMut(&str, Option<&str>) -> String,
    ) -> Option<String> {
        let Self::Items(items) = self else {
            return None;
        };
        Some(
            items
                .iter()
                .map(|item| match item {
                    ContentItem::Text(text) => text.clone(),
                    ContentItem::Attribute(name) => attribute(name).unwrap_or_default(),
                    ContentItem::Counter { name, separator } => counter(name, separator.as_deref()),
                })
                .collect(),
        )
    }
}

impl FromStr for Content {
    type Err = ParseError;
    fn from_str(source: &str) -> Result<Self, Self::Err> {
        let mut input = ParserInput::new(source);
        let mut parser = Parser::new(&mut input);
        if let Ok(keyword) = parser.try_parse(|p| p.expect_ident_cloned()) {
            parser
                .expect_exhausted()
                .map_err(|_| ParseError::expected("content"))?;
            return if keyword.eq_ignore_ascii_case("normal") {
                Ok(Self::Normal)
            } else if keyword.eq_ignore_ascii_case("none") {
                Ok(Self::None)
            } else {
                Err(ParseError::expected("content"))
            };
        }
        let mut items = Vec::new();
        while !parser.is_exhausted() {
            match parser
                .next()
                .map_err(|_| ParseError::expected("content"))?
                .clone()
            {
                Token::QuotedString(text) => items.push(ContentItem::Text(text.to_string())),
                Token::Function(name) if name.eq_ignore_ascii_case("attr") => {
                    let name = parser
                        .parse_nested_block(|p| {
                            let name = p.expect_ident_cloned()?;
                            p.expect_exhausted()?;
                            Ok::<_, cssparser::ParseError<'_, ()>>(name.to_string())
                        })
                        .map_err(|_| ParseError::expected("attr(identifier)"))?;
                    items.push(ContentItem::Attribute(name));
                },
                Token::Function(function)
                    if function.eq_ignore_ascii_case("counter")
                        || function.eq_ignore_ascii_case("counters") =>
                {
                    let nested = function.eq_ignore_ascii_case("counters");
                    let item = parser
                        .parse_nested_block(|p| {
                            let name = p.expect_ident_cloned()?;
                            if !super::counters::counter_name(&name) {
                                return Err(p.new_custom_error(()));
                            }
                            let separator = if nested {
                                p.expect_comma()?;
                                Some(p.expect_string_cloned()?.to_string())
                            } else {
                                None
                            };
                            if !p.is_exhausted() {
                                p.expect_comma()?;
                                p.expect_ident_matching("decimal")?;
                            }
                            p.expect_exhausted()?;
                            Ok::<_, cssparser::ParseError<'_, ()>>(ContentItem::Counter {
                                name: name.to_string(),
                                separator,
                            })
                        })
                        .map_err(|_| ParseError::expected("decimal counter() or counters()"))?;
                    items.push(item);
                },
                _ => return Err(ParseError::expected("string or attr(identifier)")),
            }
        }
        if items.is_empty() {
            Err(ParseError::expected("content"))
        } else {
            Ok(Self::Items(items))
        }
    }
}

impl fmt::Display for Content {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Normal => f.write_str("normal"),
            Self::None => f.write_str("none"),
            Self::Items(items) => {
                for (index, item) in items.iter().enumerate() {
                    if index != 0 {
                        f.write_str(" ")?;
                    }
                    match item {
                        ContentItem::Text(text) => {
                            Token::QuotedString(text.as_str().into()).to_css(f)?
                        },
                        ContentItem::Attribute(name) => {
                            f.write_str("attr(")?;
                            cssparser::serialize_identifier(name, f)?;
                            f.write_str(")")?;
                        },
                        ContentItem::Counter { name, separator } => {
                            f.write_str(if separator.is_some() {
                                "counters("
                            } else {
                                "counter("
                            })?;
                            cssparser::serialize_identifier(name, f)?;
                            if let Some(separator) = separator {
                                f.write_str(", ")?;
                                Token::QuotedString(separator.as_str().into()).to_css(f)?;
                            }
                            f.write_str(")")?;
                        },
                    }
                }
                Ok(())
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn text_content_is_tokenized_and_round_trips() {
        let value: Content = r#""A\41 " attr(data-label) "!""#.parse().unwrap();
        assert_eq!(
            value.resolve(|_| Some("quoted \" value".into())).unwrap(),
            "AAquoted \" value!"
        );
        assert_eq!(value.to_string().parse::<Content>().unwrap(), value);
        assert_eq!(
            "attr(missing)"
                .parse::<Content>()
                .unwrap()
                .resolve(|_| None),
            Some(String::new())
        );
        for invalid in [
            "",
            "none 'x'",
            "counter(foo, upper-roman)",
            "'x' url(a)",
            "attr(x, y)",
        ] {
            assert!(invalid.parse::<Content>().is_err(), "{invalid}");
        }
    }

    #[test]
    fn decimal_counter_functions_parse_and_resolve_through_owner() {
        let content: Content = r#"counter(chapter, decimal) "." counters(section, ".", decimal)"#
            .parse()
            .unwrap();
        assert_eq!(content.to_string().parse::<Content>().unwrap(), content);
        assert_eq!(
            content
                .resolve_with_counters(
                    |_| None,
                    |name, separator| match (name, separator) {
                        ("chapter", None) => "2".into(),
                        ("section", Some(".")) => "3.4".into(),
                        _ => panic!("unexpected counter request"),
                    }
                )
                .as_deref(),
            Some("2.3.4")
        );
        for invalid in [
            "counter(none)",
            "counter(initial)",
            "counter(x,)",
            "counters(x)",
            "counters(x, '.') extra",
            "counter(x, lower-alpha)",
        ] {
            assert!(invalid.parse::<Content>().is_err(), "{invalid}");
        }
    }
}
