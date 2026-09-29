// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Named, non-reversed CSS counter operations. Values saturate to i32.

use std::{fmt, str::FromStr};

use cssparser::{Parser, ParserInput};

use super::ParseError;

#[derive(Clone, Debug, PartialEq)]
pub struct CounterOperation {
    pub name: String,
    pub value: i32,
}

pub(super) fn counter_name(name: &str) -> bool {
    ![
        "none",
        "initial",
        "inherit",
        "unset",
        "revert",
        "revert-layer",
        "default",
    ]
    .iter()
    .any(|keyword| name.eq_ignore_ascii_case(keyword))
}

fn parse(source: &str, default: i32) -> Result<Vec<CounterOperation>, ParseError> {
    let mut input = ParserInput::new(source);
    let mut parser = Parser::new(&mut input);
    if parser
        .try_parse(|p| p.expect_ident_matching("none"))
        .is_ok()
    {
        parser
            .expect_exhausted()
            .map_err(|_| ParseError::expected("none"))?;
        return Ok(Vec::new());
    }
    let mut result = Vec::new();
    while !parser.is_exhausted() {
        let name = parser
            .expect_ident_cloned()
            .map_err(|_| ParseError::expected("counter name"))?;
        if !counter_name(&name) {
            return Err(ParseError::expected("counter name"));
        }
        let value = parser.try_parse(|p| p.expect_integer()).unwrap_or(default);
        result.push(CounterOperation {
            name: name.to_string(),
            value,
        });
    }
    if result.is_empty() {
        Err(ParseError::expected("counter operation"))
    } else {
        Ok(result)
    }
}

macro_rules! counter_property {
    ($name:ident, $default:expr) => {
        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct $name(pub Vec<CounterOperation>);

        impl FromStr for $name {
            type Err = ParseError;
            fn from_str(source: &str) -> Result<Self, Self::Err> {
                parse(source, $default).map(Self)
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                if self.0.is_empty() {
                    return f.write_str("none");
                }
                for (index, operation) in self.0.iter().enumerate() {
                    if index > 0 {
                        f.write_str(" ")?;
                    }
                    cssparser::serialize_identifier(&operation.name, f)?;
                    write!(f, " {}", operation.value)?;
                }
                Ok(())
            }
        }
    };
}

counter_property!(CounterReset, 0);
counter_property!(CounterIncrement, 1);
counter_property!(CounterSet, 0);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integer_limits_clamp_at_the_declared_implementation_range() {
        assert_eq!(
            "n 99999999999999999999".parse::<CounterReset>().unwrap().0[0].value,
            i32::MAX
        );
        assert_eq!(
            "n -99999999999999999999".parse::<CounterReset>().unwrap().0[0].value,
            i32::MIN
        );
    }

    #[test]
    fn default_values_duplicates_and_escaped_names_round_trip() {
        assert_eq!(
            "chapter section -2 chapter 4"
                .parse::<CounterReset>()
                .unwrap()
                .to_string(),
            "chapter 0 section -2 chapter 4"
        );
        assert_eq!(
            "chapter section -2"
                .parse::<CounterIncrement>()
                .unwrap()
                .to_string(),
            "chapter 1 section -2"
        );
        assert_eq!(
            "chapter".parse::<CounterSet>().unwrap().to_string(),
            "chapter 0"
        );
        let escaped: CounterReset = r"\31 name 2".parse().unwrap();
        assert_eq!(
            escaped.to_string().parse::<CounterReset>().unwrap(),
            escaped
        );
        assert_eq!(
            "NONE".parse::<CounterReset>().unwrap(),
            CounterReset::default()
        );
        for invalid in [
            "",
            "none chapter",
            "chapter none",
            "chapter 1.5",
            "chapter 2px",
            "default",
            "reversed(chapter)",
            "inherit",
            "chapter initial",
        ] {
            assert!(invalid.parse::<CounterReset>().is_err(), "{invalid}");
        }
    }
}
