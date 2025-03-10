mod char_classes;
mod cursor;
#[cfg(test)]
mod tests;

use std::ops;

use crate::parser::{
    lex::{
        char_classes::{is_id_continue, is_id_start, is_whitespace},
        cursor::Cursor,
    },
    SyntaxKind,
    SyntaxKind::*,
    T,
};

pub struct LexedStr<'a> {
    text: &'a str,
    kind: Vec<SyntaxKind>,
    start: Vec<u32>,
    error: Vec<LexError>,
}

struct LexError {
    msg: String,
    token: u32,
}

impl<'a> LexedStr<'a> {
    pub fn new(text: &'a str) -> LexedStr<'a> {
        let mut conv = LexedStrBuilder::new(text);

        let mut cursor = Cursor::new(text);

        let token_iter = std::iter::from_fn(|| {
            let token = cursor.advance_token();
            (token.kind != EOF).then_some(token)
        });

        let mut bracket_stack = Vec::new();

        for mut token in token_iter {
            let token_text = &text[conv.offset..][..token.len as usize];

            if let Some(closing_bracket) = token.kind.matching_closing_bracket() {
                bracket_stack.push(closing_bracket);
            }

            if token.kind.is_any_closing_bracket() {
                if let Some(expected_bracket) = bracket_stack.pop() {
                    if expected_bracket != token.kind {
                        assert!(token.error.is_none());
                        token.error = Some("unexpected closing bracket kind");
                    }
                } else {
                    assert!(token.error.is_none());
                    token.error = Some("unexpected closing bracket");
                }
            }

            if !bracket_stack.is_empty() && token.kind == NEWLINE {
                // demote newlines inside brackets to whitespace
                conv.push(WHITESPACE, token_text.len(), None);
            } else {
                conv.push(token.kind, token_text.len(), token.error)
            }
        }

        let err = (!bracket_stack.is_empty()).then_some("unclosed bracket");

        conv.finalize_with_eof(err)
    }

    // pub fn single_token(text: &'a str) -> Option<(SyntaxKind, Option<String>)> {
    //     if text.is_empty() {
    //         return None;
    //     }
    //
    //     let token = rustc_lexer::tokenize(text).next()?;
    //     if token.len as usize != text.len() {
    //         return None;
    //     }
    //
    //     let mut conv = Converter::new(text);
    //     conv.extend_token(&token.kind, text);
    //     match &*conv.res.kind {
    //         [kind] => Some((*kind, conv.res.error.pop().map(|it| it.msg))),
    //         _ => None,
    //     }
    // }

    pub fn as_str(&self) -> &str {
        self.text
    }

    pub fn len(&self) -> usize {
        self.kind.len() - 1
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the kind of the token at the given index.
    pub fn kind(&self, i: usize) -> SyntaxKind {
        assert!(i < self.len());
        self.kind[i]
    }

    /// Returns the text of the token at the given index.
    pub fn text(&self, i: usize) -> &str {
        self.range_text(i..i + 1)
    }

    /// Returns the text of the given token index range.
    pub fn range_text(&self, r: ops::Range<usize>) -> &str {
        assert!(r.start < r.end && r.end <= self.len());
        let lo = self.start[r.start] as usize;
        let hi = self.start[r.end] as usize;
        &self.text[lo..hi]
    }

    // Naming is hard.
    /// Returns the range of the token at the given index.
    pub fn text_range(&self, i: usize) -> ops::Range<usize> {
        assert!(i < self.len());
        let lo = self.start[i] as usize;
        let hi = self.start[i + 1] as usize;
        lo..hi
    }
    /// Returns the start of the token at the given index.
    pub fn text_start(&self, i: usize) -> usize {
        assert!(i <= self.len());
        self.start[i] as usize
    }
    /// Return the length of the token at the given index.
    pub fn text_len(&self, i: usize) -> usize {
        assert!(i < self.len());
        let r = self.text_range(i);
        r.end - r.start
    }

    /// Returns the error text of the token at the given index (if any).
    pub fn error(&self, i: usize) -> Option<&str> {
        assert!(i < self.len());
        let err = self
            .error
            .binary_search_by_key(&(i as u32), |i| i.token)
            .ok()?;
        Some(self.error[err].msg.as_str())
    }

    /// Returns an iterator over all errors.
    ///
    /// Each error is a tuple of the token index and the error message.
    pub fn errors(&self) -> impl Iterator<Item = (usize, &str)> + '_ {
        self.error
            .iter()
            .map(|it| (it.token as usize, it.msg.as_str()))
    }

    fn push(&mut self, kind: SyntaxKind, offset: usize) {
        self.kind.push(kind);
        self.start.push(offset as u32);
    }
}

struct LexedStrBuilder<'a> {
    res: LexedStr<'a>,
    offset: usize,
}

