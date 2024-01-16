// Copyright 2024 The Milton Hirsch Institute, B.V.
// SPDX-License-Identifier: Apache-2.0

use crate::param_str::format::{ParseError, Segment, Segments};
use crate::state_machine::{state, state_machine, StateMachine};
use crate::state_machine::{ParseResult, State, Termination};

type FormatState = State<BuildFormat, char, ParseError>;
type FormatResult = ParseResult<BuildFormat, char, ParseError>;
type FormatTermination = Termination<BuildFormat, char, ParseError>;

static TEXT_STATE: FormatState = state(|b, c| -> FormatResult {
    match c {
        '\\' => Ok(ESCAPE_STATE),
        '{' => {
            b.add_text_segment();
            Ok(VARIABLE_STATE)
        }
        _ => {
            b.add_text(c);
            Ok(TEXT_STATE)
        }
    }
});

static ESCAPE_STATE: FormatState = state(|b, c| -> FormatResult {
    b.add_text(c);
    Ok(TEXT_STATE)
});

static VARIABLE_STATE: FormatState = state(|b, c| -> FormatResult {
    match c {
        '}' => {
            b.add_variable();
            Ok(TEXT_STATE)
        }
        _ => Err(ParseError::InvalidFormat(
            format!("Expected }}, found {}", c).to_string(),
        )),
    }
});

static TERMINATION: FormatTermination = |last_state, b| -> Result<(), ParseError> {
    if last_state == TEXT_STATE {
        b.add_text_segment();
        Ok(())
    } else {
        Err(ParseError::InvalidFormat(
            "Unexpected end of format".to_string(),
        ))
    }
};

#[derive(Debug, PartialEq)]
struct BuildFormat {
    current_text: String,
    result: Vec<Segment>,
}

impl BuildFormat {
    #[inline(always)]
    fn result(self) -> Vec<Segment> {
        self.result
    }
    #[inline(always)]
    fn add_text(&mut self, c: char) {
        self.current_text.push(c);
    }

    fn add_text_segment(&mut self) {
        if !self.current_text.is_empty() {
            self.result.push(Segment::Text(self.current_text.clone()));
        }
        self.current_text.truncate(0);
    }

    #[inline(always)]
    fn add_variable(&mut self) {
        self.result.push(Segment::Variable);
    }
}

static FORMAT_MACHINE: StateMachine<BuildFormat, char, ParseError> =
    state_machine(TEXT_STATE, TERMINATION);

pub fn parse(format: &str) -> Result<Segments, ParseError> {
    let mut build_format = BuildFormat {
        current_text: String::new(),
        result: Vec::new(),
    };
    FORMAT_MACHINE.run(
        &mut build_format,
        format.chars().collect::<Vec<char>>().iter(),
    )?;
    Ok(build_format.result())
}

#[cfg(test)]
mod tests {
    use super::*;

    mod parse {
        use super::*;

        #[test]
        fn missing_escape_character() {
            assert_eq!(
                parse("aaa\\"),
                Err(ParseError::InvalidFormat(
                    "Unexpected end of format".to_string()
                ))
            );
        }
        #[test]
        fn unexpected_variable_character() {
            assert_eq!(
                parse("aaa{>bbb"),
                Err(ParseError::InvalidFormat("Expected }, found >".to_string()))
            );
        }
        #[test]
        fn unterminated_variable_character() {
            assert_eq!(
                parse("aaa{"),
                Err(ParseError::InvalidFormat(
                    "Unexpected end of format".to_string()
                ))
            );
        }

        #[test]
        fn empty() {
            assert_eq!(parse(""), Ok(vec![]));
        }

        #[test]
        fn text() {
            assert_eq!(parse("aaa"), Ok(vec![Segment::Text("aaa".to_string())]));
        }

        #[test]
        fn text_with_escape() {
            assert_eq!(
                parse("aaa\\\\bbb\\{}ccc"),
                Ok(vec![Segment::Text("aaa\\bbb{}ccc".to_string())])
            );
        }

        #[test]
        fn text_with_escape_at_end() {
            assert_eq!(
                parse("aaa\\\\"),
                Ok(vec![Segment::Text("aaa\\".to_string())])
            );
        }
    }
}
