#![allow(dead_code)]

// Copyright 2024 The Milton Hirsch Institute, B.V.
// SPDX-License-Identifier: Apache-2.0

use crate::param_str::errors::ParseError;
use crate::util::state_machine;
use crate::util::state_machine::{state, state_machine, ParseResult, State, Termination};

#[derive(Debug, PartialEq)]
struct BuildRange {
    current_text: String,
    start: Option<usize>,
    end: Option<usize>,
}
impl BuildRange {
    #[inline(always)]
    fn result(self) -> (usize, usize) {
        (
            self.start.expect("Start not set"),
            self.end.expect("End not set"),
        )
    }

    #[inline(always)]
    fn add_text(&mut self, c: char) {
        self.current_text.push(c);
    }

    #[inline(always)]
    fn set_start(&mut self, start: usize) {
        self.start = Some(start);
    }

    #[inline(always)]
    fn set_end(&mut self, end: usize) {
        self.end = Some(end);
    }
}

type RangeState = State<BuildRange, char, ParseError>;
type RangeResult = ParseResult<BuildRange, char, ParseError>;
type RangeTermination = Termination<BuildRange, char, ParseError>;
type RangeStateMachine = state_machine::StateMachine<BuildRange, char, ParseError>;

static INIT_STATE: RangeState = state(|_b, c| -> RangeResult {
    match c {
        '%' => Ok(START_STATE),
        _ => Err(ParseError::InvalidValue(
            format!("Expected %, found {c}").to_string(),
        )),
    }
});

static START_STATE: RangeState = state(|b, c| -> RangeResult {
    match c {
        '0'..='9' => {
            b.add_text(c);
            Ok(START_STATE)
        }
        '-' => {
            if b.current_text.is_empty() {
                return Err(ParseError::InvalidValue(
                    format!("Expected digit, found {c}").to_string(),
                ));
            }
            let start = b.current_text.parse::<usize>().unwrap();
            b.set_start(start);
            b.current_text.clear();
            Ok(END_STATE)
        }
        _ => Err(ParseError::InvalidValue(
            format!("Expected digit or '.', found {c}").to_string(),
        )),
    }
});

static END_STATE: RangeState = state(|b, c| -> RangeResult {
    match c {
        '0'..='9' => {
            b.add_text(c);
            Ok(END_STATE)
        }
        _ => Err(ParseError::InvalidValue(
            format!("Expected digit, found {c}").to_string(),
        )),
    }
});

static TERMINATION: RangeTermination = |last_state, b| -> Result<(), ParseError> {
    if last_state == END_STATE {
        if let Ok(end) = b.current_text.parse::<usize>() {
            b.set_end(end);
            return Ok(());
        }
    }
    Err(ParseError::InvalidValue(
        "Unexpected end of range".to_string(),
    ))
};

static RANGE_STATE_MACHINE: RangeStateMachine = state_machine(INIT_STATE, TERMINATION);

pub fn parse_range(range_str: &str) -> Result<Vec<String>, ParseError> {
    let mut b = BuildRange {
        current_text: String::new(),
        start: None,
        end: None,
    };
    RANGE_STATE_MACHINE.run(&mut b, range_str.chars().collect::<Vec<char>>().iter())?;

    if b.end < b.start {
        return Err(ParseError::InvalidValue(
            "End must be greater than or equal to start".to_string(),
        ));
    }

    let mut result: Vec<String> = Vec::new();
    for i in b.start.unwrap()..=b.end.unwrap() {
        result.push(i.to_string());
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_string() {
        assert_eq!(
            parse_range(""),
            Err(ParseError::InvalidValue(
                "Unexpected end of range".to_string()
            ))
        );
    }

    #[test]
    fn unexpected_start_character() {
        assert_eq!(
            parse_range("0..9"),
            Err(ParseError::InvalidValue("Expected %, found 0".to_string()))
        );
    }

    #[test]
    fn missing_start() {
        assert_eq!(
            parse_range("%-9"),
            Err(ParseError::InvalidValue(
                "Expected digit, found -".to_string()
            ))
        );
    }

    #[test]
    fn unexpected_after_start() {
        assert_eq!(
            parse_range("%0,.9"),
            Err(ParseError::InvalidValue(
                "Expected digit or '.', found ,".to_string()
            ))
        );
    }

    #[test]
    fn unexpected_after_dash() {
        assert_eq!(
            parse_range("%0-,9"),
            Err(ParseError::InvalidValue(
                "Expected digit, found ,".to_string()
            ))
        );
    }

    #[test]
    fn missing_end() {
        assert_eq!(
            parse_range("%0-"),
            Err(ParseError::InvalidValue(
                "Unexpected end of range".to_string()
            ))
        );
    }

    #[test]
    fn unexpected_after_end() {
        assert_eq!(
            parse_range("%0-9,"),
            Err(ParseError::InvalidValue(
                "Expected digit, found ,".to_string()
            ))
        );
    }

    #[test]
    fn end_less_than_start() {
        assert_eq!(
            parse_range("%9-0"),
            Err(ParseError::InvalidValue(
                "End must be greater than or equal to start".to_string()
            ))
        );
    }

    #[test]
    fn success() {
        assert_eq!(
            parse_range("%0-9"),
            Ok(vec![
                "0".to_string(),
                "1".to_string(),
                "2".to_string(),
                "3".to_string(),
                "4".to_string(),
                "5".to_string(),
                "6".to_string(),
                "7".to_string(),
                "8".to_string(),
                "9".to_string(),
            ])
        );
    }

    #[test]
    fn parse_double_digits() {
        assert_eq!(
            parse_range("%10-19"),
            Ok(vec![
                "10".to_string(),
                "11".to_string(),
                "12".to_string(),
                "13".to_string(),
                "14".to_string(),
                "15".to_string(),
                "16".to_string(),
                "17".to_string(),
                "18".to_string(),
                "19".to_string(),
            ])
        );
    }
}
