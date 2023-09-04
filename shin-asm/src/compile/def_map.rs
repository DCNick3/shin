use crate::{
    compile::{BlockId, Db, InFile, MakeInFile, Program},
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

/// Reference to a function or a label within a file
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum DefRef {
    Block(BlockId),
    Define(u32),
}

pub type FileDefRef = InFile<DefRef>;

impl MakeInFile for DefRef {}

/// This is a compile-time check that `DefRef` fits into 64 bits
const _: () = [(); 1][(core::mem::size_of::<DefRef>() == 8) as usize ^ 1];

// this size is a little sad, but I don't know how to make it smaller
const _: () = [(); 1][(core::mem::size_of::<InFile<DefRef>>() == 12) as usize ^ 1];

#[salsa::tracked]
pub struct DefMap {
    items: FxHashMap<Name, FileDefRef>,
}

#[salsa::tracked]
impl DefMap {
    #[salsa::tracked]
    pub fn get(self, db: &dyn Db, name: Name) -> Option<FileDefRef> {
        self.items(db).get(&name).cloned()
    }
}

#[salsa::tracked]
pub fn build_def_map(db: &dyn Db, program: Program) -> DefMap {
    let mut items: FxHashMap<Name, FileDefRef> = FxHashMap::default();
    let mut define = |name: Name, item: FileDefRef| match items.entry(name) {
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
                ast::Item::InstructionsBlockSet(blocks) => {
                    for (block_index, block) in blocks.blocks().enumerate() {
                        let block_index = block_index.try_into().unwrap();
                        if let Some(labels) = block.labels() {
                            for label in labels.labels() {
                                if let Some(name) = label.name() {
                                    define(
                                        Name(name.text().into()),
                                        DefRef::Block(BlockId::new_block(item_index, block_index))
                                            .in_file(file),
                                    );
                                }
                            }
                        }
                    }
                }
                ast::Item::FunctionDefinition(fun) => {
                    if let Some(name) = fun.name() {
                        if let Some(name) = name.token() {
                            define(
                                Name(name.text().into()),
                                DefRef::Block(BlockId::new_function(item_index)).in_file(file),
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
                                DefRef::Define(item_index).in_file(file),
                            );
                        }
                    }
                }
            }
        }
    }

    DefMap::new(db, items)
}

#[cfg(test)]
mod tests {
    use crate::compile::{db::Database, def_map, def_map::Name, File, Program};
    use salsa::DebugWithDb;

    #[test]
    fn def_maps() {
        let db = Database::default();
        let db = &db;
        let file = File::new(
            db,
            "test.sal".to_string(),
            r#"
def ABIBA = 3 + 3

subroutine KEKA
ABOBA:
    add $1, 2, 2
endsub

    add $2, 2, 2
LABEL1:
    sub $2, 2, 2
    j LABEL1
LABEL2:
        "#
            .to_string(),
        );
        let program = Program::new(db, vec![file]);
        let def_map = def_map::build_def_map(db, program);

        dbg!(def_map.debug_all(db));

        let abiba_def = def_map.get(db, Name("ABIBA".into())).unwrap();

        dbg!(abiba_def.file.debug_all(db));

        dbg!(abiba_def);
        dbg!(def_map.get(db, Name("ABOBA".into())));
        dbg!(def_map.get(db, Name("KEKA".into())));
        dbg!(def_map.get(db, Name("LABEL1".into())));
        dbg!(def_map.get(db, Name("LABEL2".into())));
    }
}
