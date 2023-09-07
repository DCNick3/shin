use crate::{
    compile::{
        def_map::{BlockName, Name, RegisterDefMap, RegisterName},
        emit_diagnostic, BlockId, Db, DefMap, DefRef, File, FileDefRef, MakeWithFile, Program,
        WithFile,
    },
    elements::{Register, RegisterRepr},
    syntax::{
        ast,
        ast::visit,
        ast::visit::{BlockIndex, ItemIndex},
        AstToken,
    },
};
use bind_match::bind_match;
use either::Either;
use rustc_hash::FxHashMap;
use std::collections::hash_map::Entry;

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
        global_registers: FxHashMap<RegisterName, ast::RegisterIdentKind>,
        local_registers: FxHashMap<ItemIndex, FxHashMap<RegisterName, Register>>,
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
                        return e.in_file(file).emit(self.db);
                    }
                };
                match param {
                    ast::RegisterIdentKind::Register(reg) => {
                        if reg != argument_register {
                            todo!()
                        }
                    }
                    ast::RegisterIdentKind::Alias(name) => {
                        match local_registers.entry(RegisterName(name)) {
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
                Err(e) => return e.in_file(file).emit(self.db),
            };
            let ast::RegisterIdentKind::Alias(name) = name else {
                return emit_diagnostic!(
                    self.db,
                    ident_token => file,
                    "Cannot define register alias for a built-in register"
                );
            };
            let Some(value) = def.value() else { return };
            let ast::Expr::RegisterRefExpr(value) = value else {
                return emit_diagnostic!(
                    self.db,
                    value => file,
                    "Expected a register reference"
                );
            };
            let value = match value.value().kind() {
                Ok(value) => value,
                Err(e) => return e.in_file(file).emit(self.db),
            };

            match self.global_registers.entry(RegisterName(name)) {
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

fn collect_block_names(db: &dyn Db, program: Program) -> FxHashMap<WithFile<BlockId>, BlockName> {
    struct BlockNameCollector {
        block_names: FxHashMap<WithFile<BlockId>, BlockName>,
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

    DefMap {
        items,
        registers,
        block_names,
    }
}

#[cfg(test)]
mod tests {
    use super::build_def_map;
    use crate::compile::diagnostics::{HirDiagnosticAccumulator, SourceDiagnosticAccumulator};
    use crate::compile::{db::Database, DefMap, File, Program};
    use expect_test::expect;

    fn parse_def_map(code: &str) -> (Database, DefMap) {
        let db = Database::default();
        let file = File::new(&db, "test.sal".to_string(), code.to_string());
        let program = Program::new(&db, vec![file]);
        let def_map = build_def_map(&db, program);

        let hir_errors = build_def_map::accumulated::<HirDiagnosticAccumulator>(&db, program);
        let source_errors = build_def_map::accumulated::<SourceDiagnosticAccumulator>(&db, program);
        if !source_errors.is_empty() || !hir_errors.is_empty() {
            panic!(
                "building def map produced errors:\n\
                source-level: {source_errors:?}\n\
                hir-level: {hir_errors:?}"
            );
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
