// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Selectors Level 4 grammar over `cssparser` tokens.

use std::fmt;

use cssparser::{BasicParseErrorKind, Delimiter, Parser, ParserInput, Token};

use crate::PseudoClass;
use crate::ast::{Combinator, Nth, NthKind, Selector, SelectorList, Simple};
use crate::attr::{AttrOperator, Attribute, HTML_CASE_INSENSITIVE_ATTRIBUTES, ParsedCase};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ParseErrorKind {
    /// A compound with nothing in it, or a list with no selectors.
    Empty,
    /// A pseudo-class or pseudo-element this crate or the consumer's
    /// vocabulary does not know.
    UnsupportedPseudo(String),
    /// `ns|E`. Named namespace prefixes are not supported.
    NamespacePrefix(String),
    /// A construct that is not allowed where it appeared, such as a class
    /// after `::part()`.
    Misplaced,
    Attribute,
    UnexpectedToken(String),
    UnexpectedEnd,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParseError {
    pub kind: ParseErrorKind,
    pub line: u32,
    pub column: u32,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} at {}:{}", self.kind, self.line, self.column)
    }
}

impl std::error::Error for ParseError {}

type Error<'i> = cssparser::ParseError<'i, ParseErrorKind>;

/// Where in a selector the parser stands.
#[derive(Clone, Copy, Default)]
struct State {
    /// Inside a functional pseudo-class: no `::slotted()` or `::part()`.
    nested: bool,
    /// Right of `::slotted()`: nothing but functional pseudo-classes.
    after_slotted: bool,
    /// Right of `::part()`: state pseudo-classes and functional ones.
    after_part: bool,
}

impl State {
    fn after_pseudo(self) -> bool {
        self.after_slotted || self.after_part
    }

    fn nest(self) -> Self {
        Self {
            nested: true,
            ..self
        }
    }
}

impl<P: PseudoClass> SelectorList<P> {
    /// Parse a selector list. One invalid selector fails the list.
    pub fn parse(source: &str) -> Result<Self, ParseError> {
        let mut input = ParserInput::new(source);
        let mut input = Parser::new(&mut input);
        parse_list(&mut input, State::default(), false)
            .map(Self)
            .map_err(|error| {
                let kind = match error.kind {
                    cssparser::ParseErrorKind::Custom(kind) => kind,
                    cssparser::ParseErrorKind::Basic(BasicParseErrorKind::UnexpectedToken(t)) => {
                        ParseErrorKind::UnexpectedToken(format!("{t:?}"))
                    },
                    cssparser::ParseErrorKind::Basic(_) => ParseErrorKind::UnexpectedEnd,
                };
                ParseError {
                    kind,
                    line: error.location.line,
                    column: error.location.column,
                }
            })
    }
}

fn custom<'i>(input: &Parser<'i, '_>, kind: ParseErrorKind) -> Error<'i> {
    input.new_custom_error(kind)
}

/// `forgiving` drops invalid selectors instead of failing, as `:is()` and
/// `:where()` require, and so may return an empty list.
fn parse_list<'i, P: PseudoClass>(
    input: &mut Parser<'i, '_>,
    state: State,
    forgiving: bool,
) -> Result<Vec<Selector<P>>, Error<'i>> {
    let mut list = Vec::new();
    loop {
        let parsed = input.parse_until_before(Delimiter::Comma, |input| {
            let selector = parse_selector(input, state)?;
            input.expect_exhausted()?;
            Ok(selector)
        });
        match parsed {
            Ok(selector) => list.push(selector),
            Err(_) if forgiving => {},
            Err(error) => return Err(error),
        }
        if input.next().is_err() {
            return Ok(list);
        }
    }
}

fn parse_selector<'i, P: PseudoClass>(
    input: &mut Parser<'i, '_>,
    mut state: State,
) -> Result<Selector<P>, Error<'i>> {
    let mut compounds = Vec::new();
    let mut combinators = Vec::new();
    input.skip_whitespace();
    loop {
        parse_compound(input, &mut state, &mut compounds, &mut combinators)?;
        if state.after_pseudo() {
            break;
        }
        let Some(combinator) = parse_combinator(input) else {
            break;
        };
        combinators.push(combinator);
        input.skip_whitespace();
    }
    compounds.reverse();
    combinators.reverse();
    Ok(Selector::new(compounds, combinators))
}

