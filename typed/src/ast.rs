use crate::ast::binding::Binding;

use self::expression::Expression;
use self::literal::Literal;
use anyhow::{Result, ensure};
use nom::Finish;
use nom::character::complete::multispace0;
use nom::combinator::{all_consuming, map};
use nom::multi::separated_list0;
use nom::sequence::{tuple, delimited};

pub mod literal;
pub mod expression;
pub mod binding;
pub mod ident;

/// Program abstract syntax tree
/// 
/// Program is a list of bindings, followed by an optional expression. The result of the program
/// execution is the result of the final expression. If no expression is provided, it is defaulted
/// to `()`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Ast<'a> {
    bindings: Vec<Binding<'a>>,
    expr: Expression,
}

impl<'a> Ast<'a> {
    /// Returns an AST of empty program
    pub fn new() -> Self {
        Ast {
            bindings: Vec::new(),
            expr: Expression::Literal(Literal::Unit)
        }
    }

    /// Parses an AST from a string
    pub fn parse(input: &'a str) -> Result<Self> {
        let bindings = separated_list0(multispace0, Binding::parse);
        let parser = tuple((bindings, multispace0, Expression::parse));
        let parser = map(parser, |(bindings, _, expr)| Ast { bindings, expr });
        let parser = delimited(multispace0, parser, multispace0);
        let mut parser = all_consuming(parser);
        let (tail, ast) = parser(input).map_err(nom::Err::<nom::error::Error<&str>>::to_owned).finish()?;
        ensure!(tail.is_empty(), "Unexpected trailing characters");
        Ok(ast)
    }

    /// Returns an owned version of the AST
    pub fn into_owned(self) -> Ast<'static> {
        Ast {
            bindings: self.bindings.into_iter().map(Binding::into_owned).collect(),
            expr: self.expr,
        }
    }
}