use crate::compile::diagnostics::{Diagnostic, HirLocation};
use crate::compile::{
    hir::{self, HirBlockBody},
    resolve, BlockId, MakeWithFile, WithFile,
};

#[derive(Debug, Copy, Clone)]
pub enum HirId {
    Expr(hir::ExprId),
    Instruction(hir::InstructionId),
}

#[derive(Debug, Copy, Clone)]
pub struct HirIdWithBlock {
    // TODO: naming is unclear...
    // InBlock -> identifies inside a block or HirId wrapped with a block id (like InFile)
    // Probably should rename the InFile to WithFile
    id: HirId,
    block_id: BlockId,
}

impl HirIdWithBlock {
    pub fn new(id: impl Into<HirId>, block_id: BlockId) -> Self {
        Self {
            block_id,
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

    pub fn emit(&mut self, _location: HirId, _message: String) {
        todo!()
    }

    pub fn is_empty(&self) -> bool {
        self.diagnostics.is_empty()
    }
}

pub trait FromHirExpr {
    fn from_hir_expr(
        diagnostics: &mut HirDiagnosticCollector,
        resolve_ctx: &resolve::ResolveContext,
        block: &HirBlockBody,
        expr: hir::ExprId,
    ) -> Self;
}

pub trait FromHirInstruction {
    fn from_hir_instruction(
        diagnostics: &mut HirDiagnosticCollector,
        resolve_ctx: &resolve::ResolveContext,
        block: &HirBlockBody,
        instr: hir::InstructionId,
    ) -> Self;
}
