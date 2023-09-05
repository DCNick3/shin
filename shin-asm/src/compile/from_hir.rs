use crate::compile::{
    hir::{self, HirBlockBody},
    resolve, File, InFile, MakeInFile,
};

#[derive(Debug)]
pub enum HirId {
    Expr(hir::ExprId),
    Instruction(hir::InstructionId),
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

impl MakeInFile for HirId {}

pub type HirIdInFile = InFile<HirId>;

#[derive(Debug)]
#[allow(unused)] // will be used when hir diagnostic adapters will be implemented
pub struct HirDiagnostic {
    location: HirIdInFile,
    message: String,
}

#[derive(Default, Debug)]
pub struct HirDiagnostics {
    diagnostics: Vec<HirDiagnostic>,
}

impl HirDiagnostics {
    pub fn new() -> Self {
        Self {
            diagnostics: Vec::new(),
        }
    }

    pub fn emit(&mut self, file: File, location: HirId, message: String) {
        self.diagnostics.push(HirDiagnostic {
            location: location.in_file(file),
            message,
        });
    }

    pub fn is_empty(&self) -> bool {
        self.diagnostics.is_empty()
    }
}

pub trait FromHirExpr {
    fn from_hir_expr(
        diagnostics: &mut HirDiagnostics,
        resolve_ctx: &resolve::ResolveContext,
        file: File,
        block: &HirBlockBody,
        expr: hir::ExprId,
    ) -> Self;
}

pub trait FromHirInstruction {
    fn from_hir_instruction(
        diagnostics: &mut HirDiagnostics,
        resolve_ctx: &resolve::ResolveContext,
        file: File,
        block: &HirBlockBody,
        instr: hir::InstructionId,
    ) -> Self;
}
