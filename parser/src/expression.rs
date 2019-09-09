use nom::{
    IResult,
    sequence::tuple,
    multi::separated_list,
    combinator::map,
    character::complete::multispace0 as ws,
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

/// NOM combinator for expression with definitions in scope
pub fn expression_with_defs(s: &str) ->
    IResult<&str, ast::ExpressionWithDefs>
{
    map(
        tuple((separated_list(ws, crate::definition), ws, expression)),
        |(defs, _, expr)| ast::ExpressionWithDefs::new(defs, expr)
    )(s)
}

#[cfg(test)]
mod tests {
    use crate::{test_parser, test_parser_fail};
    use super::{expression, expression_with_defs};
    use ast::builder::{unit, lit, expr, def};

    #[test]
    fn parse_unit() {
        test_parser(expression, &lit(unit()), "()");
        test_parser(
            expression_with_defs,
            &expr(None, lit(unit())),
            "()"
        );
    }

    #[test]
    fn parse_defs() {
        test_parser(
            expression_with_defs,
            &expr(
                vec![def("singleton", expr(None, lit(unit())))],
                lit(unit())
            ),
            "singleton = (); ()",
        );
    }

    #[test]
    fn parse_invalid() {
        test_parser_fail(expression, "bad");
    }
}
