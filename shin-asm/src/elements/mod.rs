use binrw::{BinRead, BinWrite};

mod prelude {
    pub use crate::compile::{
        hir, hir::ExprId, FromHirExpr, HirBlockBody, HirDiagnosticCollector, ResolveContext,
    };
    pub use crate::syntax::ast;
}

mod numbers;
mod register;

pub use shin_core::format::scenario::instruction_elements::{Register, RegisterRepr};

pub trait InstructionElement: BinRead + BinWrite {}

#[cfg(test)]
fn check_from_hir_ok<T: crate::compile::FromHirExpr + Eq + std::fmt::Debug>(
    source: &str,
    expected: &[T],
) {
    use crate::compile::{
        db::Database,
        diagnostics::{HirDiagnosticAccumulator, SourceDiagnosticAccumulator},
        file::File,
        from_hir::HirDiagnosticCollector,
        hir,
        resolve::ResolveContext,
    };

    let db = Database::default();
    let db = &db;
    let file = File::new(db, "test.sal".to_string(), source.to_string());

    let bodies = hir::collect_file_bodies(db, file);

    let hir_errors = hir::collect_file_bodies::accumulated::<HirDiagnosticAccumulator>(db, file);
    let source_errors =
        hir::collect_file_bodies::accumulated::<SourceDiagnosticAccumulator>(db, file);
    if !source_errors.is_empty() || !hir_errors.is_empty() {
        panic!(
            "lowering produced errors:\n\
                source-level: {source_errors:?}\n\
                hir-level: {hir_errors:?}"
        );
    }

    let block = bodies.get_block(db, bodies.get_block_ids(db)[0]).unwrap();

    let mut diagnostics = HirDiagnosticCollector::new();
    let resolve_ctx = ResolveContext::new(db);

    let (_, instr) = block.instructions.iter().next().unwrap();
    let args = instr.args.as_ref();

    assert_eq!(args.len(), expected.len());

    for (&expr_id, expected) in args.iter().zip(expected) {
        let register = T::from_hir_expr(&mut diagnostics, &resolve_ctx, &block, expr_id);
        assert!(diagnostics.is_empty());

        assert_eq!(register.as_ref(), Some(expected));
    }
}
