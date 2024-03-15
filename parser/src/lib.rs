use nom::bytes::complete::{tag, take_while1};
use nom::character::complete::{anychar, char as ch_, digit1, multispace0, multispace1};
use nom::combinator::{all_consuming, complete, cut, eof, map, not, opt, recognize, value, verify};
use nom::error::{context, VerboseError};
use nom::multi::many0;
use nom::sequence::{preceded, terminated, tuple};
use nom::Finish;
use nom_locate::LocatedSpan;

type Input<'a> = LocatedSpan<&'a str>;
type Error<'a> = VerboseError<Input<'a>>;
type IResult<'a, T> = nom::IResult<Input<'a>, T, Error<'a>>;

fn parse_id(input: Input) -> IResult<Input> {
    let head = verify(anychar, |c| c.is_alphabetic() || *c == '_');
    let tail = take_while1(|c: char| c.is_alphanumeric() || c == '_');
    recognize(tuple((head, tail)))(input)
}

#[derive(Debug)]
pub struct Module<'a> {
    bindings: Vec<Binding<'a>>,
}

impl<'a> Module<'a> {
    pub fn parse(input: Input<'a>) -> Result<Self, Error> {
        let binding = preceded(not(eof), cut(Binding::parse));
        let binding = preceded(multispace0, binding);
        let bindings = many0(binding);
        let bindings = terminated(bindings, multispace0);
        let bindings = all_consuming(complete(bindings));

        map(bindings, |bindings| Self { bindings })(input)
            .finish()
            .map(|(_, this)| this)
    }

    pub fn format(&self, f: &mut impl std::io::Write) -> std::io::Result<()> {
        self.format_impl(f, 0)
    }

    fn format_impl(&self, f: &mut impl std::io::Write, ind: usize) -> std::io::Result<()> {
        writeln!(f, "MOD")?;

        for bind in &self.bindings {
            bind.format(f, ind + 1)?;
        }

        Ok(())
    }
}

/// Binging in form of:
/// `[visibility] let name [: type] = expression;`
#[derive(Debug)]
pub struct Binding<'a> {
    visibility: Visibility,
    name: Input<'a>,
    ty_: Option<Type>,
    expression: Expression<'a>,
}

impl<'a> Binding<'a> {
    fn parse(input: Input<'a>) -> IResult<Self> {
        let ty_ = tuple((ch_(':'), multispace0, Type::parse));
        let ty_ = map(ty_, |(_colon, _, ty_)| ty_);

        let tpl = tuple((
            Visibility::parse,
            tag("let"),
            multispace1,
            parse_id,
            multispace0,
            opt(ty_),
            multispace0,
            ch_('='),
            multispace0,
            Expression::parse,
            ch_(';'),
        ));
        let tpl = context("Binding", tpl);

        map(
            tpl,
            |(visibility, _let, _, name, _, ty_, _, _eq, _, expression, _semi)| Self {
                visibility,
                name,
                ty_,
                expression,
            },
        )(input)
    }

    fn format(&self, f: &mut impl std::io::Write, ind: usize) -> std::io::Result<()> {
        writeln!(
            f,
            "{:indent$}BIND ({:?}) {}:",
            "",
            self.visibility,
            self.name,
            indent = ind
        )?;

        if let Some(ty_) = &self.ty_ {
            ty_.format(f, ind + 1)?;
        }

        self.expression.format(f, ind + 1)
    }
}

#[derive(Debug)]
pub enum Expression<'a> {
    Literal(ExpressionLiteral<'a>),
}

impl<'a> Expression<'a> {
    fn parse(input: Input<'a>) -> IResult<Self> {
        map(ExpressionLiteral::parse, Expression::Literal)(input)
    }

    fn format(&self, f: &mut impl std::io::Write, ind: usize) -> std::io::Result<()> {
        match self {
            Self::Literal(lit) => {
                write!(f, "{:indent$}EXPR ", "", indent = ind)?;
                lit.format(f)
            }
        }
    }
}

#[derive(Debug)]
pub enum ExpressionLiteral<'a> {
    /// Integral literal of unknown type, eg: `0`, `1357`, `-135234`
    Integral(Input<'a>),
}

impl<'a> ExpressionLiteral<'a> {
    fn parse(input: Input<'a>) -> IResult<Self> {
        let sign = opt(ch_('-'));
        let literal = tuple((sign, multispace0, digit1));
        let literal = context("Integral literal", literal);
        map(recognize(literal), ExpressionLiteral::Integral)(input)
    }

