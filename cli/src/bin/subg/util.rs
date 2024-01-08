// Copyright 2024 The Milton Hirsch Institute, B.V.
// SPDX-License-Identifier: Apache-2.0

pub(crate) fn right_pad(s: &mut String, width: usize) {
    while s.len() < width {
        s.push(' ');
    }
}
