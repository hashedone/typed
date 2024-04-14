use std::ops::{Deref, DerefMut};

use nom::{combinator::map, Parser};

use crate::{Describe, Error, Input, Meta};

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Node<T, M> {
    node: T,
    meta: M,
}

impl<T, M> Node<T, M>
where
    M: Meta,
{
    pub fn parser<'a>(
        p: impl Parser<Input<'a>, T, Error<'a>>,
    ) -> impl Parser<Input<'a>, Self, Error<'a>>
    where
        M: 'a,
        T: 'a,
    {
        map(<M as Meta>::parser(p), |(meta, node)| Node { meta, node })
    }
}

impl<T, M> Deref for Node<T, M> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.node
    }
}

impl<T, M> DerefMut for Node<T, M> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.node
    }
}

impl<T> From<T> for Node<T, ()> {
    fn from(node: T) -> Self {
        Node { node, meta: () }
    }
}

impl<T, M, W> Describe<W> for Node<T, M>
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
