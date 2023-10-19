mod from_ast;
#[cfg(test)]
mod tests;

use crate::compile::{BlockId, Db, File, WithFile};
use crate::syntax::{ast, ptr::AstPtr};
use from_ast::HirBlockCollector;
use std::rc::Rc;

use crate::compile::def_map::Name;
use crate::compile::diagnostics::Diagnostic;
use crate::syntax::ast::visit;
use crate::syntax::ast::visit::{BlockIndex, ItemIndex};
use la_arena::{Arena, Idx};
use rustc_hash::FxHashMap;
use shin_core::rational::Rational;
use smol_str::SmolStr;
use text_size::TextRange;

pub type ExprId = Idx<Expr>;
pub type ExprIdInFile = WithFile<ExprId>;
pub type ExprPtr = AstPtr<ast::Expr>;
#[allow(unused)] // Will be used when full hir source maps will be implemented
pub type ExprPtrInFile = WithFile<ExprPtr>;

pub type InstructionId = Idx<Instruction>;
pub type InstructionIdInFile = WithFile<InstructionId>;
pub type InstructionPtr = AstPtr<ast::Instruction>;
#[allow(unused)] // Will be used when full hir source maps will be implemented
pub type InstructionPtrInFile = WithFile<InstructionPtr>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Literal {
    String(SmolStr),
    IntNumber(i32),
    RationalNumber(Rational),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Expr {
    Missing,
    Literal(Literal),
    NameRef(Name),
    RegisterRef(Option<ast::RegisterIdentKind>),
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
    pub name: Option<SmolStr>,
    pub args: Box<[ExprId]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HirBlockBody {
    pub exprs: Arena<Expr>,
    pub instructions: Arena<Instruction>,
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
    pub fn get_block(self, db: &dyn Db, block_id: BlockId) -> Option<Rc<HirBlockBody>> {
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
    struct FileBodiesCollector<'a> {
        db: &'a dyn Db,
        // TODO: actually, the map from the BlockId is somewhat dense...
        // but I don't want to build a specialized container for it (yet)
        block_bodies: FxHashMap<BlockId, Rc<HirBlockBody>>,
    }

    impl visit::Visitor for FileBodiesCollector<'_> {
        fn visit_any_block(
            &mut self,
            file: File,
            item_index: ItemIndex,
            block_index: BlockIndex,
            block: ast::InstructionsBlock,
        ) {
            let mut collector = HirBlockCollector::new();

            if let Some(body) = block.body() {
                for instruction in body.instructions() {
                    collector.collect_instruction(instruction);
                }
            }

            // TODO: collect source maps
            let (block, _source_map, diagnostics) = collector.collect();

            for e in diagnostics {
                e.in_file(file).emit(self.db)
            }

            self.block_bodies
                .insert(BlockId::new_block(item_index, block_index), Rc::new(block));
        }
    }

    let mut visitor = FileBodiesCollector {
        db,
        block_bodies: FxHashMap::default(),
    };
    visit::visit_file(&mut visitor, file, file.parse(db));

    HirBlockBodies::new(db, visitor.block_bodies)
}

/// Collects an expression without a real Block into a Hir expression
///
/// It constructs a fake block to contain the expression
///
/// Note that unlike [`collect_file_bodies`], this doesn't doesn't use salsa db and doesn't emit diagnostics for you
pub fn collect_bare_expression_raw(
    expr: ast::Expr,
) -> (HirBlockBody, ExprId, Vec<Diagnostic<TextRange>>) {
    let mut collector = HirBlockCollector::new();

    let expr_id = collector.collect_expr(expr);

    // TODO: expose the source map in some way
    let (block_body, _source_map, diagnostiscs) = collector.collect();

    (block_body, expr_id, diagnostiscs)
}
