// Copyright 2024 The Milton Hirsch Institute, B.V.
// SPDX-License-Identifier: Apache-2.0

use std::error::Error;
use std::fmt::Display;

#[derive(Debug, PartialEq)]
pub(crate) enum ParseError {
    InvalidValue(String),
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::InvalidValue(message) => write!(f, "InvalidValue: {}", message),
        }
    }
}

impl Error for ParseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

#[derive(Debug, PartialEq)]
pub enum FormatError {
    NotEnoughArguments,
    TooManyArguments,
}

impl Display for FormatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FormatError::NotEnoughArguments => write!(f, "Not enough arguments"),
            FormatError::TooManyArguments => write!(f, "Too many arguments"),
        }
    }
}

impl Error for FormatError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

#[derive(Debug, PartialEq)]
pub(crate) struct ArgumentError {
    arg: String,
    range_error: ParseError,
    list_error: ParseError,
}

impl ArgumentError {
    pub(crate) fn new(
        arg: String,
        range_error: ParseError,
        list_error: ParseError,
    ) -> ArgumentError {
        ArgumentError {
            arg,
            range_error,
            list_error,
        }
    }
}

impl Display for ArgumentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "arg: {}\nif range: {}\nif list: {}",
            self.arg, self.range_error, self.list_error
        )
    }
}

impl Error for ArgumentError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
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

impl Display for FormatStringError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FormatStringError::Parse(error) => write!(f, "{}", error),
            FormatStringError::Format(error) => write!(f, "{}", error),
            FormatStringError::ArgumentParse(error) => write!(f, "{}", error),
            FormatStringError::MissingArgument => write!(f, "Missing argument"),
        }
    }
}

impl Error for FormatStringError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            FormatStringError::Parse(error) => Some(error),
            FormatStringError::Format(error) => Some(error),
            FormatStringError::ArgumentParse(error) => Some(error),
            FormatStringError::MissingArgument => None,
        }
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
    #[cfg(test)]
    mod format_error {
        use super::*;

        #[test]
        fn not_enough_arguments_display() {
            let err = FormatError::NotEnoughArguments;
            assert_eq!(format!("{}", err), "Not enough arguments");
        }

        #[test]
        fn too_many_arguments_display() {
            let err = FormatError::TooManyArguments;
            assert_eq!(format!("{}", err), "Too many arguments");
        }

        #[test]
        fn format_error_source() {
            let err = FormatError::NotEnoughArguments;
            assert!(err.source().is_none());
        }
    }

    #[cfg(test)]
    mod argument_error {
        use super::*;

        #[test]
        fn argument_error_display() {
            let err = ArgumentError::new(
                "foo".to_string(),
                ParseError::InvalidValue("bar".to_string()),
                ParseError::InvalidValue("baz".to_string()),
            );
            assert_eq!(
                format!("{}", err),
                "arg: foo\nif range: InvalidValue: bar\nif list: InvalidValue: baz"
            );
        }

        #[test]
        fn argument_error_source() {
            let err = ArgumentError::new(
                "foo".to_string(),
                ParseError::InvalidValue("bar".to_string()),
                ParseError::InvalidValue("baz".to_string()),
            );
            assert!(err.source().is_none());
        }
    }

    #[cfg(test)]
    mod format_string_error {
        use super::*;

        #[test]
        fn from_parse_error() {
            let err = ParseError::InvalidValue("foo".to_string());
            assert_eq!(
                FormatStringError::from(err),
                FormatStringError::Parse(ParseError::InvalidValue("foo".to_string()))
            );
        }

        #[test]
        fn from_format_error() {
            let err = FormatError::NotEnoughArguments;
            assert_eq!(
                FormatStringError::from(err),
                FormatStringError::Format(FormatError::NotEnoughArguments)
            );
        }

        #[test]
        fn parse_error_display() {
            let err = FormatStringError::Parse(ParseError::InvalidValue("foo".to_string()));
            assert_eq!(format!("{}", err), "InvalidValue: foo");
        }

        #[test]
        fn format_error_display() {
            let err = FormatStringError::Format(FormatError::NotEnoughArguments);
            assert_eq!(format!("{}", err), "Not enough arguments");
        }

        #[test]
        fn argument_error_display() {
            let err = FormatStringError::ArgumentParse(ArgumentError::new(
                "foo".to_string(),
                ParseError::InvalidValue("bar".to_string()),
                ParseError::InvalidValue("baz".to_string()),
            ));
            assert_eq!(
                format!("{}", err),
                "arg: foo\nif range: InvalidValue: bar\nif list: InvalidValue: baz"
            );
        }

        #[test]
        fn missing_argument_display() {
            let err = FormatStringError::MissingArgument;
            assert_eq!(format!("{}", err), "Missing argument");
        }

        #[test]
        fn parse_error_source() {
            let err = FormatStringError::Parse(ParseError::InvalidValue("foo".to_string()));
            assert_eq!(err.source().unwrap().to_string(), "InvalidValue: foo");
        }

        #[test]
        fn format_error_source() {
            let err = FormatStringError::Format(FormatError::NotEnoughArguments);
            assert_eq!(err.source().unwrap().to_string(), "Not enough arguments");
        }

        #[test]
        fn argument_error_source() {
            let err = FormatStringError::ArgumentParse(ArgumentError::new(
                "foo".to_string(),
                ParseError::InvalidValue("bar".to_string()),
                ParseError::InvalidValue("baz".to_string()),
            ));
            assert_eq!(
                err.source().unwrap().to_string(),
                "arg: foo\nif range: InvalidValue: bar\nif list: InvalidValue: baz"
            );
        }

        #[test]
        fn missing_argument_source() {
            let err = FormatStringError::MissingArgument;
            assert!(err.source().is_none());
        }
    }
}
