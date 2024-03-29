mod literal;
mod operators;

pub use literal::*;
pub use operators::*;

use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash, AstNode)]
pub enum Expr {
    #[ast(transparent)]
    Literal(Literal),
    #[ast(transparent)]
    NameRefExpr(NameRefExpr),
    #[ast(transparent)]
    RegisterRefExpr(RegisterRefExpr),
    #[ast(transparent)]
    ParenExpr(ParenExpr),
    #[ast(transparent)]
    ArrayExpr(ArrayExpr),
    #[ast(transparent)]
    MappingExpr(MappingExpr),
    #[ast(transparent)]
    BinExpr(BinExpr),
    #[ast(transparent)]
    PrefixExpr(PrefixExpr),
    #[ast(transparent)]
    CallExpr(CallExpr),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, AstNode)]
#[ast(kind = NAME_REF_EXPR)]
pub struct NameRefExpr {
    pub(crate) syntax: SyntaxNode,
}

impl NameRefExpr {
    pub fn ident(&self) -> Option<Ident> {
        support::token(self.syntax())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, AstNode)]
#[ast(kind = REGISTER_REF_EXPR)]
pub struct RegisterRefExpr {
    pub(crate) syntax: SyntaxNode,
}

impl RegisterRefExpr {
    pub fn value(&self) -> RegisterIdent {
        support::token(self.syntax())
            .expect("BUG: register ref expr should have register ident inside")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, AstNode)]
#[ast(kind = PAREN_EXPR)]
pub struct ParenExpr {
    pub(crate) syntax: SyntaxNode,
}

impl ParenExpr {
    pub fn l_paren_token(&self) -> Option<LParen> {
        support::token(self.syntax())
    }
    pub fn expr(&self) -> Option<Expr> {
        support::child(self.syntax())
    }
    pub fn r_paren_token(&self) -> Option<RParen> {
        support::token(self.syntax())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, AstNode)]
#[ast(kind = ARRAY_EXPR)]
pub struct ArrayExpr {
    pub(crate) syntax: SyntaxNode,
}

impl ArrayExpr {
    pub fn values(&self) -> AstChildren<Expr> {
        support::children(self.syntax())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, AstNode)]
#[ast(kind = MAPPING_EXPR)]
pub struct MappingExpr {
    pub(crate) syntax: SyntaxNode,
}

impl MappingExpr {
    pub fn arms(&self) -> AstChildren<MappingEntry> {
        support::children(self.syntax())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, AstNode)]
#[ast(kind = MAPPING_ENTRY)]
pub struct MappingEntry {
    pub(crate) syntax: SyntaxNode,
}

impl MappingEntry {
    pub fn key(&self) -> Option<IntNumber> {
        support::token(self.syntax())
    }

    pub fn body(&self) -> Option<Expr> {
        support::child(self.syntax())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, AstNode)]
#[ast(kind = CALL_EXPR)]
pub struct CallExpr {
    pub(crate) syntax: SyntaxNode,
}
