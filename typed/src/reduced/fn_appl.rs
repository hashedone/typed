use super::ExprId;

#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq)]
pub struct FnAppl {
    pub func: ExprId,
    pub arg: ExprId,
}
