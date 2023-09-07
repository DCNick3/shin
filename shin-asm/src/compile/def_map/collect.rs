use crate::{
    compile::{
        def_map::{BlockName, Name, RegisterName},
        diagnostics::Span,
        make_diagnostic, BlockId, Db, DefMap, File, MakeWithFile, Program, WithFile,
    },
    elements::{Register, RegisterRepr},
    syntax::{
        ast,
        ast::visit,
        ast::visit::{BlockIndex, ItemIndex},
        AstSpanned, AstToken,
    },
};
use bind_match::bind_match;
use either::Either;
use rustc_hash::FxHashMap;
use std::collections::hash_map::Entry;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum DefRef {
    Block(BlockId, Span),
    Define(ast::Expr, ItemIndex, Span),
}

fn collect_item_defs(db: &dyn Db, program: Program) -> FxHashMap<Name, DefRef> {
    struct DefCollector {
        items: FxHashMap<Name, DefRef>,
    }

    impl DefCollector {
        fn define(&mut self, name: Name, item: DefRef) {
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
                    self.define(name, DefRef::Block(block_id, block.span(file)));
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

            let span = function
                .name()
                .map_or_else(|| function.span(file), |n| n.span(file));

            self.define(name.clone(), DefRef::Block(block_id, span));
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
            let span = name.span(file);
            let name = Name(name.text().into());

            if let Some(value) = def.value() {
                self.define(name, DefRef::Define(value, item_index, span));
            }
        }
    }

    let mut visitor = DefCollector {
        items: FxHashMap::default(),
    };
    visit::visit_program(&mut visitor, db, program);

    visitor.items
}

struct UnresolvedGlobalRegister {
    register_kind: ast::RegisterIdentKind,
    definition_span: Span,
    body_span: Span,
}

type UnresolvedGlobalRegisters = FxHashMap<RegisterName, UnresolvedGlobalRegister>;
pub type ResolvedGlobalRegisters = FxHashMap<RegisterName, Register>;
pub type LocalRegisters = FxHashMap<ItemIndex, FxHashMap<RegisterName, Register>>;

fn collect_global_registers(db: &dyn Db, program: Program) -> UnresolvedGlobalRegisters {
    struct RegisterCollector<'a> {
        db: &'a dyn Db,
        global_registers: UnresolvedGlobalRegisters,
    }

    impl visit::Visitor for RegisterCollector<'_> {
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
                return make_diagnostic!(
                    ident_token => file,
                    "Cannot define register alias for a built-in register"
                )
                .emit(self.db);
            };
            let Some(value) = def.value() else { return };
            let value_span = value.span(file);
            let ast::Expr::RegisterRefExpr(value) = value else {
                return make_diagnostic!(
                    value => file,
                    "Expected a register reference"
                )
                .emit(self.db);
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
                    e.insert(UnresolvedGlobalRegister {
                        register_kind: value,
                        definition_span: ident_token.span(file),
                        body_span: value_span,
                    });
                }
            }
        }
    }

    let mut visitor = RegisterCollector {
        db,
        global_registers: FxHashMap::default(),
    };
    visit::visit_program(&mut visitor, db, program);

    visitor.global_registers
}

fn collect_local_registers(db: &dyn Db, program: Program) -> LocalRegisters {
    struct RegisterCollector<'a> {
        db: &'a dyn Db,
        local_registers: LocalRegisters,
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
    }

    let mut visitor = RegisterCollector {
        db,
        local_registers: FxHashMap::default(),
    };
    visit::visit_program(&mut visitor, db, program);

    visitor.local_registers
}

fn resolve_global_registers(
    db: &dyn Db,
    global_registers: &UnresolvedGlobalRegisters,
) -> FxHashMap<RegisterName, Register> {
    enum NodeState {
        NotVisited,
        Visiting,
        Visited(Register),
    }
    struct RegisterResolver<'a> {
        db: &'a dyn Db,
        global_registers: &'a UnresolvedGlobalRegisters,
        node_info: FxHashMap<RegisterName, NodeState>,
    }

    impl RegisterResolver<'_> {
        fn resolve(&mut self, name: RegisterName, usage_span: Option<Span>) -> Register {
            let node_entry = self.node_info.entry(name.clone());
            let node_state = node_entry.or_insert(NodeState::NotVisited);

            match *node_state {
                NodeState::NotVisited => {
                    let result = match self.global_registers.get(&name) {
                        None => {
                            make_diagnostic!(
                                usage_span.unwrap(),
                                "Could not find the definition for register ${}",
                                name
                            )
                            .emit(self.db);

                            Register::dummy()
                        }
                        Some(&UnresolvedGlobalRegister {
                            register_kind: ast::RegisterIdentKind::Register(register),
                            ..
                        }) => register,
                        Some(&UnresolvedGlobalRegister {
                            register_kind: ast::RegisterIdentKind::Alias(ref aliased_name),
                            body_span,
                            ..
                        }) => {
                            *node_state = NodeState::Visiting;
                            self.resolve(RegisterName(aliased_name.clone()), Some(body_span))
                        }
                    };

                    // can't use node_state here due to borrow checker
                    self.node_info.insert(name, NodeState::Visited(result));

                    result
                }
                NodeState::Visiting => {
                    todo!("Handle loops");
                }
                NodeState::Visited(result) => result,
            }
        }
    }

    let mut register_resolver = RegisterResolver {
        db,
        global_registers,
        node_info: FxHashMap::default(),
    };

    global_registers
        .iter()
        .map(|(name, &_)| {
            let result = register_resolver.resolve(name.clone(), None);

            (name.clone(), result)
        })
        .collect()
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
    let local_registers = collect_local_registers(db, program);
    let global_registers = collect_global_registers(db, program);
    let global_registers = resolve_global_registers(db, &global_registers);
    let block_names = collect_block_names(db, program);

    DefMap {
        // items,
        local_registers,
        global_registers,
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
def $keka = $_aboba

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
