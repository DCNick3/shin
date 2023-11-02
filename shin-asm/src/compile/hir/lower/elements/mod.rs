mod prelude {
    pub use crate::{
        compile::{
            hir,
            hir::{
                lower::{FromHirExpr, HirDiagnosticCollectorWithBlock},
                ExprId,
            },
            HirBlockBody, ResolveContext,
        },
        syntax::ast,
    };
}

mod code_address;
mod numbers;
mod register;

#[cfg(test)]
fn check_from_hir_ok<T: crate::compile::hir::lower::FromHirExpr + Eq + std::fmt::Debug>(
    source: &str,
    expected: &[T],
) {
    use crate::compile::{
        db::Database,
        def_map::{build_def_map, ResolveKind},
        hir::lower::{
            from_hir::{FromHirBlockCtx, FromHirCollectors},
            test_utils, CodeAddressCollector, HirDiagnosticCollector,
        },
        resolve::ResolveContext,
        MakeWithFile, Program,
    };

    let db = Database::default();
    let db = &db;
    let (file, block_id, block) = test_utils::lower_hir_block_ok(db, source);
    let program = Program::new(db, vec![file]);
    let def_map = build_def_map(db, program);

    let mut diagnostics = HirDiagnosticCollector::new();
    let resolve_ctx = ResolveContext::new(
        db,
        def_map,
        ResolveKind::LocalAndGlobal(block_id.in_file(file)),
    );

    let (_, instr) = block.instructions.iter().next().unwrap();
    let args = instr.args.as_ref();

    assert_eq!(args.len(), expected.len());

    let mut code_address_collector = CodeAddressCollector::new();

    let mut file_diagnostics = diagnostics.with_file(file);
    let mut block_diagnostics = file_diagnostics.with_block(block_id.into());
    let mut collectors = FromHirCollectors {
        diagnostics: &mut block_diagnostics,
        code_address_collector: &mut code_address_collector,
    };
    let ctx = FromHirBlockCtx {
        resolve_ctx: &resolve_ctx,
        block: &block,
    };

    let lowered_elements = args
        .iter()
        .map(|&expr_id| T::from_hir_expr(&mut collectors, &ctx, expr_id))
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
