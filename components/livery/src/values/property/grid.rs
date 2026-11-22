// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! CSS grid track lists and their sizing functions.

use super::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GridTrack {
    Auto,
    MinContent,
    MaxContent,
    Length(Length),
    Percent(f32),
    Fr(f32),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GridTrackSize {
    Breadth(GridTrack),
    MinMax(GridTrack, GridTrack),
}

#[derive(Clone, Debug, PartialEq)]
pub enum GridTrackComponent {
    Single(GridTrackSize),
    Repeat(u16, Vec<GridTrackSize>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum GridTemplate {
    None,
    Tracks(Vec<GridTrack>),
    TrackList(Vec<GridTrackComponent>),
}

fn function_arguments<'a>(input: &'a str, name: &str) -> Option<&'a str> {
    let (function, arguments) = input.split_once('(')?;
    function
        .eq_ignore_ascii_case(name)
        .then_some(arguments)?
        .strip_suffix(')')
}

impl FromStr for GridTrack {
    type Err = ParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let lower = input.trim().to_ascii_lowercase();
        match lower.as_str() {
            "auto" => Ok(Self::Auto),
            "min-content" => Ok(Self::MinContent),
            "max-content" => Ok(Self::MaxContent),
            value if value.ends_with("fr") => {
                Ok(Self::Fr(parse_non_negative(&value[..value.len() - 2])?))
            },
            value if value.ends_with('%') => Ok(Self::Percent(
                parse_non_negative(&value[..value.len() - 1])? / 100.0,
            )),
            value => {
                let length = value.parse::<Length>()?;
                if length.value < 0.0 {
                    return Err(ParseError::expected("a non-negative grid track length"));
                }
                Ok(Self::Length(length))
            },
        }
    }
}

impl FromStr for GridTrackSize {
    type Err = ParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        if let Some(arguments) = function_arguments(input, "minmax") {
            let (min, max) = arguments
                .split_once(',')
                .ok_or_else(|| ParseError::expected("two track breadths"))?;
            let min = min.parse()?;
            if matches!(min, GridTrack::Fr(_)) {
                return Err(ParseError::expected("an inflexible minimum track breadth"));
            }
            return Ok(Self::MinMax(min, max.parse()?));
        }
        Ok(Self::Breadth(input.parse()?))
    }
}

impl FromStr for GridTrackComponent {
    type Err = ParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        if let Some(arguments) = function_arguments(input, "repeat") {
            let (count, body) = arguments
                .split_once(',')
                .ok_or_else(|| ParseError::expected("a repeat count and tracks"))?;
            let count = count
                .trim()
                .parse::<u16>()
                .ok()
                .filter(|count| *count > 0)
                .ok_or_else(|| ParseError::expected("a positive grid repetition count"))?;
            let tracks = split_top_level(body)
                .into_iter()
                .map(str::parse)
                .collect::<Result<Vec<_>, _>>()?;
            if tracks.is_empty() {
                return Err(ParseError::expected("one or more repeated grid tracks"));
            }
            return Ok(Self::Repeat(count, tracks));
        }
        Ok(Self::Single(input.parse()?))
    }
}

impl FromStr for GridTemplate {
    type Err = ParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        if input.trim().eq_ignore_ascii_case("none") {
            return Ok(Self::None);
        }
        let parts = split_top_level(input);
        if parts.is_empty() {
            return Err(ParseError::expected("one or more grid tracks"));
        }
        // Preserve the compact public representation of plain breadth lists.
        if input.contains('(') {
            parts
                .into_iter()
                .map(str::parse)
                .collect::<Result<_, _>>()
                .map(Self::TrackList)
        } else {
            parts
                .into_iter()
                .map(str::parse)
                .collect::<Result<_, _>>()
                .map(Self::Tracks)
        }
    }
}

fn write_track_list<T: fmt::Display>(
    formatter: &mut fmt::Formatter<'_>,
    tracks: &[T],
) -> fmt::Result {
    for (index, track) in tracks.iter().enumerate() {
        if index > 0 {
            formatter.write_str(" ")?;
        }
        write!(formatter, "{track}")?;
    }
    Ok(())
}

impl fmt::Display for GridTemplate {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => formatter.write_str("none"),
            Self::Tracks(tracks) => write_track_list(formatter, tracks),
            Self::TrackList(tracks) => write_track_list(formatter, tracks),
        }
    }
}

impl fmt::Display for GridTrackComponent {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Single(track) => track.fmt(formatter),
            Self::Repeat(count, tracks) => {
                write!(formatter, "repeat({count}, ")?;
                write_track_list(formatter, tracks)?;
                formatter.write_str(")")
            },
        }
    }
}

impl fmt::Display for GridTrackSize {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Breadth(breadth) => breadth.fmt(formatter),
            Self::MinMax(min, max) => write!(formatter, "minmax({min}, {max})"),
        }
    }
}

impl fmt::Display for GridTrack {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Auto => formatter.write_str("auto"),
            Self::MinContent => formatter.write_str("min-content"),
            Self::MaxContent => formatter.write_str("max-content"),
            Self::Length(value) => value.fmt(formatter),
            Self::Percent(value) => write!(formatter, "{}%", format_number(*value * 100.0)),
            Self::Fr(value) => write!(formatter, "{}fr", format_number(*value)),
        }
    }
}

fn parse_non_negative(input: &str) -> Result<f32, ParseError> {
    input
        .parse::<f32>()
        .ok()
        .filter(|value| value.is_finite() && *value >= 0.0)
        .ok_or_else(|| ParseError::expected("a non-negative grid track number"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn track_functions_round_trip_without_expanding_the_repetition() {
        let tracks: GridTemplate = "20px REPEAT(2, minmax(0, .4fr) auto) 1FR".parse().unwrap();
        assert_eq!(
            tracks.to_string(),
            "20px repeat(2, minmax(0, 0.4fr) auto) 1fr"
        );
        assert_eq!(tracks, tracks.to_string().parse().unwrap());
    }

    #[test]
    fn invalid_track_functions_reject_the_whole_list() {
        for invalid in [
            "1fr minmax(.5fr, 1fr)",
            "minmax(0, -1px)",
            "minmax(0, 1fr, 1fr)",
            "repeat(0, 10px)",
            "repeat(2, )",
            "repeat(2, repeat(2, 1fr))",
        ] {
            assert!(invalid.parse::<GridTemplate>().is_err(), "{invalid}");
        }
    }
}
