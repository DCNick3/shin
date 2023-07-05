#![allow(non_snake_case)]

use crate::syntax::{
    ast::{self, support, AstChildren, AstNode},
    SyntaxKind::{self, *},
    SyntaxNode, SyntaxToken, T,
};
use shin_derive::AstNode;

#[derive(Debug, Clone, PartialEq, Eq, Hash, AstNode)]
#[ast(kind = SOURCE_FILE)]
pub struct SourceFile {
    pub(crate) syntax: SyntaxNode,
}

impl SourceFile {
    pub fn items(&self) -> impl Iterator<Item = Item> + '_ {
        support::children(self.syntax())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, AstNode)]
pub enum Item {
    #[ast(transparent)]
    InstructionsBlock(InstructionsBlock),
    #[ast(transparent)]
    FunctionDefinition(FunctionDefinition),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, AstNode)]
#[ast(kind = INSTRUCTIONS_BLOCK)]
pub struct InstructionsBlock {
    pub(crate) syntax: SyntaxNode,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, AstNode)]
#[ast(kind = FUNCTION_DEFINITION)]
pub struct FunctionDefinition {
    pub(crate) syntax: SyntaxNode,
}
