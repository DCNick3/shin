mod functions;

use super::*;

pub use functions::*;

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
    pub fn instructions(&self) -> AstChildren<Instruction> {
        support::children(self.syntax())
    }
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

impl InstructionName {
    pub fn token(&self) -> Option<Ident> {
        support::token(self.syntax())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, AstNode)]
#[ast(kind = INSTR_ARG_LIST)]
pub struct InstructionArgList {
    pub(crate) syntax: SyntaxNode,
}

impl InstructionArgList {
    pub fn args(&self) -> AstChildren<Expression> {
        support::children(self.syntax())
    }
}
