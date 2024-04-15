use nom::character::complete::multispace0;
use nom::combinator::{cut, eof, map, not};
use nom::error::context;
use nom::multi::many0;
use nom::sequence::{preceded, terminated};

use super::binding::BindingNode;
use super::node::Node;
use super::{Describe, IResult, Input, Meta, MetaNode};

#[derive(Debug, Clone, PartialEq)]
pub struct Module<'a, M = ()> {
    pub bindings: Vec<BindingNode<'a, M>>,
}

impl<'a, M> Node<'a> for Module<'a, M>
where
    M: Meta + 'a,
{
    fn parser(input: Input<'a>) -> IResult<Self> {
        let binding = preceded(not(eof), cut(BindingNode::parser));
        let binding = preceded(multispace0, binding);
        let bindings = many0(binding);
        let bindings = terminated(bindings, multispace0);

        context("Module", map(bindings, |bindings| Self { bindings }))(input)
    }
}

impl<'a> Module<'a, ()> {
    pub fn meta(self) -> ModuleNode<'a, ()> {
        self.into()
    }
}

impl<'a, M, W> Describe<W> for Module<'a, M>
where
    M: Meta,
    W: std::io::Write,
{
    fn describe(&self, f: &mut W) -> std::io::Result<()> {
        write!(f, "MOD")
    }

    fn subnodes(&self) -> Vec<&dyn Describe<W>> {
        self.bindings
            .iter()
            .map(|b| b as &dyn Describe<W>)
            .collect()
    }
}

pub type ModuleNode<'a, M> = MetaNode<Module<'a, M>, M>;

#[cfg(test)]
mod tests {
    use crate::ast::binding::Binding;
    use crate::ast::expression::{Expression, ExpressionLiteral};
    use crate::ast::vis::Visibility;

    use super::*;

    #[test]
    fn module() {
        let (tail, parsed) = ModuleNode::parse("pub let variable = 15; let other = 10;").unwrap();
        assert_eq!(*tail.fragment(), "");
        assert_eq!(
            parsed,
            Module {
                bindings: vec![
                    Binding {
                        name: "variable",
                        visibility: Visibility::Public.into(),
                        ty_: None,
                        expression: Expression::Literal(ExpressionLiteral::Integral("15")).into(),
                    }
                    .into(),
                    Binding {
                        name: "other",
                        visibility: Visibility::Private.into(),
                        ty_: None,
                        expression: Expression::Literal(ExpressionLiteral::Integral("10")).into(),
                    }
                    .into()
                ]
            }
            .into()
        );
    }
}
