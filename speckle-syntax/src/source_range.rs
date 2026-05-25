use proc_macro2::Span;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceRange {
    pub file: String,
    pub byte_start: usize,
    pub byte_end: usize,
}

impl From<Span> for SourceRange {
    fn from(span: Span) -> Self {
        let bytes = span.byte_range();
        Self {
            file: span.file(),
            byte_start: bytes.start,
            byte_end: bytes.end,
        }
    }
}
