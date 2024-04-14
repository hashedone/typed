use nom::bytes::complete::tag;
use nom::character::complete::multispace1;
use nom::combinator::{map, opt, value};
use nom::error::context;
use nom::sequence::terminated;

use super::{make_node, Describe, IResult, Input};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Visibility {
    /// Default visibility
    Private,
    /// Public visibility (`pub `) - have to be followed with at least one whitespace
    Public,
}

impl Visibility {
    fn parse<'a>(input: impl Into<Input<'a>>) -> IResult<'a, Self> {
        let pub_ = value(Visibility::Public, terminated(tag("pub"), multispace1));
        let vis = map(opt(pub_), |v| v.unwrap_or(Visibility::Private));

        context("Visibility", vis)(input.into())
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

make_node!(Visibility => VisibilityNode);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn visibility() {
        let (tail, parsed) = VisibilityNode::parse("").unwrap();
        assert_eq!(*tail.fragment(), "");
        assert_eq!(parsed, Visibility::Private.into());

        let (tail, parsed) = VisibilityNode::parse("pub ").unwrap();
        assert_eq!(*tail.fragment(), "");
        assert_eq!(parsed, Visibility::Public.into());
    }
}
