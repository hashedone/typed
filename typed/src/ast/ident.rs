use nom::IResult;
use nom::character::complete::satisfy;
use nom::combinator::recognize;
use nom::multi::fold_many0;
use nom::sequence::preceded;

/// Nom parser for identifier
/// 
/// Identifiers are any utf-8 strings starting with a letter, underscore, or non-ascii character,
/// and doesn't contain whitespaces.
pub fn ident(input: &str) -> IResult<&str, &str> {
    let fst = satisfy(|c| (!c.is_ascii() || c == '_') && !c.is_whitespace());
    let tail = satisfy(|c| !c.is_whitespace());

    // Parses a tail discarding the result - we will use `recognize`
    // parser to get the result directly from an input to avoid cloning
    let tail = fold_many0(tail, || (), |_, _| ());

    let parser = preceded(fst, tail);
    let mut parser = recognize(parser);

    parser(input)
}