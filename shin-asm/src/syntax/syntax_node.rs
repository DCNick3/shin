//! This module defines Concrete Syntax Tree (CST), used by shin-asm.
//!
//! The CST includes comments and whitespace, provides a single node type,
//! `SyntaxNode`, and a basic traversal API (parent, children, siblings).
//!
//! The *real* implementation is in the (language-agnostic) `rowan` crate, this
//! module just wraps its API.

use rowan::{GreenNodeBuilder, Language};

use crate::parser::SyntaxKind;
use crate::syntax::{Parse, SyntaxError};
pub(crate) use rowan::{GreenNode, GreenToken, NodeOrToken};
use text_size::TextSize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SalLanguage {}
impl Language for SalLanguage {
    type Kind = SyntaxKind;

    fn kind_from_raw(raw: rowan::SyntaxKind) -> SyntaxKind {
        SyntaxKind::from(raw.0)
    }

    fn kind_to_raw(kind: SyntaxKind) -> rowan::SyntaxKind {
        rowan::SyntaxKind(kind.into())
    }
}

pub type SyntaxNode = rowan::SyntaxNode<SalLanguage>;
pub type SyntaxToken = rowan::SyntaxToken<SalLanguage>;
pub type SyntaxElement = rowan::SyntaxElement<SalLanguage>;
pub type SyntaxNodeChildren = rowan::SyntaxNodeChildren<SalLanguage>;
pub type SyntaxElementChildren = rowan::SyntaxElementChildren<SalLanguage>;
pub type PreorderWithTokens = rowan::api::PreorderWithTokens<SalLanguage>;

#[derive(Default)]
pub struct SyntaxTreeBuilder {
    errors: Vec<SyntaxError>,
    inner: GreenNodeBuilder<'static>,
}

impl SyntaxTreeBuilder {
    pub(crate) fn finish_raw(self) -> (GreenNode, Vec<SyntaxError>) {
        let green = self.inner.finish();
        (green, self.errors)
    }

    pub fn finish(self) -> Parse<SyntaxNode> {
        let (green, errors) = self.finish_raw();
        // Disable block validation, see https://github.com/rust-lang/rust-analyzer/pull/10357
        Parse::new(green, errors)
    }

    pub fn token(&mut self, kind: SyntaxKind, text: &str) {
        let kind = SalLanguage::kind_to_raw(kind);
        self.inner.token(kind, text);
    }

    pub fn start_node(&mut self, kind: SyntaxKind) {
        let kind = SalLanguage::kind_to_raw(kind);
        self.inner.start_node(kind);
    }

    pub fn finish_node(&mut self) {
        self.inner.finish_node();
    }

    pub fn error(&mut self, error: String, text_pos: TextSize) {
        self.errors
            .push(SyntaxError::new_at_offset(error, text_pos));
    }
}
