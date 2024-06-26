use chumsky::prelude::*;
use chumsky::text::whitespace;

use crate::{spanned, Ex, Spanned};

/// Module item
#[derive(Clone, Debug, PartialEq)]
pub enum Item<'s> {
    /// Binding
    Binding(crate::binding::Binding<'s>),
    /// Invalid item
    Invalid,
}

fn item<'s>() -> impl Parser<'s, &'s str, Item<'s>, Ex<'s>> + Clone {
    let recovery = none_of(";").repeated().then(just(";")).to(Item::Invalid);
    let binding = crate::binding::binding().map(Item::Binding);

    binding.recover_with(via_parser(recovery))
}

/// Module
#[derive(Clone, Debug, PartialEq)]
pub struct Module<'s> {
    items: Vec<Spanned<Item<'s>>>,
}

pub fn module<'s>() -> impl Parser<'s, &'s str, Module<'s>, Ex<'s>> + Clone {
    let item = spanned(item());

    item.separated_by(whitespace())
        .collect::<Vec<_>>()
        .map(|items| Module { items })
        .padded()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_item() {
        let input = "let x = 42;";
        let result = item().parse(input).unwrap();
        assert_eq!(
            result,
            Item::Binding(crate::binding::Binding {
                vis: Spanned {
                    node: crate::vis::Vis::Private,
                    span: 0..0,
                },
                ident: Spanned {
                    node: "x",
                    span: 4..5,
                },
                ty: None,
                expr: Spanned {
                    node: crate::expr::Expr::Literal(crate::literal::Literal::Integer("42")),
                    span: 8..10,
                },
            })
        );

        let input = "let x: u32 = 42;";
        let result = item().parse(input).unwrap();
        assert_eq!(
            result,
            Item::Binding(crate::binding::Binding {
                vis: Spanned {
                    node: crate::vis::Vis::Private,
                    span: 0..0,
                },
                ident: Spanned {
                    node: "x",
                    span: 4..5,
                },
                ty: Some(Spanned {
                    node: crate::ty::Ty::Basic(crate::ty::BasicType::U32),
                    span: 7..10,
                }),
                expr: Spanned {
                    node: crate::expr::Expr::Literal(crate::literal::Literal::Integer("42")),
                    span: 13..15,
                },
            })
        );
    }
}
