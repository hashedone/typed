use nom::error::ContextError;
use thiserror::Error;

use crate::ast::Input;

/// Parsing failure
#[derive(Error, Debug)]
#[error("Unexpected token")]
pub struct Error<'a> {
    /// Where the token missmatch occured
    pub offset: usize,
    /// The name of last context parser
    pub context: &'a str,
    /// Span of the last context parser
    pub context_offset: usize,
}

impl<'a> nom::error::ParseError<Input<'a>> for Error<'a> {
    fn from_error_kind(input: Input<'a>, _kind: nom::error::ErrorKind) -> Self {
        let offset = input.location_offset();

        Self {
            offset,
            context: "",
            context_offset: offset,
        }
    }

    fn append(_input: Input<'a>, _kind: nom::error::ErrorKind, other: Self) -> Self {
        other
    }
}

impl<'a> ContextError<Input<'a>> for Error<'a> {
    fn add_context(input: Input<'a>, context: &'static str, other: Self) -> Self {
        match other {
            Self {
                offset,
                context: "",
                ..
            } => Self {
                offset,
                context,
                context_offset: input.location_offset(),
            },
            _ => other,
        }
    }
}
