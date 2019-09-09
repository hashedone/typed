use crate::*;

pub fn unit() -> Literal { Literal::Unit }

pub fn lit(lit: Literal) -> Expression { Expression::Literal(lit) }

pub fn def(name: impl ToString, expr: ExpressionWithDefs) -> Definition {
    Definition::new(name, expr)
}

pub fn expr(
    defs: impl IntoIterator<Item=Definition>,
    expr: Expression
) -> ExpressionWithDefs {
    ExpressionWithDefs::new(defs, expr)
}

pub fn ast(expr: ExpressionWithDefs) -> AST { AST::new(expr) }
