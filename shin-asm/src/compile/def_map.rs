use crate::elements::RegisterRepr;
use crate::{
    compile::{BlockId, Db, Diagnostics, File, InFile, MakeInFile, Program},
    elements::Register,
    syntax::ast::{
        self,
        visit::{self, BlockIndex, ItemIndex},
        AstSpanned,
    },
    syntax::AstToken,
};
use bind_match::bind_match;
use either::Either;
use miette::{diagnostic, LabeledSpan};
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
    Define(ItemIndex),
}

pub type FileDefRef = InFile<DefRef>;

impl MakeInFile for DefRef {}

/// This is a compile-time check that `DefRef` fits into 64 bits
const _: () = [(); 1][(core::mem::size_of::<DefRef>() == 8) as usize ^ 1];

// this size is a little sad, but I don't know how to make it smaller
const _: () = [(); 1][(core::mem::size_of::<InFile<DefRef>>() == 12) as usize ^ 1];

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum BlockName {
    GlobalBlock(Option<Name>),
    Function(Option<Name>),
    LocalBlock(Option<Name>),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RegisterDefMap {
    pub global_registers: FxHashMap<SmolStr, ast::RegisterIdentKind>,
    pub local_registers: FxHashMap<ItemIndex, FxHashMap<Name, Register>>,
}

#[salsa::tracked]
pub struct DefMap {
    #[return_ref]
    items: FxHashMap<Name, FileDefRef>,
    #[return_ref]
    registers: RegisterDefMap,
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

        let mut items = self.items(db).iter().collect::<Vec<_>>();
        items.sort();

        writeln!(output, "items:").unwrap();
        for (name, def_ref) in items {
            let file_name = def_ref.file.path(db);
            let def_ref = &def_ref.value;

            writeln!(output, "  {}: {:?} @ {}", name, def_ref, file_name).unwrap();
        }

        let registers = self.registers(db);
        let mut global_registers = registers.global_registers.iter().collect::<Vec<_>>();
        global_registers.sort_by_key(|(name, _)| *name);
        let mut local_registers = registers.local_registers.iter().collect::<Vec<_>>();
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

fn collect_item_defs(db: &dyn Db, program: Program) -> FxHashMap<Name, FileDefRef> {
    struct DefCollector {
        items: FxHashMap<Name, FileDefRef>,
    }

    impl DefCollector {
        fn define(&mut self, name: Name, item: FileDefRef) {
            match self.items.entry(name) {
                Entry::Occupied(_o) => {
                    todo!("report multiple definitions")
                }
                Entry::Vacant(v) => {
                    v.insert(item);
                }
            }
        }
    }

    impl visit::Visitor for DefCollector {
        fn visit_global_block(
            &mut self,
            file: File,
            item_index: ItemIndex,
            block_index: BlockIndex,
            block: ast::InstructionsBlock,
        ) {
            let block_id = BlockId::new_block(item_index, block_index);

            for label in block.labels().iter().flat_map(|v| v.labels()) {
                if let Some(name) = label.name() {
                    let name = Name(name.text().into());
                    self.define(name, DefRef::Block(block_id).in_file(file));
                }
            }
        }

        fn visit_function(
            &mut self,
            file: File,
            item_index: ItemIndex,
            function: ast::FunctionDefinition,
        ) {
            let block_id = BlockId::new_function(item_index);

            let Some(name) = function.name().and_then(|v| v.token()) else {
                return;
            };
            let name = Name(name.text().into());

            self.define(name.clone(), DefRef::Block(block_id).in_file(file));
        }

        fn visit_alias_definition(
            &mut self,
            file: File,
            item_index: ItemIndex,
            def: ast::AliasDefinition,
        ) {
            let Some(name) = def
                .name()
                .and_then(|v| bind_match!(v, Either::Left(v) => v)) // filter out only value aliases (not register aliases)
                .and_then(|v| v.token())
            else {
                return;
            };
            let name = Name(name.text().into());
            self.define(name, DefRef::Define(item_index).in_file(file));
        }
    }

    let mut visitor = DefCollector {
        items: FxHashMap::default(),
    };
    visit::visit_program(&mut visitor, db, program);

    visitor.items
}

fn collect_regiter_defs(db: &dyn Db, program: Program) -> RegisterDefMap {
    struct RegisterCollector<'a> {
        db: &'a dyn Db,
        global_registers: FxHashMap<SmolStr, ast::RegisterIdentKind>,
        local_registers: FxHashMap<ItemIndex, FxHashMap<Name, Register>>,
    }

    impl visit::Visitor for RegisterCollector<'_> {
        fn visit_function(
            &mut self,
            file: File,
            item_index: ItemIndex,
            function: ast::FunctionDefinition,
        ) {
            let mut local_registers = FxHashMap::default();

            for (param_index, param) in function
                .params()
                .iter()
                .flat_map(|v| v.params())
                .flat_map(|v| v.value())
                .enumerate()
            {
                let param_index: u16 = param_index.try_into().unwrap();

                let argument_register = RegisterRepr::Argument(param_index).register();

                let param = match param.kind() {
                    Ok(param) => param,
                    Err(e) => {
                        Diagnostics::emit(self.db, file, e);
                        continue;
                    }
                };
                match param {
                    ast::RegisterIdentKind::Register(reg) => {
                        if reg != argument_register {
                            todo!()
                        }
                    }
                    ast::RegisterIdentKind::Alias(name) => {
                        let name = Name(name);
                        match local_registers.entry(name) {
                            Entry::Occupied(_) => {
                                todo!()
                            }
                            Entry::Vacant(e) => {
                                e.insert(argument_register);
                            }
                        }
                    }
                }
            }

            assert!(self
                .local_registers
                .insert(item_index, local_registers)
                .is_none());
        }

        fn visit_alias_definition(
            &mut self,
            file: File,
            _item_index: ItemIndex,
            def: ast::AliasDefinition,
        ) {
            let Some((name, ident_token)) = def
                .name()
                .and_then(|v| bind_match!(v, Either::Right(v) => v)) // filter out only register aliases (not value aliases)
                .and_then(|v| v.token())
                .map(|v| (v.kind(), v))
            else {
                return;
            };
            let name = match name {
                Ok(name) => name,
                Err(e) => {
                    Diagnostics::emit(self.db, file, e);
                    return;
                }
            };
            let ast::RegisterIdentKind::Alias(name) = name else {
                let span = LabeledSpan::new_with_span(None, ident_token.miette_span());

                Diagnostics::emit(
                    self.db,
                    file,
                    diagnostic! {
                        labels = vec![span.clone()],
                        "Cannot define register alias for a built-in register",
                    },
                );
                return;
            };
            let Some(value) = def.value() else { return };
            let ast::Expr::RegisterRefExpr(value) = value else {
                let span = LabeledSpan::new_with_span(None, value.miette_span());

                Diagnostics::emit(
                    self.db,
                    file,
                    diagnostic! {
                        labels = vec![span.clone()],
                        "Expected a register reference",
                    },
                );
                return;
            };
            let value = match value.value().kind() {
                Ok(value) => value,
                Err(e) => {
                    Diagnostics::emit(self.db, file, e);
                    return;
                }
            };

            match self.global_registers.entry(name) {
                Entry::Occupied(_) => {
                    todo!()
                }
                Entry::Vacant(e) => {
                    e.insert(value);
                }
            }
        }
    }

    let mut visitor = RegisterCollector {
        db,
        global_registers: FxHashMap::default(),
        local_registers: FxHashMap::default(),
    };
    visit::visit_program(&mut visitor, db, program);

    RegisterDefMap {
        global_registers: visitor.global_registers,
        local_registers: visitor.local_registers,
    }
}

