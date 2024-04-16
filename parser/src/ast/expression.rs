use nom::character::complete::{char as ch_, digit1, multispace0};
use nom::combinator::{map, opt, recognize};
use nom::error::context;
use nom::sequence::tuple;

use super::node::Node;
use super::{Describe, IResult, Input, MetaNode};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExpressionLiteral<'a> {
    /// Integral literal of unknown type, eg: `0`, `1357`, `-135234`
    Integral(&'a str),
}

impl<'a> ExpressionLiteral<'a> {
    fn parse(input: Input<'a>) -> IResult<'a, Self> {
        let sign = opt(ch_('-'));
        let literal = tuple((sign, multispace0, digit1));
        let literal = context("Integral literal", literal);

        map(recognize(literal), |lit: Input<'a>| {
            ExpressionLiteral::Integral(lit.fragment())
        })(input)
    }

    fn format(&self, f: &mut impl std::io::Write) -> std::io::Result<()> {
        match self {
            Self::Integral(lit) => write!(f, "LIT INT {}", lit),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Expression<'a> {
    Literal(ExpressionLiteral<'a>),
}

impl<'a> Node<'a> for Expression<'a> {
    fn parser(input: Input<'a>) -> IResult<Self> {
        context(
            "Expression",
            map(ExpressionLiteral::parse, Expression::Literal),
        )(input)
    }
}

impl<'a, W> Describe<W> for Expression<'a>
where
    W: std::io::Write,
{
    fn describe(&self, f: &mut W) -> std::io::Result<()> {
        match self {
            Self::Literal(lit) => {
                write!(f, "EXPR ")?;
                lit.format(f)
            }
        }
    }

    fn subnodes(&self) -> Vec<&dyn Describe<W>> {
        match &self {
            Self::Literal(_) => vec![],
        }
    }
}

pub type ExprNode<'a, M> = MetaNode<Expression<'a>, M>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expression_literal_integral() {
        let (tail, parsed) = ExprNode::parse("0").unwrap();
        assert_eq!(*tail.fragment(), "");
        assert_eq!(
            parsed,
            Expression::Literal(ExpressionLiteral::Integral("0")).into()
        );

        let (tail, parsed) = ExprNode::parse("1357").unwrap();
        assert_eq!(*tail.fragment(), "");
        assert_eq!(
            parsed,
            Expression::Literal(ExpressionLiteral::Integral("1357")).into()
        );

        let (tail, parsed) = ExprNode::parse("-135234").unwrap();
        assert_eq!(*tail.fragment(), "");
        assert_eq!(
            parsed,
            Expression::Literal(ExpressionLiteral::Integral("-135234")).into()
        );

        Expression::parse("bar").unwrap_err();
    }
}
