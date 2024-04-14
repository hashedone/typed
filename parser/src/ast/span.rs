use nom::{sequence::tuple, Parser};
use nom_locate::position;

use super::{Error, Input, Meta};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Span {
    offset: usize,
    len: usize,
}

impl Meta for Span {
    fn parser<'a, O: 'a>(
        p: impl Parser<Input<'a>, O, Error<'a>>,
    ) -> impl Parser<Input<'a>, (Self, O), Error<'a>> {
        tuple((position, p, position)).map(|(beg, output, end)| {
            let span = Span {
                offset: beg.location_offset(),
                len: end.location_offset() - beg.location_offset(),
            };

            (span, output)
        })
    }

    fn describe(&self, f: &mut impl std::io::Write) -> std::io::Result<()> {
        write!(f, "[{}:{}] ", self.offset, self.offset + self.len)
    }
}
