mod prelude {
    pub use crate::compile::{
        hir, hir::ExprId, FromHirExpr, HirBlockBody, HirDiagnosticCollectorWithBlock,
        ResolveContext,
    };
    pub use crate::syntax::ast;
}

mod numbers;
mod register;

#[cfg(test)]
use crate::compile::hir::lower::test_utils;

#[cfg(test)]
fn check_from_hir_ok<T: crate::compile::FromHirExpr + Eq + std::fmt::Debug>(
    source: &str,
    expected: &[T],
) {
    use crate::compile::{db::Database, from_hir::HirDiagnosticCollector, resolve::ResolveContext};

    let db = Database::default();
    let db = &db;
    let (file, block_id, block) = test_utils::lower_hir_block_ok(db, source);

    let mut diagnostics = HirDiagnosticCollector::new();
    let resolve_ctx = ResolveContext::new(db);

    let (_, instr) = block.instructions.iter().next().unwrap();
    let args = instr.args.as_ref();

    assert_eq!(args.len(), expected.len());

    let lowered_elements = args
        .iter()
        .map(|&expr_id| {
            T::from_hir_expr(
                &mut diagnostics.with_file(file).with_block(block_id.into()),
                &resolve_ctx,
                &block,
                expr_id,
            )
        })
        .collect::<Vec<_>>();

    if !diagnostics.is_empty() {
        panic!(
            "errors while lowering hir elements:\n{}",
            test_utils::diagnostic_collector_to_str(db, diagnostics)
        );
    }

    for (lowered, expected) in lowered_elements.iter().zip(expected) {
        assert_eq!(lowered.as_ref(), Some(expected));
    }
}
