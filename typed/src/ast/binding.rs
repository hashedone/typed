use std::borrow::Cow;
use std::io::Write;

use nom::bytes::complete::tag;
use nom::character::complete::{multispace0, multispace1};
use nom::combinator::map;
use nom::error::ParseError;
use nom::sequence::tuple;
use nom::IResult;

use crate::ast::ident::ident;

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
    name: Cow<'a, str>,
    expr: Expression<'a>,
}

impl Binding<'static> {
    /// Creates new owned expression binding
    pub fn new(name: impl ToString, expr: Expression<'static>) -> Self {
        Binding {
            name: Cow::Owned(name.to_string()),
            expr,
        }
    }
}

impl<'a> Binding<'a> {
    /// Creates new borrowed expression binding
    pub fn borrowed(name: &'a str, expr: Expression<'a>) -> Self {
        Binding {
            name: Cow::Borrowed(name),
            expr,
        }
    }

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
        map(tokens, |(_let, _, name, _, _eq, _, expr, _, _semi)| {
            Binding::borrowed(name, expr)
        })(input)
    }

    /// Returns an owned version of the binding
    pub fn into_owned(self) -> Binding<'static> {
        Binding {
            name: Cow::Owned(self.name.into_owned()),
            expr: self.expr.into_owned(),
        }
    }

    /// Pretty tree-like print
    pub fn print_tree(&self, w: &mut impl Write, indent: usize) -> Result<(), std::io::Error> {
        write!(w, "{:indent$}let {} =\n", "", self.name)?;
        self.expr.print_tree(w, indent + 1)
    }
}

#[cfg(test)]
mod test {
    use nom::Finish;

    use crate::ast::{fn_decl::FnDecl, literal::Literal};

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

        let id = FnDecl::borrowed(["x"], [], Expression::Variable("x".into()));
        let expected = Binding {
            name: "id".into(),
            expr: Expression::FnDecl(Box::new(id)),
        };
        assert_eq!(expected, bound)
    }
}
