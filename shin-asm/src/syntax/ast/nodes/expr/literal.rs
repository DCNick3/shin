use super::*;
use crate::compile::diagnostics::Diagnostic;
use crate::compile::make_diagnostic;
use shin_core::rational::Rational;
use std::borrow::Cow;
use std::num::IntErrorKind;
use text_size::TextRange;

#[derive(Debug, Clone, PartialEq, Eq, Hash, AstNode)]
#[ast(kind = LITERAL)]
pub struct Literal {
    pub(crate) syntax: SyntaxNode,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, AstToken)]
#[ast(kind = STRING)]
pub struct String {
    pub(crate) syntax: SyntaxToken,
}

impl String {
    pub fn value(&self) -> Result<Cow<'_, str>, Diagnostic<TextRange>> {
        // TODO: Unescape string
        // TODO: report escape errors
        let text = self.syntax.text();
        let inner_text = text.strip_prefix('"').unwrap().strip_suffix('"').unwrap();

        Ok(Cow::Borrowed(inner_text))
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Radix {
    Binary = 2,
    Octal = 8,
    Decimal = 10,
    Hexadecimal = 16,
}

impl Radix {
    pub const ALL: &'static [Radix] = &[
        Radix::Binary,
        Radix::Octal,
        Radix::Decimal,
        Radix::Hexadecimal,
    ];

    const fn prefix_len(self) -> usize {
        match self {
            Self::Decimal => 0,
            _ => 2,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash, AstToken)]
#[ast(kind = INT_NUMBER)]
pub struct IntNumber {
    pub(crate) syntax: SyntaxToken,
}

impl IntNumber {
    pub fn radix(&self) -> Radix {
        match self.text().get(..2).unwrap_or_default() {
            "0b" => Radix::Binary,
            "0o" => Radix::Octal,
            "0x" => Radix::Hexadecimal,
            _ => Radix::Decimal,
        }
    }

    pub fn split_into_parts(&self) -> (&str, &str, &str) {
        let radix = self.radix();
        let (prefix, mut text) = self.text().split_at(radix.prefix_len());

        // TODO: do we need to handle suffixes? I _think_ the lexer doesn't allow them
        let is_suffix_start: fn(&(usize, char)) -> bool = match radix {
            Radix::Hexadecimal => |(_, c)| matches!(c, 'g'..='z' | 'G'..='Z'),
            _ => |(_, c)| c.is_ascii_alphabetic(),
        };

        let mut suffix = "";
        if let Some((suffix_start, _)) = text.char_indices().find(is_suffix_start) {
            let (text2, suffix2) = text.split_at(suffix_start);
            text = text2;
            suffix = suffix2;
        };

        (prefix, text, suffix)
    }

    pub fn value(&self) -> Result<i32, Diagnostic<TextRange>> {
        let (_, text, _) = self.split_into_parts();
        i32::from_str_radix(&text.replace('_', ""), self.radix() as u32).map_err(|e| {
            match e.kind() {
                IntErrorKind::Empty => unreachable!(), // I think??
                IntErrorKind::InvalidDigit => {
                    make_diagnostic!(self.text_range(), "Invalid digit in integer literal")
                }
                IntErrorKind::PosOverflow => {
                    make_diagnostic!(self.text_range(), "Integer literal is too large")
                }
                IntErrorKind::NegOverflow => {
                    make_diagnostic!(self.text_range(), "Integer literal is too small")
                }
                IntErrorKind::Zero => unreachable!(),
                _ => make_diagnostic!(
                    self.text_range(),
                    "Unknown error occurred while parsing integer literal: {:?}",
                    e
                ),
            }
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, AstToken)]
#[ast(kind = RATIONAL_NUMBER)]
pub struct RationalNumber {
    pub(crate) syntax: SyntaxToken,
}

impl RationalNumber {
    pub fn value(&self) -> Result<Rational, Diagnostic<TextRange>> {
        let text = self.text().replace('_', "");
        text.parse()
            .map_err(|_| make_diagnostic!(self.text_range(), "Invalid float literal"))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LiteralKind {
    String(String),
    IntNumber(IntNumber),
    RationalNumber(RationalNumber),
    // Bool(bool),
}

impl Literal {
    pub fn token(&self) -> SyntaxToken {
        self.syntax()
            .children_with_tokens()
            .find(|e| !e.kind().is_trivia())
            .and_then(|e| e.into_token())
            .unwrap()
    }

    pub fn kind(&self) -> LiteralKind {
        let token = self.token();

        if let Some(t) = String::cast(token.clone()) {
            LiteralKind::String(t)
        } else if let Some(t) = IntNumber::cast(token.clone()) {
            LiteralKind::IntNumber(t)
        } else if let Some(t) = RationalNumber::cast(token.clone()) {
            LiteralKind::RationalNumber(t)
        } else {
            unreachable!()
        }
    }
}
