// Copyright 2024 The Milton Hirsch Institute, B.V.
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

use crate::param_str::errors::ParseError;
use crate::param_str::parsers::format::{Segment, Segments};
use crate::param_str::parsers::{format, list, range};
use crate::util::iter;

pub type Args<'a> = Vec<&'a str>;

#[derive(Debug, PartialEq)]
pub enum FormatError {
    NotEnoughArguments,
    TooManyArguments,
}

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

    fn parse(text: &str) -> Result<Self, ParseError> {
        Ok(StringFormat {
            segments: format::parse(text)?,
        })
    }
}

#[derive(Debug, PartialEq)]
pub(crate) struct ArgumentError {
    arg: String,
    range_error: ParseError,
    list_error: ParseError,
}

impl ArgumentError {
    #[inline(always)]
    pub(crate) fn arg(&self) -> &str {
        &self.arg
    }

    #[inline(always)]
    pub(crate) fn range_error(&self) -> &ParseError {
        &self.range_error
    }

    #[inline(always)]
    pub(crate) fn list_error(&self) -> &ParseError {
        &self.list_error
    }
}

#[derive(Debug, PartialEq)]
enum FormatStringError {
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

fn format_strings(format: &str, args: &Args) -> Result<Vec<String>, FormatStringError> {
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
                        return Err(FormatStringError::ArgumentParse(ArgumentError {
                            arg: arg.to_string(),
                            range_error,
                            list_error,
                        }));
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

    mod parse {
        use super::*;

        #[test]
        fn invalid_format() {
            assert_eq!(
                StringFormat::parse("aaa\\"),
                Err(ParseError::InvalidValue(
                    "Unexpected end of format".to_string()
                ))
            );
        }

        #[test]
        fn success() {
            assert_eq!(
                StringFormat::parse("aaa\\{}bbb{}ccc"),
                Ok(StringFormat::new(vec![
                    Segment::Text("aaa{}bbb".to_string()),
                    Segment::Variable,
                    Segment::Text("ccc".to_string())
                ]))
            );
        }
    }
}
