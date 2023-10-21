use crate::{
    compile::{def_map::RegisterName, diagnostics::Span, make_diagnostic, Db, File, Program},
    syntax::{
        ast::{self, visit, visit::ItemIndex},
        AstSpanned,
    },
};
use bind_match::bind_match;
use either::Either;
use rustc_hash::FxHashMap;
use shin_core::format::scenario::instruction_elements::{Register, RegisterRepr};
use std::collections::hash_map::Entry;

pub struct UnresolvedGlobalRegister {
    register_kind: ast::RegisterIdentKind,
    definition_span: Span,
    body_span: Span,
}

type UnresolvedGlobalRegisters = FxHashMap<RegisterName, UnresolvedGlobalRegister>;
pub type ResolvedGlobalRegisters = FxHashMap<RegisterName, Option<Register>>;
pub type LocalRegisters = FxHashMap<ItemIndex, FxHashMap<RegisterName, Register>>;

pub fn collect_global_registers(db: &dyn Db, program: Program) -> UnresolvedGlobalRegisters {
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

pub fn collect_local_registers(db: &dyn Db, program: Program) -> LocalRegisters {
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

pub fn resolve_global_registers(
    db: &dyn Db,
    global_registers: &UnresolvedGlobalRegisters,
) -> FxHashMap<RegisterName, Option<Register>> {
    enum NodeState {
        NotVisited,
        Visiting,
        Visited(Option<Register>),
    }
    struct RegisterResolver<'a> {
        db: &'a dyn Db,
        global_registers: &'a UnresolvedGlobalRegisters,
        node_info: FxHashMap<RegisterName, NodeState>,
    }

    impl RegisterResolver<'_> {
        fn resolve(&mut self, name: RegisterName, usage_span: Option<Span>) -> Option<Register> {
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

                            None
                        }
                        Some(&UnresolvedGlobalRegister {
                            register_kind: ast::RegisterIdentKind::Register(register),
                            ..
                        }) => Some(register),
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
                    make_diagnostic!(
                        usage_span.unwrap(),
                        "Encountered a loop while resolving register ${}",
                        name
                    )
                    .emit(self.db);

                    return None;
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
