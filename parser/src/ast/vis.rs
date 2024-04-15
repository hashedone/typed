use nom::bytes::complete::tag;
use nom::character::complete::one_of;
use nom::combinator::{map, opt, peek, value};
use nom::error::context;
use nom::sequence::terminated;

use super::node::Node;
use super::{Describe, IResult, Input, MetaNode};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Visibility {
    /// Default visibility
    Private,
    /// Public visibility (`pub `) - have to be followed with at least one whitespace
    Public,
}

impl<'a> Node<'a> for Visibility {
    fn parser(input: Input<'a>) -> IResult<Self> {
        let pub_ = value(
            Visibility::Public,
            terminated(tag("pub"), peek(one_of(" \t\r\n"))),
        );
        let vis = map(opt(pub_), |v| v.unwrap_or(Visibility::Private));

        context("Visibility", vis)(input)
    }
}

impl<W> Describe<W> for Visibility
where
    W: std::io::Write,
{
    fn describe(&self, f: &mut W) -> std::io::Result<()> {
        match self {
            Self::Private => write!(f, "PRIV"),
            Self::Public => write!(f, "PUB"),
        }
    }

    fn subnodes(&self) -> Vec<&dyn Describe<W>> {
        vec![]
    }
}

pub type VisibilityNode<M> = MetaNode<Visibility, M>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn visibility() {
        let (tail, parsed) = VisibilityNode::parse("").unwrap();
        assert_eq!(*tail.fragment(), "");
        assert_eq!(parsed, Visibility::Private.into());

        let (tail, parsed) = VisibilityNode::parse("pub ").unwrap();
        assert_eq!(*tail.fragment(), " ");
        assert_eq!(parsed, Visibility::Public.into());
    }
}
