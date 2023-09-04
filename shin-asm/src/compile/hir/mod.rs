mod lower;
#[cfg(test)]
mod tests;

use crate::compile::{BlockId, Db, File, InFile};
use crate::syntax::{ast, ptr::AstPtr};
use lower::BlockCollector;
use std::rc::Rc;

use la_arena::{Arena, Idx};
use rustc_hash::FxHashMap;
use smol_str::SmolStr;

type ExprId = Idx<Expr>;
type ExprPtr = AstPtr<ast::Expr>;
#[allow(unused)] // Will be used when full hir source maps will be implemented
type ExprInFile = InFile<ExprPtr>;

type InstructionId = Idx<Instruction>;
type InstructionPtr = AstPtr<ast::Instruction>;
#[allow(unused)] // Will be used when full hir source maps will be implemented
type InstructionInFile = InFile<InstructionPtr>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Literal {
    String(SmolStr),
    IntNumber(i32),
    FloatNumber(i32), // TODO: this should be fixed decimal point (1.0 is represented as 1000)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Expr {
    Missing,
    Literal(Literal),
    NameRef(SmolStr),
    RegisterRef(SmolStr),
    Array(Box<[ExprId]>),
    Mapping(Box<[(Option<i32>, ExprId)]>),
    UnaryOp {
        expr: ExprId,
        op: ast::UnaryOp,
    },
    BinaryOp {
        lhs: ExprId,
        rhs: ExprId,
        op: Option<ast::BinaryOp>,
    },
    Call {
        target: SmolStr,
        args: Box<[ExprId]>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Instruction {
    name: Option<SmolStr>,
    args: Box<[ExprId]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HirBlockBody {
    exprs: Arena<Expr>,
    instructions: Arena<Instruction>,
}

impl HirBlockBody {
    pub fn debug_dump(&self) -> String {
        use std::fmt::Write as _;

        let mut output = String::new();
        writeln!(output, "exprs:").unwrap();
        for (id, expr) in self.exprs.iter() {
            writeln!(output, "  {}: {:?}", id.into_raw(), expr).unwrap();
        }
        writeln!(output, "isns:").unwrap();
        for (id, instruction) in self.instructions.iter() {
            writeln!(output, "  {}: {:?}", id.into_raw(), instruction).unwrap();
        }

        output
    }
}

#[derive(Default, Clone, Debug, Eq, PartialEq)]
pub struct BlockSourceMap {
    exprs_source_map: FxHashMap<ExprId, ExprPtr>,
    instructions_source_map: FxHashMap<InstructionId, InstructionPtr>,
}

#[salsa::tracked]
pub struct HirBlockBodies {
    #[return_ref]
    bodies: FxHashMap<BlockId, Rc<HirBlockBody>>,
}

#[salsa::tracked]
impl HirBlockBodies {
    #[salsa::tracked]
    pub fn get(self, db: &dyn Db, block_id: BlockId) -> Option<Rc<HirBlockBody>> {
        self.bodies(db).get(&block_id).cloned()
    }

    pub fn get_block_ids(self, db: &dyn Db) -> Vec<BlockId> {
        let mut bodies = self.bodies(db).keys().cloned().collect::<Vec<_>>();
        bodies.sort();

        bodies
    }
}

#[salsa::tracked]
pub fn collect_file_bodies(db: &dyn Db, file: File) -> HirBlockBodies {
    // TODO: actually, the map from the BlockId is somewhat dense...
    // but I don't want to build a specialized container for it (yet)
    let mut result = FxHashMap::default();

    let source_file = file.parse(db);
    for (item_index, item) in source_file.items().enumerate() {
        let item_index = item_index.try_into().unwrap();
        let mut collect_blocks = |blocks: ast::AstChildren<ast::InstructionsBlock>| {
            for (block_index, block) in blocks.enumerate() {
                let block_index = block_index.try_into().unwrap();
                let mut collector = BlockCollector::new(db, file);

                if let Some(body) = block.body() {
                    for instruction in body.instructions() {
                        collector.collect_instruction(instruction);
                    }
                }

                // TODO: collect source maps
                let (block, _source_map) = collector.collect();

                result.insert(BlockId::new_block(item_index, block_index), Rc::new(block));
            }
        };

        match item {
            ast::Item::InstructionsBlockSet(block_set) => {
                collect_blocks(block_set.blocks());
            }
            ast::Item::FunctionDefinition(function) => {
                if let Some(block_set) = function.instruction_block_set() {
                    collect_blocks(block_set.blocks());
                }
            }
            ast::Item::AliasDefinition(_) => {} // nothing to do here
        }
    }

    HirBlockBodies::new(db, result)
}
