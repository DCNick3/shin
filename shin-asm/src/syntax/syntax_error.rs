//! See docs for `SyntaxError`.

use text_size::{TextRange, TextSize};

use crate::compile::diagnostics::Diagnostic;

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
        Self(
            message.into(),
            // A hack to get syntax errors to show up
            TextRange::at(offset, TextSize::new(1)),
        )
    }

    pub fn range(&self) -> TextRange {
        self.1
    }

    pub fn with_range(mut self, range: TextRange) -> Self {
        self.1 = range;
        self
    }

    pub fn into_diagnostic(self) -> Diagnostic<TextRange> {
        Diagnostic::new(self.0, self.1)
    }
}
