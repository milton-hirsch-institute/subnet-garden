// Copyright 2024 The Milton Hirsch Institute, B.V.
// SPDX-License-Identifier: Apache-2.0

use crate::param_str::errors::ParseError;
use crate::util::state_machine::{state, state_machine, StateMachine};
use crate::util::state_machine::{ParseResult, State, Termination};

pub type Segments = Vec<Segment>;

#[derive(Debug, PartialEq)]
pub(crate) struct VarFormat {
    is_numeric: bool,
    padding: usize,
}

impl VarFormat {
    pub(crate) fn is_numeric(&self) -> bool {
        self.is_numeric
    }

    pub(crate) fn padding(&self) -> usize {
        self.padding
    }

    pub(crate) fn new(is_numeric: bool, padding: usize) -> Self {
        VarFormat {
            is_numeric,
            padding,
        }
    }
}

#[cfg(test)]
impl Default for VarFormat {
    fn default() -> Self {
        VarFormat::new(false, 0)
    }
}

#[derive(Debug, PartialEq)]
pub(crate) enum Segment {
    Text(String),
    Variable(VarFormat),
}

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

static PADDING_STATE: FormatState = state(|b, c| -> FormatResult {
    match c {
        '0'..='9' => {
            b.padding = b.padding * 10 + c.to_digit(10).unwrap() as usize;
            Ok(PADDING_STATE)
        }
        '}' => VARIABLE_STATE.next(b, c),
        _ => Err(ParseError::InvalidValue(
            format!("Expected 0-9, found {}", c).to_string(),
        )),
    }
});

static START_PADDING_STATE: FormatState = state(|b, c| -> FormatResult {
    match c {
        '0' => {
            b.is_numeric = true;
            Ok(PADDING_STATE)
        }
        _ => PADDING_STATE.next(b, c),
    }
});

static VARIABLE_STATE: FormatState = state(|b, c| -> FormatResult {
    match c {
        '}' => {
            b.add_variable();
            Ok(TEXT_STATE)
        }
        ':' => Ok(START_PADDING_STATE),
        _ => Err(ParseError::InvalidValue(
            format!("Expected }}, found {}", c).to_string(),
        )),
    }
});

static TERMINATION: FormatTermination = |last_state, b| -> Result<(), ParseError> {
    if last_state == TEXT_STATE {
        b.add_text_segment();
        Ok(())
    } else {
        Err(ParseError::InvalidValue(
            "Unexpected end of format".to_string(),
        ))
    }
};

#[derive(Debug, PartialEq)]
struct BuildFormat {
    current_text: String,
    result: Vec<Segment>,
    is_numeric: bool,
    padding: usize,
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
        self.result.push(Segment::Variable(VarFormat::new(
            self.is_numeric,
            self.padding,
        )));
        self.is_numeric = false;
        self.padding = 0;
    }
}

static FORMAT_MACHINE: StateMachine<BuildFormat, char, ParseError> =
    state_machine(TEXT_STATE, TERMINATION);

pub(crate) fn parse(format: &str) -> Result<Segments, ParseError> {
    let mut build_format = BuildFormat {
        current_text: String::new(),
        result: Vec::new(),
        is_numeric: false,
        padding: 0,
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
                Err(ParseError::InvalidValue(
                    "Unexpected end of format".to_string()
                ))
            );
        }
        #[test]
        fn unexpected_variable_character() {
            assert_eq!(
                parse("aaa{>bbb"),
                Err(ParseError::InvalidValue("Expected }, found >".to_string()))
            );
        }
        #[test]
        fn unterminated_variable_character() {
            assert_eq!(
                parse("aaa{"),
                Err(ParseError::InvalidValue(
                    "Unexpected end of format".to_string()
                ))
            );
        }

        #[test]
        fn unexepected_padding_character() {
            assert_eq!(
                parse("aaa{:x}"),
                Err(ParseError::InvalidValue(
                    "Expected 0-9, found x".to_string()
                ))
            );
        }
        #[test]
        fn unexepected_numeric_padding_character() {
            assert_eq!(
                parse("aaa{:05x}"),
                Err(ParseError::InvalidValue(
                    "Expected 0-9, found x".to_string()
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
        fn variables() {
            assert_eq!(
                parse("aaa\\{}bbb{}ccc"),
                Ok(vec![
                    Segment::Text("aaa{}bbb".to_string()),
                    Segment::Variable(VarFormat::default()),
                    Segment::Text("ccc".to_string())
                ])
            );
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

        #[test]
        fn non_numeric_padding() {
            assert_eq!(
                parse("x{:5}y"),
                Ok(vec![
                    Segment::Text("x".to_string()),
                    Segment::Variable(VarFormat::new(false, 5)),
                    Segment::Text("y".to_string())
                ])
            );
        }

        #[test]
        fn numeric_padding() {
            assert_eq!(
                parse("x{:05}y"),
                Ok(vec![
                    Segment::Text("x".to_string()),
                    Segment::Variable(VarFormat::new(true, 5)),
                    Segment::Text("y".to_string())
                ])
            );
        }
    }
}
