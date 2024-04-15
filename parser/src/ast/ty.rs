use nom::bytes::complete::tag;
use nom::combinator::{map, value};
use nom::error::context;

use super::node::Node;
use super::{Describe, IResult, Input, MetaNode};

/// Built-in type: `u32`
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BasicType {
    U32,
}

impl BasicType {
    fn parser(input: Input<'_>) -> IResult<Self> {
        context("BasicType", value(BasicType::U32, tag("u32")))(input)
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

impl<'a> Node<'a> for Type {
    fn parser(input: Input<'a>) -> IResult<Self> {
        context("Type", map(BasicType::parser, Type::Basic))(input)
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

pub type TypeNode<M> = MetaNode<Type, M>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_type() {
        let (tail, parsed) = BasicType::parser("u32".into()).unwrap();
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
