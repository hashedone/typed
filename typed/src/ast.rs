use std::fmt::Debug;
use std::io::Write;

use crate::ast::binding::Binding;

use self::expression::Expression;
use self::literal::Literal;
use anyhow::{anyhow, ensure, Context, Result};
use nom::character::complete::multispace0;
use nom::combinator::{all_consuming, map};
use nom::error::convert_error;
use nom::multi::separated_list0;
use nom::sequence::{delimited, tuple};
use nom::Finish;

pub mod binding;
pub mod expression;
pub mod fn_appl;
pub mod fn_decl;
pub mod ident;
pub mod literal;

/// Program abstract syntax tree
///
/// Program is a list of bindings, followed by an optional expression. The result of the program
/// execution is the result of the final expression. If no expression is provided, it is defaulted
/// to `()`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Ast<'a> {
    bindings: Vec<Binding<'a>>,
    expr: Expression<'a>,
}

impl<'a> Ast<'a> {
    /// Returns an AST of empty program
    pub fn new() -> Self {
        Ast {
            bindings: Vec::new(),
            expr: Expression::Literal(Literal::Unit),
        }
    }

    /// Parses an AST from a string
    pub fn parse(input: &'a str) -> Result<Self> {
        let bindings = separated_list0(multispace0, Binding::parse);
        let parser = tuple((bindings, multispace0, Expression::parse));
        let parser = map(parser, |(bindings, _, expr)| Ast { bindings, expr });
        let parser = delimited(multispace0, parser, multispace0);
        let mut parser = all_consuming(parser);
        let (tail, ast) = parser(input)
            .map_err(nom::Err::<nom::error::Error<&str>>::to_owned)
            .finish()?;
        ensure!(tail.is_empty(), "Unexpected trailing characters");
        Ok(ast)
    }

    /// Parses an AST from a string with custom `ParseError`
    pub fn parse_verbose(input: &'a str) -> Result<Self> {
        let bindings = separated_list0(multispace0, Binding::parse);
        let parser = tuple((bindings, multispace0, Expression::parse));
        let parser = map(parser, |(bindings, _, expr)| Ast { bindings, expr });
        let parser = delimited(multispace0, parser, multispace0);
        let mut parser = all_consuming(parser);
        let (tail, ast) = parser(input)
            .finish()
            .map_err(|e| anyhow!("{}", convert_error(input, e)))
            .context("Parsing failure")?;
        ensure!(tail.is_empty(), "Unexpected trailing characters");
        Ok(ast)
    }

    /// Returns an owned version of the AST
    pub fn into_owned(self) -> Ast<'static> {
        Ast {
            bindings: self.bindings.into_iter().map(Binding::into_owned).collect(),
            expr: self.expr.into_owned(),
        }
    }

    /// Pretty tree-like print
    pub fn print_tree(&self, w: &mut impl Write) -> Result<(), std::io::Error> {
        for binding in &self.bindings {
            binding.print_tree(w, 0)?;
        }

        self.expr.print_tree(w, 0)
    }
}
