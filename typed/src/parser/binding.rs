use nom::bytes::complete::tag;
use nom::character::complete::{multispace0, multispace1};
use nom::combinator::map;
use nom::error::ParseError;
use nom::sequence::tuple;
use nom::IResult;

use super::ident::ident;

use super::expression::Expression;

/// The binding expression
///
/// Binding expression binds the expression to the value:
///
/// ```typed
/// let unit = ();
/// ```
///
/// Binding expressions are in the form of:
///
/// let <name> = <expression>;
///
/// In PEG:
///
/// binding <- "let" <name> "=" <expression> ";"
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Binding<'a> {
    pub name: &'a str,
    pub expr: Expression<'a>,
}

impl<'a> Binding<'a> {
    /// Nom parser for a binding
    pub fn parse<E>(input: &'a str) -> IResult<&'a str, Self, E>
    where
        E: ParseError<&'a str>,
    {
        let tokens = tuple((
            tag("let"),
            multispace1,
            ident,
            multispace0,
            tag("="),
            multispace0,
            Expression::parse,
            multispace0,
            tag(";"),
        ));
        map(tokens, |(_let, _, name, _, _eq, _, expr, _, _semi)| Self {
            name,
            expr,
        })(input)
    }
}

#[cfg(test)]
mod test {
    use nom::Finish;

    use crate::parser::{fn_decl::FnDecl, literal::Literal};

    use super::*;

    type Err<'a> = nom::error::Error<&'a str>;

    #[test]
    fn binding_expr() {
        let (tail, bound) = Binding::parse::<Err>("let var = 123;").finish().unwrap();
        assert_eq!(tail, "");

        let expected = Binding {
            name: "var".into(),
            expr: Expression::Literal(Literal::Unsigned(123)),
        };
        assert_eq!(expected, bound)
    }

    #[test]
    fn binding_fn() {
        let (tail, bound) = Binding::parse::<Err>("let id = fn(x) { x };")
            .finish()
            .unwrap();
        assert_eq!(tail, "");

        let id = FnDecl {
            args: vec!["x"],
            bindings: vec![],
            expr: Expression::Variable("x"),
        };
        let expected = Binding {
            name: "id".into(),
            expr: Expression::FnDecl(Box::new(id)),
        };
        assert_eq!(expected, bound)
    }
}
