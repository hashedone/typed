use chumsky::prelude::*;
use chumsky::text::unicode::ident;
use chumsky::text::whitespace;

use crate::expr::{expr, Expr};
use crate::ty::{ty, Ty};
use crate::vis::{vis, Vis};
use crate::{spanned, Ex, Spanned};

/// Binding:
///
/// `[vis] let ident [: ty] = expr;`
#[derive(Clone, Debug, PartialEq)]
pub struct Binding<'s> {
    pub vis: Spanned<Vis>,
    pub ident: Spanned<&'s str>,
    pub ty: Option<Spanned<Ty>>,
    pub expr: Spanned<Expr<'s>>,
}

pub fn binding<'s>() -> impl Parser<'s, &'s str, Binding<'s>, Ex<'s>> + Clone {
    let vis = spanned(vis());
    let let_ = just("let");
    let ident = spanned(ident());
    let ty = just(":")
        .then(whitespace())
        .ignore_then(spanned(ty()))
        .or_not();
    let eq = just("=");
    let expr = spanned(expr());
    let semi = just(";");

    vis.then_ignore(whitespace())
        .then_ignore(let_)
        .then_ignore(whitespace())
        .then(ident)
        .then_ignore(whitespace())
        .then(ty)
        .then_ignore(whitespace())
        .then_ignore(eq)
        .then_ignore(whitespace())
        .then(expr)
        .then_ignore(whitespace())
        .then_ignore(semi)
        .map(|(((vis, ident), ty), expr)| Binding {
            vis,
            ident,
            ty,
            expr,
        })
}

#[cfg(test)]
mod tests {
    use crate::literal::Literal;
    use crate::ty::BasicType;

    use super::*;

    #[test]
    fn test_binding() {
        let input = "pub let x = 42;";
        let result = binding().parse(input).unwrap();
        assert_eq!(
            result,
            Binding {
                vis: Spanned {
                    node: Vis::Public,
                    span: 0..3,
                },
                ident: Spanned {
                    node: "x",
                    span: 8..9,
                },
                ty: None,
                expr: Spanned {
                    node: Expr::Literal(Literal::Integer("42")),
                    span: 12..14,
                },
            }
        );

        let input = "let x: u32 = 42;";
        let result = binding().parse(input).unwrap();
        assert_eq!(
            result,
            Binding {
                vis: Spanned {
                    node: Vis::Private,
                    span: 0..0,
                },
                ident: Spanned {
                    node: "x",
                    span: 4..5,
                },
                ty: Some(Spanned {
                    node: Ty::Basic(BasicType::U32),
                    span: 7..10,
                }),
                expr: Spanned {
                    node: Expr::Literal(Literal::Integer("42")),
                    span: 13..15,
                },
            }
        );
    }
}
