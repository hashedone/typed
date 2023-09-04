use std::fmt::Display;

use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::combinator::{map, value};
use nom::error::ParseError;
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
    pub fn parse<'a, E>(input: &'a str) -> IResult<&'a str, Self, E>
    where
        E: ParseError<&'a str>,
    {
        let unit = value(Literal::Unit, tag("()"));
        let unsigned = map(character::complete::u64, Literal::Unsigned);

        alt((unit, unsigned))(input)
    }
}

impl Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unit => write!(f, "()"),
            Self::Unsigned(val) => write!(f, "{val}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use nom::Finish;

    use super::*;

    type Err<'a> = nom::error::Error<&'a str>;

    #[test]
    fn unit_literal() {
        let (tail, unit) = Literal::parse::<Err>("()").finish().unwrap();
        assert_eq!(tail, "");
        assert_eq!(unit, Literal::Unit);
    }

    #[test]
    fn unsigned_literal() {
        let (tail, unit) = Literal::parse::<Err>("123").finish().unwrap();
        assert_eq!(tail, "");
        assert_eq!(unit, Literal::Unsigned(123));
    }
}
