use std::{fmt::Display, mem::take};

use parser::expression::Expression as ParsedExpr;
use tracing::{debug, instrument, warn};

use crate::parser::{self, literal::Literal, root::Root};

use super::{fn_appl::FnAppl, fn_decl::FnDecl, BuildingContext, Context};

use anyhow::Result;

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub enum Expression {
    Literal(Literal),
    Variable(usize),
    FnDecl(Box<FnDecl>),
    FnAppl(Box<FnAppl>),
}

impl Expression {
    #[instrument(skip(root, context, builder), name = "Reduced::new")]
    pub(super) fn new<'a>(
        root: Root<'a>,
        context: &mut Context<'a>,
        builder: &mut BuildingContext<'a>,
    ) -> Result<Self> {
        for binding in root.bindings {
            let expr = Expression::from_expr(binding.expr, context, builder)?;
            debug!(name = binding.name, expr = %expr.p(context), "Binding");
            builder.bind(binding.name, expr);
        }

        let expr = Expression::from_expr(root.expr, context, builder)?;

        Ok(expr)
    }

    pub(super) fn from_expr<'a>(
        expr: ParsedExpr<'a>,
        context: &mut Context<'a>,
        builder: &mut BuildingContext<'a>,
    ) -> Result<Self> {
        let expr = match expr {
            ParsedExpr::Literal(lit) => Self::Literal(lit),
            ParsedExpr::Variable(v) => {
                let mut expr = builder.binding(v)?;
                expr.alfa_conversion(context);
                expr
            }
            ParsedExpr::FnDecl(decl) => {
                Self::FnDecl(Box::new(FnDecl::new(*decl, context, builder)?))
            }
            ParsedExpr::FnAppl(appl) => {
                let func = Self::from_expr(appl.func, context, builder)?;
                let args = appl
                    .args
                    .into_iter()
                    .map(|arg| Self::from_expr(arg, context, builder));
                let mut appl = Self::apply(func, args)?;
                appl.reduce(context);
                appl
            }
        };

        Ok(expr)
    }

    fn apply(
        func: Expression,
        args: impl IntoIterator<Item = Result<Expression>>,
    ) -> Result<Expression> {
        args.into_iter().try_fold(func, |func, arg| {
            let arg = arg?;
            let expr = {
                let appl = FnAppl { func, arg };
                Self::FnAppl(Box::new(appl))
            };
            Ok(expr)
        })
    }

    /// Performes single alfa reduction of all the non-free variables occuring in the expression to
    /// the newly created variables.
    #[instrument(name = "alfa", skip(self, context))]
    fn alfa_conversion(&mut self, context: &mut Context<'_>) {
        // Mappings to be performed in the `(from, to)` form
        let mut mapping = vec![];
        debug!(expr = %self.p(context), "α-conversion");

        fn convert_inner(
            expr: &mut Expression,
            context: &mut Context,
            mapping: &mut Vec<(usize, usize)>,
        ) {
            match expr {
                Expression::Variable(var) => {
                    if let Some((_, to)) = mapping.iter().find(|(from, _)| *from == *var) {
                        *var = *to
                    }
                }
                Expression::FnDecl(ref mut decl) => {
                    if let Some(idx) = mapping.iter().position(|(from, _)| *from == decl.arg) {
                        // Shadowing detected.
                        //
                        // To deal with it, the mapping is temporarly substituted for entirely
                        // fresh variable, and will be remapped when this node is converted
                        // entirely.
                        warn!(
                            var = decl.arg,
                            name = context.variable(decl.arg).unwrap_or_default(),
                            "Shadowing detected"
                        );

                        let prev = mapping[idx].1;
                        mapping[idx].1 = context.duplicate_variable(decl.arg);
                        debug!(from = mapping[idx].0, to = mapping[idx].1, "New α mapping");
                        decl.arg = mapping[idx].1;
                        convert_inner(&mut decl.expr, context, mapping);
                        mapping[idx].1 = prev;
                    } else {
                        let new = context.duplicate_variable(decl.arg);
                        mapping.push((decl.arg, new));
                        decl.arg = new;
                        convert_inner(&mut decl.expr, context, mapping);
                    }
                }
                Expression::FnAppl(appl) => {
                    convert_inner(&mut appl.func, context, mapping);
                    convert_inner(&mut appl.arg, context, mapping);
                }
                Expression::Literal(_) => (),
            }
        }

        convert_inner(self, context, &mut mapping);

        debug!(expr = %self.p(context), "α-reduction complete");
    }

    /// Performs beta-reduction and eta-reduction on a tree node. It assumes that everything below the
    /// reduced note is already beta-reduced, so it reduces only expressions in the form of:
    /// * `(\x.e1) e2`, aplying the `e2` as a variable x in `e1`, and the recursively reducing the new expression.
    ///
    /// Alfa-conversion is performed only when expression is branched to keep unique variables in
    /// the entire tree.
    #[instrument(skip(self, context))]
    fn reduce(&mut self, context: &mut Context<'_>) {
        fn substitute(
            expr: &mut Expression,
            var: usize,
            sub: Expression,
            context: &mut Context<'_>,
        ) {
            match expr {
                Expression::Variable(v) if *v == var => *expr = sub,
                Expression::FnDecl(decl) => substitute(&mut decl.expr, var, sub, context),
                Expression::FnAppl(appl) => {
                    let mut left = sub.clone();
                    left.alfa_conversion(context);
                    substitute(&mut appl.func, var, left, context);
                    substitute(&mut appl.arg, var, sub, context);

                    // After substitution, it may be a case that the reduceable node was created, reducing it
                    expr.reduce(context);
                }
                Expression::Variable(_) | Expression::Literal(_) => (),
            }
        }

        debug!(expr = %self.p(context), "β-conversion");
        if let Expression::FnAppl(appl) = self {
            // Nodes in the form of `(\x.e1) e2` can be b-reduced
            if let Expression::FnDecl(ref mut decl) = appl.func {
                let arg = take(&mut appl.arg);
                substitute(&mut decl.expr, decl.arg, arg, context);
                *self = take(&mut decl.expr);
                debug!(expr = %self.p(context), "β-conversion complete");
            }
        }
    }

    pub(super) fn format(&self, w: &mut std::fmt::Formatter, ctx: &Context) -> std::fmt::Result {
        match self {
            Self::Variable(v) => ctx
                .variable(*v)
                .map(|v| write!(w, "{v}"))
                .unwrap_or_else(|| write!(w, "_{v}")),
            Self::Literal(lit) => write!(w, "{lit}"),
            Self::FnAppl(appl) => {
                write!(w, "(")?;
                appl.func.format(w, ctx)?;
                write!(w, " ")?;
                appl.arg.format(w, ctx)?;
                write!(w, ")")
            }
            Self::FnDecl(decl) => {
                ctx.variable(decl.arg)
                    .map(|v| write!(w, "\\{v}. "))
                    .unwrap_or_else(|| write!(w, "\\_{}", decl.arg))?;
                decl.expr.format(w, ctx)
            }
        }
    }

    /// Makes expression printing friendly
    pub fn p<'a>(&'a self, context: &'a Context<'a>) -> PrettyExpression<'a> {
        PrettyExpression {
            context,
            expr: self,
        }
    }
}

impl Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Variable(v) => write!(f, "_{v}"),
            Self::Literal(lit) => write!(f, "{lit}"),
            Self::FnAppl(appl) => {
                write!(f, "({} {})", appl.func, appl.arg)
            }
            Self::FnDecl(decl) => {
                write!(f, "\\_{}. {}", decl.arg, decl.expr)
            }
        }
    }
}

/// Expression wrapped with the context for better printing
pub struct PrettyExpression<'a> {
    context: &'a Context<'a>,
    expr: &'a Expression,
}

impl Display for PrettyExpression<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.expr.format(f, self.context)
    }
}

impl Default for Expression {
    fn default() -> Self {
        Self::Literal(Literal::Unit)
    }
}
