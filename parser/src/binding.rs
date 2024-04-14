use nom::bytes::complete::tag;
use nom::character::complete::{char as ch_, multispace0, multispace1};
use nom::combinator::{map, opt};
use nom::error::context;
use nom::sequence::tuple;

use crate::{
    make_node, parse_id, Describe, ExpressionNode, IResult, Input, Meta, TypeNode, VisibilityNode,
};

/// Binging in form of:
/// `[visibility] let name [: type] = expression;`
#[derive(Debug, Clone, PartialEq)]
pub struct Binding<'a, M> {
    visibility: VisibilityNode<M>,
    name: &'a str,
    ty_: Option<TypeNode<M>>,
    expression: ExpressionNode<'a, M>,
}

impl<'a, M> Binding<'a, M>
where
    M: Meta + 'a,
{
    pub fn new(
        name: &'a str,
        visibility: impl Into<VisibilityNode<M>>,
        ty_: impl Into<Option<TypeNode<M>>>,
        expression: impl Into<ExpressionNode<'a, M>>,
    ) -> Self {
        Self {
            name,
            visibility: visibility.into(),
            ty_: ty_.into(),
            expression: expression.into(),
        }
    }

    fn parse(input: impl Into<Input<'a>>) -> IResult<'a, Self> {
        let ty_ = tuple((ch_(':'), multispace0, TypeNode::parse));
        let ty_ = map(ty_, |(_colon, _, ty_)| ty_);

        let tpl = tuple((
            VisibilityNode::parse,
            tag("let"),
            multispace1,
            parse_id,
            multispace0,
            opt(ty_),
            multispace0,
            ch_('='),
            multispace0,
            ExpressionNode::parse,
            ch_(';'),
        ));
        let tpl = context("Binding", tpl);

        map(
            tpl,
            |(visibility, _let, _, name, _, ty_, _, _eq, _, expression, _semi)| Self {
                visibility,
                name: name.fragment(),
                ty_,
                expression,
            },
        )(input.into())
    }
}

impl<'a, M, W> Describe<W> for Binding<'a, M>
where
    M: Meta,
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

make_node!(Binding<'a, M> => BindingNode<'a, M>);

#[cfg(test)]
mod tests {
    use crate::expression::{Expression, ExpressionLiteral};
    use crate::ty::{BasicType, Type};
    use crate::vis::Visibility;

    use super::*;

    #[test]
    fn binding() {
        let (tail, parsed) = BindingNode::parse("let variable = 15;").unwrap();
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

        let (tail, parsed) = BindingNode::parse("pub let other = 10;").unwrap();
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

        let (tail, parsed) = BindingNode::parse("let two: u32 = 2;").unwrap();
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
    }
}
