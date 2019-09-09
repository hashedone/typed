pub mod builder;

/// Literal constant
#[derive(Debug, PartialEq)]
pub enum Literal {
    /// () in any value position
    Unit,
}

#[derive(Debug, PartialEq)]
pub enum Expression {
    /// Literal constant expression
    Literal(Literal),
}

#[derive(Debug, PartialEq)]
pub struct AST {
    /// Top-level expression
    expr: Expression,
}

impl AST {
    /// Builds AST with only top-level expression
    pub fn new(expr: Expression) -> Self {
        Self { expr }
    }

    /// Expression accessor
    pub fn expr(&self) -> &Expression {
        &self.expr
    }

    /// Morphism to expression
    pub fn into_expr(self) -> Expression {
        self.expr
    }
}
