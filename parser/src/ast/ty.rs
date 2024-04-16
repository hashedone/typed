use nom::bytes::complete::tag;
use nom::combinator::{map, value};
use nom::error::context;

use super::spanned::{spanned, Spanned};
use super::{Describe, IResult, Input};

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
    context("BasicType", value(BasicType::U32, tag("u32")))(input)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Type {
    Basic(BasicType),
}

impl<W> Describe<W> for Type
where
    W: std::io::Write,
{
    fn describe(&self, f: &mut W) -> std::io::Result<()> {
        match self {
            Self::Basic(ty_) => ty_.describe(f),
        }
    }

    fn subnodes(&self) -> Vec<&dyn Describe<W>> {
        vec![]
    }
}

pub type TypeNode = Spanned<Type>;

pub fn type_(input: Input) -> IResult<TypeNode> {
    context("Type", spanned(map(basic_type, Type::Basic)))(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_basic_type() {
        let (tail, parsed) = basic_type("u32".into()).unwrap();
        assert_eq!(*tail.fragment(), "");
        assert_eq!(parsed, BasicType::U32);
    }

    #[test]
    fn parse_type() {
        let (tail, parsed) = type_("u32".into()).unwrap();
        assert_eq!(*tail.fragment(), "");
        assert_eq!(parsed, Type::Basic(BasicType::U32).into());
    }
}
