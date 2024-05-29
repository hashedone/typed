use nom::character::complete::{char as ch_, multispace0, none_of};
use nom::combinator::map;
use nom::multi::many0_count;
use nom::sequence::tuple;
use nom_locate::{position, LocatedSpan};

use super::spanned::Span;
use super::{Input, ParseError};

// Parser that tries to reach the next point from whitch it is worth to recover parsing for next
// tokens. It returns the `Span` that is skipped part up until the recovery point.
//
// It is intended to be used after the parsing failure, when we know what other entities we are
// looking for after recovery.
pub fn recover<'a>(input: Input<'a>) -> nom::IResult<Input<'a>, Span, ParseError> {
    map(
        tuple((
            multispace0,
            position,
            many0_count(none_of(";")),
            position,
            multispace0,
            ch_(';'),
        )),
        |(_, beg, _, end, _, _): (_, LocatedSpan<&'a str>, _, LocatedSpan<&'a str>, _, _)| {
            beg.location_offset()..end.location_offset()
        },
    )(input)
}
