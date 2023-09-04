use crate::{
    compile::{BlockId, Db, InFile, MakeInFile, Program},
    syntax::ast,
    syntax::AstToken,
};
use either::Either;
use rustc_hash::FxHashMap;
use salsa::DebugWithDb;
use smol_str::SmolStr;
use std::collections::hash_map::Entry;
use std::fmt::Display;

#[derive(Clone, Eq, PartialEq, Hash, Debug, Ord, PartialOrd)]
pub struct Name(pub SmolStr);

impl Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

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
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
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

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum BlockName {
    GlobalUnnamed(u32),
    GlobalNamed(Name),
    Function(Name),
    LocalUnnamed {
        function_name: Name,
        block_index: u32,
    },
    LocalNamed {
        function_name: Name,
        block_name: Name,
    },
}

#[salsa::tracked]
pub struct DefMap {
    #[return_ref]
    items: FxHashMap<Name, FileDefRef>,
    #[return_ref]
    block_names: FxHashMap<InFile<BlockId>, BlockName>,
}

#[salsa::tracked]
impl DefMap {
    #[salsa::tracked]
    pub fn get(self, db: &dyn Db, name: Name) -> Option<FileDefRef> {
        self.items(db).get(&name).cloned()
    }

    pub fn get_block_name(self, db: &dyn Db, block: InFile<BlockId>) -> Option<BlockName> {
        self.block_names(db).get(&block).cloned()
    }

    pub fn get_defined_names(self, db: &dyn Db) -> Vec<Name> {
        self.items(db).keys().cloned().collect()
    }
}

impl DefMap {
    pub fn debug_dump(&self, db: &dyn Db) -> String {
        use std::fmt::Write as _;

        let mut output = String::new();

        let mut items = self.items(db).into_iter().collect::<Vec<_>>();
        items.sort();

        writeln!(output, "items:").unwrap();
        for (name, def_ref) in items {
            let file_name = def_ref.file.path(db);
            let def_ref = &def_ref.value;

            writeln!(output, "  {}: {:?} @ {}", name, def_ref, file_name).unwrap();
        }

        let mut block_names = self.block_names(db).into_iter().collect::<Vec<_>>();
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

#[salsa::tracked]
pub fn build_def_map(db: &dyn Db, program: Program) -> DefMap {
    let mut items: FxHashMap<Name, FileDefRef> = FxHashMap::default();
    let mut block_names = FxHashMap::default();
    let mut define = |name: Name, item: FileDefRef| match items.entry(name) {
        Entry::Occupied(_o) => {
            todo!("report multiple definitions")
        }
        Entry::Vacant(v) => {
            v.insert(item);
        }
    };

    for (file, tree) in program.parse_files(db) {
        for (item_index, item) in tree.items().enumerate() {
            let item_index = item_index.try_into().unwrap();
            match item {
                ast::Item::InstructionsBlockSet(blocks) => {
                    for (block_index, block) in blocks.blocks().enumerate() {
                        let block_index = block_index.try_into().unwrap();

                        let block_id = BlockId::new_block(item_index, block_index);
                        let mut block_name = None;

                        if let Some(labels) = block.labels() {
                            for label in labels.labels() {
                                if let Some(name) = label.name() {
                                    let name = Name(name.text().into());
                                    block_name = Some(name.clone());
                                    define(name, DefRef::Block(block_id).in_file(file));
                                }
                            }
                        }

                        block_names.insert(
                            block_id.in_file(file),
                            match block_name {
                                None => BlockName::GlobalUnnamed(block_index),
                                Some(name) => BlockName::GlobalNamed(name),
                            },
                        );
                    }
                }
                ast::Item::FunctionDefinition(fun) => {
                    let block_id = BlockId::new_function(item_index);

                    if let Some(name) = fun.name() {
                        if let Some(name) = name.token() {
                            let function_name = Name(name.text().into());
                            define(function_name.clone(), DefRef::Block(block_id).in_file(file));

                            block_names.insert(
                                block_id.in_file(file),
                                BlockName::Function(function_name.clone()),
                            );

                            if let Some(block_set) = fun.instruction_block_set() {
                                for (block_index, block) in block_set.blocks().enumerate() {
                                    let block_index = block_index.try_into().unwrap();

                                    let block_id = BlockId::new_block(item_index, block_index);
                                    let block_name = block
                                        .labels()
                                        .and_then(|l| l.labels().next())
                                        .and_then(|l| l.name())
                                        .map(|name| Name(name.text().into()));

                                    block_names.insert(
                                        block_id.in_file(file),
                                        if let Some(name) = block_name {
                                            BlockName::LocalNamed {
                                                function_name: function_name.clone(),
                                                block_name: name,
                                            }
                                        } else {
                                            BlockName::LocalUnnamed {
                                                function_name: function_name.clone(),
                                                block_index,
                                            }
                                        },
                                    );
                                }
                            }
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

    DefMap::new(db, items, block_names)
}

#[cfg(test)]
mod tests {
    use crate::compile::{db::Database, def_map, DefMap, Diagnostics, File, Program};
    use expect_test::expect;

    fn parse_def_map(code: &str) -> (Database, DefMap) {
        let db = Database::default();
        let file = File::new(&db, "test.sal".to_string(), code.to_string());
        let program = Program::new(&db, vec![file]);
        let def_map = def_map::build_def_map(&db, program);

        let errors = Diagnostics::debug_dump(
            &db,
            def_map::build_def_map::accumulated::<Diagnostics>(&db, program),
        );
        if !errors.is_empty() {
            panic!("building def map produced errors:\n{}", errors);
        }

        (db, def_map)
    }

    #[test]
    fn check_map_dump() {
        let (db, def_map) = parse_def_map(
            r#"
def ABIBA = 3 + 3

subroutine KEKA
    add $1, 2, 2
ABOBA:
    add $1, 3, 3
endsub

    add $2, 2, 2
LABEL1:
    sub $2, 2, 2
    j LABEL1
LABEL2:
        "#,
        );

        expect![[r#"
            items:
              ABIBA: Define(0) @ test.sal
              KEKA: Block(BlockId { item_index: 1, block_index: None }) @ test.sal
              LABEL1: Block(BlockId { item_index: 2, block_index: Some(1) }) @ test.sal
              LABEL2: Block(BlockId { item_index: 2, block_index: Some(2) }) @ test.sal
            block names:
              BlockId { item_index: 1, block_index: None } @ test.sal: Function(Name("KEKA"))
              BlockId { item_index: 1, block_index: Some(0) } @ test.sal: LocalUnnamed { function_name: Name("KEKA"), block_index: 0 }
              BlockId { item_index: 1, block_index: Some(1) } @ test.sal: LocalNamed { function_name: Name("KEKA"), block_name: Name("ABOBA") }
              BlockId { item_index: 2, block_index: Some(0) } @ test.sal: GlobalUnnamed(0)
              BlockId { item_index: 2, block_index: Some(1) } @ test.sal: GlobalNamed(Name("LABEL1"))
              BlockId { item_index: 2, block_index: Some(2) } @ test.sal: GlobalNamed(Name("LABEL2"))
        "#]].assert_eq(&def_map.debug_dump(&db));
    }
}
