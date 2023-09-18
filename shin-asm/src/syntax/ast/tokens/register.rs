use crate::compile::diagnostics::make_diagnostic;
use crate::compile::diagnostics::Diagnostic;
use smol_str::SmolStr;
use text_size::TextRange;

use super::*;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
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
    pub fn kind(&self) -> Result<RegisterIdentKind, Diagnostic<TextRange>> {
        let reg_str = self
            .text()
            .strip_prefix('$')
            .expect("RegisterIdent must start with '$'");

        parse::parse_register_kind(reg_str)
            .map_err(|e| make_diagnostic!(self.text_range(), "{}", e))
    }
}

mod parse {
    use crate::elements::Register;
    use crate::syntax::ast::RegisterIdentKind;
    use std::num::IntErrorKind;

    fn try_parse_predefined_register(reg_str: &str) -> Result<Option<RegisterIdentKind>, String> {
        let mut chars = reg_str.chars();

        let Some(first_char) = chars.next() else {
            // Register name cannot be empty
            return Err("Expected register name".to_string());
        };

        if let c @ ('a' | 'v') = first_char {
            // try to parse the integer after the prefix
            // NOTE: technically, this will parse a+1 and a-1 as valid register names
            // we don't care because the lexer will not produce those as single tokens
            match chars.as_str().parse() {
                Ok(v) => {
                    let register = match c {
                        'v' => Register::try_from_regular_register(v),
                        'a' => Register::try_from_argument(v),
                        _ => unreachable!(),
                    }
                    .ok_or_else(|| "Register index is too big".to_string())?;

                    return Ok(Some(RegisterIdentKind::Register(register)));
                }
                Err(e) => {
                    match e.kind() {
                        IntErrorKind::PosOverflow => {
                            // this is an error though
                            return Err("Register index is too big".to_string());
                        }

                        IntErrorKind::Empty | IntErrorKind::InvalidDigit => {
                            // $a and $v are fine, but is not a predefined register
                        }
                        // we don't use a non-zero type for register index
                        // `-` couldn't have allowed here to parse a negative number
                        IntErrorKind::Zero | IntErrorKind::NegOverflow => unreachable!(),
                        _ => {}
                    }
                }
            }
        }

        Ok(None)
    }

    pub fn parse_register_kind(reg_str: &str) -> Result<RegisterIdentKind, String> {
        Ok(
            if let Some(predefined) = try_parse_predefined_register(reg_str)? {
                predefined
            } else {
                RegisterIdentKind::Alias(reg_str.into())
            },
        )
    }

    #[cfg(test)]
    mod tests {
        use super::parse_register_kind;
        use crate::syntax::ast::RegisterIdentKind;

        #[test]
        fn parse_argument() {
            assert_eq!(
                parse_register_kind("a0"),
                Ok(RegisterIdentKind::Register("$a0".parse().unwrap()))
            );
            assert_eq!(
                parse_register_kind("a1"),
                Ok(RegisterIdentKind::Register("$a1".parse().unwrap()))
            );
            assert_eq!(
                parse_register_kind("a4095"),
                Ok(RegisterIdentKind::Register("$a4095".parse().unwrap()))
            );
            assert_eq!(
                parse_register_kind("a4096"),
                Err("Register index is too big".to_string())
            );
            assert_eq!(
                parse_register_kind("a999999999"),
                Err("Register index is too big".to_string())
            );
        }

        #[test]
        fn parse_regular() {
            assert_eq!(
                parse_register_kind("v0"),
                Ok(RegisterIdentKind::Register("$v0".parse().unwrap()))
            );
            assert_eq!(
                parse_register_kind("v1"),
                Ok(RegisterIdentKind::Register("$v1".parse().unwrap()))
            );
            assert_eq!(
                parse_register_kind("v4095"),
                Ok(RegisterIdentKind::Register("$v4095".parse().unwrap()))
            );
            assert_eq!(
                parse_register_kind("v4096"),
                Err("Register index is too big".to_string())
            );
            assert_eq!(
                parse_register_kind("v999999999"),
                Err("Register index is too big".to_string())
            );
        }

        #[test]
        fn parse_alias() {
            assert_eq!(
                parse_register_kind("a"),
                Ok(RegisterIdentKind::Alias("a".into()))
            );
            assert_eq!(
                parse_register_kind("v"),
                Ok(RegisterIdentKind::Alias("v".into()))
            );
            assert_eq!(
                parse_register_kind("a1a"),
                Ok(RegisterIdentKind::Alias("a1a".into()))
            );
            assert_eq!(
                parse_register_kind("v1a"),
                Ok(RegisterIdentKind::Alias("v1a".into()))
            );
            assert_eq!(
                parse_register_kind("dummy"),
                Ok(RegisterIdentKind::Alias("dummy".into()))
            );
        }
    }
}
