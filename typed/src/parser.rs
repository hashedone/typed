use self::expression::Expression;
use self::literal::Literal;
use anyhow::{Result, ensure};
use nom::Finish;

mod literal;
mod expression;

/// Program abstract syntax tree
/// 
/// Result of parsing
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Ast {
    expr: Expression,
}

impl Ast {
    /// Returns an AST of empty program
    pub fn new() -> Self {
        Ast {
            expr: Expression::Literal(Literal::Unit)
        }
    }

    /// Parses an AST from a string
    pub fn parse(input: &str) -> Result<Self> {
        let (tail, expr) = Expression::parse(input).map_err(nom::Err::<nom::error::Error<&str>>::to_owned).finish()?;
        ensure!(tail.is_empty(), "Unexpected trailing characters");
        Ok(Ast { expr })
    }
}