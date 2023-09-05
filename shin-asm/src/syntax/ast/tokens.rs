use crate::syntax::{
    ast::{AstToken, SyntaxToken},
    SyntaxKind::*,
};
use miette::{diagnostic, LabeledSpan};
use smol_str::SmolStr;

#[derive(Debug, Clone, PartialEq, Eq, Hash, AstToken)]
#[ast(kind = IDENT)]
pub struct Ident {
    pub(crate) syntax: SyntaxToken,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RegisterIdentKind {
    Register(crate::elements::Register),
    Alias(SmolStr),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, AstToken)]
#[ast(kind = REGISTER_IDENT)]
pub struct RegisterIdent {
    pub(crate) syntax: SyntaxToken,
}

impl RegisterIdent {
    pub fn kind(&self) -> Result<RegisterIdentKind, miette::MietteDiagnostic> {
        let span = LabeledSpan::new_with_span(None, self.miette_span());

        let mut chars = self.text().chars();
        assert_eq!(chars.next(), Some('$'));
        match chars.clone().next() /* peek */ {
            None => Err(diagnostic! {
                labels = vec![span],
                "Expected register name",
            }),
            Some(c @ ('a' | 'v')) => {
                chars.next();
                let index = chars.as_str().parse().map_err(|e| {
                    diagnostic! {
                        labels = vec![span.clone()],
                        "Failed to parse register index: {:?}",
                        e,
                    }
                })?;

                let register = match c {
                    'a' => crate::elements::Register::try_from_argument(index),
                    'v' => crate::elements::Register::try_from_regular_register(index),
                    _ => unreachable!(),
                }
                .ok_or_else(|| {
                    diagnostic! {
                        labels = vec![span],
                        "Invalid register index: {}",
                        index,
                    }
                })?;

                Ok(RegisterIdentKind::Register(register))
            }
            Some(_) => {
                let alias = chars.as_str();
                Ok(RegisterIdentKind::Alias(alias.into()))
            }
        }
    }
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
