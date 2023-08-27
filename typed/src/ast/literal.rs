use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::combinator::{map, value};
use nom::{character, IResult};

/// Literal value
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Literal {
    /// Unit value, only value of an unit type
    Unit,
    /// Unsigened integral literal
    Unsigned(u64),
}

impl Literal {
    /// Nom parser for a literal
    pub fn parse(input: &str) -> IResult<&str, Self> {
        let unit = value(Literal::Unit, tag("()"));
        let unsigned = map(character::complete::u64, Literal::Unsigned);

        alt((unit, unsigned))(input)
    }
}

