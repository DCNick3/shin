use crate::parser::SyntaxKind;

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd, Hash, Clone, Copy)]
pub enum Lang {}

impl rowan::Language for Lang {
    type Kind = SyntaxKind;

    fn kind_from_raw(raw: rowan::SyntaxKind) -> Self::Kind {
        Self::Kind::from_u16(raw.0).unwrap()
    }

    fn kind_to_raw(kind: Self::Kind) -> rowan::SyntaxKind {
        rowan::SyntaxKind(kind.into_u16())
    }
}
