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

    pub fn find(&self, name: &str) -> Option<&'a Definition> {
        self.defs
            .iter()
            .find(|def| def.name() == name)
            .or_else(|| self.parent.clone().and_then(|ctx| ctx.find(name)))
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
