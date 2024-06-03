use chumsky::prelude::*;

use super::Ex;

/// Built-in primitive type
#[derive(Debug, Clone, PartialEq)]
pub enum BasicType {
    /// Unit type
    Unit,
    /// U32 type
    U32,
}

fn basic_type<'s>() -> impl Parser<'s, &'s str, BasicType, Ex<'s>> + Clone {
    let unit = just("()").to(BasicType::Unit);
    let u32_ = just("u32").to(BasicType::U32);

    choice((unit, u32_))
}

/// Type
#[derive(Debug, Clone, PartialEq)]
pub enum Ty {
    /// Basic type
    Basic(BasicType),
}

pub fn ty<'s>() -> impl Parser<'s, &'s str, Ty, Ex<'s>> + Clone {
    let basic = basic_type().map(Ty::Basic);

    basic.labelled("type")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_type() {
        let input = "u32";
        let result = basic_type().parse(input).unwrap();
        assert_eq!(result, BasicType::U32);
    }

    #[test]
    fn test_ty() {
        let input = "u32";
        let result = ty().parse(input).unwrap();
        assert_eq!(result, Ty::Basic(BasicType::U32));
    }
}
