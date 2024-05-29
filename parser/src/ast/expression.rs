use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{char as ch_, digit1, multispace0};
use nom::combinator::{map, opt, recognize, value};
use nom::error::context;
use nom::sequence::{terminated, tuple};

use super::spanned::{spanned, Spanned};
use super::{mape, noerr, Describe, IResult, Input};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExpressionLiteral<'a> {
    /// Integral literal of unknown type, eg: `0`, `1357`, `-135234`
    Integral(&'a str),
    /// `u32` ilteral
    /// Either `Self::Integral` with type elided to `u32`, or integral suffixed with `u32`, eg.
    /// `1357u32`.
    U32(u32),
    /// Unit type literal
    Unit,
}

impl<'a> ExpressionLiteral<'a> {
    fn format(&self, f: &mut impl std::io::Write) -> std::io::Result<()> {
        match self {
            Self::Integral(lit) => write!(f, "LIT INT {}", lit),
            Self::U32(lit) => write!(f, "LIT U32 {}", lit),
            Self::Unit => write!(f, "LIT ()"),
        }
    }
}

fn expression_literal(input: Input) -> IResult<'_, ExpressionLiteral<'_>> {
    let u32_ = context(
        "U32 literal",
        noerr(map(terminated(digit1, tag("u32")), |lit: Input| {
            ExpressionLiteral::U32(lit.parse().unwrap())
        })),
    );

    let sign = opt(ch_('-'));
    let signed = tuple((sign, multispace0, digit1));
    let signed = context("Integral literal", signed);

    let integral_untyped = noerr(map(recognize(signed), |lit: Input| {
        ExpressionLiteral::Integral(lit.fragment())
    }));

    let unit = context(
        "Unit literal",
        noerr(value(
            ExpressionLiteral::Unit,
            nom::sequence::delimited(ch_('('), multispace0, ch_(')')),
        )),
    );

    context("Literal", alt((u32_, integral_untyped, unit)))(input)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Expression<'a> {
    Literal(ExpressionLiteral<'a>),
}

pub type ExprNode<'a> = Spanned<Expression<'a>>;

pub fn expression(input: Input) -> IResult<ExprNode> {
    context(
        "Expression",
        spanned(mape(expression_literal, Expression::Literal)),
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
        let (tail, (parsed, err)) = expression("0".into()).unwrap();
        assert_eq!(*tail.fragment(), "");
        assert_eq!(
            parsed,
            Expression::Literal(ExpressionLiteral::Integral("0")).into()
        );
        assert_eq!(err, vec![]);

        let (tail, (parsed, err)) = expression("1357".into()).unwrap();
        assert_eq!(*tail.fragment(), "");
        assert_eq!(
            parsed,
            Expression::Literal(ExpressionLiteral::Integral("1357")).into()
        );
        assert_eq!(err, vec![]);

        let (tail, (parsed, err)) = expression("-135234".into()).unwrap();
        assert_eq!(*tail.fragment(), "");
        assert_eq!(
            parsed,
            Expression::Literal(ExpressionLiteral::Integral("-135234")).into()
        );
        assert_eq!(err, vec![]);

        expression("bar".into()).unwrap_err();
    }

    #[test]
    fn parse_expression_literal_unit() {
        let (tail, (parsed, err)) = expression("()".into()).unwrap();
        assert_eq!(*tail.fragment(), "");
        assert_eq!(parsed, Expression::Literal(ExpressionLiteral::Unit).into());
        assert_eq!(err, vec![]);
    }

    #[test]
    fn parse_expression_literal_u32() {
        let (tail, (parsed, err)) = expression("1234u32".into()).unwrap();
        assert_eq!(*tail.fragment(), "");
        assert_eq!(
            parsed,
            Expression::Literal(ExpressionLiteral::U32(1234)).into()
        );
        assert_eq!(err, []);
    }
}
