use nom::bytes::complete::tag;
use nom::character::complete::one_of;
use nom::combinator::{map, opt, peek, value};
use nom::error::context;
use nom::sequence::terminated;

use super::spanned::{spanned, Spanned};
use super::{noerr, Describe, IResult, Input};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Visibility {
    /// Default visibility
    Private,
    /// Public visibility (`pub `) - have to be followed with at least one whitespace
    Public,
}

pub type VisibilityNode = Spanned<Visibility>;

pub fn visibility(input: Input) -> IResult<Spanned<Visibility>> {
    let pub_ = value(
        Visibility::Public,
        terminated(tag("pub"), peek(one_of(" \t\r\n"))),
    );
    let vis = map(opt(pub_), |v| v.unwrap_or(Visibility::Private));

    context("Visibility", spanned(noerr(vis)))(input)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_visibility() {
        let (tail, (parsed, err)) = visibility("".into()).unwrap();

        assert_eq!(*tail.fragment(), "");
        assert_eq!(parsed, Visibility::Private.into());
        assert_eq!(err, []);

        let (tail, (parsed, err)) = visibility("pub ".into()).unwrap();
        assert_eq!(*tail.fragment(), " ");
        assert_eq!(parsed, Visibility::Public.into());
        assert_eq!(err, []);
    }
}
