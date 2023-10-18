use binrw::{BinRead, BinWrite};
use std::fmt::{Debug, Display};
use std::num::ParseIntError;
use std::str::FromStr;

/// Register address in the vm
///
/// It can either refer to an argument register (implemented as a stack) or a regular global register.
#[derive(BinRead, BinWrite, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
// TODO: add a niche for `Option<Register>` to have an efficient representation
pub struct Register(u16);

impl Register {
    /// Addresses larger than 0x1000 are treated as relative to the stack top (Aka mem3)
    const REGULAR_REGISTERS_START: u16 = 0;
    const REGULAR_REGISTERS_END: u16 = Self::ARGUMENTS_START - 1;
    const ARGUMENTS_START: u16 = 0x1000;
    const ARGUMENTS_END: u16 = 0x1fff;

    pub fn try_from_regular_register(index: u16) -> Option<Self> {
        if index <= Self::REGULAR_REGISTERS_END - Self::REGULAR_REGISTERS_START {
            Some(Self::from(RegisterRepr::Regular(index)))
        } else {
            None
        }
    }

    pub fn from_regular_register(index: u16) -> Self {
        Self::try_from_regular_register(index).expect("Regular register index out of range")
    }

    pub fn try_from_argument(index: u16) -> Option<Self> {
        if index <= Self::ARGUMENTS_END - Self::ARGUMENTS_START {
            Some(Self::from(RegisterRepr::Argument(index)))
        } else {
            None
        }
    }

    pub fn from_argument(index: u16) -> Self {
        Self::try_from_argument(index).expect("Argument register index out of range")
    }

    pub fn repr(self) -> RegisterRepr {
        self.into()
    }
}

impl Debug for Register {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.repr(), f)
    }
}

impl Display for Register {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.repr(), f)
    }
}

impl From<Register> for RegisterRepr {
    fn from(value: Register) -> Self {
        match value.0 {
            Register::REGULAR_REGISTERS_START..=Register::REGULAR_REGISTERS_END => {
                RegisterRepr::Regular(value.0)
            }
            Register::ARGUMENTS_START..=Register::ARGUMENTS_END => {
                RegisterRepr::Argument(value.0 - Register::ARGUMENTS_START)
            }
            value => unreachable!("Invalid register: {}", value),
        }
    }
}

impl From<RegisterRepr> for Register {
    fn from(value: RegisterRepr) -> Self {
        match value {
            RegisterRepr::Regular(regular) => Self(regular),
            RegisterRepr::Argument(argument) => Self(Register::ARGUMENTS_START + argument),
        }
    }
}

impl FromStr for Register {
    type Err = RegisterReprParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        RegisterRepr::from_str(s).map(RegisterRepr::register)
    }
}

#[derive(Debug, Copy, Clone)]
pub enum RegisterRepr {
    Argument(u16),
    Regular(u16),
}

impl RegisterRepr {
    pub fn register(self) -> Register {
        self.into()
    }
}

impl Display for RegisterRepr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegisterRepr::Argument(argument) => write!(f, "$a{}", argument),
            RegisterRepr::Regular(regular) => write!(f, "$v{}", regular),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegisterReprParseError {
    InvalidPrefix,
    InvalidIndex(ParseIntError),
}

impl FromStr for RegisterRepr {
    type Err = RegisterReprParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s
            .strip_prefix('$')
            .ok_or(RegisterReprParseError::InvalidPrefix)?;
        if let Some(s) = s.strip_prefix('a') {
            let index = s.parse().map_err(RegisterReprParseError::InvalidIndex)?;
            Ok(RegisterRepr::Argument(index))
        } else if let Some(s) = s.strip_prefix('v') {
            let index = s.parse().map_err(RegisterReprParseError::InvalidIndex)?;
            Ok(RegisterRepr::Regular(index))
        } else {
            Err(RegisterReprParseError::InvalidPrefix)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Register;

    fn assert_register_roundtrip(s: &str) {
        let register: Register = s.parse().unwrap();
        assert_eq!(register.to_string(), s);
    }

    #[test]
    fn roundtrip() {
        assert_register_roundtrip("$v0");
        assert_register_roundtrip("$v1");
        assert_register_roundtrip("$v2");
        assert_register_roundtrip("$v3");
        assert_register_roundtrip("$v4095");
        assert_register_roundtrip("$a0");
        assert_register_roundtrip("$a1");
        assert_register_roundtrip("$a2");
        assert_register_roundtrip("$a3");
        assert_register_roundtrip("$a4095");
    }

    fn assert_register_value(s: &str, value: u16) {
        let register: Register = s.parse().unwrap();
        assert_eq!(register.0, value);
    }
    #[test]
    fn value() {
        assert_register_value("$v0", 0);
        assert_register_value("$v1", 1);
        assert_register_value("$v2", 2);
        assert_register_value("$v3", 3);
        assert_register_value("$v4095", 4095);
        assert_register_value("$a0", 0x1000);
        assert_register_value("$a1", 0x1001);
        assert_register_value("$a2", 0x1002);
        assert_register_value("$a3", 0x1003);
        assert_register_value("$a4095", 0x1fff);
    }

    fn assert_constructor(r: Register, s: &str) {
        assert_eq!(r.to_string(), s);
    }
    #[test]
    fn constructors() {
        assert_constructor(Register::from_regular_register(0), "$v0");
        assert_constructor(Register::from_regular_register(1), "$v1");
        assert_constructor(Register::from_regular_register(2), "$v2");
        assert_constructor(Register::from_regular_register(3), "$v3");
        assert_constructor(Register::from_regular_register(4095), "$v4095");
        assert_constructor(Register::from_argument(0), "$a0");
        assert_constructor(Register::from_argument(1), "$a1");
        assert_constructor(Register::from_argument(2), "$a2");
        assert_constructor(Register::from_argument(3), "$a3");
        assert_constructor(Register::from_argument(4095), "$a4095");
    }
}
