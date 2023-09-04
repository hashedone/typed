use nom::character::complete::satisfy;
use nom::combinator::recognize;
use nom::error::ParseError;
use nom::multi::fold_many0;
use nom::sequence::preceded;
use nom::IResult;

/// Nom parser for identifier
///
/// Identifiers are any utf-8 strings starting with a letter, underscore, or non-ascii character,
/// and doesn't contain whitespaces.
pub fn ident<'a, E>(input: &'a str) -> IResult<&'a str, &'a str, E>
where
    E: ParseError<&'a str>,
{
    let fst = satisfy(|c| (!c.is_ascii() || c == '_' || c.is_alphabetic()) && !c.is_whitespace());
    let tail = satisfy(|c| !c.is_ascii() || c == '_' || c.is_alphanumeric());

    // Parses a tail discarding the result - we will use `recognize`
    // parser to get the result directly from an input to avoid cloning
    let tail = fold_many0(tail, || (), |_, _| ());

    let parser = preceded(fst, tail);
    let mut parser = recognize(parser);

    parser(input)
}

#[cfg(test)]
mod tests {
    use nom::Finish;

    use super::*;

    type Err<'a> = nom::error::Error<&'a str>;

    #[test]
    fn simple_ident() {
        let (tail, res) = ident::<Err>("ident").finish().unwrap();
        assert_eq!(tail, "");
        assert_eq!(res, "ident");
    }

    #[test]
    fn underscore_start() {
        let (tail, res) = ident::<Err>("_123").finish().unwrap();
        assert_eq!(tail, "");
        assert_eq!(res, "_123");
    }

    #[test]
    fn emoji() {
        let (tail, res) = ident::<Err>("😀").finish().unwrap();
        assert_eq!(tail, "");
        assert_eq!(res, "😀");
    }

    #[test]
    fn number_start() {
        ident::<Err>("123").finish().unwrap_err();
    }
}
