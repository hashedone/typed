use nom::{
    IResult,
    sequence::tuple,
    combinator::map,
    character::complete::{multispace0 as ws, char},
};
use crate::Context;

/// NOM combinator for definition
pub fn definition<'a>(ctx: &'a Context) ->
    impl Fn(&str) -> IResult<&str, ast::Definition> + 'a
{
    move |s| map(
        tuple((
                crate::ident,
                ws, char('='), ws,
                crate::expression_with_defs(ctx),
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
            definition(&Default::default()),
            &def("singleton", expr(None, lit(unit()))),
            "singleton = ();",
        )
    }

    #[test]
    fn parse_invalid() {
        test_parser_fail(
            definition(&Default::default()),
            "bad",
        )
    }
}
