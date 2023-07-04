#![allow(non_snake_case)]
use crate::syntax::{
    ast::{self, support, AstChildren, AstNode},
    SyntaxKind::{self, *},
    SyntaxNode, SyntaxToken, T,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SourceFile {
    pub(crate) syntax: SyntaxNode,
}

impl AstNode for SourceFile {
    fn can_cast(kind: SyntaxKind) -> bool
    where
        Self: Sized,
    {
        kind == SOURCE_FILE
    }

    fn cast(syntax: SyntaxNode) -> Option<Self>
    where
        Self: Sized,
    {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }

    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
