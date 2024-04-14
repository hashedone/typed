use nom::character::complete::multispace0;
use nom::combinator::{cut, eof, map, not};
use nom::error::context;
use nom::multi::many0;
use nom::sequence::{preceded, terminated};
use nom_locate::LocatedSpan;

use super::binding::BindingNode;
use super::{make_node, Describe, IResult, Meta};

#[derive(Debug, Clone, PartialEq)]
pub struct Module<'a, M> {
    bindings: Vec<BindingNode<'a, M>>,
}

impl<'a, M> Module<'a, M>
where
    M: Meta + 'a,
{
    pub fn new(bidnings: impl IntoIterator<Item = BindingNode<'a, M>>) -> Self {
        Self {
            bindings: bidnings.into_iter().collect(),
        }
    }

    fn parse(input: impl Into<LocatedSpan<&'a str>>) -> IResult<'a, Self> {
        let binding = preceded(not(eof), cut(BindingNode::parse));
        let binding = preceded(multispace0, binding);
        let bindings = many0(binding);
        let bindings = terminated(bindings, multispace0);

        context("Module", map(bindings, |bindings| Self { bindings }))(input.into())
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

make_node!(Module<'a, M> => ModuleNode<'a, M>);

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
                    Binding::new(
                        "variable",
                        Visibility::Public,
                        None,
                        Expression::Literal(ExpressionLiteral::Integral("15")),
                    )
                    .into(),
                    Binding::new(
                        "other",
                        Visibility::Private,
                        None,
                        Expression::Literal(ExpressionLiteral::Integral("10")),
                    )
                    .into()
                ]
            }
            .into()
        );
    }
}
