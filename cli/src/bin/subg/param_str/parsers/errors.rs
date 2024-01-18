// Copyright 2024 The Milton Hirsch Institute, B.V.
// SPDX-License-Identifier: Apache-2.0

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_error_display() {
        let err = ParseError::InvalidValue("foo".to_string());
        assert_eq!(format!("{}", err), "InvalidValue: foo");
    }
}
