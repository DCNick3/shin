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

#[derive(Debug, Clone, PartialEq, Eq, Hash, AstNode)]
pub enum Item {
    // NOTE: this is not what we want to do with enums...
    // we should support a enum of AstNodes, not of raw syntax nodes
    #[ast(kind = INSTRUCTIONS_BLOCK)]
    InstructionsBlock(SyntaxNode),
    // FunctionDefinition(),
    // SubroutineDefinition(),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, AstNode)]
#[ast(kind = INSTRUCTIONS_BLOCK)]
pub struct InstructionsBlock {
    pub(crate) syntax: SyntaxNode,
}