fn collect_block_names(db: &dyn Db, program: Program) -> FxHashMap<InFile<BlockId>, BlockName> {
    struct BlockNameCollector {
        block_names: FxHashMap<InFile<BlockId>, BlockName>,
    }

    fn block_name(block: &ast::InstructionsBlock) -> Option<Name> {
        block
            .labels()
            .and_then(|l| l.labels().next())
            .and_then(|l| l.name())
            .map(|name| Name(name.text().into()))
    }

    fn function_name(function: &ast::FunctionDefinition) -> Option<Name> {
        function
            .name()
            .and_then(|v| v.token())
            .map(|name| Name(name.text().into()))
    }

    impl visit::Visitor for BlockNameCollector {
        fn visit_global_block(
            &mut self,
            file: File,
            item_index: ItemIndex,
            block_index: BlockIndex,
            block: ast::InstructionsBlock,
        ) {
            let block_id = BlockId::new_block(item_index, block_index);

            self.block_names.insert(
                block_id.in_file(file),
                BlockName::GlobalBlock(block_name(&block)),
            );
        }

        fn visit_function(
            &mut self,
            file: File,
            item_index: ItemIndex,
            function: ast::FunctionDefinition,
        ) {
            self.block_names.insert(
                BlockId::new_function(item_index).in_file(file),
                BlockName::Function(function_name(&function)),
            );

            visit::visit_function(self, file, item_index, function)
        }

        fn visit_function_block(
            &mut self,
            file: File,
            item_index: ItemIndex,
            block_index: BlockIndex,
            block: ast::InstructionsBlock,
        ) {
            self.block_names.insert(
                BlockId::new_block(item_index, block_index).in_file(file),
                BlockName::LocalBlock(block_name(&block)),
            );
        }
    }

    let mut visitor = BlockNameCollector {
        block_names: FxHashMap::default(),
    };
    visit::visit_program(&mut visitor, db, program);

    visitor.block_names
}

