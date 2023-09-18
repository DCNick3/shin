mod register;

use crate::syntax::{
    ast::{AstSpanned, AstToken, SyntaxToken},
    SyntaxKind::*,
};

pub use register::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash, AstToken)]
#[ast(kind = IDENT)]
pub struct Ident {
    pub(crate) syntax: SyntaxToken,
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
