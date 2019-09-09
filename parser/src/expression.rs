use nom::{
    IResult,
    combinator::map
};

/// NOM combinator for single literal expression
fn literal(s: &str) -> IResult<&str, ast::Expression> {
    map(
        crate::literal,
        |lit| ast::Expression::Literal(lit),
    )(s)
}

/// NOM combinator for any expression
pub fn expression(s: &str) -> IResult<&str, ast::Expression> {
    literal(s)
}

#[cfg(test)]
mod tests {
    use crate::{test_parser, test_parser_fail};
    use super::expression;
    use ast::builder::{unit, literal};

    #[test]
    fn parse_unit() {
        test_parser(expression, literal(unit()), "()");
    }

    #[test]
    fn invalid_expr() {
        test_parser_fail(expression, "bad");
    }
}
