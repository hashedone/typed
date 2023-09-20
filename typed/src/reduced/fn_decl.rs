use crate::parser::fn_decl::FnDecl as ParsedDecl;

use super::{expression::Expression, BuildingContext, Context};

use anyhow::{anyhow, Result};

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct FnDecl {
    pub arg: usize,
    pub expr: Expression,
}

impl FnDecl {
    pub(super) fn new<'a>(
        decl: ParsedDecl<'a>,
        context: &mut Context<'a>,
        builder: &mut BuildingContext<'a>,
    ) -> Result<Self> {
        builder.new_frame();

        for binding in decl.bindings {
            let expr = Expression::from_expr(binding.expr, context, builder)?;
            builder.bind(binding.name, expr);
        }

        let mut args: Vec<_> = decl
            .args
            .into_iter()
            .map(|arg| {
                let var = context.create_variable(arg);
                builder.bind(arg, Expression::Variable(var));
                var
            })
            .collect();

        let expr = Expression::from_expr(decl.expr, context, builder)?;
        builder.close_frame()?;

        let decl = FnDecl {
            arg: args
                .pop()
                .ok_or_else(|| anyhow!("No arguments on function declaration"))?,
            expr,
        };

        let decl = args.into_iter().rev().fold(decl, |expr, arg| FnDecl {
            arg,
            expr: Expression::FnDecl(Box::new(expr)),
        });

        Ok(decl)
    }
}
