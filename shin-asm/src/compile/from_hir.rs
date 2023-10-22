use crate::compile::diagnostics::{Diagnostic, HirLocation};
use crate::compile::{
    hir::{self, HirBlockBody},
    resolve, BlockId, File, MakeWithFile, WithFile,
};
use crate::syntax::ast::visit::ItemIndex;

#[derive(Debug, Copy, Clone)]
pub enum HirId {
    Expr(hir::ExprId),
    Instruction(hir::InstructionId),
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

impl From<hir::ExprId> for HirId {
    fn from(id: hir::ExprId) -> Self {
        Self::Expr(id)
    }
}

impl From<hir::InstructionId> for HirId {
    fn from(id: hir::InstructionId) -> Self {
        Self::Instruction(id)
    }
}

impl MakeWithFile for HirIdWithBlock {}

pub type HirIdWithFile = WithFile<HirIdWithBlock>;

type HirDiagnostic = Diagnostic<HirLocation>;

#[derive(Default, Debug)]
pub struct HirDiagnosticCollector {
    diagnostics: Vec<HirDiagnostic>,
}

impl HirDiagnosticCollector {
    pub fn new() -> Self {
        Self {
            diagnostics: Vec::new(),
        }
    }

    pub fn emit(&mut self, location: HirLocation, message: String) {
        self.diagnostics.push(HirDiagnostic::new(message, location));
    }

    pub fn is_empty(&self) -> bool {
        self.diagnostics.is_empty()
    }

    pub fn with_file(&mut self, file: File) -> HirDiagnosticCollectorWithFile<'_> {
        HirDiagnosticCollectorWithFile::new(self, file)
    }

    pub fn into_diagnostics(self) -> Vec<HirDiagnostic> {
        self.diagnostics
    }
}

pub struct HirDiagnosticCollectorWithFile<'a> {
    diagnostics: &'a mut HirDiagnosticCollector,
    file: File,
}

impl<'a> HirDiagnosticCollectorWithFile<'a> {
    pub fn new(
        diagnostics: &'a mut HirDiagnosticCollector,
        file: File,
    ) -> HirDiagnosticCollectorWithFile<'a> {
        HirDiagnosticCollectorWithFile { diagnostics, file }
    }

    pub fn with_block(&'a mut self, block: HirBlockId) -> HirDiagnosticCollectorWithBlock<'a> {
        HirDiagnosticCollectorWithBlock::new(self, block)
    }

    pub fn emit(&mut self, location: HirIdWithBlock, message: String) {
        self.diagnostics.emit(location.in_file(self.file), message);
    }
}

pub struct HirDiagnosticCollectorWithBlock<'a> {
    diagnostics: &'a mut HirDiagnosticCollectorWithFile<'a>,
    block: HirBlockId,
}

impl<'a> HirDiagnosticCollectorWithBlock<'a> {
    pub fn new(
        diagnostics: &'a mut HirDiagnosticCollectorWithFile<'a>,
        block: HirBlockId,
    ) -> HirDiagnosticCollectorWithBlock<'a> {
        HirDiagnosticCollectorWithBlock { diagnostics, block }
    }

    pub fn emit(&mut self, location: HirId, message: String) {
        self.diagnostics
            .emit(HirIdWithBlock::new(location, self.block), message);
    }
}

pub trait FromHirExpr: Sized {
    fn from_hir_expr(
        diagnostics: &mut HirDiagnosticCollectorWithBlock<'_>,
        resolve_ctx: &resolve::ResolveContext,
        block: &HirBlockBody,
        expr: hir::ExprId,
    ) -> Option<Self>;
}
