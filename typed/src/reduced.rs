use std::{borrow::Cow, fmt::Display};

use crate::parser::root::Root;

use self::expression::Expression;

use anyhow::{anyhow, Result};

pub mod expression;
pub mod fn_appl;
pub mod fn_decl;

/// Reduction context
///
/// Contains common reduced nodes
#[derive(Debug, Clone, PartialEq, Hash, Eq, Default)]
pub struct Context<'a> {
    /// Variables in the reduced tree as kept as `usize` to simplify processing, but here is the
    /// mapping of those variables to their names in the original source.
    variables: Vec<Cow<'a, str>>,
}

impl<'a> Context<'a> {
    fn create_variable(&mut self, var: impl Into<Cow<'a, str>>) -> usize {
        self.variables.push(var.into());
        self.variables.len() - 1
    }

    fn variable(&self, id: usize) -> Option<&str> {
        self.variables.get(id).map(Cow::as_ref)
    }

    /// Creates a new variable, with debug information derived from the given variable
    fn duplicate_variable(&mut self, var: usize) -> usize {
        let name = self
            .variable(var)
            .map_or_else(|| format!("_{var}"), |name| format!("{name}"));
        self.create_variable(name)
    }
}

/// The context used to build the reduced tree
#[derive(Debug, Clone, PartialEq, Hash, Eq, Default)]
struct BuildingContext<'a> {
    /// Stack of active bindings mapping the binding name to the expression.
    ///
    /// This is not a `HashMap`, because single name might be bound multiple times, new binding
    /// should shadow the previous occurence - therefore bindings are in the form of the stack,
    /// and when binding is looked up, it should always be scanned from the end.
    bindings: Vec<(&'a str, Expression)>,
    /// Stack frames in which bindings are introduced. Whenever new scope is created, the new stack
    /// frame should be introduced, and when the scope is processed, the stack frame should be
    /// closed - this way, all the bindings introduced in this frame would immediately be removes.
    stack: Vec<usize>,
}

impl<'a> BuildingContext<'a> {
    /// Creates new stack frame
    pub fn new_frame(&mut self) {
        self.stack.push(self.bindings.len())
    }

    /// Closes the most recent stack frame, failing if there is no opened frame
    fn close_frame(&mut self) -> Result<()> {
        self.bindings.shrink_to(
            self.stack
                .pop()
                .ok_or_else(|| anyhow!("No pending stack frame"))?,
        );

        Ok(())
    }

    /// Returns an expression bound to the variable
    pub fn binding(&self, var: &str) -> Result<Expression> {
        self.bindings
            .iter()
            .rev()
            .find(|(name, _)| *name == var)
            .map(|(_, r)| r.clone())
            .ok_or_else(|| anyhow!("Unbound variable {var}"))
    }

    /// Create new binding
    pub fn bind(&mut self, var: &'a str, expr: Expression) {
        self.bindings.push((var, expr))
    }
}

/// The type of the Ast after applying reductions
#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct Ast<'a> {
    pub context: Context<'a>,
    pub root: Expression,
}

impl<'a> Ast<'a> {
    pub fn new(root: Root<'a>) -> Result<Self> {
        let mut context = Context::default();
        let mut builder = BuildingContext::default();

        let root = Expression::new(root, &mut context, &mut builder)?;

        Ok(Self { context, root })
    }
}

impl Display for Ast<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.root.format(f, &self.context)
    }
}