/// `None` at the end of the selector. A bare non-combinator token is left for
/// the caller's exhaustion check to reject.
fn parse_combinator(input: &mut Parser<'_, '_>) -> Option<Combinator> {
    let mut whitespace = false;
    loop {
        let before = input.state();
        match input.next_including_whitespace() {
            Ok(Token::WhiteSpace(_)) => whitespace = true,
            Ok(Token::Delim('>')) => return Some(Combinator::Child),
            Ok(Token::Delim('+')) => return Some(Combinator::NextSibling),
            Ok(Token::Delim('~')) => return Some(Combinator::LaterSibling),
            Ok(_) => {
                input.reset(&before);
                return whitespace.then_some(Combinator::Descendant);
            },
            Err(_) => return None,
        }
    }
}

/// Parse one compound onto `compounds`. `::slotted()` and `::part()` close the
/// compound they follow and open the subject compound, joined by their own
/// combinator, so one call may push two.
fn parse_compound<'i, P: PseudoClass>(
    input: &mut Parser<'i, '_>,
    state: &mut State,
    compounds: &mut Vec<Vec<Simple<P>>>,
    combinators: &mut Vec<Combinator>,
) -> Result<(), Error<'i>> {
    let mut simples = Vec::new();
    parse_type_selector(input, &mut simples)?;
    if state.after_pseudo() && !simples.is_empty() {
        return Err(custom(input, ParseErrorKind::Misplaced));
    }
    loop {
        let before = input.state();
        let token = match input.next_including_whitespace() {
            Ok(token) => token.clone(),
            Err(_) => break,
        };
        match token {
            Token::IDHash(id) if !state.after_pseudo() => simples.push(Simple::Id((&*id).into())),
            Token::Delim('.') if !state.after_pseudo() => {
                let class = expect_adjacent_ident(input)?;
                simples.push(Simple::Class(class));
            },
            Token::SquareBracketBlock if !state.after_pseudo() => {
                let attribute = input.parse_nested_block(parse_attribute)?;
                simples.push(Simple::Attribute(attribute));
            },
            Token::IDHash(_) | Token::Delim('.') | Token::SquareBracketBlock => {
                return Err(custom(input, ParseErrorKind::Misplaced));
            },
            Token::Colon => {
                let double = input
                    .try_parse(|input| match input.next_including_whitespace() {
                        Ok(Token::Colon) => Ok(()),
                        _ => Err(()),
                    })
                    .is_ok();
                if double {
                    // Close the compound so far (possibly empty, as in a bare
                    // `::slotted(p)`) and open the subject compound.
                    compounds.push(std::mem::take(&mut simples));
                    combinators.push(parse_pseudo_element(input, state, &mut simples)?);
                } else {
                    simples.push(parse_pseudo_class(input, *state)?);
                }
            },
            _ => {
                input.reset(&before);
                break;
            },
        }
    }
    if simples.is_empty() {
        return Err(custom(input, ParseErrorKind::Empty));
    }
    compounds.push(simples);
    Ok(())
}

fn expect_adjacent_ident<'i>(input: &mut Parser<'i, '_>) -> Result<Box<str>, Error<'i>> {
    let location = input.current_source_location();
    match input.next_including_whitespace()? {
        Token::Ident(name) => Ok((&**name).into()),
        token => Err(location.new_unexpected_token_error(token.clone())),
    }
}

/// `E`, `*`, `*|E`, `*|*`, `|E`, `|*`. Pushes nothing when the compound has no
/// type selector.
fn parse_type_selector<'i, P>(
    input: &mut Parser<'i, '_>,
    simples: &mut Vec<Simple<P>>,
) -> Result<(), Error<'i>> {
    fn local_name<P>(name: &str) -> Simple<P> {
        Simple::LocalName {
            name: name.into(),
            lower: name.to_ascii_lowercase().into(),
        }
    }
    fn bar(input: &mut Parser<'_, '_>) -> bool {
        input
            .try_parse(|input| match input.next_including_whitespace() {
                Ok(Token::Delim('|')) => Ok(()),
                _ => Err(()),
            })
            .is_ok()
    }
    fn after_bar<'i, P>(input: &mut Parser<'i, '_>) -> Result<Simple<P>, Error<'i>> {
        let location = input.current_source_location();
        match input.next_including_whitespace()? {
            Token::Ident(name) => Ok(local_name(name)),
            Token::Delim('*') => Ok(Simple::Universal),
            token => Err(location.new_unexpected_token_error(token.clone())),
        }
    }

    let before = input.state();
    let token = match input.next_including_whitespace() {
        Ok(token) => token.clone(),
        Err(_) => return Ok(()),
    };
    match token {
        Token::Ident(name) => {
            if bar(input) {
                return Err(custom(input, ParseErrorKind::NamespacePrefix(name.to_string())));
            }
            simples.push(local_name(&name));
        },
        Token::Delim('*') => {
            let simple = if bar(input) { after_bar(input)? } else { Simple::Universal };
            simples.push(simple);
        },
        Token::Delim('|') => {
            simples.push(Simple::NoNamespace);
            simples.push(after_bar(input)?);
        },
        _ => input.reset(&before),
    }
    Ok(())
}

