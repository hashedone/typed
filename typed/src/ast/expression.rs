use std::borrow::Cow;
use std::io::Write;

use nom::branch::alt;
use nom::character::complete::{char as ch_, multispace0};
use nom::combinator::map;
use nom::error::ParseError;
use nom::sequence::tuple;
use nom::IResult;

use super::fn_appl::FnAppl;
use super::fn_decl::FnDecl;
use super::ident::ident;
use super::literal::Literal;

/// Single expression
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Expression<'a> {
    /// Literal value
    Literal(Literal),
    /// Variable
    Variable(Cow<'a, str>),
    /// Function declaration
    ///
    /// Boxed, as it contains expression itself internally
    FnDecl(Box<FnDecl<'a>>),
    /// Function application
    ///
    /// Boxed, as it contains expressions itself internally
    FnAppl(Box<FnAppl<'a>>),
}

impl Expression<'static> {
    /// Creates new literal expression
    pub fn literal(literal: Literal) -> Self {
        Expression::Literal(literal)
    }

    /// Creates new variable expression
    pub fn variable(variable: String) -> Self {
        Expression::Variable(Cow::Owned(variable))
    }
}

impl<'a> Expression<'a> {
    /// Creates new variable expression
    pub fn variable_borrowed(variable: &'a str) -> Self {
        Expression::Variable(Cow::Borrowed(variable))
    }

    /// Nom parser for a trivial expression
    ///
    /// Trivial expression is any expression which is not left recursive. Any expression which
    /// expects to start with another expression should always expect only the `trivial_expression`
    /// instead of the full expression.
    ///
    /// If for any reason one want to left-associate the expression, the parenthised might be used
    pub fn parse_trivial<E>(input: &'a str) -> IResult<&'a str, Self, E>
    where
        E: ParseError<&'a str>,
    {
        let paren = map(
            tuple((
                ch_('('),
                multispace0,
                Expression::parse,
                multispace0,
                ch_(')'),
            )),
            |(_bro, _, expr, _, _brc)| expr,
        );
        let fn_decl = map(FnDecl::parse, |f| Expression::FnDecl(Box::new(f)));
        let literal = map(Literal::parse, Expression::Literal);
        let variable = map(ident, |v| Expression::Variable(Cow::Borrowed(v)));
        //        let fn_appl = map(FnAppl::parse, |f| Expression::FnAppl(Box::new(f)));
        alt((paren, fn_decl, /* fn_appl, */ literal, variable))(input)
    }

    /// Nom parser for an expression
    pub fn parse<E>(input: &'a str) -> IResult<&'a str, Self, E>
    where
        E: ParseError<&'a str>,
    {
        let fn_appl = map(FnAppl::parse, |f| Expression::FnAppl(Box::new(f)));
        alt((fn_appl, Self::parse_trivial))(input)
    }

    /// Returns an owned version of the expression
    pub fn into_owned(self) -> Expression<'static> {
        match self {
            Expression::Literal(lit) => Expression::Literal(lit),
            Expression::Variable(v) => Expression::Variable(Cow::Owned(v.into_owned())),
            Expression::FnDecl(f) => Expression::FnDecl(Box::new(f.into_owned())),
            Expression::FnAppl(f) => Expression::FnAppl(Box::new(f.into_owned())),
        }
    }

    /// Pretty tree-like print
    pub fn print_tree(&self, w: &mut impl Write, indent: usize) -> Result<(), std::io::Error> {
        match self {
            Expression::Literal(lit) => write!(w, "{:indent$}LIT: {lit}\n", ""),
            Expression::Variable(v) => write!(w, "{:indent$}VAR: {v}\n", ""),
            Expression::FnDecl(f) => f.print_tree(w, indent),
            Expression::FnAppl(f) => f.print_tree(w, indent),
        }
    }
}

#[cfg(test)]
mod tests {
    use nom::Finish;

    use super::*;

    type Err<'a> = nom::error::Error<&'a str>;

    #[test]
    fn variable() {
        let (tail, ex) = Expression::parse::<Err>("ident").finish().unwrap();
        assert_eq!(tail, "");
        assert_eq!(ex, Expression::variable("ident".into()));
    }

    #[test]
    fn literal() {
        let (tail, ex) = Expression::parse::<Err>("()").finish().unwrap();
        assert_eq!(tail, "");
        assert_eq!(ex, Expression::Literal(Literal::Unit));

        let (tail, ex) = Expression::parse::<Err>("123").finish().unwrap();
        assert_eq!(tail, "");
        assert_eq!(ex, Expression::Literal(Literal::Unsigned(123)));
    }

    #[test]
    fn paren() {
        let (tail, ex) = Expression::parse::<Err>("(123)").finish().unwrap();
        assert_eq!(tail, "");
        assert_eq!(ex, Expression::Literal(Literal::Unsigned(123)));
    }
}
