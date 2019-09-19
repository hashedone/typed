use nom::{
    IResult,
    combinator::map,
    bytes::complete::tag,
};

/// NOM combinator for unit literal
fn unit(s: &str) -> IResult<&str, ast::Literal> {
    map(
        tag("()"),
        |_| ast::Literal::new(ast::Value::Unit),
    )(s)
}

/// NOM combinator for any literal
pub fn literal(s: &str) -> IResult<&str, ast::Literal> {
    unit(s)
}

#[cfg(test)]
mod tests {
    use crate::{test_parser, test_parser_fail};
    use super::literal;
    use ast::builder::unit;

    #[test]
    fn parse_unit() {
        test_parser(literal, &unit(), "()");
        test_parser_fail(literal, "( )");
        test_parser_fail(literal, "(\t)");
        test_parser_fail(literal, "(\n)");
    }

    #[test]
    fn parse_invalid() {
        test_parser_fail(literal, "bad");
    }
}
