use std::io::Write;

use nom::{
    character::complete::{char as ch_, multispace0},
    combinator::map,
    error::ParseError,
    multi::separated_list1,
    sequence::tuple,
    IResult,
};

use super::expression::Expression;

/// Function application
///
/// Functions are applied like this:
/// ```typed
/// fn_name(arg1, arg2)
/// ````
///
/// In PEG:
///
/// fn_appl <- <expression> "(" <call_arg_list> ")"
/// call_arg_list <- <expression> | <expression> "," <expression>
///
/// Functions can be called immediately when defined:
/// ```typed
/// fn (a, b) { b }(15, 10)
/// ```
///
/// It might be a good idea to wrap the function declaration into parens for more clarity:
/// ```typed
/// (fn (a, b) { b })(15, 10)
/// ```
///
/// Not all the function arguments have to be passed - the functions are automatically curried. The
/// code above can be written as:
///
/// ```typed
/// (fn (a, b) { b })(15)(10)
/// ````
///
/// Also the arguments passed to the function do not need to be limited to the function arguments -
/// if the function would return another function, arguments can be passed over to returned
/// function. The following expression evaluates to `5`:
///
/// ```typed
/// (fn (a) { fn (b) { a + b }})(3, 2)
/// ```
///
/// Note, that the function application requires at least one argument - typed do not support
/// zero-arguments zero-argument function, as all functions in Typed are pure. Zero-argument
/// function is equivalent of the constant in functional paradigm.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FnAppl<'a> {
    func: Expression<'a>,
    args: Vec<Expression<'a>>,
}

impl<'a> FnAppl<'a> {
    pub fn new(func: Expression<'a>, args: impl IntoIterator<Item = Expression<'a>>) -> Self {
        Self {
            func,
            args: args.into_iter().collect(),
        }
    }

    pub fn parse<E>(input: &'a str) -> IResult<&'a str, Self, E>
    where
        E: ParseError<&'a str>,
    {
        let arg_list = separated_list1(
            tuple((multispace0, ch_(','), multispace0)),
            Expression::parse,
        );

        map(
            tuple((
                Expression::parse_trivial,
                multispace0,
                ch_('('),
                multispace0,
                arg_list,
                multispace0,
                ch_(')'),
            )),
            |(func, _, _bro, _, args, _, _brc)| Self { func, args },
        )(input)
    }

    pub fn into_owned(self) -> FnAppl<'static> {
        FnAppl {
            func: self.func.into_owned(),
            args: self.args.into_iter().map(Expression::into_owned).collect(),
        }
    }

    pub fn print_tree(&self, w: &mut impl Write, indent: usize) -> Result<(), std::io::Error> {
        self.func.print_tree(w, indent)?;
        write!(w, "{:indent$} <-\n", "", indent = indent)?;
        for arg in &self.args {
            arg.print_tree(w, indent + 1)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use nom::Finish;

    use crate::ast::literal::Literal;

    use super::*;

    type Err<'a> = nom::error::Error<&'a str>;

    #[test]
    fn call() {
        let (tail, appl) = FnAppl::parse::<Err>("add(2, 3)").finish().unwrap();
        assert_eq!(tail, "");

        let expected = FnAppl {
            func: Expression::Variable("add".into()),
            args: vec![
                Expression::Literal(Literal::Unsigned(2)),
                Expression::Literal(Literal::Unsigned(3)),
            ],
        };
        assert_eq!(appl, expected)
    }
}
