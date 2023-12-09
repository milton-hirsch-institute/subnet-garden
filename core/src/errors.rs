// Copyright 2023 The Milton Hirsch Institute, B.V.
// SPDX-License-Identifier: Apache-2.0

#[derive(Debug, Clone)]
pub enum CreateError {
    DuplicateObject,
}

impl std::fmt::Display for CreateError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            CreateError::DuplicateObject => write!(f, "Duplicate object"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum RemoveError {
    NoSuchObject,
}

impl std::fmt::Display for RemoveError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            RemoveError::NoSuchObject => write!(f, "No such object"),
        }
    }
}

mod tests {
    use super::*;

    #[test]
    fn display_create_error_duplicate_object() {
        assert_eq!(
            format!("{}", CreateError::DuplicateObject),
            "Duplicate object"
        );
    }

    #[test]
    fn display_remove_error_no_such_object() {
        assert_eq!(format!("{}", RemoveError::NoSuchObject), "No such object");
    }
}
