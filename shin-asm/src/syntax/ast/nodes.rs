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

impl InstructionsBlock {
    pub fn instructions(&self) -> impl Iterator<Item = Instruction> + '_ {
        support::children(self.syntax())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, AstNode)]
#[ast(kind = FUNCTION_DEFINITION)]
pub struct FunctionDefinition {
    pub(crate) syntax: SyntaxNode,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, AstNode)]
#[ast(kind = INSTRUCTION)]
pub struct Instruction {
    pub(crate) syntax: SyntaxNode,
}

impl Instruction {
    pub fn name(&self) -> Option<InstructionName> {
        support::child(self.syntax())
    }

    pub fn args(&self) -> Option<InstructionArgList> {
        support::child(self.syntax())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, AstNode)]
#[ast(kind = INSTRUCTION_NAME)]
pub struct InstructionName {
    pub(crate) syntax: SyntaxNode,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, AstNode)]
#[ast(kind = INSTR_ARG_LIST)]
pub struct InstructionArgList {
    pub(crate) syntax: SyntaxNode,
}

impl InstructionArgList {
    pub fn args(&self) -> impl Iterator<Item = Expression> + '_ {
        support::children(self.syntax())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, AstNode)]
pub enum Expression {
    #[ast(transparent)]
    Literal(Literal),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, AstNode)]
#[ast(kind = LITERAL)]
pub struct Literal {
    pub(crate) syntax: SyntaxNode,
}
