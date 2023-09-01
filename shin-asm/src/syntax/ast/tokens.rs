use crate::syntax::{
    ast::{AstToken, SyntaxToken},
    SyntaxKind::{self, *},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, AstToken)]
#[ast(kind = IDENT)]
pub struct Ident {
    pub(crate) syntax: SyntaxToken,
}

pub enum RegisterIdentKind<'a> {
    Argument(u32),
    Value(u32),
    Alias(&'a str),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, AstToken)]
#[ast(kind = REGISTER_IDENT)]
pub struct RegisterIdent {
    pub(crate) syntax: SyntaxToken,
}

impl RegisterIdent {
    pub fn kind(&self) -> RegisterIdentKind {
        // TODO: we want to validate the register ident somewhere
        todo!()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, AstToken)]
#[ast(kind = L_PAREN)]
pub struct LParen {
    pub(crate) syntax: SyntaxToken,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, AstToken)]
#[ast(kind = R_PAREN)]
pub struct RParen {
    pub(crate) syntax: SyntaxToken,
}