impl<'a> LexedStrBuilder<'a> {
    fn new(text: &'a str) -> Self {
        Self {
            res: LexedStr {
                text,
                kind: Vec::new(),
                start: Vec::new(),
                error: Vec::new(),
            },
            offset: 0,
        }
    }

    fn finalize_with_eof(mut self, error: Option<&str>) -> LexedStr<'a> {
        self.res.push(EOF, self.offset);
        if let Some(err) = error {
            self.res.error.push(LexError {
                token: self.res.len().checked_sub(1).unwrap() as u32,
                msg: err.into(),
            });
        }
        self.res
    }

    fn push(&mut self, mut kind: SyntaxKind, len: usize, err: Option<&str>) {
        // Handle keywords
        // they are first parsed as IDENT, but if they are actually keywords, we change the kind
        if kind == IDENT {
            if let Some(kw_kind) =
                SyntaxKind::from_keyword_str(&self.res.text[self.offset..][..len])
            {
                kind = kw_kind;
            }
        }

        // coalesce adjacent whitespace tokens (needed for demoted newlines)
        if kind == WHITESPACE && self.res.kind.last().is_some_and(|&k| k == WHITESPACE) {
            self.offset += len;
            return;
        }

        self.res.push(kind, self.offset);
        self.offset += len;

        if let Some(err) = err {
            let token = self.res.len() as u32;
            let msg = err.to_string();
            self.res.error.push(LexError { msg, token });
        }
    }
}

struct Token {
    kind: SyntaxKind,
    len: u32,
    error: Option<&'static str>,
}

