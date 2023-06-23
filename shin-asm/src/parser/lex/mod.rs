mod char_classes;
mod cursor;
#[cfg(test)]
mod tests;

use crate::parser::lex::char_classes::{is_id_continue, is_id_start, is_whitespace};
use crate::parser::{lex::cursor::Cursor, SyntaxKind, SyntaxKind::*, T};
use std::ops;

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

        for token in token_iter {
            let token_text = &text[conv.offset..][..token.len as usize];

            conv.push(token.kind, token_text.len(), token.error)
        }

        conv.finalize_with_eof()
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

    fn finalize_with_eof(mut self) -> LexedStr<'a> {
        self.res.push(EOF, self.offset);
        self.res
    }

    fn push(&mut self, kind: SyntaxKind, len: usize, err: Option<&str>) {
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
            // '@' => T![@],
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
        debug_assert!(is_whitespace(self.prev()));
        self.eat_while(is_whitespace);
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
                '.' | 'e' | 'E' => {}

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
                let mut empty_exponent = false;
                if self.first().is_digit(10) {
                    self.eat_decimal_digits();
                    match self.first() {
                        'e' | 'E' => {
                            self.bump();
                            empty_exponent = !self.eat_float_exponent();
                        }
                        _ => (),
                    }
                }
                if empty_exponent {
                    self.emit_error("Missing digits after the exponent symbol");
                }
                FLOAT_NUMBER
            }
            'e' | 'E' => {
                self.bump();
                let empty_exponent = !self.eat_float_exponent();
                self.emit_error("Missing digits after the exponent symbol");
                FLOAT_NUMBER
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

    /// Eats the float exponent. Returns true if at least one digit was met,
    /// and returns false otherwise.
    fn eat_float_exponent(&mut self) -> bool {
        debug_assert!(self.prev() == 'e' || self.prev() == 'E');
        if self.first() == '-' || self.first() == '+' {
            self.bump();
        }
        self.eat_decimal_digits()
    }
}
