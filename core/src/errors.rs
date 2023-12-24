// Copyright 2023 The Milton Hirsch Institute, B.V.
// SPDX-License-Identifier: Apache-2.0

use std::error::Error;

#[derive(Debug, Clone, PartialEq)]
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

impl Error for CreateError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum DeleteError {
    NoSuchObject,
}

impl std::fmt::Display for DeleteError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            DeleteError::NoSuchObject => write!(f, "No such object"),
        }
    }
}

impl Error for DeleteError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AllocateError {
    DuplicateName,
    NoSpaceAvailable,
}

impl std::fmt::Display for AllocateError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            AllocateError::DuplicateName => write!(f, "Duplicate name"),
            AllocateError::NoSpaceAvailable => write!(f, "No space available"),
        }
    }
}

impl Error for AllocateError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum RenameError {
    DuplicateName,
    NoSuchObject,
}

impl std::fmt::Display for RenameError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            RenameError::DuplicateName => write!(f, "Duplicate name"),
            RenameError::NoSuchObject => write!(f, "No such object"),
        }
    }
}
impl Error for RenameError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

#[cfg(test)]
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
    fn display_delete_error_no_such_object() {
        assert_eq!(format!("{}", DeleteError::NoSuchObject), "No such object");
    }
}
