use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash, AstNode)]
#[ast(kind = FUNCTION_DEFINITION)]
pub struct FunctionDefinition {
    pub(crate) syntax: SyntaxNode,
}

impl FunctionDefinition {
    pub fn name(&self) -> Option<NameDef> {
        support::child(self.syntax())
    }

    pub fn params(&self) -> Option<FunctionDefinitionParams> {
        support::child(self.syntax())
    }

    pub fn preserves(&self) -> Option<FunctionDefinitionPreserves> {
        support::child(self.syntax())
    }

    pub fn instruction_block_set(&self) -> Option<InstructionsBlockSet> {
        support::child(self.syntax())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, AstNode)]
#[ast(kind = FUNCTION_DEFINITION_PARAMS)]
pub struct FunctionDefinitionParams {
    pub(crate) syntax: SyntaxNode,
}

impl FunctionDefinitionParams {
    pub fn params(&self) -> AstChildren<FunctionDefinitionParam> {
        support::children(self.syntax())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, AstNode)]
#[ast(kind = FUNCTION_DEFINITION_PARAM)]
pub struct FunctionDefinitionParam {
    pub(crate) syntax: SyntaxNode,
}

impl FunctionDefinitionParam {
    pub fn value(&self) -> Option<RegisterIdent> {
        support::token(self.syntax())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, AstNode)]
#[ast(kind = FUNCTION_DEFINITION_PRESERVES)]
pub struct FunctionDefinitionPreserves {
    pub(crate) syntax: SyntaxNode,
}

pub enum RegisterRangeKind {
    Single(RegisterIdent),
    Range(RegisterIdent, RegisterIdent),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, AstNode)]
#[ast(kind = REGISTER_RANGE)]
pub struct RegisterRange {
    pub(crate) syntax: SyntaxNode,
}

impl RegisterRange {
    pub fn kind(&self) -> Option<RegisterRangeKind> {
        let mut iter = self
            .syntax()
            .children_with_tokens()
            .filter_map(rowan::NodeOrToken::into_token)
            .filter_map(RegisterIdent::cast);

        let first = iter.next()?;
        match iter.next() {
            None => Some(RegisterRangeKind::Single(first)),
            Some(second) => Some(RegisterRangeKind::Range(first, second)),
        }
    }
}
