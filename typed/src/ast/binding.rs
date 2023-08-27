use std::borrow::Cow;

use nom::IResult;
use nom::bytes::complete::tag;
use nom::character::complete::{multispace0, multispace1};
use nom::sequence::tuple;
use nom::combinator::map;

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
    expr: Expression,
}

impl Binding<'static> {
    /// Creates new owned expression binding
    pub fn new(name: impl ToString, expr: Expression) -> Self {
        Binding { name: Cow::Owned(name.to_string()), expr }
    }
}

impl<'a> Binding<'a> {
    /// Creates new borrowed expression binding
    pub fn borrowed(name: &'a str, expr: Expression) -> Self {
        Binding { name: Cow::Borrowed(name), expr }
    }

    /// Nom parser for a binding
    pub fn parse(input: &'a str) -> IResult<&str, Self> {
        let tokens = tuple((tag("let"), multispace1, ident, multispace0, tag("="), multispace0, Expression::parse, multispace0, tag(";")));
        map(tokens, |(_let, _, name, _, _eq, _, expr, _, _semi)| Binding::borrowed(name, expr))(input)
    }

    /// Returns an owned version of the binding
    pub fn into_owned(self) -> Binding<'static> {
        Binding { name: Cow::Owned(self.name.into_owned()), expr: self.expr }
    }
}
