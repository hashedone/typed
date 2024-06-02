use std::ops::Range;

use nom::error::{ContextError, FromExternalError};
use thiserror::Error;

use crate::ast::Input;

/// Parsing failure
#[derive(Error, Debug, Clone, PartialEq)]
pub enum ParseError {
    /// Unexpected token while parsing
    #[error("Unexpected token")]
    Unexpected {
        /// Where the token missmatch occured
        offset: usize,
        /// The name of last context parser
        context: &'static str,
    },
    #[error("Literal out of range")]
    LiteralOutOfRange {
        literal: Range<usize>,
        ty: &'static str,
    },
}

impl<I> FromExternalError<I, ParseError> for ParseError {
    fn from_external_error(_input: I, _kind: nom::error::ErrorKind, e: ParseError) -> Self {
        e
    }
}

impl<'a> nom::error::ParseError<Input<'a>> for ParseError {
    fn from_error_kind(input: Input<'a>, _kind: nom::error::ErrorKind) -> Self {
        let offset = input.location_offset();

        Self::Unexpected {
            offset,
            context: "",
        }
    }

    fn append(_input: Input<'a>, _kind: nom::error::ErrorKind, other: Self) -> Self {
        other
    }

    fn or(self, other: Self) -> Self {
        match (self, other) {
            (Self::Unexpected { offset, .. }, Self::Unexpected { .. }) => Self::Unexpected {
                offset,
                context: "",
            },
            (Self::Unexpected { .. }, err) => err,
            (err, _) => err,
        }
    }
}

impl<'a> ContextError<Input<'a>> for ParseError {
    fn add_context(_input: Input<'a>, context: &'static str, other: Self) -> Self {
        match other {
            Self::Unexpected {
                offset,
                context: "",
                ..
            } => Self::Unexpected { offset, context },
            _ => other,
        }
    }
}

#[derive(Error, Debug, Clone, PartialEq)]
pub enum Error {
    /// Error occured while parsing
    #[error("{error}")]
    Parse {
        error: ParseError,
        recovery_point: usize,
    },
}
