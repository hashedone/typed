use crate::*;

pub fn unit() -> Literal { Literal::Unit }

pub fn literal(lit: Literal) -> Expression { Expression::Literal(lit) }

pub fn ast(expr: Expression) -> AST { AST::new(expr) }
