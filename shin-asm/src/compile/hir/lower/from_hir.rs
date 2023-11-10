use shin_core::format::scenario::instruction_elements::{CodeAddress, Register};

use crate::{
    compile::{
        def_map::{DefValue, Name},
        diagnostics::HirLocation,
        hir::{
            self,
            lower::{LowerError, LowerResult},
            HirBlockBody, HirBlockId, HirDiagnostic, HirId, HirIdWithBlock,
        },
        resolve, BlockIdWithFile, File, MakeWithFile,
    },
    syntax::ast,
};

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

pub struct CodeAddressCollector {
    block_ids: Vec<BlockIdWithFile>,
}

impl CodeAddressCollector {
    pub fn new() -> Self {
        Self {
            block_ids: Vec::new(),
        }
    }

    pub fn allocate(&mut self, block: BlockIdWithFile) -> CodeAddress {
        let index = self.block_ids.len() as u32;
        self.block_ids.push(block);
        CodeAddress(index)
    }

    pub fn into_block_ids(self) -> Vec<BlockIdWithFile> {
        self.block_ids
    }
}

/// Represents the mutable state used during hir lowering.
///
/// This is used to pass out-of-band data related to the results of lowering like emitted diagnostics and allocated code addresses.
pub struct FromHirCollectors<'d, 'di, 'cac> {
    pub diagnostics: &'d mut HirDiagnosticCollectorWithBlock<'di>,
    pub code_address_collector: &'cac mut CodeAddressCollector,
}

impl<'d, 'di, 'cac> FromHirCollectors<'d, 'di, 'cac> {
    #[inline]
    pub fn emit_diagnostic<T>(&mut self, location: HirId, message: String) -> LowerResult<T> {
        self.diagnostics.emit(location, message);
        Err(LowerError)
    }

    #[inline]
    pub fn emit_unexpected_type<T>(
        &mut self,
        ctx: &FromHirBlockCtx,
        expected: &str,
        unexpected: hir::ExprId,
    ) -> LowerResult<T> {
        self.emit_diagnostic(
            unexpected.into(),
            format!(
                "Expected {}, but got {}",
                expected,
                ctx.expr(unexpected).describe_ty()
            ),
        )
    }

    #[inline]
    pub fn allocate_code_address(&mut self, block: BlockIdWithFile) -> CodeAddress {
        self.code_address_collector.allocate(block)
    }
}

/// Represents the immutable state used during hir lowering.
///
/// This includes context information like the block being lowered and the resolve context.
pub struct FromHirBlockCtx<'r, 'db, 'b> {
    pub resolve_ctx: &'r resolve::ResolveContext<'db>,
    pub block: &'b HirBlockBody,
}

impl<'r, 'db, 'b> FromHirBlockCtx<'r, 'db, 'b> {
    #[inline]
    pub fn resolve_register(&self, register: &ast::RegisterIdentKind) -> Option<Register> {
        self.resolve_ctx.resolve_register(register)
    }

    #[inline]
    pub fn resolve_item(&self, name: &Name) -> Option<DefValue> {
        self.resolve_ctx.resolve_item(name)
    }

    #[inline]
    pub fn instr(&self, id: hir::InstructionId) -> &hir::Instruction {
        &self.block.instructions[id]
    }

    #[inline]
    pub fn expr(&self, id: hir::ExprId) -> &hir::Expr {
        &self.block.exprs[id]
    }
}

pub trait FromHirExpr: Sized {
    fn from_hir_expr(
        collectors: &mut FromHirCollectors,
        ctx: &FromHirBlockCtx,
        expr: hir::ExprId,
    ) -> LowerResult<Self>;
}
