use chumsky::prelude::*;
use chumsky::text::whitespace;

use crate::Ex;

/// Visibility
#[derive(Clone, Debug, PartialEq)]
pub enum Vis {
    /// Public visibility
    Public,
    /// Private visibility
    Private,
}

pub fn vis<'s>() -> impl Parser<'s, &'s str, Vis, Ex<'s>> + Clone {
    let public = just("pub")
        .then_ignore(whitespace().rewind())
        .to(Vis::Public)
        .labelled("pub");
    let private = empty().to(Vis::Private);

    choice((public, private))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vis() {
        let input = "pub ";
        let result = vis().padded().parse(input).unwrap();
        assert_eq!(result, Vis::Public);

        let input = "";
        let result = vis().parse(input).unwrap();
        assert_eq!(result, Vis::Private);
    }
}
