#![allow(non_snake_case)]

mod expr;
mod items;

pub use expr::*;
pub use items::*;

use super::tokens::*;
use crate::syntax::{
    ast::{support, AstChildren, AstNode, AstSpanned, AstToken},
    SyntaxKind::*,
    SyntaxNode, SyntaxToken, T,
};

/// `SourceFile` represents a parse tree for a single Rust file.
#[derive(Debug, Clone, PartialEq, Eq, Hash, AstNode)]
#[ast(kind = SOURCE_FILE)]
pub struct SourceFile {
    pub(crate) syntax: SyntaxNode,
}

impl SourceFile {
    pub fn items(&self) -> AstChildren<Item> {
        support::children(self.syntax())
    }

    // pub fn numbered_items(&self) -> impl Iterator<Item = (ItemNumber, Item)> + '_ {
    //     self.items().enumerate().map(|(i, item)| (i.into(), item))
    // }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, AstNode)]
#[ast(kind = NAME_DEF)]
pub struct NameDef {
    pub(crate) syntax: SyntaxNode,
}

impl NameDef {
    pub fn token(&self) -> Option<Ident> {
        support::token(self.syntax())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, AstNode)]
#[ast(kind = REGISTER_NAME_DEF)]
pub struct RegisterNameDef {
    pub(crate) syntax: SyntaxNode,
}

impl RegisterNameDef {
    pub fn token(&self) -> Option<RegisterIdent> {
        support::token(self.syntax())
    }
}
