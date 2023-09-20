use super::expression::Expression;

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct FnAppl {
    pub func: Expression,
    pub arg: Expression,
}
