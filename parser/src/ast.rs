use nom::bytes::complete::take_while1;
use nom::character::complete::anychar;
use nom::combinator::{all_consuming, complete, map, recognize, verify};
use nom::sequence::tuple;
use nom::{Finish, Parser};
use nom_locate::LocatedSpan;

pub mod binding;
pub mod expression;
pub mod module;
pub mod recover;
pub mod spanned;
pub mod tuple;
pub mod ty;
pub mod vis;

pub use module::{module, ModuleNode};
pub use ty::TypeNode;

use crate::error::{Error, ParseError};

pub(crate) type Input<'a> = LocatedSpan<&'a str>;
type IResult<'a, T> = nom::IResult<Input<'a>, (T, Vec<Error>), ParseError>;

#[derive(Debug, Clone, PartialEq)]
pub struct Ast<'a> {
    pub module: Option<ModuleNode<'a>>,
    pub errors: Vec<Error>,
}

impl<'a> Ast<'a> {
    pub fn parse(input: &'a str) -> Self {
        let input = LocatedSpan::new(input);
        let res = all_consuming(complete(module))(input).finish();

        match res {
            Ok((_, (module, errors))) => Self {
                module: Some(module),
                errors,
            },
            Err(err) => Self {
                module: None,
                errors: vec![Error::Parse {
                    error: err,
                    recovery_point: input.len(),
                }],
            },
        }
    }

    pub fn format<W>(&self, f: &mut W) -> std::io::Result<()>
    where
        W: std::io::Write,
        ModuleNode<'a>: Describe<W>,
    {
        let mut stack: Vec<_> = self
            .module
            .iter()
            .map(|m| (0, m as &dyn Describe<W>))
            .collect();

        while let Some((ind, node)) = stack.pop() {
            write!(f, "{:ind$}", "", ind = ind)?;
            node.describe(f)?;
            writeln!(f)?;
            stack.extend(
                node.subnodes()
                    .into_iter()
                    .rev()
                    .map(|node| (ind + 1, node)),
            )
        }

        Ok(())
    }
}

pub trait Describe<W> {
    fn describe(&self, f: &mut W) -> std::io::Result<()>;
    fn subnodes(&self) -> Vec<&dyn Describe<W>>;
}

fn parse_id(input: Input) -> IResult<Input> {
    let head = verify(anychar, |c| c.is_alphabetic() || *c == '_');
    let tail = take_while1(|c: char| c.is_alphanumeric() || c == '_');
    map(recognize(tuple((head, tail))), |id| (id, vec![]))(input)
}

fn noerr<'a, T>(
    parser: impl Parser<Input<'a>, T, ParseError>,
) -> impl Parser<Input<'a>, (T, Vec<Error>), ParseError> {
    map(parser, |t| (t, vec![]))
}

fn mape<'a, T, U>(
    parser: impl Parser<Input<'a>, (T, Vec<Error>), ParseError>,
    mut f: impl FnMut(T) -> U,
) -> impl Parser<Input<'a>, (U, Vec<Error>), ParseError> {
    map(parser, move |(t, e)| (f(t), e))
}

#[cfg(test)]
mod tests {
    use super::binding::Binding;
    use super::expression::{Expression, ExpressionLiteral};
    use super::module::Module;
    use super::vis::Visibility;

    use super::*;

    #[test]
    fn ast() {
        let parsed = Ast::parse("pub let variable = 15; let other = 10;");
        assert_eq!(
            parsed,
            Ast {
                module: Some(
                    Module {
                        bindings: vec![
                            Binding {
                                name: "variable",
                                visibility: Visibility::Public.into(),
                                ty_: None,
                                expression: Expression::Literal(ExpressionLiteral::Integral("15"))
                                    .into(),
                            }
                            .into(),
                            Binding {
                                name: "other",
                                visibility: Visibility::Private.into(),
                                ty_: None,
                                expression: Expression::Literal(ExpressionLiteral::Integral("10"))
                                    .into(),
                            }
                            .into()
                        ]
                    }
                    .into()
                ),
                errors: vec![],
            }
        );
    }
}
