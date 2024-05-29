use nom::character::complete::multispace0;
use nom::combinator::{cut, eof, map, not};
use nom::error::context;
use nom::multi::many0;
use nom::sequence::{preceded, terminated};
use nom::Err;

use crate::error::Error;

use super::binding::{binding, BindingNode};
use super::recover::recover;
use super::spanned::{spanned, Spanned};
use super::{mape, Describe, IResult, Input};

#[derive(Debug, Clone, PartialEq)]
pub struct Module<'a> {
    pub bindings: Vec<BindingNode<'a>>,
}

impl<'a, W> Describe<W> for Module<'a>
where
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

pub type ModuleNode<'a> = Spanned<Module<'a>>;

pub fn module(input: Input) -> IResult<ModuleNode> {
    let binding = preceded(not(eof), cut(binding));
    let mut binding = preceded(multispace0, binding);

    let binding_recovered = move |input| match binding(input) {
        Ok((tail, binding)) => Ok((tail, Ok(binding))),
        Err(Err::Failure(err)) => {
            let (tail, span) = recover(input)?;
            let err = Error::Parse {
                error: err,
                recovery_point: span.end,
            };
            Ok((tail, Err(err)))
        }
        Err(err) => Err(err),
    };

    let bindings = many0(binding_recovered);
    let bindings = terminated(bindings, multispace0);

    let bindings = map(bindings, |b| {
        let mut bindings = vec![];
        let mut errors = vec![];

        for binding in b {
            match binding {
                Ok((binding, err)) => {
                    bindings.push(binding);
                    errors.extend(err);
                }
                Err(err) => errors.push(err),
            }
        }

        (bindings, errors)
    });

    let module = spanned(mape(bindings, |bindings| Module { bindings }));

    context("Module", module)(input)
}

#[cfg(test)]
mod tests {
    use crate::ast::binding::Binding;
    use crate::ast::expression::{Expression, ExpressionLiteral};
    use crate::ast::vis::Visibility;

    use super::*;

    #[test]
    fn parse_module() {
        let (tail, (parsed, errors)) =
            module("pub let variable = 15; let other = 10;".into()).unwrap();
        assert_eq!(errors, []);
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
