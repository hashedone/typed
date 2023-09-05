use nom::{
    character::complete::multispace0,
    combinator::{all_consuming, map},
    error::ParseError,
    multi::separated_list0,
    sequence::{delimited, tuple},
    IResult,
};

use super::{binding::Binding, expression::Expression};

/// Program abstract syntax tree
///
/// Program is a list of bindings, followed by an optional expression. The result of the program
/// execution is the result of the final expression. If no expression is provided, it is defaulted
/// to `()`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Root<'a> {
    pub bindings: Vec<Binding<'a>>,
    pub expr: Expression<'a>,
}

impl<'a> Root<'a> {
    /// Parses an AST from a string
    pub fn parse<E>(input: &'a str) -> IResult<&'a str, Self, E>
    where
        E: ParseError<&'a str>,
    {
        let bindings = separated_list0(multispace0, Binding::parse);
        let parser = tuple((bindings, multispace0, Expression::parse));
        let parser = map(parser, |(bindings, _, expr)| Root { bindings, expr });
        let parser = delimited(multispace0, parser, multispace0);
        all_consuming(parser)(input)
    }
}
