use std::{borrow::Cow, io::Write};

use nom::{
    bytes::complete::tag,
    character::complete::{char as ch_, multispace0},
    combinator::map,
    error::ParseError,
    multi::{separated_list0, separated_list1},
    sequence::tuple,
    IResult,
};

use super::{binding::Binding, expression::Expression, ident::ident};

/// Function declaration
///
/// Function declaration looks like this:
/// ```typed
/// fn (arg1, arg2, arg3) {
///     let var1 = ...;
///     let var2 = ...;
///     expr
/// }
/// ```
///
/// In PEG:
///
/// fn_decl <- "fn" "(" <fn_arg_list> ")" "{" <binding>* <expression> "}"
/// fn_arg_list <- <fn_arg> | <fn_arg> "," <fn_arg_list>
/// fn_arg <- <ident>
///
/// In lambda calculus functions are taking only one arguments, which is how typed works -
/// multi-argument functions are syntactic sugar for currying. Internally for structure
/// simplification we keep as multi-argument function, however from type point of view
/// the function above have a signature of:
///
/// ```typed
/// <A, B, C> A -> B -> C -> R
/// ````
///
/// `R` is not generic here, as the return type has to be always possible to calculate from
/// argument types.
///
/// `let` bindings in the function are lazy evaluated - the only relevant part of a function is
/// an `expr` - all the `let` bindings are guaranteed to be evaluate only if they are needed for
/// the final expression evaluation.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FnDecl<'a> {
    args: Vec<Cow<'a, str>>,
    bindings: Vec<Binding<'a>>,
    expr: Expression<'a>,
}

impl FnDecl<'static> {
    pub fn new<Arg: ToString>(
        args: impl IntoIterator<Item = Arg>,
        bindings: impl IntoIterator<Item = Binding<'static>>,
        expr: Expression<'static>,
    ) -> Self {
        Self {
            args: args
                .into_iter()
                .map(|arg| Cow::Owned(arg.to_string()))
                .collect(),
            bindings: bindings.into_iter().collect(),
            expr,
        }
    }
}

impl<'a> FnDecl<'a> {
    pub fn borrowed(
        args: impl IntoIterator<Item = &'a str>,
        bindings: impl IntoIterator<Item = Binding<'a>>,
        expr: Expression<'a>,
    ) -> Self {
        Self {
            args: args.into_iter().map(|arg| Cow::Borrowed(arg)).collect(),
            bindings: bindings.into_iter().collect(),
            expr,
        }
    }

    pub fn parse<E>(input: &'a str) -> IResult<&'a str, Self, E>
    where
        E: ParseError<&'a str>,
    {
        let arg = map(ident, Cow::Borrowed);
        let arg_list = separated_list1(tuple((multispace0, tag(","), multispace0)), arg);
        let bindings = separated_list0(multispace0, Binding::parse);
        let decl = tuple((
            tag("fn"),
            multispace0,
            ch_('('),
            multispace0,
            arg_list,
            multispace0,
            ch_(')'),
            multispace0,
            ch_('{'),
            multispace0,
            bindings,
            multispace0,
            Expression::parse,
            multispace0,
            ch_('}'),
        ));

        map(
            decl,
            |(_fn, _, _obr, _, args, _, _cbr, _, _ocbr, _, bindings, _, expr, _, _ccbr)| Self {
                args,
                bindings,
                expr,
            },
        )(input)
    }

    pub fn into_owned(self) -> FnDecl<'static> {
        let args = self
            .args
            .into_iter()
            .map(|arg| Cow::Owned(arg.into_owned()))
            .collect();
        let bindings = self
            .bindings
            .into_iter()
            .map(|binding| binding.into_owned())
            .collect();
        let expr = self.expr.into_owned();

        FnDecl {
            args,
            bindings,
            expr,
        }
    }

    /// Pretty tree-like print
    pub fn print_tree(&self, w: &mut impl Write, indent: usize) -> Result<(), std::io::Error> {
        let args = self.args.join(" ");
        write!(w, "{:indent$}fn ({args}):\n", "")?;

        for binding in &self.bindings {
            binding.print_tree(w, indent + 1)?;
        }

        self.expr.print_tree(w, indent + 1)
    }
}

#[cfg(test)]
mod tests {
    use nom::Finish;

    use super::*;

    type Err<'a> = nom::error::Error<&'a str>;

    #[test]
    fn id() {
        let (tail, func) = FnDecl::parse::<Err>("fn(x) { x }").finish().unwrap();
        assert_eq!(tail, "");

        let expected = FnDecl {
            args: vec!["x".into()],
            bindings: vec![],
            expr: Expression::Variable("x".into()),
        };
        assert_eq!(func, expected);
    }
}
