use std::fmt::Display;

use parser::expression::Expression as ParsedExpr;
use tracing::{debug, instrument, warn};

use crate::parser::{self, literal::Literal, root::Root};

use super::{fn_appl::FnAppl, fn_decl::FnDecl, BuildingContext, Context, ExprId};

use anyhow::Result;

#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq)]
pub enum Expression {
    Literal(Literal),
    Variable(usize),
    FnDecl(FnDecl),
    FnAppl(FnAppl),
}

impl Expression {
    #[instrument(skip(root, context, builder), name = "Reduced::new")]
    pub(super) fn new<'a>(
        root: Root<'a>,
        context: &mut Context<'a>,
        builder: &mut BuildingContext<'a>,
    ) -> Result<ExprId> {
        for binding in root.bindings {
            let expr = Expression::from_expr(binding.expr, context, builder)?;
            debug!(name = binding.name, expr = %context.pexpr(expr), "Binding");
            builder.bind(binding.name, expr);
        }

        let expr = Expression::from_expr(root.expr, context, builder)?;

        Ok(expr)
    }

    pub(super) fn from_expr<'a>(
        expr: ParsedExpr<'a>,
        context: &mut Context<'a>,
        builder: &mut BuildingContext<'a>,
    ) -> Result<ExprId> {
        let expr = match expr {
            ParsedExpr::Literal(lit) => context.create_expr(Self::Literal(lit)),
            ParsedExpr::Variable(v) => {
                let expr = builder.binding(v)?;
                let expr = Self::alfa_conversion(expr, context);
                expr
            }
            ParsedExpr::FnDecl(decl) => {
                let expr = Self::FnDecl(FnDecl::new(*decl, context, builder)?);
                context.create_expr(expr)
            }
            ParsedExpr::FnAppl(appl) => {
                let func = Self::from_expr(appl.func, context, builder)?;
                let expr = appl
                    .args
                    .into_iter()
                    .try_fold(func, |func, arg| -> Result<_> {
                        let arg = Self::from_expr(arg, context, builder)?;
                        let appl = FnAppl { func, arg };
                        let expr = context.create_expr(Self::FnAppl(appl));
                        Ok(expr)
                    })?;

                Self::reduce(expr, context)
            }
        };

        Ok(expr)
    }

    /// Performes single alfa reduction of all the non-free variables occuring in the expression to
    /// the newly created variables, creating a new expression from it.
    #[instrument(name = "alfa", skip(context))]
    fn alfa_conversion(expr: ExprId, context: &mut Context<'_>) -> ExprId {
        // Mappings to be performed in the `(from, to)` form
        //
        // Note that mappings never have to be popped, as we assume that expressions are always
        // alpha-normalized, so the same variable is never used in two separated for many subtrees.
        let mut mapping = vec![];
        debug!(expr = %context.pexpr(expr), "α-conversion");

        let expr = context.clone_expr(expr);

        // Expressions to be transformed, paired with the depth of the `mapping` when pushed
        // to the stack
        let mut stack = vec![(expr, 0)];

        while let Some((id, mlen)) = stack.pop() {
            use Expression::*;

            mapping.shrink_to(mlen);

            let mut expr = context.expr(id);
            match &mut expr {
                Literal(_) => (),
                Variable(var) => {
                    if let Some((_, to)) = mapping.iter().rev().find(|(from, _)| *from == *var) {
                        *var = *to
                    }
                }
                FnDecl(decl) => {
                    let var = context.duplicate_variable(decl.arg);
                    debug!(from = decl.arg, to = var, "New α mapping");
                    mapping.push((decl.arg, var));
                    decl.arg = var;

                    decl.expr = context.clone_expr(decl.expr);
                    stack.push((decl.expr, mapping.len()));
                }
                FnAppl(appl) => {
                    appl.func = context.clone_expr(appl.func);
                    stack.push((appl.func, mapping.len()));

                    appl.arg = context.clone_expr(appl.arg);
                    stack.push((appl.arg, mapping.len()));
                }
            }

            *context.expr_mut(id) = expr;
        }

        debug!(expr = %context.pexpr(expr), "α-reduction complete");
        expr
    }

    /// Performs beta-reduction on a tree node. It assumes that everything below the reduced note is already beta-reduced,
    /// so it reduces only expressions in the form of `(\x.e1) e2`, aplying the `e2` as a variable x in `e1`, and the
    /// recursively reducing the new expression.
    ///
    /// Alfa-conversion is performed only when expression is branched to keep unique variables in
    /// the entire tree.
    #[instrument(skip(context))]
    fn reduce(id: ExprId, context: &mut Context<'_>) -> ExprId {
        use Expression::*;

        let result = context.clone_expr(id);
        // Application nodes occured in the process - they shall be double-checked after the proces
        // as reduceable node could be created
        let mut applications = vec![result];

        // Expressions substitutions to be performed, pairs of `(var, expr_id)`
        let mut substitutions = vec![];

        // Expressions to be reduced with length of substiturions when pushed
        let mut stack = vec![];

        while let Some(root) = applications.pop() {
            // Reducing only if top-level expression is `(\x.e1) e2`
            // `appl` becomes `(\x.e1) e2
            // `decl` becomes `\x.e1`
            let FnAppl(mut appl) = context.expr(root) else { continue };
            let FnDecl(decl) = context.expr(appl.func) else { continue };

            // Recursion check
            //
            // The node shaped as `(e1 e1)` where `e1` is a function declaration node, leading to
            // the infite recursion. We still want to reduce the `e1` node, but without reducing
            // top-level expression.
            if Self::equivalent(appl.func, appl.arg, context) {
                debug!(expr = %context.pexpr(root), "β-conversion recursion");
                applications.push(appl.func);
                if appl.arg != appl.func {
                    appl.arg = appl.func;
                    *context.expr_mut(root) = FnAppl(appl);
                }

                continue;
            }

            debug!(expr = %context.pexpr(root), "β-conversion");

            substitutions.clear();
            substitutions.push((decl.arg, context.expr(appl.arg).clone()));

            let expr = context.expr(decl.expr).clone();
            *context.expr_mut(root) = expr;
            stack.push((root, substitutions.len()));

            while let Some((id, depth)) = stack.pop() {
                substitutions.shrink_to(depth);

                let expr = context.expr(id).clone();
                let expr = match expr {
                    Literal(_) => expr,
                    Variable(var) => {
                        if let Some((_, to)) =
                            substitutions.iter().rev().find(|(from, _)| *from == var)
                        {
                            to.clone()
                        } else {
                            expr
                        }
                    }
                    FnDecl(mut decl) => {
                        decl.expr = context.clone_expr(decl.expr);
                        stack.push((decl.expr, substitutions.len()));
                        FnDecl(decl)
                    }
                    FnAppl(mut appl) => {
                        if let FnDecl(decl) = context.expr(appl.func) {
                            // Immediately reduceable expression
                            //
                            // Recursion check - we do not reduce if expresions are the same
                            // TODO: Double check it its enough, possibly its neccessary if the
                            // `appl.arg` is anywhere down in `appl.func`
                            //
                            // If recursion is found, we do not reduce this node, but we reduce the
                            // underlying expression
                            if Self::equivalent(appl.func, appl.arg, context) {
                                debug!(expr = %expr.p(context), "β-conversion recursion");
                                appl.func = context.clone_expr(appl.arg);
                                appl.arg = appl.func;

                                stack.push((appl.func, substitutions.len()));

                                FnAppl(appl)
                            } else {
                                let substitution = Self::alfa_conversion(appl.arg, context);
                                let substitution = context.expr(substitution);
                                substitutions.push((decl.arg, substitution));

                                // Recalculate the same node - it will be reassigned with the `decl.expr``
                                stack.push((id, substitutions.len()));

                                context.expr(decl.expr)
                            }
                        } else {
                            appl.func = context.clone_expr(appl.func);
                            stack.push((appl.func, substitutions.len()));

                            appl.arg = context.clone_expr(appl.arg);
                            stack.push((appl.arg, substitutions.len()));

                            applications.push(id);

                            FnAppl(appl)
                        }
                    }
                };

                *context.expr_mut(id) = expr;
            }

            debug!(expr = %context.pexpr(root), "β-conversion complete");
        }

        result
    }

    /// Checks if two expressions are quivalent
    fn equivalent(l: ExprId, r: ExprId, context: &Context) -> bool {
        use Expression::*;

        let mut stack = vec![(l, r)];
        let mut vars = vec![];

        while let Some((l, r)) = stack.pop() {
            match (context.expr(l), context.expr(r)) {
                (Literal(l), Literal(r)) if l == r => (),
                (Variable(l), Variable(r)) if l == r => (),
                (Variable(l), Variable(r)) if vars.contains(&(l, r)) => (),
                (FnDecl(l), FnDecl(r)) => {
                    if l.arg != r.arg {
                        vars.push((l.arg, r.arg));
                    }

                    stack.push((l.expr, r.expr));
                }
                (FnAppl(l), FnAppl(r)) => {
                    stack.push((l.func, r.func));
                    stack.push((l.arg, r.arg));
                }
                _ => return false,
            }
        }

        true
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
                ctx.expr(appl.func).format(w, ctx)?;
                write!(w, " ")?;
                ctx.expr(appl.arg).format(w, ctx)?;
                write!(w, ")")
            }
            Self::FnDecl(decl) => {
                ctx.variable(decl.arg)
                    .map(|v| write!(w, "\\{v}. "))
                    .unwrap_or_else(|| write!(w, "\\_{}", decl.arg))?;
                ctx.expr(decl.expr).format(w, ctx)
            }
        }
    }

    /// Makes expression printing friendly
    pub fn p<'a>(self, context: &'a Context<'a>) -> PrettyExpression<'a> {
        PrettyExpression {
            context,
            expr: self,
        }
    }
}

/// Expression wrapped with the context for better printing
pub struct PrettyExpression<'a> {
    context: &'a Context<'a>,
    expr: Expression,
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
