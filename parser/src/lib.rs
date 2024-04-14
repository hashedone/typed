use nom::bytes::complete::take_while1;
use nom::character::complete::anychar;
use nom::combinator::{all_consuming, complete, map, recognize, verify};
use nom::error::VerboseError;
use nom::sequence::tuple;
use nom::{Finish, Parser};
use nom_locate::LocatedSpan;

pub mod binding;
pub mod expression;
pub mod module;
pub mod node;
pub mod span;
pub mod ty;
pub mod vis;

use binding::BindingNode;
use expression::ExpressionNode;
use module::ModuleNode;
use node::Node;
use span::Span;
use ty::TypeNode;
use vis::VisibilityNode;

macro_rules! make_node {
    ($type:ident<$($lt:lifetime,)* $($gen:ident),*> => $node:ident<$input_lt:lifetime, $meta:ident>) => {
        pub type $node<$($lt,)* $($gen),*> = crate::Node<$type<$($lt,)* $($gen),*>, $meta>;

        impl<$($lt,)* $($gen),*> $node<$($lt,)* $($gen),*>
        where
            $meta: crate::Meta + $input_lt,
        {
            pub fn parse(input: impl Into<crate::Input<$input_lt>>) -> IResult<$input_lt, Self> {
                use nom::Parser;
                crate::Node::parser($type::parse).parse(input.into())
            }
        }
    };
    ($type:ident<$($lt:lifetime),*> => $node:ident<$input_lt:lifetime>) => {
        pub type $node<$($lt,)* M> = crate::Node<$type<$($lt),*>, M>;

        impl<$($lt,)* M> $node<$($lt,)* M>
        where
            M: crate::Meta + $input_lt,
        {
            pub fn parse(input: impl Into<crate::Input<$input_lt>>) -> IResult<$input_lt, Self> {
                use nom::Parser;
                crate::Node::parser($type::parse).parse(input.into())
            }
        }
    };
    ($type:ident => $node:ident) => {
        pub type $node<M> = crate::Node<$type, M>;

        impl<M> $node<M>
        where
            M: crate::Meta,
        {
            pub fn parse<'a>(input: impl Into<Input<'a>>) -> IResult<'a, Self> where M: 'a {
                use nom::Parser;
                crate::Node::parser($type::parse).parse(input.into())
            }
        }
    };
}

pub(crate) use make_node;

type Input<'a> = LocatedSpan<&'a str>;
type Error<'a> = VerboseError<Input<'a>>;
type IResult<'a, T> = nom::IResult<Input<'a>, T, Error<'a>>;

#[derive(Debug, Clone, PartialEq)]
pub struct AST<'a, M = ()> {
    module: ModuleNode<'a, M>,
}

pub type RawAST<'a> = AST<'a, ()>;
pub type SpannedAST<'a> = AST<'a, Span>;

impl<'a, M> AST<'a, M>
where
    M: Meta + 'a,
{
    pub fn parse(input: &'a str) -> Result<Self, Error> {
        let input = LocatedSpan::new(input);
        all_consuming(complete(ModuleNode::parse))(input)
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
        p: impl Parser<Input<'a>, O, Error<'a>>,
    ) -> impl Parser<Input<'a>, (Self, O), Error<'a>>
    where
        O: 'a,
        Self: 'a;

    fn describe(&self, f: &mut impl std::io::Write) -> std::io::Result<()>;
}

impl Meta for () {
    fn parser<'a, O: 'a>(
        p: impl Parser<Input<'a>, O, Error<'a>>,
    ) -> impl Parser<Input<'a>, (Self, O), Error<'a>> {
        map(p, |o| ((), o))
    }

    fn describe(&self, _f: &mut impl std::io::Write) -> std::io::Result<()> {
        Ok(())
    }
}

fn parse_id(input: Input) -> IResult<Input> {
    let head = verify(anychar, |c| c.is_alphabetic() || *c == '_');
    let tail = take_while1(|c: char| c.is_alphanumeric() || c == '_');
    recognize(tuple((head, tail)))(input)
}

#[cfg(test)]
mod tests {
    use crate::{
        binding::Binding,
        expression::{Expression, ExpressionLiteral},
        module::Module,
        vis::Visibility,
    };

    use super::*;

    #[test]
    fn ast() {
        let parsed = AST::parse("pub let variable = 15; let other = 10;").unwrap();
        assert_eq!(
            parsed,
            AST {
                module: Module::new([
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
                ])
                .into()
            }
        );
    }
}
