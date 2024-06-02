use nom::bytes::complete::tag;
use nom::character::complete::{char as ch_, multispace0, multispace1};
use nom::combinator::{map, opt};
use nom::error::context;
use nom::sequence::tuple;
use nom::Parser;

use super::expression::{expression, ExprNode};
use super::spanned::{spanned, Spanned};
use super::tuple::collect_err;
use super::ty::{type_, TypeNode};
use super::vis::{visibility, Visibility, VisibilityNode};
use super::{parse_id, Describe, IResult, Input};

/// Binging in form of:
/// `[visibility] let name [: type] = expression;`
#[derive(Debug, Clone, PartialEq)]
pub struct Binding<'a> {
    pub visibility: VisibilityNode,
    pub name: &'a str,
    pub ty_: Option<TypeNode>,
    pub expression: ExprNode<'a>,
}

impl<'a, W> Describe<W> for Binding<'a>
where
    W: std::io::Write,
{
    fn describe(&self, f: &mut W) -> std::io::Result<()> {
        write!(f, "BIND {}:", self.name)
    }

    fn subnodes(&self) -> Vec<&dyn Describe<W>> {
        if let Some(ty_) = &self.ty_ {
            vec![&self.visibility, ty_, &self.expression]
        } else {
            vec![&self.visibility, &self.expression]
        }
    }
}

pub type BindingNode<'a> = Spanned<Binding<'a>>;

pub fn binding(input: Input) -> IResult<BindingNode> {
    let ty_ = tuple((ch_(':'), multispace0, type_));
    let ty_ = map(ty_, |(_colon, _, ty_)| ty_);

    let tpl = tuple((
        visibility,
        multispace0,
        tag("let"),
        multispace1,
        parse_id,
        multispace0,
        opt(ty_),
        multispace0,
        ch_('='),
        multispace0,
        expression,
        ch_(';'),
    ));
    let tpl = context("Binding", tpl);

    spanned(map(
        tpl,
        |(visibility, _, _let, _, name, _, ty_, _, _eq, _, expression, _semi)| {
            let (ty_, tyerr) = ty_.unzip();
            let (visibility, name, _, expression, err) = collect_err((
                visibility,
                name,
                ((), tyerr.unwrap_or_default()),
                expression,
            ));

            (
                Binding {
                    visibility,
                    name: name.fragment(),
                    ty_,
                    expression,
                },
                err,
            )
        },
    ))
    .parse(input)
}

#[cfg(test)]
mod tests {
    use crate::ast::expression::{Expression, ExpressionLiteral};
    use crate::ast::ty::{BasicType, Type};
    use crate::ast::vis::Visibility;

    use super::*;

    #[test]
    fn parse_binding() {
        let (tail, (parsed, err)) = binding("let variable = 15;".into()).unwrap();
        assert_eq!(*tail.fragment(), "");
        assert_eq!(
            parsed,
            Binding {
                visibility: Visibility::Private.into(),
                name: "variable",
                ty_: None,
                expression: Expression::Literal(ExpressionLiteral::Integral("15")).into()
            }
            .into()
        );
        assert_eq!(err, []);

        let (tail, (parsed, err)) = binding("pub let other = 10;".into()).unwrap();
        assert_eq!(*tail.fragment(), "");
        assert_eq!(
            parsed,
            Binding {
                visibility: Visibility::Public.into(),
                name: "other",
                ty_: None,
                expression: Expression::Literal(ExpressionLiteral::Integral("10")).into()
            }
            .into()
        );
        assert_eq!(err, []);

        let (tail, (parsed, err)) = binding("let two: u32 = 2;".into()).unwrap();
        assert_eq!(*tail.fragment(), "");
        assert_eq!(
            parsed,
            Binding {
                visibility: Visibility::Private.into(),
                name: "two",
                ty_: Some(Type::Basic(BasicType::U32).into()),
                expression: Expression::Literal(ExpressionLiteral::Integral("2")).into()
            }
            .into()
        );
        assert_eq!(err, []);
    }
}
