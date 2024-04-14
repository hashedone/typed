use nom::bytes::complete::tag;
use nom::combinator::{map, value};
use nom::error::context;

use crate::{make_node, Describe, IResult, Input};

/// Built-in type: `u32`
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BasicType {
    U32,
}

impl BasicType {
    fn parse<'a>(input: impl Into<Input<'a>>) -> IResult<'a, Self> {
        context("BasicType", value(BasicType::U32, tag("u32")))(input.into())
    }

    fn describe(&self, f: &mut impl std::io::Write) -> std::io::Result<()> {
        match self {
            Self::U32 => write!(f, "BTYPE u32"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Type {
    Basic(BasicType),
}

impl Type {
    fn parse<'a>(input: impl Into<Input<'a>>) -> IResult<'a, Self> {
        context("Type", map(BasicType::parse, Type::Basic))(input.into())
    }
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

make_node!(Type => TypeNode);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_type() {
        let (tail, parsed) = BasicType::parse("u32").unwrap();
        assert_eq!(*tail.fragment(), "");
        assert_eq!(parsed, BasicType::U32);
    }

    #[test]
    fn type_() {
        let (tail, parsed) = TypeNode::parse("u32").unwrap();
        assert_eq!(*tail.fragment(), "");
        assert_eq!(parsed, Type::Basic(BasicType::U32).into());
    }
}
