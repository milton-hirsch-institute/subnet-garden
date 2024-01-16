// Copyright 2024 The Milton Hirsch Institute, B.V.
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

use crate::param_str::parsers::errors::ParseError;
use crate::util::state_machine;
use crate::util::state_machine::{state, state_machine, ParseResult, State, Termination};

#[derive(Debug, PartialEq)]
struct BuildList {
    current_text: String,
    result: Vec<String>,
}

impl BuildList {
    #[inline(always)]
    fn result(self) -> Vec<String> {
        self.result
    }

    #[inline(always)]
    fn add_text(&mut self, c: char) {
        self.current_text.push(c);
    }

    #[inline(always)]
    fn add_list_item(&mut self) -> Result<(), ParseError> {
        if self.current_text.is_empty() {
            return Err(ParseError::InvalidValue("Empty string".to_string()));
        }
        self.result.push(self.current_text.clone());
        self.current_text.clear();
        Ok(())
    }
}

type ListState = State<BuildList, char, ParseError>;
type ListResult = ParseResult<BuildList, char, ParseError>;
type ListTermination = Termination<BuildList, char, ParseError>;
type ListStateMachine = state_machine::StateMachine<BuildList, char, ParseError>;

static TEXT_STATE: ListState = state(|b, c| -> ListResult {
    match c {
        ',' => {
            b.add_list_item()?;
            Ok(DELIMIT_STATE)
        }
        _ => {
            b.add_text(c);
            Ok(TEXT_STATE)
        }
    }
});
static DELIMIT_STATE: ListState = state(|b, c| -> ListResult {
    b.add_text(c);
    Ok(TEXT_STATE)
});

static TERMINATION: ListTermination = |last_state, b| -> Result<(), ParseError> {
    if last_state == TEXT_STATE {
        b.add_list_item()?;
        Ok(())
    } else {
        Err(ParseError::InvalidValue(
            "Unexpected end of list".to_string(),
        ))
    }
};

static LIST_STATE_MACHINE: ListStateMachine = state_machine(TEXT_STATE, TERMINATION);

pub(crate) fn parse_list(list_str: &str) -> Result<Vec<String>, ParseError> {
    let mut b = BuildList {
        current_text: String::new(),
        result: Vec::new(),
    };
    LIST_STATE_MACHINE.run(&mut b, list_str.chars().collect::<Vec<char>>().iter())?;
    Ok(b.result())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_string() {
        assert_eq!(
            parse_list(""),
            Err(ParseError::InvalidValue("Empty string".to_string()))
        );
    }

    #[test]
    fn one_item() {
        assert_eq!(parse_list("aaa"), Ok(vec!["aaa".to_string()]));
    }

    #[test]
    fn multiple_items() {
        assert_eq!(
            parse_list("aaa,bbb,ccc"),
            Ok(vec![
                "aaa".to_string(),
                "bbb".to_string(),
                "ccc".to_string()
            ])
        );
    }

    #[test]
    fn trailing_comma() {
        assert_eq!(
            parse_list("aaa,bbb,"),
            Err(ParseError::InvalidValue(
                "Unexpected end of list".to_string()
            ))
        );
    }
}