fn parse_attribute<'i>(input: &mut Parser<'i, '_>) -> Result<Attribute, Error<'i>> {
    let bad = |input: &Parser<'i, '_>| custom(input, ParseErrorKind::Attribute);
    input.skip_whitespace();
    let (any_namespace, explicit_namespace, name) = match input.next_including_whitespace()?.clone()
    {
        Token::Ident(name) => {
            let prefixed = input
                .try_parse(|input| match input.next_including_whitespace() {
                    Ok(Token::Delim('|')) => match input.next_including_whitespace() {
                        Ok(Token::Ident(_)) => Ok(()),
                        _ => Err(()),
                    },
                    _ => Err(()),
                })
                .is_ok();
            if prefixed {
                return Err(custom(input, ParseErrorKind::NamespacePrefix(name.to_string())));
            }
            (false, false, name)
        },
        Token::Delim('*') => match (
            input.next_including_whitespace().cloned(),
            input.next_including_whitespace().cloned(),
        ) {
            (Ok(Token::Delim('|')), Ok(Token::Ident(name))) => (true, true, name),
            _ => return Err(bad(input)),
        },
        Token::Delim('|') => match input.next_including_whitespace().cloned() {
            Ok(Token::Ident(name)) => (false, true, name),
            _ => return Err(bad(input)),
        },
        _ => return Err(bad(input)),
    };
    let name_lower: Box<str> = name.to_ascii_lowercase().into();

    let operator = match input.next() {
        Err(_) => None,
        Ok(Token::Delim('=')) => Some(AttrOperator::Equal),
        Ok(Token::IncludeMatch) => Some(AttrOperator::Includes),
        Ok(Token::DashMatch) => Some(AttrOperator::DashMatch),
        Ok(Token::PrefixMatch) => Some(AttrOperator::Prefix),
        Ok(Token::SubstringMatch) => Some(AttrOperator::Substring),
        Ok(Token::SuffixMatch) => Some(AttrOperator::Suffix),
        Ok(_) => return Err(bad(input)),
    };
    let operation = match operator {
        None => None,
        Some(operator) => {
            let value: Box<str> = (&**input.expect_ident_or_string()?).into();
            let flag = match input.next() {
                Err(_) => None,
                Ok(Token::Ident(flag)) => Some(flag.to_ascii_lowercase()),
                Ok(_) => return Err(bad(input)),
            };
            let case = match flag.as_deref() {
                Some("i") => ParsedCase::Insensitive,
                Some("s") => ParsedCase::Sensitive,
                Some(_) => return Err(bad(input)),
                None if !explicit_namespace
                    && HTML_CASE_INSENSITIVE_ATTRIBUTES.contains(&&*name_lower) =>
                {
                    ParsedCase::InsensitiveInHtml
                },
                None => ParsedCase::Sensitive,
            };
            Some((operator, case, value))
        },
    };
    Ok(Attribute {
        any_namespace,
        name: (&*name).into(),
        name_lower,
        operation,
    })
}

/// After `::`. Pushes the pseudo-element onto `simples` and returns the
/// combinator that joins it to the compound on its left.
fn parse_pseudo_element<'i, P: PseudoClass>(
    input: &mut Parser<'i, '_>,
    state: &mut State,
    simples: &mut Vec<Simple<P>>,
) -> Result<Combinator, Error<'i>> {
    let location = input.current_source_location();
    let name = match input.next_including_whitespace()?.clone() {
        Token::Function(name) => name,
        Token::Ident(name) => {
            return Err(custom(input, ParseErrorKind::UnsupportedPseudo(format!("::{name}"))));
        },
        token => return Err(location.new_unexpected_token_error(token)),
    };
    if state.nested || state.after_pseudo() {
        return Err(custom(input, ParseErrorKind::Misplaced));
    }
    let nested = state.nest();
    if name.eq_ignore_ascii_case("slotted") {
        let inner = input.parse_nested_block(|input| parse_compound_only(input, nested))?;
        simples.push(Simple::Slotted(Box::new(inner)));
        state.after_slotted = true;
        Ok(Combinator::SlotAssignment)
    } else if name.eq_ignore_ascii_case("part") {
        let names = input.parse_nested_block(|input| {
            let mut names = vec![Box::<str>::from(&**input.expect_ident()?)];
            while let Ok(name) = input.try_parse(|input| input.expect_ident_cloned()) {
                names.push((&*name).into());
            }
            Ok(names)
        })?;
        simples.push(Simple::Part(names));
        state.after_part = true;
        Ok(Combinator::Part)
    } else {
        Err(custom(input, ParseErrorKind::UnsupportedPseudo(format!("::{name}()"))))
    }
}

