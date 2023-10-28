mod register;

pub use register::*;

use crate::syntax::{
    ast::{AstSpanned, AstToken, SyntaxToken},
    SyntaxKind::*,
};

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