    fn format(&self, f: &mut impl std::io::Write) -> std::io::Result<()> {
        match self {
            Self::Integral(lit) => writeln!(f, "{}", lit),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Visibility {
    /// Default visibility
    Private,
    /// Public visibility (`pub `) - have to be followed with at least one whitespace
    Public,
}

impl Visibility {
    fn parse(input: Input) -> IResult<Self> {
        let pub_ = value(Visibility::Public, terminated(tag("pub"), multispace1));
        let vis = map(opt(pub_), |v| v.unwrap_or(Visibility::Private));

        context("Visibility", vis)(input)
    }
}

/// Built-in type: `u32`
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BasicType {
    U32,
}

impl BasicType {
    fn parse(input: Input) -> IResult<Self> {
        context("BasicType", value(BasicType::U32, tag("u32")))(input)
    }
}

#[derive(Debug, PartialEq)]
pub enum Type {
    Basic(BasicType),
}

impl Type {
    fn parse(input: Input) -> IResult<Self> {
        context("Type", map(BasicType::parse, Type::Basic))(input)
    }

    fn format(&self, f: &mut impl std::io::Write, ind: usize) -> std::io::Result<()> {
        match self {
            Self::Basic(ty_) => {
                writeln!(f, "{:indent$}TY {:?}", "", ty_, indent = ind)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn module() {
        let parsed = Module::parse(LocatedSpan::new("let variable = 15; let other = 10;")).unwrap();
        assert_eq!(parsed.bindings.len(), 2);
        assert_eq!(*parsed.bindings[0].name.fragment(), "variable");
        assert_eq!(*parsed.bindings[1].name.fragment(), "other");
        assert!(
            matches!(parsed.bindings[0].expression, Expression::Literal(ExpressionLiteral::Integral(s)) if *s.fragment() == "15")
        );
        assert!(
            matches!(parsed.bindings[1].expression, Expression::Literal(ExpressionLiteral::Integral(s)) if *s.fragment() == "10")
        );
    }

    #[test]
    fn binding() {
        let (tail, parsed) = Binding::parse(LocatedSpan::new("let variable = 15;")).unwrap();
        assert_eq!(*tail.fragment(), "");
        assert_eq!(parsed.visibility, Visibility::Private);
        assert_eq!(*parsed.name.fragment(), "variable");
        assert_eq!(parsed.ty_, None);
        assert!(
            matches!(parsed.expression, Expression::Literal(ExpressionLiteral::Integral(s)) if *s.fragment() == "15")
        );

        let (tail, parsed) = Binding::parse(LocatedSpan::new("pub let other = 10;")).unwrap();
        assert_eq!(*tail.fragment(), "");
        assert_eq!(parsed.visibility, Visibility::Public);
        assert_eq!(*parsed.name.fragment(), "other");
        assert_eq!(parsed.ty_, None);
        assert!(
            matches!(parsed.expression, Expression::Literal(ExpressionLiteral::Integral(s)) if *s.fragment() == "10")
        );

        let (tail, parsed) = Binding::parse(LocatedSpan::new("let two: u32 = 2;")).unwrap();
        assert_eq!(*tail.fragment(), "");
        assert_eq!(parsed.visibility, Visibility::Private);
        assert_eq!(*parsed.name.fragment(), "two");
        assert_eq!(parsed.ty_, Some(Type::Basic(BasicType::U32)));
        assert!(
            matches!(parsed.expression, Expression::Literal(ExpressionLiteral::Integral(s)) if *s.fragment() == "2")
        );
    }

    #[test]
    fn expression_literal_integral() {
        let (tail, parsed) = Expression::parse(LocatedSpan::new("0")).unwrap();
        assert_eq!(*tail.fragment(), "");
        assert!(
            matches!(parsed, Expression::Literal(ExpressionLiteral::Integral(s)) if *s.fragment() == "0")
        );

        let (tail, parsed) = Expression::parse(LocatedSpan::new("1357")).unwrap();
        assert_eq!(*tail.fragment(), "");
        assert!(
            matches!(parsed, Expression::Literal(ExpressionLiteral::Integral(s)) if *s.fragment() == "1357")
        );

        let (tail, parsed) = Expression::parse(LocatedSpan::new("-135234")).unwrap();
        assert_eq!(*tail.fragment(), "");
        assert!(
            matches!(parsed, Expression::Literal(ExpressionLiteral::Integral(s)) if *s.fragment() == "-135234")
        );

        Expression::parse(LocatedSpan::new("bar")).unwrap_err();
    }

    #[test]
    fn visibility() {
        let (tail, parsed) = Visibility::parse(LocatedSpan::new("")).unwrap();
        assert_eq!(*tail.fragment(), "");
        assert_eq!(parsed, Visibility::Private);

        let (tail, parsed) = Visibility::parse(LocatedSpan::new("pub ")).unwrap();
        assert_eq!(*tail.fragment(), "");
        assert_eq!(parsed, Visibility::Public);
    }

    #[test]
    fn basic_type() {
        let (tail, parsed) = BasicType::parse(LocatedSpan::new("u32")).unwrap();
        assert_eq!(*tail.fragment(), "");
        assert_eq!(parsed, BasicType::U32);
    }

    #[test]
    fn type_() {
        let (tail, parsed) = Type::parse(LocatedSpan::new("u32")).unwrap();
        assert_eq!(*tail.fragment(), "");
        assert_eq!(parsed, Type::Basic(BasicType::U32));
    }
}
