use nom::bytes::complete::take_while1;
use nom::character::complete::anychar;
use nom::combinator::{all_consuming, complete, map, recognize, verify};
use nom::sequence::tuple;
use nom::Finish;
use nom_locate::{position, LocatedSpan};

pub mod binding;
pub mod expression;
pub mod module;
pub mod node;
pub mod ty;
pub mod vis;

pub use module::ModuleNode;
pub use node::MetaNode;
pub use ty::TypeNode;
pub use vis::VisibilityNode;

use crate::error::Error;
use crate::Span;

use self::node::Node;

pub(crate) type Input<'a> = LocatedSpan<&'a str>;
type IResult<'a, T> = nom::IResult<Input<'a>, T, Error<'a>>;

#[derive(Debug, Clone, PartialEq)]
pub struct Ast<'a, M = ()> {
    module: ModuleNode<'a, M>,
}

pub type Raw<'a> = Ast<'a, ()>;
pub type Spanned<'a> = Ast<'a, Span>;

impl<'a, M> Ast<'a, M>
where
    M: Meta + 'a,
{
    pub fn parse(input: &'a str) -> Result<Self, Error> {
        let input = LocatedSpan::new(input);
        all_consuming(complete(ModuleNode::parser))(input)
            .finish()
            .map(|(_, module)| Self { module })
    }

    pub fn format<W>(&self, f: &mut W) -> std::io::Result<()>
    where
        W: std::io::Write,
        ModuleNode<'a, M>: Describe<W>,
    {
        let mut stack = vec![(0, &self.module as &dyn Describe<W>)];
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

pub trait Meta: Sized {
    fn parser<'a, O>(
        p: impl nom::Parser<Input<'a>, O, Error<'a>>,
    ) -> impl nom::Parser<Input<'a>, (Self, O), Error<'a>>
    where
        O: 'a,
        Self: 'a;

    fn describe(&self, f: &mut impl std::io::Write) -> std::io::Result<()>;
}

impl Meta for () {
    fn parser<'a, O: 'a>(
        p: impl nom::Parser<Input<'a>, O, Error<'a>>,
    ) -> impl nom::Parser<Input<'a>, (Self, O), Error<'a>> {
        map(p, |o| ((), o))
    }

    fn describe(&self, _f: &mut impl std::io::Write) -> std::io::Result<()> {
        Ok(())
    }
}

impl Meta for Span {
    fn parser<'a, O: 'a>(
        p: impl nom::Parser<Input<'a>, O, Error<'a>>,
    ) -> impl nom::Parser<Input<'a>, (Self, O), Error<'a>> {
        map(tuple((position, p, position)), |(beg, output, end)| {
            (
                (beg.location_offset()..end.location_offset()).into(),
                output,
            )
        })
    }

    fn describe(&self, f: &mut impl std::io::Write) -> std::io::Result<()> {
        write!(f, "[@{}:{}] ", self.range().start, self.range().end)
    }
}

fn parse_id(input: Input) -> IResult<Input> {
    let head = verify(anychar, |c| c.is_alphabetic() || *c == '_');
    let tail = take_while1(|c: char| c.is_alphanumeric() || c == '_');
    recognize(tuple((head, tail)))(input)
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
        let parsed = Ast::parse("pub let variable = 15; let other = 10;").unwrap();
        assert_eq!(
            parsed,
            Ast {
                module: Module {
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
            }
        );
    }
}
