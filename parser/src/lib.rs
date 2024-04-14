use std::ops::{Deref, DerefMut};

use nom::bytes::complete::{tag, take_while1};
use nom::character::complete::{anychar, char as ch_, digit1, multispace0, multispace1};
use nom::combinator::{all_consuming, complete, cut, eof, map, not, opt, recognize, value, verify};
use nom::error::{context, VerboseError};
use nom::multi::many0;
use nom::sequence::{preceded, terminated, tuple};
use nom::{Finish, Parser};
use nom_locate::{position, LocatedSpan};

type Input<'a> = LocatedSpan<&'a str>;
type Error<'a> = VerboseError<Input<'a>>;
type IResult<'a, T> = nom::IResult<Input<'a>, T, Error<'a>>;

#[derive(Debug, Clone, PartialEq)]
pub struct AST<'a, M = ()> {
    module: ModuleNode<'a, M>,
}

pub type RawAST<'a> = AST<'a, ()>;
pub type SpannedAST<'a> = AST<'a, Span>;

impl<'a, M> AST<'a, M>
where
    M: Meta + 'a,
{
    pub fn parse(input: &'a str) -> Result<Self, Error> {
        let input = LocatedSpan::new(input);
        all_consuming(complete(ModuleNode::parse))(input)
            .finish()
            .map(|(_, module)| Self { module })
    }

    pub fn format<W>(&self, f: &mut W) -> std::io::Result<()>
    where
        W: std::io::Write,
        ModuleNode<'a, M>: Describe<W>,
    {
        let mut stack = vec![(0, &self.module as &dyn Describe<W>)];
        while let Some((ind, node)) = stack.pop() {
            write!(f, "{:ind$}", "", ind = ind)?;
            node.describe(f)?;
            writeln!(f)?;
            stack.extend(
                node.subnodes()
                    .into_iter()
                    .rev()
                    .map(|node| (ind + 1, node)),
            )
        }

        Ok(())
    }
}

pub trait Describe<W> {
    fn describe(&self, f: &mut W) -> std::io::Result<()>;
    fn subnodes(&self) -> Vec<&dyn Describe<W>>;
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Node<T, M> {
    node: T,
    meta: M,
}

impl<T, M> Node<T, M>
where
    M: Meta,
{
    fn parser<'a>(
        p: impl Parser<Input<'a>, T, Error<'a>>,
    ) -> impl Parser<Input<'a>, Self, Error<'a>>
    where
        T: 'a,
    {
        map(<M as Meta>::parser(p), |(meta, node)| Node { meta, node })
    }
}

impl<T, M> Deref for Node<T, M> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.node
    }
}

impl<T, M> DerefMut for Node<T, M> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.node
    }
}

impl<T> From<T> for Node<T, ()> {
    fn from(node: T) -> Self {
        Node { node, meta: () }
    }
}

impl<T, M, W> Describe<W> for Node<T, M>
where
    T: Describe<W>,
    M: Meta,
    W: std::io::Write,
{
    fn describe(&self, f: &mut W) -> std::io::Result<()> {
        self.meta.describe(f)?;
        self.node.describe(f)
    }

    fn subnodes(&self) -> Vec<&dyn Describe<W>> {
        self.node.subnodes()
    }
}

pub trait Meta: Sized {
    fn parser<'a, O: 'a>(
        p: impl Parser<Input<'a>, O, Error<'a>>,
    ) -> impl Parser<Input<'a>, (Self, O), Error<'a>>;

    fn describe(&self, f: &mut impl std::io::Write) -> std::io::Result<()>;
}

impl Meta for () {
    fn parser<'a, O: 'a>(
        p: impl Parser<Input<'a>, O, Error<'a>>,
    ) -> impl Parser<Input<'a>, (Self, O), Error<'a>> {
        map(p, |o| ((), o))
    }

