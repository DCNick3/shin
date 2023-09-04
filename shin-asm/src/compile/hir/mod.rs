mod lower;

use crate::compile::{BlockId, Db, File, InFile};
use crate::syntax::{ast, ptr::AstPtr};
use lower::BlockCollector;
use std::rc::Rc;

use la_arena::{Arena, Idx};
use rustc_hash::FxHashMap;
use smol_str::SmolStr;

type ExprId = Idx<Expr>;
type ExprPtr = AstPtr<ast::Expr>;
type ExprInFile = InFile<ExprPtr>;

type InstructionId = Idx<Instruction>;
type InstructionPtr = AstPtr<ast::Instruction>;
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

    pub fn get_block_ids(self, db: &dyn Db) -> impl Iterator<Item = BlockId> {
        let mut bodies = self.bodies(db).keys().cloned().collect::<Vec<_>>();
        bodies.sort();

        bodies.into_iter()
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

                let (block, source_map) = collector.collect();

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

#[cfg(test)]
mod tests {
    use crate::compile::{db::Database, hir, Diagnostics, File};

    #[test]
    fn bodies() {
        let db = Database::default();
        let db = &db;
        let file = File::new(
            db,
            "test.sal".to_string(),
            r#"
LABEL_1:
    add "ass", 2, 2
    MSGINIT -1

LABEL_2:
    add $1, -1, 14
    jt $v0, {
        0 => SNR_0,
        1 => SNR_1,
    } // parser does not split these blocks for some reason..
LABEL_3:
    SELECT 1, 2, $choice, 14, "NCSELECT", [
        "To Be",
        "Not to Be",
    ]
    
LABEL_4:
    // eh, parser seems to get stuck on parenthesis
    exp $result, 1 * ($2 + $3 & 7)
    
function FUN_1($a, $b)[$v2-$v3]
LABEL_5:
    MSGSET 12, "HELLO"
LABEL_6:
    MSGSET 13, "WORLD" 
endfun
            "#
            .to_string(),
        );

        // eprintln!("{}", file.parse_debug_dump(db));

        let bodies = hir::collect_file_bodies(db, file);

        for block_id in bodies.get_block_ids(db) {
            // TODO: resolve block names
            // DefMap would have to be augmented for this
            eprintln!("{:?}:", block_id.repr());
            let block = bodies.get(db, block_id).unwrap();
            eprintln!("  exprs:");
            for (id, expr) in block.exprs.iter() {
                eprintln!("    {:?}: {:?}", id, expr);
            }
            eprintln!("  isns:");
            for (id, instruction) in block.instructions.iter() {
                eprintln!("    {:?}: {:?}", id, instruction)
            }
            eprintln!();
        }

        let diagnostics = hir::collect_file_bodies::accumulated::<Diagnostics>(db, file);
        for diagnostic in Diagnostics::with_source(db, diagnostics) {
            eprintln!("{:?}", diagnostic);
        }
    }
}
