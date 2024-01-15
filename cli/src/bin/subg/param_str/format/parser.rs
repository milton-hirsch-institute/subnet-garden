// Copyright 2024 The Milton Hirsch Institute, B.V.
// SPDX-License-Identifier: Apache-2.0

use crate::param_str::format::{ParseError, Segment, StringFormat};

#[derive(Debug, PartialEq)]
struct State<B, L> {
    transition: fn(b: &mut B, l: L) -> Result<Self, ParseError>,
}

type ParseResult<B, L> = Result<State<B, L>, ParseError>;

type Transition<B, L> = fn(b: &mut B, l: L) -> ParseResult<B, L>;

impl<B, L> State<B, L> {
    fn next(&self, b: &mut B, l: L) -> ParseResult<B, L> {
        let transition = self.transition;
        transition(b, l)
    }
}

impl<B, L> Clone for State<B, L> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<B, L> Copy for State<B, L> {}

const fn state<B, L>(transition: Transition<B, L>) -> State<B, L> {
    State { transition }
}

type FormatState = State<BuildFormat, char>;
type FormatResult = ParseResult<BuildFormat, char>;

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

#[derive(Debug, PartialEq)]
struct FormatMachine<B, L> {
    current_state: State<B, L>,
}

impl<B, L> FormatMachine<B, L> {
    #[inline(always)]
    fn current_state(&self) -> State<B, L> {
        self.current_state
    }

    #[inline(always)]
    fn set_state(&mut self, state: State<B, L>) {
        self.current_state = state;
    }
}

pub fn parse(format: &str) -> Result<StringFormat, ParseError> {
    let mut m = FormatMachine {
        current_state: TEXT_STATE,
    };
    let mut b = BuildFormat {
        current_text: String::new(),
        result: Vec::new(),
    };

    for c in format.chars() {
        let current_state = m.current_state();
        let next_state = current_state.next(&mut b, c)?;
        m.set_state(next_state)
    }

    if m.current_state() == TEXT_STATE {
        b.add_text_segment();
        return Ok(StringFormat::new(b.result()));
    }

    Err(ParseError::InvalidFormat(
        "Unexpected end of format".to_string(),
    ))
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
            assert_eq!(parse(""), Ok(StringFormat::new(vec![])));
        }

        #[test]
        fn text() {
            assert_eq!(
                parse("aaa"),
                Ok(StringFormat::new(vec![Segment::Text("aaa".to_string())]))
            );
        }

        #[test]
        fn text_with_escape() {
            assert_eq!(
                parse("aaa\\\\bbb\\{}ccc"),
                Ok(StringFormat::new(vec![Segment::Text(
                    "aaa\\bbb{}ccc".to_string()
                )]))
            );
        }

        #[test]
        fn text_with_escape_at_end() {
            assert_eq!(
                parse("aaa\\\\"),
                Ok(StringFormat::new(vec![Segment::Text("aaa\\".to_string())]))
            );
        }
    }
}
