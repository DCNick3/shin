mod lower;

use crate::db::file::File;
use crate::db::in_file::InFile;
use crate::db::Db;
use crate::syntax::{ast, ptr::AstPtr};
use lower::BlockCollector;

use la_arena::{Arena, Idx};
use rustc_hash::FxHashMap;
use smol_str::SmolStr;

type ExprId = Idx<Expr>;
type ExprPtr = AstPtr<ast::Expr>;
type ExprSource = InFile<ExprPtr>;

type InstructionId = Idx<Instruction>;
type InstructionPtr = AstPtr<ast::Instruction>;
type InstructionSource = InFile<InstructionPtr>;

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
pub struct Block {
    exprs: Arena<Expr>,
    instructions: Arena<Instruction>,
}

#[derive(Default, Clone, Debug, Eq, PartialEq)]
pub struct BlockSourceMap {
    exprs_source_map: FxHashMap<ExprId, ExprPtr>,
    instructions_source_map: FxHashMap<InstructionId, InstructionPtr>,
}

#[salsa::tracked]
pub fn collect_file_bodies(db: &dyn Db, file: File) -> Vec<Block> {
    let mut result = Vec::new();

    let source_file = file.parse(db).syntax(db);
    for item in source_file.items() {
        match item {
            ast::Item::InstructionsBlock(block) => {
                let mut collector = BlockCollector::new(db, file);

                for instruction in block.instructions() {
                    collector.collect_instruction(instruction);
                }

                let (block, source_map) = collector.collect();

                result.push(block);
            }
            ast::Item::FunctionDefinition(_) => {
                todo!()
            }
            ast::Item::AliasDefinition(_) => {} // nothing to do here
        }
    }

    result
}
