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

mod tests {
    use super::*;

    #[test]
    fn display_subnet_gardener_error() {
        assert_eq!(
            format!("{}", CreateError::DuplicateObject),
            "Duplicate object"
        );
    }
}
