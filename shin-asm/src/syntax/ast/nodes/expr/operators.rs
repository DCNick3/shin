use super::*;

//         T![||]   => (3,  T![||],  Left),
//         T![|]    => (6,  T![|],   Left),
//         T![>>]   => (9,  T![>>],  Left),
//         T![>=]   => (5,  T![>=],  Left),
//         T![>]    => (5,  T![>],   Left),
//         T![==]   => (5,  T![==],  Left),
//         T![<=]   => (5,  T![<=],  Left),
//         T![<<]   => (9,  T![<<],  Left),
//         T![<]    => (5,  T![<],   Left),
//         T![+]    => (10, T![+],   Left),
//         T![^]    => (7,  T![^],   Left),
//         T![mod]  => (11, T![mod], Left),
//         T![&&]   => (4,  T![&&],  Left),
//         T![&]    => (8,  T![&],   Left),
//         T![/]    => (11, T![/],   Left),
//         T![*]    => (11, T![*],   Left),
//         T![./]   => (11, T![./],  Left),
//         T![.*]   => (11, T![.*],  Left),
//         T![!=]   => (5,  T![!=],  Left),
//         T![-]    => (10, T![-],   Left),
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum BinaryOp {
    /// `||`
    LogicalOr,
    /// `|`
    BitwiseOr,
    /// `>>`
    ShiftRight,
    /// `>=`
    GreaterThanOrEqual,
    /// `>`
    GreaterThan,
    /// `==`
    Equal,
    /// `<=`
    LessThanOrEqual,
    /// `<<`
    ShiftLeft,
    /// `<`
    LessThan,
    /// `+`
    Add,
    /// `^`
    BitwiseXor,
    /// `mod`
    Modulo,
    /// `&&`
    LogicalAnd,
    /// `&`
    BitwiseAnd,
    /// `/`
    Divide,
    /// `*`
    Multiply,
    /// `./`
    DivideReal,
    /// `.*`
    MultiplyReal,
    /// `!=`
    NotEqual,
    /// `-`
    Subtract,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, AstNode)]
#[ast(kind = BIN_EXPR)]
pub struct BinExpr {
    pub(crate) syntax: SyntaxNode,
}

impl BinExpr {
    pub fn op_details(&self) -> Option<(SyntaxToken, BinaryOp)> {
        self.syntax()
            .children_with_tokens()
            .filter_map(|it| it.into_token())
            .find_map(|c| {
                let bin_op = match c.kind() {
                    T![||] => BinaryOp::LogicalOr,
                    T![|] => BinaryOp::BitwiseOr,
                    T![>>] => BinaryOp::ShiftRight,
                    T![>=] => BinaryOp::GreaterThanOrEqual,
                    T![>] => BinaryOp::GreaterThan,
                    T![==] => BinaryOp::Equal,
                    T![<=] => BinaryOp::LessThanOrEqual,
                    T![<<] => BinaryOp::ShiftLeft,
                    T![<] => BinaryOp::LessThan,
                    T![+] => BinaryOp::Add,
                    T![^] => BinaryOp::BitwiseXor,
                    T![mod] => BinaryOp::Modulo,
                    T![&&] => BinaryOp::LogicalAnd,
                    T![&] => BinaryOp::BitwiseAnd,
                    T![/] => BinaryOp::Divide,
                    T![*] => BinaryOp::Multiply,
                    T![./] => BinaryOp::DivideReal,
                    T![.*] => BinaryOp::MultiplyReal,
                    T![!=] => BinaryOp::NotEqual,
                    T![-] => BinaryOp::Subtract,

                    _ => return None,
                };
                Some((c, bin_op))
            })
    }

    pub fn op_kind(&self) -> Option<BinaryOp> {
        self.op_details().map(|t| t.1)
    }

    pub fn op_token(&self) -> Option<SyntaxToken> {
        self.op_details().map(|t| t.0)
    }

    pub fn lhs(&self) -> Option<Expr> {
        support::children(self.syntax()).next()
    }

    pub fn rhs(&self) -> Option<Expr> {
        support::children(self.syntax()).nth(1)
    }

    pub fn sub_exprs(&self) -> (Option<Expr>, Option<Expr>) {
        let mut children = support::children(self.syntax());
        let first = children.next();
        let second = children.next();
        (first, second)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum UnaryOp {
    /// `-`
    Negate,
    /// `!`
    LogigalNot,
    /// `~`
    BitwiseNot,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, AstNode)]
#[ast(kind = PREFIX_EXPR)]
pub struct PrefixExpr {
    pub(crate) syntax: SyntaxNode,
}

impl PrefixExpr {
    pub fn expr(&self) -> Option<Expr> {
        support::child(&self.syntax)
    }
    pub fn op_kind(&self) -> Option<UnaryOp> {
        let res = match self.op_token()?.kind() {
            T![-] => UnaryOp::Negate,
            T![!] => UnaryOp::LogigalNot,
            T![~] => UnaryOp::BitwiseNot,
            _ => return None,
        };
        Some(res)
    }

    pub fn op_token(&self) -> Option<SyntaxToken> {
        self.syntax().first_child_or_token()?.into_token()
    }
}
