//! See docs for `SyntaxError`.

use std::error::Error;
use std::fmt;

use text_size::{TextRange, TextSize};

/// Represents the result of unsuccessful tokenization, parsing
/// or tree validation.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SyntaxError(String, TextRange);

// FIXME: there was an unused SyntaxErrorKind previously (before this enum was removed)
// It was introduced in this PR: https://github.com/rust-lang/rust-analyzer/pull/846/files#diff-827da9b03b8f9faa1bade5cdd44d5dafR95
// but it was not removed by a mistake.
//
// So, we need to find a place where to stick validation for attributes in match clauses.
// Code before refactor:
// InvalidMatchInnerAttr => {
//    write!(f, "Inner attributes are only allowed directly after the opening brace of the match expression")
// }

impl SyntaxError {
    pub fn new(message: impl Into<String>, range: TextRange) -> Self {
        Self(message.into(), range)
    }
    pub fn new_at_offset(message: impl Into<String>, offset: TextSize) -> Self {
        Self(message.into(), TextRange::empty(offset))
    }

    pub fn range(&self) -> TextRange {
        self.1
    }

    pub fn with_range(mut self, range: TextRange) -> Self {
        self.1 = range;
        self
    }
}

impl fmt::Display for SyntaxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Error for SyntaxError {}

impl miette::Diagnostic for SyntaxError {
    fn severity(&self) -> Option<miette::Severity> {
        Some(miette::Severity::Error)
    }

    fn labels(&self) -> Option<Box<dyn Iterator<Item = miette::LabeledSpan> + '_>> {
        let Self(_, range) = self;

        let start = range.start().into();
        let len: usize = range.len().into();
        // let len = len.max(1);

        Some(Box::new(std::iter::once(miette::LabeledSpan::new(
            None, start, len,
        ))))
    }
}
