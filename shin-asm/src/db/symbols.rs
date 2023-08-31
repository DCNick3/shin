use crate::{
    db::{file::Program, in_file::InFile, in_file::MakeInFile, Db},
    syntax::ast,
    syntax::AstToken,
};
use either::Either;
use rustc_hash::FxHashMap;
use salsa::DebugWithDb;
use std::collections::hash_map::Entry;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Name(pub smol_str::SmolStr);

impl DebugWithDb<<crate::Jar as salsa::jar::Jar<'_>>::DynDb> for Name {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        _db: &<crate::Jar as salsa::jar::Jar<'_>>::DynDb,
        _include_all_fields: bool,
    ) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[salsa::interned]
pub struct DefRefId {
    pub code_ref: InFile<DefRef>,
}

/// Reference to a function or a label within a file
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum DefRef {
    Function {
        /// Index of the function item
        item_index: u32,
    },
    Label {
        /// Index of the block or function item
        item_index: u32,
        /// Index of the label within the block
        ///
        /// it's `u16` to make the `DefRef` fit into 64 bits
        label_index: u16,
    },
    Define {
        /// Index of the define item
        item_index: u32,
    },
}

impl MakeInFile for DefRef {}

/// This is a compile-time check that `CodeRef` fits into 64 bits
const _: () = [(); 1][(core::mem::size_of::<DefRef>() == 8) as usize ^ 1];

// this size is a little sad, but I don't know how to make it smaller
const _: () = [(); 1][(core::mem::size_of::<InFile<DefRef>>() == 12) as usize ^ 1];

// #[salsa::tracked]
// pub struct FunctionDefMap {
//     local_code_refs: FxHashMap<SmolStr, DefRefId>,
// }

#[salsa::tracked]
pub struct DefMap {
    items: FxHashMap<Name, DefRefId>,
}

#[salsa::tracked]
impl DefMap {
    #[salsa::tracked]
    pub fn get(self, db: &dyn Db, name: Name) -> Option<DefRefId> {
        self.items(db).get(&name).cloned()
    }
}

#[salsa::tracked]
pub fn build_def_map(db: &dyn Db, program: Program) -> DefMap {
    let mut items: FxHashMap<Name, DefRefId> = FxHashMap::default();
    let mut define = |name: Name, item: DefRefId| match items.entry(name) {
        Entry::Occupied(_o) => {
            todo!("report multiple definitions")
        }
        Entry::Vacant(v) => {
            v.insert(item);
        }
    };

    for (file, tree) in program.file_trees(db) {
        for (item_index, item) in tree.items().enumerate() {
            let item_index = item_index.try_into().unwrap();
            match item {
                ast::Item::InstructionsBlock(block) => {
                    for (label_index, label) in block.labels().enumerate() {
                        let label_index = label_index.try_into().unwrap();
                        if let Some(name) = label.name() {
                            define(
                                Name(name.text().into()),
                                DefRefId::new(
                                    db,
                                    DefRef::Label {
                                        item_index,
                                        label_index,
                                    }
                                    .in_file(file),
                                ),
                            );
                        }
                    }
                }
                ast::Item::FunctionDefinition(fun) => {
                    if let Some(name) = fun.name() {
                        if let Some(name) = name.token() {
                            define(
                                Name(name.text().into()),
                                DefRefId::new(db, DefRef::Function { item_index }.in_file(file)),
                            );
                        }
                    }
                }
                ast::Item::AliasDefinition(def) => {
                    // NOTE: here we are interested only in value aliases
                    // we would need to collect register aliases in a different pass
                    if let Some(Either::Left(name)) = def.name() {
                        if let Some(name) = name.token() {
                            define(
                                Name(name.text().into()),
                                DefRefId::new(db, DefRef::Define { item_index }.in_file(file)),
                            );
                        }
                    }
                }
            }
        }
    }

    DefMap::new(db, items)
}
