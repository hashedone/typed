use nom::{
    IResult,
    sequence::tuple,
    combinator::map,
    character::complete::{multispace0 as ws, char},
};

/// NOM combinator for definition
pub fn definition(s: &str) -> IResult<&str, ast::Definition> {
    map(
        tuple((
                crate::ident,
                ws, char('='), ws,
                crate::expression_with_defs,
                ws, char(';')
        )),
        |(ident, _, _, _, expr, _, _)| ast::Definition::new(ident, expr)
    )(s)
}

#[cfg(test)]
mod tests {
    use crate::{test_parser, test_parser_fail};
    use super::definition;
    use ast::builder::{def, expr, lit, unit};

    #[test]
    fn parse_unit() {
        test_parser(
            definition,
            &def("singleton", expr(None, lit(unit()))),
            "singleton = ();",
        )
    }

    #[test]
    fn parse_invalid() {
        test_parser_fail(
            definition,
            "bad",
        )
    }
}