    fn describe(&self, _f: &mut impl std::io::Write) -> std::io::Result<()> {
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Span {
    offset: usize,
    len: usize,
}

impl Meta for Span {
    fn parser<'a, O: 'a>(
        p: impl Parser<Input<'a>, O, Error<'a>>,
    ) -> impl Parser<Input<'a>, (Self, O), Error<'a>> {
        tuple((position, p, position)).map(|(beg, output, end)| {
            let span = Span {
                offset: beg.location_offset(),
                len: end.location_offset() - beg.location_offset(),
            };

            (span, output)
        })
    }

    fn describe(&self, f: &mut impl std::io::Write) -> std::io::Result<()> {
        write!(f, "[{}:{}] ", self.offset, self.offset + self.len)
    }
}

fn parse_id(input: Input) -> IResult<Input> {
    let head = verify(anychar, |c| c.is_alphabetic() || *c == '_');
    let tail = take_while1(|c: char| c.is_alphanumeric() || c == '_');
    recognize(tuple((head, tail)))(input)
}

#[derive(Debug, Clone, PartialEq)]
pub struct Module<'a, M> {
    bindings: Vec<BindingNode<'a, M>>,
}

impl<'a, M> Module<'a, M>
where
    M: Meta + 'a,
{
    fn parse(input: impl Into<LocatedSpan<&'a str>>) -> IResult<'a, Self> {
        let binding = preceded(not(eof), cut(BindingNode::parse));
        let binding = preceded(multispace0, binding);
        let bindings = many0(binding);
        let bindings = terminated(bindings, multispace0);

        context("Module", map(bindings, |bindings| Self { bindings }))(input.into())
    }
}

impl<'a, M, W> Describe<W> for Module<'a, M>
where
    M: Meta,
    W: std::io::Write,
{
    fn describe(&self, f: &mut W) -> std::io::Result<()> {
        write!(f, "MOD")
    }

    fn subnodes(&self) -> Vec<&dyn Describe<W>> {
        self.bindings
            .iter()
            .map(|b| b as &dyn Describe<W>)
            .collect()
    }
}

type ModuleNode<'a, M> = Node<Module<'a, M>, M>;

impl<'a, M> ModuleNode<'a, M>
where
    M: Meta + 'a,
{
    fn parse(input: impl Into<LocatedSpan<&'a str>>) -> IResult<'a, Self> {
        Node::parser(Module::parse).parse(input.into())
    }
}

/// Binging in form of:
/// `[visibility] let name [: type] = expression;`
#[derive(Debug, Clone, PartialEq)]
pub struct Binding<'a, M> {
    visibility: VisibilityNode<M>,
    name: &'a str,
    ty_: Option<TypeNode<M>>,
    expression: ExpressionNode<'a, M>,
}

impl<'a, M> Binding<'a, M>
where
    M: Meta + 'a,
{
    fn parse(input: impl Into<LocatedSpan<&'a str>>) -> IResult<'a, Self> {
        let ty_ = tuple((ch_(':'), multispace0, TypeNode::parse));
        let ty_ = map(ty_, |(_colon, _, ty_)| ty_);

        let tpl = tuple((
            VisibilityNode::parse,
            tag("let"),
            multispace1,
            parse_id,
            multispace0,
            opt(ty_),
            multispace0,
            ch_('='),
            multispace0,
            ExpressionNode::parse,
            ch_(';'),
        ));
        let tpl = context("Binding", tpl);

        map(
            tpl,
            |(visibility, _let, _, name, _, ty_, _, _eq, _, expression, _semi)| Self {
                visibility,
                name: name.fragment(),
                ty_,
                expression,
            },
        )(input.into())
    }
}

type BindingNode<'a, M> = Node<Binding<'a, M>, M>;

impl<'a, M> BindingNode<'a, M>
where
    M: Meta + 'a,
{
    fn parse(input: impl Into<LocatedSpan<&'a str>>) -> IResult<'a, Self> {
        Node::parser(Binding::parse).parse(input.into())
    }
}

impl<'a, M, W> Describe<W> for Binding<'a, M>
where
    M: Meta,
    W: std::io::Write,
{
    fn describe(&self, f: &mut W) -> std::io::Result<()> {
        write!(f, "BIND {}:", self.name)
    }

    fn subnodes(&self) -> Vec<&dyn Describe<W>> {
        if let Some(ty_) = &self.ty_ {
            vec![&self.visibility, ty_, &self.expression]
        } else {
            vec![&self.visibility, &self.expression]
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Expression<'a> {
    Literal(ExpressionLiteral<'a>),
}

impl<'a> Expression<'a> {
    fn parse(input: impl Into<LocatedSpan<&'a str>>) -> IResult<'a, Self> {
        map(ExpressionLiteral::parse, Expression::Literal)(input.into())
    }
}

impl<'a, W> Describe<W> for Expression<'a>
where
    W: std::io::Write,
{
    fn describe(&self, f: &mut W) -> std::io::Result<()> {
        match self {
            Self::Literal(lit) => {
                write!(f, "EXPR ")?;
                lit.format(f)
            }
        }
    }

    fn subnodes(&self) -> Vec<&dyn Describe<W>> {
        match &self {
            Self::Literal(_) => vec![],
        }
    }
}

type ExpressionNode<'a, M> = Node<Expression<'a>, M>;

impl<'a, M> ExpressionNode<'a, M>
where
    M: Meta + 'a,
{
    fn parse(input: impl Into<LocatedSpan<&'a str>>) -> IResult<'a, Self> {
        Node::parser(Expression::parse).parse(input.into())
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExpressionLiteral<'a> {
    /// Integral literal of unknown type, eg: `0`, `1357`, `-135234`
    Integral(&'a str),
}

impl<'a> ExpressionLiteral<'a> {
    fn parse(input: impl Into<LocatedSpan<&'a str>>) -> IResult<'a, Self> {
        let sign = opt(ch_('-'));
        let literal = tuple((sign, multispace0, digit1));
        let literal = context("Integral literal", literal);

        map(recognize(literal), |lit: Input<'a>| {
            ExpressionLiteral::Integral(lit.fragment())
        })(input.into())
    }

    fn format(&self, f: &mut impl std::io::Write) -> std::io::Result<()> {
        match self {
            Self::Integral(lit) => write!(f, "LIT INT {}", lit),
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
    fn parse<'a>(input: impl Into<LocatedSpan<&'a str>>) -> IResult<'a, Self> {
        let pub_ = value(Visibility::Public, terminated(tag("pub"), multispace1));
        let vis = map(opt(pub_), |v| v.unwrap_or(Visibility::Private));

        context("Visibility", vis)(input.into())
    }
}

impl<W> Describe<W> for Visibility
where
    W: std::io::Write,
{
    fn describe(&self, f: &mut W) -> std::io::Result<()> {
        match self {
            Self::Private => write!(f, "PRIV"),
            Self::Public => write!(f, "PUB"),
        }
    }

    fn subnodes(&self) -> Vec<&dyn Describe<W>> {
        vec![]
    }
}

type VisibilityNode<M> = Node<Visibility, M>;

impl<M> VisibilityNode<M>
where
    M: Meta,
{
    fn parse<'a>(input: impl Into<LocatedSpan<&'a str>>) -> IResult<'a, Self> {
        Node::parser(Visibility::parse).parse(input.into())
    }
}

/// Built-in type: `u32`
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BasicType {
    U32,
}

impl BasicType {
    fn parse<'a>(input: impl Into<LocatedSpan<&'a str>>) -> IResult<'a, Self> {
        context("BasicType", value(BasicType::U32, tag("u32")))(input.into())
    }

    fn describe(&self, f: &mut impl std::io::Write) -> std::io::Result<()> {
        match self {
            Self::U32 => write!(f, "BTYPE u32"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Type {
    Basic(BasicType),
}

impl Type {
    fn parse<'a>(input: impl Into<LocatedSpan<&'a str>>) -> IResult<'a, Self> {
        context("Type", map(BasicType::parse, Type::Basic))(input.into())
    }
}

impl<W> Describe<W> for Type
where
    W: std::io::Write,
{
    fn describe(&self, f: &mut W) -> std::io::Result<()> {
        match self {
            Self::Basic(ty_) => ty_.describe(f),
        }
    }

    fn subnodes(&self) -> Vec<&dyn Describe<W>> {
        vec![]
    }
}

type TypeNode<M> = Node<Type, M>;

impl<M> TypeNode<M>
where
    M: Meta,
{
    fn parse<'a>(input: impl Into<LocatedSpan<&'a str>>) -> IResult<'a, Self> {
        Node::parser(Type::parse).parse(input.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ast() {
        let parsed = AST::parse("pub let variable = 15; let other = 10;").unwrap();
        assert_eq!(
            parsed,
            AST {
                module: Module {
                    bindings: vec![
                        Binding {
                            visibility: Visibility::Public.into(),
                            name: "variable",
                            ty_: None,
                            expression: Expression::Literal(ExpressionLiteral::Integral("15"))
                                .into(),
                        }
                        .into(),
                        Binding {
                            visibility: Visibility::Private.into(),
                            name: "other",
                            ty_: None,
                            expression: Expression::Literal(ExpressionLiteral::Integral("10"))
                                .into(),
                        }
                        .into()
                    ]
                }
                .into()
            }
        );
    }

    #[test]
    fn binding() {
        let (tail, parsed) = BindingNode::parse("let variable = 15;").unwrap();
        assert_eq!(*tail.fragment(), "");
        assert_eq!(
            parsed,
            Binding {
                visibility: Visibility::Private.into(),
                name: "variable",
                ty_: None,
                expression: Expression::Literal(ExpressionLiteral::Integral("15")).into()
            }
            .into()
        );

        let (tail, parsed) = BindingNode::parse("pub let other = 10;").unwrap();
        assert_eq!(*tail.fragment(), "");
        assert_eq!(
            parsed,
            Binding {
                visibility: Visibility::Public.into(),
                name: "other",
                ty_: None,
                expression: Expression::Literal(ExpressionLiteral::Integral("10")).into()
            }
            .into()
        );

        let (tail, parsed) = BindingNode::parse("let two: u32 = 2;").unwrap();
        assert_eq!(*tail.fragment(), "");
        assert_eq!(
            parsed,
            Binding {
                visibility: Visibility::Private.into(),
                name: "two",
                ty_: Some(Type::Basic(BasicType::U32).into()),
                expression: Expression::Literal(ExpressionLiteral::Integral("2")).into()
            }
            .into()
        );
    }

    #[test]
    fn expression_literal_integral() {
        let (tail, parsed) = ExpressionNode::parse("0").unwrap();
        assert_eq!(*tail.fragment(), "");
        assert_eq!(
            parsed,
            Expression::Literal(ExpressionLiteral::Integral("0")).into()
        );

        let (tail, parsed) = ExpressionNode::parse("1357").unwrap();
        assert_eq!(*tail.fragment(), "");
        assert_eq!(
            parsed,
            Expression::Literal(ExpressionLiteral::Integral("1357")).into()
        );

        let (tail, parsed) = ExpressionNode::parse("-135234").unwrap();
        assert_eq!(*tail.fragment(), "");
        assert_eq!(
            parsed,
            Expression::Literal(ExpressionLiteral::Integral("-135234")).into()
        );

        Expression::parse("bar").unwrap_err();
    }

    #[test]
    fn visibility() {
        let (tail, parsed) = VisibilityNode::parse("").unwrap();
        assert_eq!(*tail.fragment(), "");
        assert_eq!(parsed, Visibility::Private.into());

        let (tail, parsed) = VisibilityNode::parse("pub ").unwrap();
        assert_eq!(*tail.fragment(), "");
        assert_eq!(parsed, Visibility::Public.into());
    }

    #[test]
    fn basic_type() {
        let (tail, parsed) = BasicType::parse("u32").unwrap();
        assert_eq!(*tail.fragment(), "");
        assert_eq!(parsed, BasicType::U32);
    }

    #[test]
    fn type_() {
        let (tail, parsed) = TypeNode::parse("u32").unwrap();
        assert_eq!(*tail.fragment(), "");
        assert_eq!(parsed, Type::Basic(BasicType::U32).into());
    }
}
