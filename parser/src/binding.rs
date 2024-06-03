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
    pub let_: Spanned<()>,
    pub ident: Spanned<&'s str>,
    pub colon: Option<Spanned<()>>,
    pub ty: Option<Spanned<Ty>>,
    pub eq: Spanned<()>,
    pub expr: Spanned<Expr<'s>>,
    pub semi: Spanned<()>,
}

pub fn binding<'s>() -> impl Parser<'s, &'s str, Binding<'s>, Ex<'s>> + Clone {
    let vis = spanned(vis());
    let let_ = spanned(just("let").to(()));
    let ident = spanned(ident());
    let ty = spanned(just(":").to(()))
        .then_ignore(whitespace())
        .then(spanned(ty()))
        .or_not();
    let eq = spanned(just("=").to(()));
    let expr = spanned(expr());
    let semi = spanned(just(";").to(()));

    vis.then_ignore(whitespace())
        .then(let_)
        .then_ignore(whitespace())
        .then(ident)
        .then_ignore(whitespace())
        .then(ty)
        .then_ignore(whitespace())
        .then(eq)
        .then_ignore(whitespace())
        .then(expr)
        .then_ignore(whitespace())
        .then(semi)
        .map(|((((((vis, let_), ident), ty), eq), expr), semi)| {
            let (colon, ty) = ty.unzip();

            Binding {
                vis,
                let_,
                ident,
                colon,
                ty,
                eq,
                expr,
                semi,
            }
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
                let_: Spanned {
                    node: (),
                    span: 4..7,
                },
                ident: Spanned {
                    node: "x",
                    span: 8..9,
                },
                colon: None,
                ty: None,
                eq: Spanned {
                    node: (),
                    span: 10..11,
                },
                expr: Spanned {
                    node: Expr::Literal(Literal::Integer("42")),
                    span: 12..14,
                },
                semi: Spanned {
                    node: (),
                    span: 14..15,
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
                let_: Spanned {
                    node: (),
                    span: 0..3,
                },
                ident: Spanned {
                    node: "x",
                    span: 4..5,
                },
                colon: Some(Spanned {
                    node: (),
                    span: 5..6,
                }),
                ty: Some(Spanned {
                    node: Ty::Basic(BasicType::U32),
                    span: 7..10,
                }),
                eq: Spanned {
                    node: (),
                    span: 11..12,
                },
                expr: Spanned {
                    node: Expr::Literal(Literal::Integer("42")),
                    span: 13..15,
                },
                semi: Spanned {
                    node: (),
                    span: 15..16,
                },
            }
        );
    }
}
