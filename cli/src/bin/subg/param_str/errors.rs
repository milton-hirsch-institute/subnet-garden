// Copyright 2024 The Milton Hirsch Institute, B.V.
// SPDX-License-Identifier: Apache-2.0

use crate::param_str::format::ArgumentError;
use std::error::Error;
use std::fmt::Display;

#[derive(Debug, PartialEq)]
pub(crate) enum ParseError {
    InvalidValue(String),
}

impl Error for ParseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::InvalidValue(message) => write!(f, "InvalidValue: {}", message),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum FormatError {
    NotEnoughArguments,
    TooManyArguments,
}

#[derive(Debug, PartialEq)]
pub enum FormatStringError {
    Parse(ParseError),
    Format(FormatError),
    ArgumentParse(ArgumentError),
    MissingArgument,
}

impl From<ParseError> for FormatStringError {
    fn from(error: ParseError) -> Self {
        FormatStringError::Parse(error)
    }
}

impl From<FormatError> for FormatStringError {
    fn from(error: FormatError) -> Self {
        FormatStringError::Format(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod parse_error {
        use super::*;

        #[test]
        fn parse_error_display() {
            let err = ParseError::InvalidValue("foo".to_string());
            assert_eq!(format!("{}", err), "InvalidValue: foo");
        }

        #[test]
        fn parse_error_source() {
            let err = ParseError::InvalidValue("foo".to_string());
            assert!(err.source().is_none());
        }
    }
}
