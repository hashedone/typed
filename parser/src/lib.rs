use chumsky::prelude::*;

mod binding;
mod expr;
mod literal;
mod module;
mod ty;
mod vis;

use std::ops::Range;

use self::module::Module;

pub type Error<'s> = Rich<'s, char>;
type Ex<'s> = extra::Err<Error<'s>>;

/// Parse entry point
pub fn parse(input: &str) -> ParseResult<Module, Rich<char>> {
    module::module().parse(input)
}

/// Spanned node
#[derive(Clone, Debug, PartialEq)]
struct Spanned<T> {
    node: T,
    span: Range<usize>,
}

/// Parses spanned node
fn spanned<'s, T>(
    parser: impl Parser<'s, &'s str, T, Ex<'s>> + Clone,
) -> impl Parser<'s, &'s str, Spanned<T>, Ex<'s>> + Clone {
    parser.map_with(|node, extra| Spanned {
        node,
        span: extra.span().start()..extra.span().end(),
    })
}
