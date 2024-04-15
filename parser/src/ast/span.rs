use std::ops::Range;

use nom::sequence::tuple;
use nom::Parser;
use nom_locate::position;

use super::{Error, Input, Meta};

#[derive(Debug, Clone, PartialEq)]
pub struct Span(Range<usize>);

impl Meta for Span {
    fn parser<'a, O: 'a>(
        p: impl Parser<Input<'a>, O, Error<'a>>,
    ) -> impl Parser<Input<'a>, (Self, O), Error<'a>> {
        tuple((position, p, position))
            .map(|(beg, output, end)| (Span(beg.location_offset()..end.location_offset()), output))
    }

    fn describe(&self, f: &mut impl std::io::Write) -> std::io::Result<()> {
        write!(f, "[@{}:{}] ", self.0.start, self.0.end)
    }
}
