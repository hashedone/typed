use ast;

mod literal;
mod expression;

use nom::{
    self,
    IResult,
    sequence::tuple,
    combinator::map,
    character::complete::multispace0 as ws,
};

use literal::literal;
use expression::expression;

/// Helper function for testing NOM combinators.
/// Asserts if parser consumes whole string and
/// produces expected result
#[cfg(test)]
fn test_parser<T: PartialEq + std::fmt::Debug>(
    parser: impl Fn(&str) -> IResult<&str, T>,
    expected: T,
    source: &str
) {
    let (rest, given) = parser(source).unwrap();
    assert!(rest.is_empty());
    assert_eq!(given, expected);
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

/// NOM combinator for parsing whole AST
fn parse_ast(s: &str) -> IResult<&str, ast::AST> {
    map(
        tuple((ws, expression, ws)),
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
        parse_ast,
        parse,
    };

    #[test]
    fn unit_ast() {
        test_parser(parse_ast, ast(literal(unit())), "()");
        test_parser(parse_ast, ast(literal(unit())), "    ()  ");
        test_parser(parse_ast, ast(literal(unit())), "\n\t() \n");
    }

    #[test]
    fn invalid_ast() {
        test_parser_fail(parse_ast, "bad");
    }

    #[test]
    fn parse_unit() {
        assert_eq!(
            ast(literal(unit())),
            parse("()").unwrap()
        )
    }

    #[test]
    fn parse_invalid() {
        parse("bad").unwrap_err()
    }
}
