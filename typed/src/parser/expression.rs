use nom::combinator::map;
use nom::IResult;

use super::literal::Literal;

/// Single expression
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Expression {
    /// Literal value
    Literal(Literal)
}

impl Expression {
    /// Nom parser for an expression
    pub fn parse(input: &str) -> IResult<&str, Self> {
        map(Literal::parse, Expression::Literal)(input)
    }
}