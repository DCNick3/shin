#![allow(non_snake_case)]

mod expressions;
mod items;

use super::tokens::*;
use crate::syntax::{
    ast::{self, support, AstChildren, AstNode, AstToken},
    SyntaxKind::{self, *},
    SyntaxNode, SyntaxToken, T,
};

pub use expressions::*;
pub use items::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash, AstNode)]
#[ast(kind = SOURCE_FILE)]
pub struct SourceFile {
    pub(crate) syntax: SyntaxNode,
}

impl SourceFile {
    pub fn items(&self) -> AstChildren<Item> {
        support::children(self.syntax())
    }
}
