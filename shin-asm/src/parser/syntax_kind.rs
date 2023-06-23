use shin_derive::syntax_kind;

syntax_kind! {
    technical: [
        EOF
    ],
    punct: {
        COMMA => ",",
        L_PAREN => "(",
        R_PAREN => ")",
        L_CURLY => "{",
        R_CURLY => "}",
        L_ANGLE => "<",
        R_ANGLE => ">",
        L_BRACK => "[",
        R_BRACK => "]",
        AMP => "&",
        PIPE => "|",
        PLUS => "+",
        STAR => "*",
        SLASH => "/",
        CARET => "^",
        MOD => "%",
        DOT => ".",
        EQ => "=",
        EQ2 => "==",
        FAT_ARROW => "=>",
        BANG => "!",
        NEQ => "!=",
        MINUS => "-",
        LTEQ => "<=",
        GTEQ => ">=",
        AMP2 => "&&",
        PIPE2 => "||",
        SHL => "<<",
        SHR => ">>",
    },
    literals: [
        INT_NUMBER,
        FLOAT_NUMBER,
        STRING,
    ],
    tokens: [
        ERROR,
        IDENT,
        WHITESPACE,
        COMMENT,
    ],
    nodes: [
        SOURCE_FILE,
        // TODO: add more nodes here
    ],
}