impl Cursor<'_> {
    fn advance_token(&mut self) -> Token {
        let first_char = match self.bump() {
            Some(c) => c,
            None => {
                return Token {
                    kind: EOF,
                    len: 0,
                    error: None,
                }
            }
        };

        let token_kind = match first_char {
            // Slash, comment or block comment.
            '/' => match self.first() {
                '/' => self.line_comment(),
                '*' => self.block_comment(),
                _ => SLASH,
            },
            // Whitespace sequence.
            c if is_whitespace(c) => self.whitespace(),
            '\\' => match self.first() {
                '\n' => {
                    self.bump();
                    self.whitespace()
                }
                _ => {
                    self.emit_error("expected newline after backslash");
                    WHITESPACE
                }
            },

            // Identifier (this should be checked after other variant that can
            // start as identifier).
            c if is_id_start(c) => self.ident(),

            // Numeric literal.
            c @ '0'..='9' => self.number(c),

            // One-symbol tokens.
            '\n' => NEWLINE,
            ',' => T![,],
            '.' => match self.first() {
                '/' => {
                    self.bump();
                    T![./]
                }
                '*' => {
                    self.bump();
                    T![.*]
                }
                _ => T![.],
            },
            '(' => T!['('],
            ')' => T![')'],
            '{' => T!['{'],
            '}' => T!['}'],
            '[' => T!['['],
            ']' => T![']'],
            '<' => match self.first() {
                '=' => {
                    self.bump();
                    T![<=]
                }
                '<' => {
                    self.bump();
                    T![<<]
                }
                _ => T![<],
            },
            '>' => match self.first() {
                '=' => {
                    self.bump();
                    T![>=]
                }
                '>' => {
                    self.bump();
                    T![>>]
                }
                _ => T![>],
            },
            '@' => T![@],
            // '#' => T![#],
            '~' => T![~],
            // '?' => T![?],
            ':' => T![:],
            // '$' => T![$],
            '=' => match self.first() {
                '=' => {
                    self.bump();
                    T![==]
                }
                '>' => {
                    self.bump();
                    T![=>]
                }
                _ => T![=],
            },
            '!' => match self.first() {
                '=' => {
                    self.bump();
                    T![!=]
                }
                _ => T![!],
            },
            '-' => T![-],
            '&' => match self.first() {
                '&' => {
                    self.bump();
                    T![&&]
                }
                _ => T![&],
            },
            '|' => match self.first() {
                '|' => {
                    self.bump();
                    T![||]
                }
                _ => T![|],
            },
            '+' => T![+],
            '*' => T![*],
            '^' => T![^],
            '%' => T![%],
            '$' if is_id_continue(self.first()) => self.register_ident(),
            '$' if !self.first().is_ascii() && unic_emoji_char::is_emoji(self.first()) => {
                self.fake_register_ident()
            }

            // String literal.
            '"' => {
                let terminated = self.double_quoted_string();
                if !terminated {
                    self.emit_error("Missing trailing `\"` symbol to terminate the string literal");
                }
                STRING
            }

            // Identifier starting with an emoji. Only lexed for graceful error recovery.
            c if !c.is_ascii() && unic_emoji_char::is_emoji(c) => self.fake_ident(),

            _ => ERROR,
        };

        let len = self.pos_within_token();
        self.reset_pos_within_token();

        Token {
            kind: token_kind,
            len,
            error: self.take_error(),
        }
    }

    fn line_comment(&mut self) -> SyntaxKind {
        debug_assert!(self.prev() == '/' && self.first() == '/');
        self.bump();

        self.eat_while(|c| c != '\n');

        COMMENT
    }

    fn block_comment(&mut self) -> SyntaxKind {
        debug_assert!(self.prev() == '/' && self.first() == '*');
        self.bump();

        let mut depth = 1usize;
        while let Some(c) = self.bump() {
            match c {
                '/' if self.first() == '*' => {
                    self.bump();
                    depth += 1;
                }
                '*' if self.first() == '/' => {
                    self.bump();
                    depth -= 1;
                    if depth == 0 {
                        // This block comment is closed, so for a construction like "/* */ */"
                        // there will be a successfully parsed block comment "/* */"
                        // and " */" will be processed separately.
                        break;
                    }
                }
                _ => (),
            }
        }

        if depth != 0 {
            self.emit_error("Missing trailing `*/` symbols to terminate the block comment");
        }

        COMMENT
    }

    fn whitespace(&mut self) -> SyntaxKind {
        debug_assert!(is_whitespace(self.prev()) || self.prev() == '\n');
        loop {
            self.eat_while(is_whitespace);
            if self.first() == '\\' && self.second() == '\n' {
                self.bump();
                self.bump();
            } else {
                break;
            }
        }
        WHITESPACE
    }

    fn ident(&mut self) -> SyntaxKind {
        debug_assert!(is_id_start(self.prev()));
        // Start is already eaten, eat the rest of identifier.
        self.eat_while(is_id_continue);
        match self.first() {
            c if !c.is_ascii() && unic_emoji_char::is_emoji(c) => self.fake_ident(),
            _ => IDENT,
        }
    }

    fn fake_ident(&mut self) -> SyntaxKind {
        // Start is already eaten, eat the rest of identifier.
        self.eat_while(|c| {
            unicode_xid::UnicodeXID::is_xid_continue(c)
                || (!c.is_ascii() && unic_emoji_char::is_emoji(c))
                || c == '\u{200d}'
        });
        self.emit_error("Ident contains invalid characters");
        IDENT
    }

    fn register_ident(&mut self) -> SyntaxKind {
        // Unlike `fake_ident`, the first character is not eaten yet!
        self.eat_while(is_id_continue);
        match self.first() {
            c if !c.is_ascii() && unic_emoji_char::is_emoji(c) => self.fake_ident(),
            _ => REGISTER_IDENT,
        }
    }

    fn fake_register_ident(&mut self) -> SyntaxKind {
        // Unlike `fake_ident`, the first character is not eaten yet
        self.eat_while(|c| {
            unicode_xid::UnicodeXID::is_xid_continue(c)
                || (!c.is_ascii() && unic_emoji_char::is_emoji(c))
                || c == '\u{200d}'
        });
        self.emit_error("Register ident contains invalid characters");
        REGISTER_IDENT
    }

    fn number(&mut self, first_digit: char) -> SyntaxKind {
        debug_assert!('0' <= self.prev() && self.prev() <= '9');
        if first_digit == '0' {
            // Attempt to parse encoding base.
            match self.first() {
                'b' => {
                    self.bump();
                    if !self.eat_decimal_digits() {
                        self.emit_error("Missing digits after the integer base prefix");
                        return INT_NUMBER;
                    }
                }
                'o' => {
                    self.bump();
                    if !self.eat_decimal_digits() {
                        self.emit_error("Missing digits after the integer base prefix");
                        return INT_NUMBER;
                    }
                }
                'x' => {
                    self.bump();
                    if !self.eat_hexadecimal_digits() {
                        self.emit_error("Missing digits after the integer base prefix");
                        return INT_NUMBER;
                    }
                }
                // Not a base prefix; consume additional digits.
                '0'..='9' | '_' => {
                    self.eat_decimal_digits();
                }

                // Also not a base prefix; nothing more to do here.
                '.' => {}

                // Just a 0.
                _ => {
                    return INT_NUMBER;
                }
            }
        } else {
            // No base prefix, parse number in the usual way.
            self.eat_decimal_digits();
        };

        match self.first() {
            // Don't be greedy if this is actually an
            // integer literal followed by field/method access or a range pattern
            // (`0..2` and `12.foo()`)
            '.' if self.second() != '.' && !is_id_start(self.second()) => {
                // might have stuff after the ., and if it does, it needs to start
                // with a number
                self.bump();
                if self.first().is_ascii_digit() {
                    self.eat_decimal_digits();
                }
                RATIONAL_NUMBER
            }
            _ => INT_NUMBER,
        }
    }

    /// Eats double-quoted string and returns true
    /// if string is terminated.
    fn double_quoted_string(&mut self) -> bool {
        debug_assert!(self.prev() == '"');
        while let Some(c) = self.bump() {
            match c {
                '"' => {
                    return true;
                }
                '\\' if self.first() == '\\' || self.first() == '"' => {
                    // Bump again to skip escaped character.
                    self.bump();
                }
                _ => (),
            }
        }
        // End of file reached.
        false
    }

    fn eat_decimal_digits(&mut self) -> bool {
        let mut has_digits = false;
        loop {
            match self.first() {
                '_' => {
                    self.bump();
                }
                '0'..='9' => {
                    has_digits = true;
                    self.bump();
                }
                _ => break,
            }
        }
        has_digits
    }

    fn eat_hexadecimal_digits(&mut self) -> bool {
        let mut has_digits = false;
        loop {
            match self.first() {
                '_' => {
                    self.bump();
                }
                '0'..='9' | 'a'..='f' | 'A'..='F' => {
                    has_digits = true;
                    self.bump();
                }
                _ => break,
            }
        }
        has_digits
    }
}
