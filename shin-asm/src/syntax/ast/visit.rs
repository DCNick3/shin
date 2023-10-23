//! AST visitor.
//!
//! The API is copied from `syn`

use crate::compile::{Db, File, Program};
use crate::syntax::ast;
use shin_asm::compile::MakeWithFile;
use std::fmt::Display;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ItemIndex(u32);
impl Display for ItemIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{}", self.0)
    }
}
impl From<u32> for ItemIndex {
    fn from(v: u32) -> Self {
        Self(v)
    }
}
impl From<ItemIndex> for u32 {
    fn from(v: ItemIndex) -> Self {
        v.0
    }
}

impl MakeWithFile for ItemIndex {}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct BlockIndex(u32);
impl Display for BlockIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{}", self.0)
    }
}
impl From<u32> for BlockIndex {
    fn from(v: u32) -> Self {
        Self(v)
    }
}
impl From<BlockIndex> for u32 {
    fn from(v: BlockIndex) -> Self {
        v.0
    }
}

pub fn visit_program<V: Visitor>(visitor: &mut V, db: &dyn Db, program: Program) {
    for &file in program.files(db) {
        let syntax = file.parse(db);
        visitor.visit_file(file, syntax);
    }
}

pub fn visit_file<V: Visitor>(visitor: &mut V, file: File, syntax: ast::SourceFile) {
    for (item_index, item) in syntax.items().enumerate() {
        let item_index = item_index.try_into().unwrap();
        visitor.visit_item(file, ItemIndex(item_index), item);
    }
}
pub fn visit_item<V: Visitor>(visitor: &mut V, file: File, item_index: ItemIndex, item: ast::Item) {
    match item {
        ast::Item::InstructionsBlockSet(block_set) => {
            visitor.visit_global_block_set(file, item_index, block_set)
        }
        ast::Item::FunctionDefinition(function) => {
            visitor.visit_function(file, item_index, function);
        }
        ast::Item::AliasDefinition(alias) => {
            visitor.visit_alias_definition(file, item_index, alias)
        }
    }
}

pub fn visit_global_block_set<V: Visitor>(
    visitor: &mut V,
    file: File,
    item_index: ItemIndex,
    block_set: ast::InstructionsBlockSet,
) {
    for (block_index, block) in block_set.blocks().enumerate() {
        let block_index = block_index.try_into().unwrap();
        visitor.visit_global_block(file, item_index, BlockIndex(block_index), block);
    }
}

pub fn visit_global_block<V: Visitor>(
    visitor: &mut V,
    file: File,
    item_index: ItemIndex,
    block_index: BlockIndex,
    block: ast::InstructionsBlock,
) {
    visitor.visit_any_block(file, item_index, block_index, block);
}

pub fn visit_function<V: Visitor>(
    visitor: &mut V,
    file: File,
    item_index: ItemIndex,
    function: ast::FunctionDefinition,
) {
    for (block_index, block) in function
        .instruction_block_set()
        .iter()
        .flat_map(|v| v.blocks())
        .enumerate()
    {
        let block_index = block_index.try_into().unwrap();
        visitor.visit_function_block(file, item_index, BlockIndex(block_index), block);
    }
}

pub fn visit_function_block<V: Visitor>(
    visitor: &mut V,
    file: File,
    item_index: ItemIndex,
    block_index: BlockIndex,
    block: ast::InstructionsBlock,
) {
    visitor.visit_any_block(file, item_index, block_index, block);
}

pub fn visit_alias_definition<V: Visitor>(
    _visitor: &mut V,
    _file: File,
    _item_index: ItemIndex,
    _def: ast::AliasDefinition,
) {
}

pub trait Visitor: Sized {
    fn visit_file(&mut self, file: File, syntax: ast::SourceFile) {
        visit_file(self, file, syntax);
    }
    fn visit_item(&mut self, file: File, item_index: ItemIndex, item: ast::Item) {
        visit_item(self, file, item_index, item);
    }
    fn visit_global_block_set(
        &mut self,
        file: File,
        item_index: ItemIndex,
        block_set: ast::InstructionsBlockSet,
    ) {
        visit_global_block_set(self, file, item_index, block_set);
    }
    fn visit_global_block(
        &mut self,
        file: File,
        item_index: ItemIndex,
        block_index: BlockIndex,
        block: ast::InstructionsBlock,
    ) {
        visit_global_block(self, file, item_index, block_index, block);
    }
    fn visit_function(
        &mut self,
        file: File,
        item_index: ItemIndex,
        function: ast::FunctionDefinition,
    ) {
        visit_function(self, file, item_index, function);
    }
    fn visit_function_block(
        &mut self,
        file: File,
        item_index: ItemIndex,
        block_index: BlockIndex,
        block: ast::InstructionsBlock,
    ) {
        visit_function_block(self, file, item_index, block_index, block);
    }
    fn visit_any_block(
        &mut self,
        _file: File,
        _item_index: ItemIndex,
        _block_index: BlockIndex,
        _block: ast::InstructionsBlock,
    ) {
    }
    fn visit_alias_definition(
        &mut self,
        file: File,
        item_index: ItemIndex,
        def: ast::AliasDefinition,
    ) {
        visit_alias_definition(self, file, item_index, def);
    }
}
