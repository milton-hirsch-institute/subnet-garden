// Copyright 2024 The Milton Hirsch Institute, B.V.
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

use crate::param_str::errors::{ArgumentError, FormatError, FormatStringError, ParseError};
use crate::param_str::parsers::format::{Segment, Segments};
use crate::param_str::parsers::{format, list, range};
use crate::util::iter;

pub type Args<'a> = Vec<&'a str>;

#[derive(Debug, PartialEq)]
pub struct StringFormat {
    segments: Segments,
}

impl StringFormat {
    pub fn new(segments: Segments) -> StringFormat {
        StringFormat { segments }
    }

    pub fn format(&self, args: &Args) -> Result<String, FormatError> {
        let mut result = String::new();
        let mut arg_iter = args.iter();
        for segment in &self.segments {
            match segment {
                Segment::Text(text) => result.push_str(text),
                Segment::Variable(var_format) => match arg_iter.next() {
                    Some(arg) => {
                        let field = match var_format.pad_char() {
                            '0' => format!("{:0>width$}", arg, width = var_format.padding()),
                            ' ' => format!("{:width$}", arg, width = var_format.padding()),
                            _ => panic!("Invalid padding character: {}", var_format.pad_char()),
                        };
                        result.push_str(&field);
                    }
                    None => return Err(FormatError::NotEnoughArguments),
                },
            }
        }
        match arg_iter.next() {
            Some(_) => Err(FormatError::TooManyArguments),
            None => Ok(result),
        }
    }

    fn parse(text: &str) -> Result<Self, ParseError> {
        Ok(StringFormat {
            segments: format::parse(text)?,
        })
    }
}

pub(crate) fn format_strings(format: &str, args: &Args) -> Result<Vec<String>, FormatStringError> {
    let format = StringFormat::parse(format)?;
    let mut parsed_args = Vec::<Vec<String>>::new();
    let mut result = Vec::<String>::new();
    for arg in args.iter().copied() {
        let range_result = range::parse_range(arg);
        match range_result {
            Ok(range) => {
                parsed_args.push(range);
            }
            Err(range_error) => {
                let list_result = list::parse_list(arg);
                match list_result {
                    Ok(list) => {
                        parsed_args.push(list);
                    }
                    Err(list_error) => {
                        return Err(FormatStringError::ArgumentParse(ArgumentError::new(
                            arg.to_string(),
                            range_error,
                            list_error,
                        )));
                    }
                }
            }
        }
    }

    let iterator = iter::ListCombinationIterator::new(parsed_args)
        .ok_or(FormatStringError::MissingArgument)?;
    for params in iterator {
        let next_args = params.iter().map(|s| s.as_str()).collect::<Vec<&str>>();
        result.push(format.format(&next_args)?);
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    mod format {
        use super::*;
        use crate::param_str::parsers::format::VarFormat;

        fn format() -> StringFormat {
            StringFormat::new(vec![
                Segment::Text("aaa".to_string()),
                Segment::Variable(VarFormat::default()),
                Segment::Text("bbb".to_string()),
                Segment::Variable(VarFormat::default()),
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

        #[test]
        fn string_padding() {
            let format = StringFormat::parse("aaa{:5}bbb").unwrap();
            assert_eq!(format.format(&vec!["111"]), Ok("aaa111  bbb".to_string()));
        }

        #[test]
        fn numeric_padding() {
            let format = StringFormat::parse("aaa{:05}bbb").unwrap();
            assert_eq!(format.format(&vec!["111"]), Ok("aaa00111bbb".to_string()));
        }
    }
}
