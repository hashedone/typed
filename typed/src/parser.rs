use anyhow::{anyhow, ensure, Context, Result};
use nom::{error::convert_error, Finish};

use self::root::Root;

pub mod binding;
pub mod expression;
pub mod fn_appl;
pub mod fn_decl;
pub mod ident;
pub mod literal;
pub mod root;

/// Description of Ast parsed from input. Lifetime is lifetime of the input.
pub struct Ast<'a> {
    pub root: Root<'a>,
}

impl<'a> Ast<'a> {
    /// Parses an AST from a string
    pub fn parse(input: &'a str) -> Result<Self> {
        let (tail, root) = Root::parse(input)
            .map_err(nom::Err::<nom::error::Error<&str>>::to_owned)
            .finish()?;
        ensure!(tail.is_empty(), "Unexpected trailing characters");
        Ok(Self { root })
    }

    /// Parses an AST from a string with custom `ParseError`
    pub fn parse_verbose(input: &'a str) -> Result<Self> {
        let (tail, root) = Root::parse(input)
            .finish()
            .map_err(|e| anyhow!("{}", convert_error(input, e)))
            .context("Parsing failure")?;
        ensure!(tail.is_empty(), "Unexpected trailing characters");
        Ok(Self { root })
    }
}
