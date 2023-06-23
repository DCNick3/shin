// TODO: codegen this file, as rust-analyzer does
// we can put it into the shin-derive crate =)

use shin_derive::syntax_kind;

syntax_kind! {
    technical: [
        EOF
    ],
    punct: {
        EQ => "=",
        EQ2 => "==",
    },
    literals: [],
    tokens: [],
    nodes: [],
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
#[repr(u16)]
pub enum SyntaxKind {
    // Technical SyntaxKinds: they appear temporally during parsing,
    // but never end up in the final tree
    EOF,

    // Punctuation
    COMMA,
    L_PAREN,
    R_PAREN,
    L_CURLY,
    R_CURLY,
    L_ANGLE,
    R_ANGLE,
    L_BRACK,
    R_BRACK,
    AMP,
    PIPE,
    PLUS,
    STAR,
    SLASH,
    CARET,
    MOD,
    DOT,
    EQ,
    EQ2,
    FAT_ARROW,
    BANG,
    NEQ,
    MINUS,
    LTEQ,
    GTEQ,
    AMP2,
    PIPE2,
    SHL,
    SHR,

    // Literals
    INT_NUMBER,
    FLOAT_NUMBER,
    STRING,

    // Tokens
    ERROR,
    IDENT,
    WHITESPACE,
    COMMENT,

    // Nodes
    SOURCE_FILE,
    // TODO: add more nodes here
}

#[macro_export]
macro_rules! T {
    [,] => {
        $crate::SyntaxKind::COMMA
    };
    ['('] => {
        $crate::SyntaxKind::L_PAREN
    };
    [')'] => {
        $crate::SyntaxKind::R_PAREN
    };
    ['{'] => {
        $crate::SyntaxKind::L_CURLY
    };
    ['}'] => {
        $crate::SyntaxKind::R_CURLY
    };
    ['<'] => {
        $crate::SyntaxKind::L_ANGLE
    };
    ['>'] => {
        $crate::SyntaxKind::R_ANGLE
    };
    ['['] => {
        $crate::SyntaxKind::L_BRACK
    };
    [']'] => {
        $crate::SyntaxKind::R_BRACK
    };
    [&] => {
        $crate::SyntaxKind::AMP
    };
    [|] => {
        $crate::SyntaxKind::PIPE
    };
    [+] => {
        $crate::SyntaxKind::PLUS
    };
    [*] => {
        $crate::SyntaxKind::STAR
    };
    [/] => {
        $crate::SyntaxKind::SLASH
    };
    [^] => {
        $crate::SyntaxKind::CARET
    };
    [%] => {
        $crate::SyntaxKind::MOD
    };
    [.] => {
        $crate::SyntaxKind::DOT
    };
    [=] => {
        $crate::SyntaxKind::EQ
    };
    [==] => {
        $crate::SyntaxKind::EQ2
    };
    [=>] => {
        $crate::SyntaxKind::FAT_ARROW
    };
    [!] => {
        $crate::SyntaxKind::BANG
    };
    [!=] => {
        $crate::SyntaxKind::NEQ
    };
    [-] => {
        $crate::SyntaxKind::MINUS
    };
    [<=] => {
        $crate::SyntaxKind::LTEQ
    };
    [>=] => {
        $crate::SyntaxKind::GTEQ
    };
    [&&] => {
        $crate::SyntaxKind::AMP2
    };
    [||] => {
        $crate::SyntaxKind::PIPE2
    };
    [<<] => {
        $crate::SyntaxKind::SHL
    };
    [>>] => {
        $crate::SyntaxKind::SHR
    };
}
