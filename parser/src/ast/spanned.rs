use std::ops::Range;

use derivative::Derivative;
use nom::combinator::map;
use nom::sequence::tuple;
use nom::Parser;
use nom_locate::position;

use super::{Describe, Input};
use crate::error::Error;

pub type Span = Range<usize>;

/// Any node with attached span
///
/// While comparing Spanned nodes spans are ignored
#[derive(Clone, Derivative, Debug)]
#[derivative(PartialEq)]
pub struct Spanned<T> {
    pub node: T,
    #[derivative(PartialEq = "ignore")]
    pub span: Span,
}

impl<T> From<T> for Spanned<T> {
    fn from(node: T) -> Self {
        Self { node, span: 0..0 }
    }
}

pub fn spanned<'a, T>(
    parser: impl Parser<Input<'a>, T, Error>,
) -> impl Parser<Input<'a>, Spanned<T>, Error> {
    map(tuple((position, parser, position)), |(beg, node, end)| {
        Spanned {
            node,
            span: beg.location_offset()..end.location_offset(),
        }
    })
}

impl<T, W> Describe<W> for Spanned<T>
where
    T: Describe<W>,
    W: std::io::Write,
{
    fn describe(&self, f: &mut W) -> std::io::Result<()> {
        self.node.describe(f)?;
        write!(f, " @{}:{}", self.span.start, self.span.end)
    }

    fn subnodes(&self) -> Vec<&dyn Describe<W>> {
        self.node.subnodes()
    }
}
