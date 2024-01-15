// Copyright 2024 The Milton Hirsch Institute, B.V.
// SPDX-License-Identifier: Apache-2.0

pub(crate) type Segments = Vec<Segment>;
pub(crate) type Args<'a> = Vec<&'a str>;

#[derive(Debug, PartialEq)]
pub(crate) enum FormatError {
    NotEnoughArguments,
    TooManyArguments,
}

#[derive(Debug, PartialEq)]
pub(crate) enum Segment {
    Text(String),
    Variable,
}
#[derive(Debug)]
pub(crate) struct StringFormat {
    segments: Segments,
}

impl StringFormat {
    pub(crate) fn new(segments: Segments) -> StringFormat {
        StringFormat { segments }
    }

    pub(crate) fn format(&self, args: &Args) -> Result<String, FormatError> {
        let mut result = String::new();
        let mut arg_iter = args.iter();
        for segment in &self.segments {
            match segment {
                Segment::Text(text) => result.push_str(text),
                Segment::Variable => match arg_iter.next() {
                    Some(arg) => result.push_str(arg),
                    None => return Err(FormatError::NotEnoughArguments),
                },
            }
        }
        match arg_iter.next() {
            Some(_) => Err(FormatError::TooManyArguments),
            None => Ok(result),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod format {
        use super::*;

        fn format() -> StringFormat {
            StringFormat::new(vec![
                Segment::Text("aaa".to_string()),
                Segment::Variable,
                Segment::Text("bbb".to_string()),
                Segment::Variable,
                Segment::Text("ccc".to_string()),
            ])
        }

        #[test]
        fn not_enough_arguments() {
            let format = format();
            assert_eq!(
                format.format(&vec!["111"]),
                Err(FormatError::NotEnoughArguments)
            );
        }

        #[test]
        fn too_many_arguments() {
            let format = format();
            assert_eq!(
                format.format(&vec!["111", "222", "333"]),
                Err(FormatError::TooManyArguments)
            );
        }
        #[test]
        fn success() {
            let format = format();
            assert_eq!(
                format.format(&vec!["111", "222"]),
                Ok("aaa111bbb222ccc".to_string())
            );
        }
    }
}
