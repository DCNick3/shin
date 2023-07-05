use crate::syntax::{
    ast::{self, support, AstChildren, AstNode},
    SyntaxKind::{self, *},
    SyntaxNode, SyntaxToken, T,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, AstNode)]
pub enum Expression {
    #[ast(transparent)]
    Literal(Literal),
    #[ast(transparent)]
    NameRefExpr(NameRefExpr),
    #[ast(transparent)]
    RegisterRefExpr(RegisterRefExpr),
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
#[ast(kind = LITERAL)]
pub struct Literal {
    pub(crate) syntax: SyntaxNode,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, AstNode)]
#[ast(kind = NAME_REF_EXPR)]
pub struct NameRefExpr {
    pub(crate) syntax: SyntaxNode,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, AstNode)]
#[ast(kind = REGISTER_REF_EXPR)]
pub struct RegisterRefExpr {
    pub(crate) syntax: SyntaxNode,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, AstNode)]
#[ast(kind = ARRAY_EXPR)]
pub struct ArrayExpr {
    pub(crate) syntax: SyntaxNode,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, AstNode)]
#[ast(kind = MAPPING_EXPR)]
pub struct MappingExpr {
    pub(crate) syntax: SyntaxNode,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, AstNode)]
#[ast(kind = BIN_EXPR)]
pub struct BinExpr {
    pub(crate) syntax: SyntaxNode,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, AstNode)]
#[ast(kind = PREFIX_EXPR)]
pub struct PrefixExpr {
    pub(crate) syntax: SyntaxNode,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, AstNode)]
#[ast(kind = CALL_EXPR)]
pub struct CallExpr {
    pub(crate) syntax: SyntaxNode,
}
