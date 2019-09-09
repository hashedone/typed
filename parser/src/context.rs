use ast::Definition;

/// Context of expression evaluation, eg. all definitions
/// available for this evaluation
pub struct Context<'a> {
    parent: Option<&'a Context<'a>>,
    defs: &'a [Definition],
}

impl<'a> Context<'a> {
    pub fn new(defs: &'a [Definition]) -> Self {
        Self {
            parent: None,
            defs,
        }
    }

    pub fn extend(&'a self, defs: &'a [Definition]) -> Self {
        Self {
            parent: Some(self),
            defs,
        }
    }
}

impl<'a> Default for Context<'a> {
    fn default() -> Self {
        Self {
            parent: None,
            defs: &[],
        }
    }
}
