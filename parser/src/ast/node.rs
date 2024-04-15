use std::ops::{Deref, DerefMut};

use super::Input;
use nom::combinator::map;

use super::{Describe, IResult, Meta};

pub trait Node<'a>: Sized {
    fn parser(input: Input<'a>) -> IResult<Self>;

    fn parse(input: &'a str) -> IResult<Self> {
        Self::parser(input.into())
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct MetaNode<T, M> {
    node: T,
    meta: M,
}

impl<'a, T, M> Node<'a> for MetaNode<T, M>
where
    M: Meta + 'a,
    T: Node<'a> + 'a,
{
    fn parser(input: Input<'a>) -> IResult<Self> {
        map(<M as Meta>::parser(T::parser), |(meta, node)| MetaNode {
            meta,
            node,
        })(input)
    }
}

impl<T, M> Deref for MetaNode<T, M> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.node
    }
}

impl<T, M> DerefMut for MetaNode<T, M> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.node
    }
}

impl<T> From<T> for MetaNode<T, ()> {
    fn from(node: T) -> Self {
        MetaNode { node, meta: () }
    }
}

impl<T, M, W> Describe<W> for MetaNode<T, M>
where
    T: Describe<W>,
    M: Meta,
    W: std::io::Write,
{
    fn describe(&self, f: &mut W) -> std::io::Result<()> {
        self.meta.describe(f)?;
        self.node.describe(f)
    }

    fn subnodes(&self) -> Vec<&dyn Describe<W>> {
        self.node.subnodes()
    }
}
