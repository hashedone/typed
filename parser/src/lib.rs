use ast;

mod literal;
mod expression;
mod definition;
mod context;

use nom::{
    self,
    IResult,
    sequence::tuple,
    combinator::map,
    character::complete::multispace0 as ws,
    bytes::complete::{take_while, take_while1},
};

use literal::literal;
use expression::expression_with_defs;
use definition::definition;

use context::Context;

/// Helper function for testing NOM combinators.
/// Asserts if parser consumes whole string and
/// produces expected result
#[cfg(test)]
fn test_parser<T: PartialEq + std::fmt::Debug>(
    parser: impl Fn(&str) -> IResult<&str, T>,
    expected: &T,
    source: &str
) {
    let (rest, given) = parser(source).unwrap();
    assert!(rest.is_empty());
    assert_eq!(&given, expected);
}

/// Helper function for testing NOM combinators.
/// Asserts if parser gives error parsing given
/// source
#[cfg(test)]
fn test_parser_fail<T: std::fmt::Debug>(
    parser: impl Fn(&str) -> IResult<&str, T>,
    source: &str
) {
    parser(source).unwrap_err();
}

/// NOM combinator for identifier
fn ident(s: &str) -> IResult<&str, String> {
    let head_pred = |c: char| (c.is_alphabetic() || c == '_');
    let tail_pred = |c: char| (c.is_alphanumeric() || c == '_');

    map(
        tuple((take_while1(head_pred), take_while(tail_pred))),
        |(h, t)| format!("{}{}", h, t)
    )(s)
}

/// NOM combinator for parsing whole AST
fn parse_ast(s: &str) -> IResult<&str, ast::AST> {
    map(
        tuple((ws, expression_with_defs(&Default::default()), ws)),
        |(_, expr, _)| ast::AST::new(expr),
    )(s)
}

/// Parses given string into AST, aligning error type.
/// Whole string has to parse so this will succeed
pub fn parse(s: &str) -> Result<ast::AST, ()> {
    match parse_ast(s) {
        Ok(("", ast)) => Ok(ast),
        _ => Err(())
    }
}

#[cfg(test)]
mod test {
    use ast::builder::*;
    use super::{
        test_parser,
        test_parser_fail,
        parse,
    };

    #[test]
    fn ident() {
        test_parser(super::ident, &String::from("ident"), "ident");
        test_parser_fail(super::ident, "132d");
    }

    #[test]
    fn parse_unit() {
        assert_eq!(
            ast(expr(None, lit(unit()))),
            parse("()").unwrap()
        )
    }

    #[test]
    fn parse_invalid() {
        parse("bad").unwrap_err()
    }
}
