// Copyright 2024 The Milton Hirsch Institute, B.V.
// SPDX-License-Identifier: Apache-2.0

use crate::format_str::{ParseError, Segment, StringFormat};

type State = fn(&mut Machine, char) -> Result<(), ParseError>;

static TEXT_STATE: State = |m, c| {
    match c {
        '\\' => m.set_state(ESCAPE_STATE),
        '{' => {
            m.add_text_segment();
            m.set_state(VARIABLE_STATE);
        }
        _ => m.add_text(c),
    }
    Ok(())
};

static ESCAPE_STATE: State = |m, c| {
    m.add_text(c);
    m.set_state(TEXT_STATE);
    Ok(())
};

static VARIABLE_STATE: State = |m, c| {
    match c {
        '}' => {
            m.add_variable();
            m.set_state(TEXT_STATE);
        }
        _ => {
            return Err(ParseError::InvalidFormat(
                format!("Expected }}, found {}", c).to_string(),
            ));
        }
    }
    Ok(())
};

#[derive(Debug, PartialEq)]
struct Machine {
    current_state: State,
    current_text: String,
    result: Vec<Segment>,
}

impl Machine {
    #[inline(always)]
    fn current_state(&self) -> State {
        self.current_state
    }

    fn set_state(&mut self, state: State) {
        self.current_state = state;
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

    fn add_variable(&mut self) {
        self.result.push(Segment::Variable);
    }
}

pub(super) fn parse(format: &str) -> Result<StringFormat, ParseError> {
    let mut m = Machine {
        current_state: TEXT_STATE,
        current_text: String::new(),
        result: Vec::new(),
    };

    for c in format.chars() {
        m.current_state()(&mut m, c)?;
    }

    if m.current_state() == TEXT_STATE {
        m.add_text_segment();
        return Ok(StringFormat::new(m.result));
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
