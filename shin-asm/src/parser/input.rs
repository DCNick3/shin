//! See [`Input`].

use crate::parser::SyntaxKind;

/// Input for the parser -- a sequence of tokens.
///
/// As of now, parser doesn't have access to the *text* of the tokens, and makes
/// decisions based solely on their classification. Unlike `LexerToken`, the
/// `Tokens` doesn't include whitespace and comments. Main input to the parser.
///
/// Struct of arrays internally, but this shouldn't really matter.
#[derive(Default)]
pub struct Input {
    kind: Vec<SyntaxKind>,
}

/// `pub` impl used by callers to create `Tokens`.
impl Input {
    #[inline]
    pub fn push(&mut self, kind: SyntaxKind) {
        self.kind.push(kind)
    }
}

/// pub(crate) impl used by the parser to consume `Tokens`.
impl Input {
    pub(crate) fn kind(&self, idx: usize) -> SyntaxKind {
        self.kind.get(idx).copied().unwrap_or(SyntaxKind::EOF)
    }
}
