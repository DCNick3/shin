use crate::compile::constexpr::ConstexprValue;
use crate::{
    compile::{def_map::Name, diagnostics::Span, BlockId, Db, File, Program},
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
    Define(ast::Expr, ItemIndex, Span),
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum DefValue {
    Block(BlockId),
    Define(ConstexprValue),
}

type UnresolvedDefMap = FxHashMap<Name, DefRef>;
pub type ResolvedDefMap = FxHashMap<Name, DefValue>;

pub fn collect_item_defs(db: &dyn Db, program: Program) -> UnresolvedDefMap {
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

pub fn resolve_item_defs(db: &dyn Db, def_map: &UnresolvedDefMap) -> ResolvedDefMap {
    // TODO: implement
    ResolvedDefMap::default()
}
