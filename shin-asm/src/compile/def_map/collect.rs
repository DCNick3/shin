use rustc_hash::FxHashMap;

use crate::{
    compile::{
        def_map::{BlockName, Name},
        BlockId, Db, File, MakeWithFile, Program, WithFile,
    },
    syntax::{
        ast,
        ast::{
            visit,
            visit::{BlockIndex, ItemIndex},
        },
        AstToken,
    },
};

pub fn collect_block_names(
    db: &dyn Db,
    program: Program,
) -> FxHashMap<WithFile<BlockId>, BlockName> {
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
