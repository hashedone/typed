use std::marker::PhantomData;
use std::ops::{Deref, Range};

use ariadne::{Report, ReportKind};

pub enum Error {
    MatchFailure {
        expected: Vec<&'static str>,
        offset: usize,
    },
}

impl Error {
    pub fn match_failure(expected: &'static str, offset: usize) -> Self {
        Self::MatchFailure {
            expected: vec![expected],
            offset,
        }
    }

    fn match_failure_report(source: &str, expected: &[&str], offset: usize) -> Report<'static> {
        Report::build(ReportKind::Error, source, offset)
            .with_code(1)
            .with_message("Unexpected token".to_owned())
            .with_note(format!("Expected one of: {}", expected.join(", ")))
            .finish()
    }

    pub fn prepare_report(&self, source: &str) -> Report<'static> {
        match self {
            Self::MatchFailure { expected, offset } => {
                Self::match_failure_report(source, expected, *offset)
            }
        }
    }
}
struct Context {
    source_name: &'static str,
    errors: Vec<Error>,
}

pub struct Spanned<T> {
    node: T,
    span: Range<usize>,
}

impl<T> Deref for Spanned<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.node
    }
}
pub struct Input<'a> {
    data: &'a str,
    offset: usize,
}

impl<'a> Input<'a> {
    pub fn new(data: &'a str) -> Self {
        Self { data, offset: 0 }
    }

    fn offset(&self) -> usize {
        self.offset
    }

    fn consume_while(&self, predicate: impl Fn(char) -> bool) -> (Spanned<&'a str>, Self) {
        let len = self.data.chars().take_while(|c| predicate(*c)).count();
        let node = &self.data[..len];
        let span = self.offset..self.offset + len;
        let token = Spanned { node, span };
        let input = Input {
            data: &self.data[len..],
            offset: self.offset + len,
        };

        (token, input)
    }
}

impl Deref for Input<'_> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.data
    }
}

pub trait Parser<'a> {
    type Output: 'a;

    fn parse(
        &self,
        context: &mut Context,
        input: Input<'a>,
    ) -> Result<(Input<'a>, Spanned<Self::Output>), Error>;
}

pub struct FnParser<O, F>(F, PhantomData<O>);

pub fn from_fn<'a, O, F> (f: F) -> FnParser<O, F> where F: Fn(&mut Context, Input<'a>) -> Result<(Input<'a>, Spanned<O>), Error>, O: 'a{
    FnParser(f, PhantomData)
}

impl<'a, O, P> Parser<'a> for FnParser<O, P>
where
    P: Fn(&mut Context, Input<'a>) -> Result<(Input<'a>, Spanned<O>), Error>,
    O: 'a,
{
    type Output = O;

    fn parse(
        &self,
        context: &mut Context,
        input: Input<'a>,
    ) -> Result<(Input<'a>, Spanned<Self::Output>), Error> {
        self.0(context, input)
    }
}

impl<'a, O> Parser<'a> for fn(&mut Context, Input<'a>) -> Result<(Input<'a>, Spanned<O>), Error>
where
    O: 'a,
{
    type Output = O;

    fn parse(
        &self,
        context: &mut Context,
        input: Input<'a>,
    ) -> Result<(Input<'a>, Spanned<Self::Output>), Error> {
        self(context, input)
    }
}

fn parse_id<'a>(_context: &mut Context, input: Input<'a>) -> Result<(Input<'a>, &'a str), Error> {
    let head = verify(anychar, |c| c.is_alphabetic() || *c == '_');
    let tail = take_while1(|c: char| c.is_alphanumeric() || c == '_');
    map(recognize(tuple((head, tail))), |id| (id, vec![]))(input)
}
