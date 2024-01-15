// Copyright 2024 The Milton Hirsch Institute, B.V.
// SPDX-License-Identifier: Apache-2.0

use crate::param_str::format::{ParseError, Segment, StringFormat};

type State = fn(&mut Machine, &mut Building, char) -> Result<(), ParseError>;

static TEXT_STATE: State = |m, b, c| {
    match c {
        '\\' => m.set_state(ESCAPE_STATE),
        '{' => {
            b.add_text_segment();
            m.set_state(VARIABLE_STATE);
        }
        _ => b.add_text(c),
    }
    Ok(())
};

static ESCAPE_STATE: State = |m, b, c| {
    b.add_text(c);
    m.set_state(TEXT_STATE);
    Ok(())
};

static VARIABLE_STATE: State = |m, b, c| {
    match c {
        '}' => {
            b.add_variable();
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
struct Building {
    current_text: String,
    result: Vec<Segment>,
}

impl Building {
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
struct Machine {
    current_state: State,
}

impl Machine {
    #[inline(always)]
    fn current_state(&self) -> State {
        self.current_state
    }

    #[inline(always)]
    fn set_state(&mut self, state: State) {
        self.current_state = state;
    }
}

pub fn parse(format: &str) -> Result<StringFormat, ParseError> {
    let mut m = Machine {
        current_state: TEXT_STATE,
    };
    let mut b = Building {
        current_text: String::new(),
        result: Vec::new(),
    };

    for c in format.chars() {
        m.current_state()(&mut m, &mut b, c)?;
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