#[salsa::tracked]
pub fn build_def_map(db: &dyn Db, program: Program) -> DefMap {
    let items = collect_item_defs(db, program);
    let registers = collect_regiter_defs(db, program);
    let block_names = collect_block_names(db, program);

    DefMap::new(db, items, registers, block_names)
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
def $_aboba = $v17
def $keka = $_abiba

function KEKA($a0, $hello, $keka)
    add $v1, 2, 2
ABOBA:
    add $v1, 3, 3
endfun

    add $v2, 2, 2
LABEL1:
    sub $v2, 2, 2
    j LABEL1
LABEL2:
        "#,
        );

        expect![[r#"
            items:
              ABIBA: Define(ItemIndex(0)) @ test.sal
              KEKA: Block(BlockId { item_index: 3, block_index: None }) @ test.sal
              LABEL1: Block(BlockId { item_index: 4, block_index: Some(1) }) @ test.sal
              LABEL2: Block(BlockId { item_index: 4, block_index: Some(2) }) @ test.sal
            registers:
              global:
                _aboba: Register($v17)
                keka: Alias("_abiba")
              local:
                item #3: 
                  hello: $a1
                  keka: $a2
            block names:
              BlockId { item_index: 3, block_index: None } @ test.sal: Function(Some(Name("KEKA")))
              BlockId { item_index: 3, block_index: Some(0) } @ test.sal: LocalBlock(None)
              BlockId { item_index: 3, block_index: Some(1) } @ test.sal: LocalBlock(Some(Name("ABOBA")))
              BlockId { item_index: 4, block_index: Some(0) } @ test.sal: GlobalBlock(None)
              BlockId { item_index: 4, block_index: Some(1) } @ test.sal: GlobalBlock(Some(Name("LABEL1")))
              BlockId { item_index: 4, block_index: Some(2) } @ test.sal: GlobalBlock(Some(Name("LABEL2")))
        "#]].assert_eq(&def_map.debug_dump(&db));
    }
}