/// One compound and nothing else, as `:host()` and `::slotted()` take.
fn parse_compound_only<'i, P: PseudoClass>(
    input: &mut Parser<'i, '_>,
    mut state: State,
) -> Result<Selector<P>, Error<'i>> {
    let mut compounds = Vec::new();
    input.skip_whitespace();
    parse_compound(input, &mut state, &mut compounds, &mut Vec::new())?;
    input.skip_whitespace();
    Ok(Selector::new(compounds, Vec::new()))
}

/// After a single `:`.
fn parse_pseudo_class<'i, P: PseudoClass>(
    input: &mut Parser<'i, '_>,
    state: State,
) -> Result<Simple<P>, Error<'i>> {
    let location = input.current_source_location();
    let unsupported =
        |input: &Parser<'i, '_>, name: &str| custom(input, ParseErrorKind::UnsupportedPseudo(format!(":{name}")));
    let edge = |kind| {
        Simple::Nth(Nth {
            kind,
            a: 0,
            b: 1,
            of: Vec::new(),
        })
    };
    match input.next_including_whitespace()?.clone() {
        Token::Ident(name) => {
            let lower = name.to_ascii_lowercase();
            if state.after_slotted {
                return Err(custom(input, ParseErrorKind::Misplaced));
            }
            let structural = match &*lower {
                "root" => Some(Simple::Root),
                "empty" => Some(Simple::Empty),
                "scope" => Some(Simple::Scope),
                "host" => Some(Simple::Host(None)),
                "first-child" => Some(edge(NthKind::Child)),
                "last-child" => Some(edge(NthKind::LastChild)),
                "first-of-type" => Some(edge(NthKind::OfType)),
                "last-of-type" => Some(edge(NthKind::LastOfType)),
                "only-child" => Some(Simple::Only { of_type: false }),
                "only-of-type" => Some(Simple::Only { of_type: true }),
                _ => None,
            };
            match structural {
                Some(_) if state.after_pseudo() => Err(custom(input, ParseErrorKind::Misplaced)),
                Some(simple) => Ok(simple),
                None => P::parse(&lower)
                    .map(Simple::State)
                    .ok_or_else(|| unsupported(input, &lower)),
            }
        },
        Token::Function(name) => {
            let lower = name.to_ascii_lowercase();
            let nested = state.nest();
            let nth = match &*lower {
                "nth-child" => Some(NthKind::Child),
                "nth-last-child" => Some(NthKind::LastChild),
                "nth-of-type" => Some(NthKind::OfType),
                "nth-last-of-type" => Some(NthKind::LastOfType),
                _ => None,
            };
            if (nth.is_some() || lower == "host") && state.after_pseudo() {
                return Err(custom(input, ParseErrorKind::Misplaced));
            }
            input.parse_nested_block(|input| {
                if let Some(kind) = nth {
                    let (a, b) = cssparser::parse_nth(input)?;
                    let of = if !kind.of_type()
                        && input.try_parse(|input| input.expect_ident_matching("of")).is_ok()
                    {
                        parse_list(input, nested, false)?
                    } else {
                        Vec::new()
                    };
                    return Ok(Simple::Nth(Nth { kind, a, b, of }));
                }
                match &*lower {
                    "not" => Ok(Simple::Not(parse_list(input, nested, false)?)),
                    "is" => Ok(Simple::Is(parse_list(input, nested, true)?)),
                    "where" => Ok(Simple::Where(parse_list(input, nested, true)?)),
                    "host" => Ok(Simple::Host(Some(Box::new(parse_compound_only(input, nested)?)))),
                    _ => Err(unsupported(input, &format!("{lower}()"))),
                }
            })
        },
        token => Err(location.new_unexpected_token_error(token)),
    }
}
