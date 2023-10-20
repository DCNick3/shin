use crate::compile::constexpr::{constexpr_evaluate, ContexprContextValue};
use crate::compile::{hir, make_diagnostic, MakeWithFile};
use crate::{
    compile::{
        constexpr::ConstexprValue, def_map::Name, diagnostics::Span, BlockId, BlockIdWithFile, Db,
        File, Program,
    },
    syntax::{
        ast::visit,
        ast::visit::BlockIndex,
        ast::{self, visit::ItemIndex},
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
    Value(ast::Expr, ItemIndex, Span),
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum DefValue {
    Block(BlockIdWithFile),
    Value(ConstexprValue),
}

type UnresolvedItems = FxHashMap<Name, DefRef>;
pub type ResolvedItems = FxHashMap<Name, DefValue>;

pub fn collect_item_defs(db: &dyn Db, program: Program) -> UnresolvedItems {
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
                self.define(name, DefRef::Value(value, item_index, span));
            }
        }
    }

    let mut visitor = DefCollector {
        items: FxHashMap::default(),
    };
    visit::visit_program(&mut visitor, db, program);

    visitor.items
}

pub fn resolve_item_defs(db: &dyn Db, def_map: &UnresolvedItems) -> ResolvedItems {
    enum NodeState {
        NotVisited,
        Visiting,
        Visited((DefValue, Option<Span>)),
    }
    struct DefResolver<'a> {
        db: &'a dyn Db,
        def_map: &'a UnresolvedItems,
        node_info: FxHashMap<Name, NodeState>,
    }

    impl DefResolver<'_> {
        fn resolve(&mut self, name: Name, usage_span: Option<Span>) -> (DefValue, Option<Span>) {
            let node_entry = self.node_info.entry(name.clone());
            let node_state = node_entry.or_insert(NodeState::NotVisited);

            match &*node_state {
                NodeState::NotVisited => {
                    let result = match self.def_map.get(&name) {
                        None => {
                            make_diagnostic!(
                                usage_span.unwrap(),
                                "Could not find the definition of `{}`",
                                name
                            )
                            .emit(self.db);

                            (DefValue::Value(ConstexprValue::dummy()), None)
                        }
                        Some(&DefRef::Block(block_id, span)) => {
                            (DefValue::Block(block_id.in_file(span.file())), Some(span))
                        }
                        Some(&DefRef::Value(ref expr, _item_index, span)) => {
                            let (block, root_expr_id, expr_source_map, diagnostics) =
                                hir::collect_bare_expression_raw(expr.clone());

                            for diag in diagnostics {
                                diag.in_file(span.file()).emit(self.db);
                            }

                            *node_state = NodeState::Visiting;

                            let mut constexpr_context = FxHashMap::default();
                            for expr in block.exprs.values() {
                                if let hir::Expr::NameRef(name) = expr {
                                    // TODO: use hir source map to provide a more granular `usage_span`
                                    let (value, span) = self.resolve(name.clone(), Some(span));

                                    let value = match value {
                                        DefValue::Block(_) => ContexprContextValue::Block(
                                            span.expect("BUG: block value without span"),
                                        ),
                                        DefValue::Value(value) => {
                                            ContexprContextValue::Value(value, span)
                                        }
                                    };

                                    constexpr_context.insert(name.clone(), value);
                                }
                            }

                            let (value, diagnostics) =
                                constexpr_evaluate(&constexpr_context, &block, root_expr_id);

                            for diag in diagnostics {
                                diag.map_location(|location| {
                                    location.right_or_else(|id| {
                                        expr_source_map.get(&id).unwrap().span(span.file())
                                    })
                                })
                                .emit(self.db);
                            }

                            (DefValue::Value(value), Some(span))
                        }
                    };

                    // can't use node_state here due to borrow checker
                    self.node_info
                        .insert(name, NodeState::Visited(result.clone()));

                    result
                }
                NodeState::Visiting => {
                    make_diagnostic!(
                        usage_span.unwrap(),
                        "Encountered a loop while resolving the definition of `{}`",
                        name
                    )
                    .emit(self.db);

                    return (DefValue::Value(ConstexprValue::dummy()), None);
                }
                NodeState::Visited(result) => result.clone(),
            }
        }
    }

    let mut def_map_resolver = DefResolver {
        db,
        def_map,
        node_info: FxHashMap::default(),
    };

    def_map
        .iter()
        .map(|(name, &_)| {
            let (value, _span) = def_map_resolver.resolve(name.clone(), None);

            (name.clone(), value)
        })
        .collect()
}
