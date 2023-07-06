mod resolve;

use crate::file_db::InFile;
use rustc_hash::FxHashMap;
use smol_str::SmolStr;
use std::fmt;
use std::sync::Arc;

/// Reference to a function or a label within a file
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum CodeRef {
    Function {
        /// Index of the function item
        item_index: u32,
    },
    Label {
        /// Index of the block or function item
        item_index: u32,
        /// Index of the label within the block or function item
        ///
        /// it's `u16` to make the `CodeRef` fit into 64 bits
        label_index: u16,
    },
}

/// This is a compile-time check that `CodeRef` fits into 64 bits
const _: () = [(); 1][(core::mem::size_of::<CodeRef>() == 8) as usize ^ 1];

// this size is a little sad, but I don't know how to make it smaller
const _: () = [(); 1][(core::mem::size_of::<InFile<CodeRef>>() == 12) as usize ^ 1];

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum SymbolValue {
    Number(i32),
    CodeRef(InFile<CodeRef>),
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum RegisterName {
    A(u16),
    V(u16),
}

impl fmt::Display for RegisterName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (prefix, i) = match self {
            RegisterName::A(i) => ("a", i),
            RegisterName::V(i) => ("v", i),
        };
        write!(f, "${}{}", prefix, i)
    }
}

pub struct Scope {
    parent: Option<Arc<Scope>>,
    symbols: FxHashMap<SmolStr, SymbolValue>,
    registers: FxHashMap<SmolStr, RegisterName>,
}

impl Scope {
    pub fn resolve_symbol(&self, name: &str) -> Option<SymbolValue> {
        self.symbols.get(name).copied().or_else(|| {
            self.parent
                .as_ref()
                .and_then(|parent| parent.resolve_symbol(name))
        })
    }

    pub fn resolve_register(&self, name: &str) -> Option<RegisterName> {
        self.registers.get(name).copied().or_else(|| {
            self.parent
                .as_ref()
                .and_then(|parent| parent.resolve_register(name))
        })
    }
}
