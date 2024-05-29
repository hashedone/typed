use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{char as ch_, multispace0};
use nom::combinator::value;
use nom::error::context;
use nom::sequence::delimited;

use super::spanned::{spanned, Spanned};
use super::{mape, noerr, Describe, IResult, Input};

/// Built-in type: `u32`
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BasicType {
    U32,
}

impl BasicType {
    fn describe(&self, f: &mut impl std::io::Write) -> std::io::Result<()> {
        match self {
            Self::U32 => write!(f, "BTYPE u32"),
        }
    }
}

fn basic_type(input: Input<'_>) -> IResult<BasicType> {
    context("Basic type", noerr(value(BasicType::U32, tag("u32"))))(input)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Type {
    Basic(BasicType),
    Unit,
}

impl<W> Describe<W> for Type
where
    W: std::io::Write,
{
    fn describe(&self, f: &mut W) -> std::io::Result<()> {
        match self {
            Self::Basic(ty_) => ty_.describe(f),
            Self::Unit => write!(f, "UNIT"),
        }
    }

    fn subnodes(&self) -> Vec<&dyn Describe<W>> {
        vec![]
    }
}

pub type TypeNode = Spanned<Type>;

pub fn type_(input: Input) -> IResult<TypeNode> {
    let unit = context(
        "Unit type",
        noerr(value(
            Type::Unit,
            delimited(ch_('('), multispace0, ch_(')')),
        )),
    );
    let basic = mape(basic_type, Type::Basic);

    context("Type", spanned(alt((unit, basic))))(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_basic_type() {
        let (tail, (parsed, err)) = basic_type("u32".into()).unwrap();
        assert_eq!(*tail.fragment(), "");
        assert_eq!(parsed, BasicType::U32);
        assert_eq!(err, []);
    }

    #[test]
    fn parse_type() {
        let (tail, (parsed, err)) = type_("u32".into()).unwrap();
        assert_eq!(*tail.fragment(), "");
        assert_eq!(parsed, Type::Basic(BasicType::U32).into());
        assert_eq!(err, []);
    }

    #[test]
    fn parse_unit() {
        let (tail, (parsed, err)) = type_("()".into()).unwrap();
        assert_eq!(*tail.fragment(), "");
        assert_eq!(parsed, Type::Unit.into());
        assert_eq!(err, []);
    }
}
