// Copyright 2024 The Milton Hirsch Institute, B.V.
// SPDX-License-Identifier: Apache-2.0

pub(crate) fn right_pad(s: &mut String, width: usize) {
    while s.len() < width {
        s.push(' ');
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_right_pad() {
        let mut s = String::from("foo");
        right_pad(&mut s, 5);
        assert_eq!(s, "foo  ");
    }

    #[test]
    fn test_right_pad_too_wide() {
        let mut s = String::from("foo-bar");
        right_pad(&mut s, 5);
        assert_eq!(s, "foo-bar");
    }
}
