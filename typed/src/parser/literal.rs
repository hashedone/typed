use nom::IResult;
use nom::bytes::complete::tag;
use nom::combinator::value;

/// Literal value
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Literal {
    /// Unit value, only value of an unit type
    Unit,
}

impl Literal {
    /// Nom parser for a literal
    pub fn parse(input: &str) -> IResult<&str, Self> {
        value(Literal::Unit, tag("()"))(input)
    }
}