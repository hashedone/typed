use nom::{
    IResult,
    sequence::tuple,
    multi::separated_list,
    combinator::map,
    character::complete::multispace0 as ws,
};
use crate::Context;

/// NOM combinator for single literal expression
fn literal(s: &str) -> IResult<&str, ast::Expression> {
    map(
        crate::literal,
        |lit| ast::Expression::Literal(lit),
    )(s)
}

/// NOM combinator for any expression
pub fn expression(_ctx: &Context) ->
    impl Fn(&str) -> IResult<&str, ast::Expression>
{
    literal
}

/// NOM combinator for expression with definitions in scope
pub fn expression_with_defs<'a>(ctx: &'a Context) ->
    impl Fn(&str) -> IResult<&str, ast::ExpressionWithDefs> + 'a
{
    move |s| {
        let (tail, (defs, _)) = tuple((
            separated_list(ws, crate::definition(ctx)),
            ws,
        ))(s)?;

        let ctx = ctx.extend(&defs);
        let (tail, expr) = expression(&ctx)(tail)?;

        Ok((tail, ast::ExpressionWithDefs::new(defs, expr)))
    }
}

#[cfg(test)]
mod tests {
    use crate::{test_parser, test_parser_fail};
    use super::{expression, expression_with_defs};
    use ast::builder::{unit, lit, expr, def};

    #[test]
    fn parse_unit() {
        test_parser(expression(&Default::default()), &lit(unit()), "()");
        test_parser(
            expression_with_defs(&Default::default()),
            &expr(None, lit(unit())),
            "()"
        );
    }

    #[test]
    fn parse_defs() {
        test_parser(
            expression_with_defs(&Default::default()),
            &expr(
                vec![def("singleton", expr(None, lit(unit())))],
                lit(unit())
            ),
            "singleton = (); ()",
        );
    }

    #[test]
    fn parse_invalid() {
        test_parser_fail(expression(&Default::default()), "bad");
    }
}
