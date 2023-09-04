mod lower;

use crate::compile::{Db, File, InFile};
use crate::syntax::{ast, ptr::AstPtr};
use lower::BlockCollector;

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
            ast::Item::InstructionsBlockSet(blocks) => {
                for block in blocks.blocks() {
                    let mut collector = BlockCollector::new(db, file);

                    if let Some(body) = block.body() {
                        for instruction in body.instructions() {
                            collector.collect_instruction(instruction);
                        }
                    }

                    let (block, source_map) = collector.collect();

                    result.push(block);
                }
            }
            ast::Item::FunctionDefinition(_) => {
                todo!()
            }
            ast::Item::AliasDefinition(_) => {} // nothing to do here
        }
    }

    result
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
            "#
            .to_string(),
        );

        let bodies = hir::collect_file_bodies(db, file);

        dbg!(bodies);

        let diagnostics = hir::collect_file_bodies::accumulated::<Diagnostics>(db, file);
        for diagnostic in Diagnostics::with_source(db, diagnostics) {
            dbg!(diagnostic.labels().unwrap().collect::<Vec<_>>());
            eprintln!("{:?}", diagnostic);
        }
    }
}
