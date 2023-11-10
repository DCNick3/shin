mod from_ast;
pub mod lower;
#[cfg(test)]
mod tests;

use std::rc::Rc;

use from_ast::HirBlockCollector;
use la_arena::{Arena, Idx};
use rustc_hash::FxHashMap;
use shin_core::rational::Rational;
use smol_str::SmolStr;
use text_size::TextRange;

use crate::{
    compile::{
        def_map::Name,
        diagnostics::{Diagnostic, HirLocation},
        hir::lower::LowerResult,
        BlockId, Db, File, MakeWithFile, WithFile,
    },
    syntax::{
        ast,
        ast::{
            visit,
            visit::{BlockIndex, ItemIndex},
        },
        ptr::AstPtr,
        AstSpanned,
    },
};

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

#[derive(Debug, Copy, Clone)]
pub enum HirId {
    Expr(ExprId),
    Instruction(InstructionId),
}

#[derive(Debug, Copy, Clone)]
pub enum HirBlockId {
    Block(BlockId),
    Alias(ItemIndex),
}

impl From<BlockId> for HirBlockId {
    fn from(value: BlockId) -> Self {
        Self::Block(value)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct HirIdWithBlock {
    // TODO: naming is unclear...
    // InBlock -> identifies inside a block or HirId wrapped with a block id (like InFile)
    // Probably should rename the InFile to WithFile
    pub id: HirId,
    pub block_id: HirBlockId,
}

impl HirIdWithBlock {
    pub fn new(id: impl Into<HirId>, block_id: impl Into<HirBlockId>) -> Self {
        Self {
            block_id: block_id.into(),
            id: id.into(),
        }
    }
}

impl From<ExprId> for HirId {
    fn from(id: ExprId) -> Self {
        Self::Expr(id)
    }
}

impl From<InstructionId> for HirId {
    fn from(id: InstructionId) -> Self {
        Self::Instruction(id)
    }
}

impl MakeWithFile for HirIdWithBlock {}

pub type HirIdWithFile = WithFile<HirIdWithBlock>;

type HirDiagnostic = Diagnostic<HirLocation>;

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
    RegisterRef(LowerResult<ast::RegisterIdentKind>),
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

impl Expr {
    pub fn describe_ty(&self) -> String {
        match self {
            Expr::Missing => "a missing expression",
            Expr::Literal(Literal::String(_)) => "a string literal",
            Expr::Literal(Literal::IntNumber(_)) => "an integer literal",
            Expr::Literal(Literal::RationalNumber(_)) => "a rational literal",
            Expr::NameRef(_) => "a name reference",
            Expr::RegisterRef(_) => "a register reference",
            Expr::Array(_) => "an array",
            Expr::Mapping(_) => "a mapping",
            _ => "an expression",
        }
        .to_string()
    }
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
    expressions_source_map: FxHashMap<ExprId, ExprPtr>,
    instructions_source_map: FxHashMap<InstructionId, InstructionPtr>,
}

impl BlockSourceMap {
    pub fn get_text_range(&self, id: HirId) -> Option<TextRange> {
        match id {
            HirId::Expr(id) => self
                .expressions_source_map
                .get(&id)
                .map(|ptr| ptr.text_range()),
            HirId::Instruction(id) => self
                .instructions_source_map
                .get(&id)
                .map(|ptr| ptr.text_range()),
        }
    }
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

    #[salsa::tracked]
    pub fn get_block_ids(self, db: &dyn Db) -> Vec<BlockId> {
        let mut bodies = self.bodies(db).keys().cloned().collect::<Vec<_>>();
        bodies.sort();

        bodies
    }
}

#[salsa::tracked]
pub struct HirBlockBodySourceMaps {
    #[return_ref]
    bodies: FxHashMap<BlockId, Rc<BlockSourceMap>>,
}

#[salsa::tracked]
impl HirBlockBodySourceMaps {
    #[salsa::tracked]
    pub fn get_block(self, db: &dyn Db, block_id: BlockId) -> Option<Rc<BlockSourceMap>> {
        self.bodies(db).get(&block_id).cloned()
    }
}

#[salsa::tracked]
pub fn collect_file_bodies_with_source_maps(
    db: &dyn Db,
    file: File,
) -> (HirBlockBodies, HirBlockBodySourceMaps) {
    struct FileBodiesCollector<'a> {
        db: &'a dyn Db,
        // TODO: actually, the map from the BlockId is somewhat dense...
        // but I don't want to build a specialized container for it (yet)
        block_bodies: FxHashMap<BlockId, Rc<HirBlockBody>>,
        block_source_maps: FxHashMap<BlockId, Rc<BlockSourceMap>>,
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

            let (block, source_map, diagnostics) = collector.collect();

            for e in diagnostics {
                e.in_file(file).emit(self.db)
            }

            let block_id = BlockId::new_block(item_index, block_index);
            self.block_bodies.insert(block_id, Rc::new(block));
            self.block_source_maps.insert(block_id, Rc::new(source_map));
        }
    }

    let mut visitor = FileBodiesCollector {
        db,
        block_bodies: FxHashMap::default(),
        block_source_maps: FxHashMap::default(),
    };
    visit::visit_file(&mut visitor, file, file.parse(db));

    let bodies = HirBlockBodies::new(db, visitor.block_bodies);
    let source_maps = HirBlockBodySourceMaps::new(db, visitor.block_source_maps);

    (bodies, source_maps)
}

#[salsa::tracked]
pub fn collect_file_bodies(db: &dyn Db, file: File) -> HirBlockBodies {
    let (bodies, _) = collect_file_bodies_with_source_maps(db, file);

    bodies
}

/// Collects an expression without a real Block into a Hir expression
///
/// It constructs a fake block to contain the expression
///
/// Note that unlike [`collect_file_bodies`], this doesn't doesn't use salsa db and doesn't emit diagnostics for you
pub fn collect_bare_expression_raw(
    expr: ast::Expr,
) -> (
    HirBlockBody,
    ExprId,
    FxHashMap<ExprId, ExprPtr>,
    Vec<Diagnostic<TextRange>>,
) {
    let mut collector = HirBlockCollector::new();

    let expr_id = collector.collect_expr(expr);

    let (block_body, source_map, diagnostiscs) = collector.collect();

    (
        block_body,
        expr_id,
        source_map.expressions_source_map,
        diagnostiscs,
    )
}
