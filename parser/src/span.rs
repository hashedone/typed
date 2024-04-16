use std::ops::Range;

#[derive(Debug, Clone, PartialEq)]
pub struct Span(Range<usize>);

impl Span {
    pub fn range(&self) -> &Range<usize> {
        &self.0
    }
}

impl From<Range<usize>> for Span {
    fn from(value: Range<usize>) -> Self {
        Self(value)
    }
}
