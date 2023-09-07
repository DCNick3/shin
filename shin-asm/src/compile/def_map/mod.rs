mod collect;

pub use collect::{build_def_map, LocalRegisters, ResolvedGlobalRegisters};

use crate::{
    compile::{BlockId, BlockIdWithFile, Db, MakeWithFile, WithFile},
    elements::Register,
    syntax::ast::{self, visit::ItemIndex},
};
use rustc_hash::FxHashMap;
use smol_str::SmolStr;
use std::fmt::Display;

#[derive(Clone, Eq, PartialEq, Hash, Debug, Ord, PartialOrd)]
pub struct Name(pub SmolStr);

impl Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Debug, Ord, PartialOrd)]
pub struct RegisterName(pub SmolStr);

impl Display for RegisterName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum BlockName {
    GlobalBlock(Option<Name>),
    Function(Option<Name>),
    LocalBlock(Option<Name>),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DefMap {
    // items: FxHashMap<Name, DefRef>,
    global_registers: ResolvedGlobalRegisters,
    local_registers: LocalRegisters,
    block_names: FxHashMap<BlockIdWithFile, BlockName>,
}

impl DefMap {
    // pub fn get_item(&self, name: &Name) -> Option<FileDefRef> {
    //     self.items.get(name).copied()
    // }
}

impl DefMap {
    pub fn debug_dump(&self, db: &dyn Db) -> String {
        use std::fmt::Write as _;

        let mut output = String::new();

        // let mut items = self.items.iter().collect::<Vec<_>>();
        // items.sort();
        //
        // writeln!(output, "items:").unwrap();
        // for (name, def_ref) in items {
        //     let file_name = def_ref.file.path(db);
        //     let def_ref = &def_ref.value;
        //
        //     writeln!(output, "  {}: {:?} @ {}", name, def_ref, file_name).unwrap();
        // }

        let mut global_registers = self.global_registers.iter().collect::<Vec<_>>();
        global_registers.sort_by_key(|(name, _)| *name);
        let mut local_registers = self.local_registers.iter().collect::<Vec<_>>();
        local_registers.sort_by_key(|(&index, _)| index);

        writeln!(output, "registers:").unwrap();
        writeln!(output, "  global:").unwrap();
        for (name, value) in global_registers {
            writeln!(output, "    {}: {:?}", name, value).unwrap();
        }
        writeln!(output, "  local:").unwrap();
        for (item_index, registers) in local_registers {
            writeln!(output, "    item {}: ", item_index).unwrap();
            for (name, value) in registers {
                writeln!(output, "      {}: {:?}", name, value).unwrap();
            }
        }

        let mut block_names = self.block_names.iter().collect::<Vec<_>>();
        block_names.sort();

        writeln!(output, "block names:").unwrap();
        for (block_id, name) in block_names {
            let file_name = block_id.file.path(db);
            let block_id = &block_id.value;

            writeln!(output, "  {:?} @ {}: {:?}", block_id, file_name, name).unwrap();
        }

        output
    }
}
