use std::ops::Range;

pub struct Input<'a> {
    input: &'a str,
    offset: usize,
}

impl<'a> Input<'a> {
    pub fn new(input: &'a str) -> Self {
        Input {
            input,
            offset: 0,
        }
    }
}

/// Single AST node
#[derive(Clone, Debug, PartialEq)]
struct Node<Kind> {
    kind: Kind,
    span: Range<usize>,
    children: usize,
}

pub struct Context<NodeKind, Ctx=()> {
    context: Ctx,
    nodes: Vec<Node<NodeKind>>,
}

impl Context<()> {
    pub fn new() -> Self {
        Context { context: () }
    }

    pub fn context(&self) -> &() {
        &self.context
    }
}

impl<Ctx> Context<Ctx> {
    pub fn with_context(context: Ctx) -> Self {
        Context { context }
    }
}
