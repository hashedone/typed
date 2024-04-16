use nom::character::complete::{char as ch_, digit1, multispace0};
use nom::combinator::{map, opt, recognize};
use nom::error::context;
use nom::sequence::tuple;

use super::spanned::{spanned, Spanned};
use super::{Describe, IResult, Input};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExpressionLiteral<'a> {
    /// Integral literal of unknown type, eg: `0`, `1357`, `-135234`
    Integral(&'a str),
}

impl<'a> ExpressionLiteral<'a> {
    fn format(&self, f: &mut impl std::io::Write) -> std::io::Result<()> {
        match self {
            Self::Integral(lit) => write!(f, "LIT INT {}", lit),
        }
    }
}

fn expression_literal(input: Input) -> IResult<'_, ExpressionLiteral<'_>> {
    let sign = opt(ch_('-'));
    let literal = tuple((sign, multispace0, digit1));
    let literal = context("Integral literal", literal);

    map(recognize(literal), |lit: Input| {
        ExpressionLiteral::Integral(lit.fragment())
    })(input)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Expression<'a> {
    Literal(ExpressionLiteral<'a>),
}

pub type ExprNode<'a> = Spanned<Expression<'a>>;

pub fn expression(input: Input) -> IResult<ExprNode> {
    context(
        "Expression",
        spanned(map(expression_literal, Expression::Literal)),
    )(input)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_expression_literal_integral() {
        let (tail, parsed) = expression("0".into()).unwrap();
        assert_eq!(*tail.fragment(), "");
        assert_eq!(
            parsed,
            Expression::Literal(ExpressionLiteral::Integral("0")).into()
        );

        let (tail, parsed) = expression("1357".into()).unwrap();
        assert_eq!(*tail.fragment(), "");
        assert_eq!(
            parsed,
            Expression::Literal(ExpressionLiteral::Integral("1357")).into()
        );

        let (tail, parsed) = expression("-135234".into()).unwrap();
        assert_eq!(*tail.fragment(), "");
        assert_eq!(
            parsed,
            Expression::Literal(ExpressionLiteral::Integral("-135234")).into()
        );

        expression("bar".into()).unwrap_err();
    }
}
