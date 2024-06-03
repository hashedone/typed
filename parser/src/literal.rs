use chumsky::prelude::*;
use chumsky::text::{digits, whitespace};

use crate::ty::BasicType;
use crate::Ex;

/// Compile time literal
#[derive(Debug, Clone, PartialEq)]
pub enum Literal<'s> {
    /// Untyped integer
    Integer(&'s str),
    /// Unit literal
    Unit,
    /// Typed literal
    Typed {
        /// Literal
        value: &'s str,
        /// Type
        ty: BasicType,
    },
}

pub fn literal<'s>() -> impl Parser<'s, &'s str, Literal<'s>, Ex<'s>> + Clone {
    let unsigned = digits(10).separated_by(just("_")).at_least(1);
    let signed = just("-").then(whitespace()).or_not().then(unsigned);

    let integer = signed.to_slice().map(Literal::Integer);

    let unit = just("()").to(Literal::Unit);

    let u32_ = unsigned
        .to_slice()
        .then_ignore(just("u32"))
        .map(|value| Literal::Typed {
            value,
            ty: BasicType::U32,
        });

    choice((unit, u32_, integer)).labelled("literal")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_literal() {
        let input = "42";
        let result = literal().parse(input).unwrap();
        assert_eq!(result, Literal::Integer("42"));

        let input = "-42";
        let result = literal().parse(input).unwrap();
        assert_eq!(result, Literal::Integer("-42"));

        let input = "53u32";
        let result = literal().parse(input).unwrap();
        assert_eq!(
            result,
            Literal::Typed {
                value: "53",
                ty: BasicType::U32,
            }
        );

        let input = "()";
        let result = literal().parse(input).unwrap();
        assert_eq!(result, Literal::Unit);
    }
}
