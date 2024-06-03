use chumsky::prelude::*;

use crate::literal::{literal, Literal};
use crate::{spanned, Ex, Spanned};

/// Exppression
#[derive(Clone, Debug, PartialEq)]
pub enum Expr<'s> {
    /// Literal
    Literal(Literal<'s>),

    /// Parenthesized expression
    Paren(Box<Spanned<Self>>),
}

pub fn expr<'s>() -> impl Parser<'s, &'s str, Expr<'s>, Ex<'s>> + Clone {
    recursive(|expr| {
        let literal = literal().map(Expr::Literal);

        let paren = spanned(expr)
            .padded()
            .delimited_by(just("("), just(")"))
            .map(|expr| Expr::Paren(Box::new(expr)));

        choice((paren, literal))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expr() {
        let input = "42";
        let result = expr().parse(input).unwrap();
        assert_eq!(result, Expr::Literal(Literal::Integer("42")));

        let input = "(42)";
        let result = expr().parse(input).unwrap();
        assert_eq!(
            result,
            Expr::Paren(Box::new(Spanned {
                node: Expr::Literal(Literal::Integer("42")),
                span: 1..3,
            }))
        );

        let input = "()";
        let result = expr().parse(input).unwrap();
        assert_eq!(result, Expr::Literal(Literal::Unit));

        let input = "(())";
        let result = expr().parse(input).unwrap();
        assert_eq!(
            result,
            Expr::Paren(Box::new(Spanned {
                node: Expr::Literal(Literal::Unit),
                span: 1..3,
            }))
        );
    }
}
