mod functions;

use super::*;
use either::Either;
use smol_str::SmolStr;

pub use functions::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash, AstNode)]
pub enum Item {
    #[ast(transparent)]
    InstructionsBlockSet(InstructionsBlockSet),
    #[ast(transparent)]
    FunctionDefinition(FunctionDefinition),
    #[ast(transparent)]
    AliasDefinition(AliasDefinition),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, AstNode)]
#[ast(kind = INSTRUCTIONS_BLOCK_SET)]
pub struct InstructionsBlockSet {
    pub(crate) syntax: SyntaxNode,
}

impl InstructionsBlockSet {
    pub fn blocks(&self) -> AstChildren<InstructionsBlock> {
        support::children(self.syntax())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, AstNode)]
#[ast(kind = INSTRUCTIONS_BLOCK)]
pub struct InstructionsBlock {
    pub(crate) syntax: SyntaxNode,
}

impl InstructionsBlock {
    pub fn labels(&self) -> Option<InstructionsBlockLabels> {
        support::child(self.syntax())
    }
    pub fn body(&self) -> Option<InstructionsBlockBody> {
        support::child(self.syntax())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, AstNode)]
#[ast(kind = INSTRUCTIONS_BLOCK_LABELS)]
pub struct InstructionsBlockLabels {
    pub(crate) syntax: SyntaxNode,
}

impl InstructionsBlockLabels {
    pub fn labels(&self) -> AstChildren<Label> {
        support::children(self.syntax())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, AstNode)]
#[ast(kind = INSTRUCTIONS_BLOCK_BODY)]
pub struct InstructionsBlockBody {
    pub(crate) syntax: SyntaxNode,
}

impl InstructionsBlockBody {
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

    pub fn value(&self) -> Option<SmolStr> {
        self.token().map(|t| t.text().into())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, AstNode)]
#[ast(kind = INSTR_ARG_LIST)]
pub struct InstructionArgList {
    pub(crate) syntax: SyntaxNode,
}

impl InstructionArgList {
    pub fn args(&self) -> AstChildren<Expr> {
        support::children(self.syntax())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, AstNode)]
#[ast(kind = LABEL)]
pub struct Label {
    pub(crate) syntax: SyntaxNode,
}

impl Label {
    pub fn name(&self) -> Option<Ident> {
        support::token(self.syntax())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, AstNode)]
#[ast(kind = ALIAS_DEFINITION)]
pub struct AliasDefinition {
    pub(crate) syntax: SyntaxNode,
}

impl AliasDefinition {
    pub fn name(&self) -> Option<Either<NameDef, RegisterNameDef>> {
        support::child(self.syntax())
    }

    pub fn value(&self) -> Option<Expr> {
        support::child(self.syntax())
    }
}
