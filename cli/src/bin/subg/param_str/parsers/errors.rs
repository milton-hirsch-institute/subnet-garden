// Copyright 2024 The Milton Hirsch Institute, B.V.
// SPDX-License-Identifier: Apache-2.0

#[derive(Debug, PartialEq)]
pub(crate) enum ParseError {
    InvalidValue(String),
}
