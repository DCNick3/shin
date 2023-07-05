use shin_derive::syntax_kind;

syntax_kind! {
    technical: [
        EOF,
        TOMBSTONE,
    ],
    punct: {
        NEWLINE => "\n",
        COMMA => ",",
        L_PAREN => "(",
        R_PAREN => ")",
        L_CURLY => "{",
        R_CURLY => "}",
        L_BRACK => "[",
        R_BRACK => "]",
        L_ANGLE => "<",
        R_ANGLE => ">",
        AT => "@",
        TILDE => "~",
        AMP => "&",
        PIPE => "|",
        PLUS => "+",
        STAR => "*",
        SLASH => "/",
        CARET => "^",
        PERCENT => "%",
        DOT => ".",
        DOT_SLASH => "./",
        DOT_STAR => ".*",
        COLON => ":",
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
    keywords: {
        MOD_KW => "mod",
        FUNCTION_KW => "function",
        ENDFUN_KW => "endfun",
        SUBROUTINE_KW => "subroutine",
        ENDSUB_KW => "endsub",
    },
    literals: [
        INT_NUMBER,
        FLOAT_NUMBER,
        STRING,
    ],
    tokens: [
        ERROR,
        IDENT,
        REGISTER_IDENT,
        WHITESPACE,
        COMMENT,
    ],
    nodes: [
        SOURCE_FILE,
        FUNCTION,
        NAME,

        REGISTER_SET,
        REGISTER_RANGE,

        LABEL,
        INSTRUCTION,

        INSTRUCTIONS_BLOCK,

        INSTR_ARG_LIST,
        CALL_ARG_LIST,

        NAME_REF_EXPR,
        REGISTER_REF_EXPR,

        CALL_EXPR,
        ARRAY_EXPR,
        MAPPING_EXPR,
        PREFIX_EXPR,
        BIN_EXPR,
        LITERAL,

        MAPPING_ENTRY,
    ],
}

impl SyntaxKind {
    #[inline]
    pub fn is_trivia(self) -> bool {
        matches!(self, SyntaxKind::WHITESPACE | SyntaxKind::COMMENT)
    }

    #[inline]
    pub fn is_any_opening_bracket(self) -> bool {
        matches!(self, T!['('] | T!['['] | T!['{'])
    }

    #[inline]
    pub fn is_any_closing_bracket(self) -> bool {
        matches!(self, T![')'] | T![']'] | T!['}'])
    }

    #[inline]
    pub fn matching_closing_bracket(self) -> Option<SyntaxKind> {
        match self {
            T!['('] => Some(T![')']),
            T!['['] => Some(T![']']),
            T!['{'] => Some(T!['}']),
            _ => None,
        }
    }
}
