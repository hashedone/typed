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

/// Function/variable definition
#[derive(Debug, PartialEq)]
pub struct Definition {
    /// Definition name
    name: String,
    /// Evaluation expression
    expr: ExpressionWithDefs,
}

impl Definition {
    pub fn new(name: impl ToString, expr: ExpressionWithDefs) -> Self {
        Self {
            name: name.to_string(),
            expr
        }
    }

    pub fn name(&self) -> &str { &self.name }
    pub fn expr(&self) -> &ExpressionWithDefs { &self.expr }
}

#[derive(Debug, PartialEq)]
pub struct ExpressionWithDefs {
    /// Definitions in scope of this expression
    defs: Vec<Definition>,
    /// Expression
    expr: Expression,
}

impl ExpressionWithDefs {
    pub fn new(
        defs: impl IntoIterator<Item=Definition>,
        expr: Expression,
    ) -> Self {
        Self {
            defs: defs.into_iter().collect(),
            expr
        }
    }

    pub fn defs(&self) -> &[Definition] { &self.defs }
    pub fn expr(&self) -> &Expression { &self.expr }
}

#[derive(Debug, PartialEq)]
pub struct AST {
    /// Top-level expression
    expr: ExpressionWithDefs,
}

impl AST {
    pub fn new(expr: ExpressionWithDefs) -> Self {
        Self { expr }
    }

    pub fn expr(&self) -> &ExpressionWithDefs { &self.expr }
}
