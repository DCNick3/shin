use shin_core::format::scenario::instruction_elements::CodeAddress;

use super::prelude::*;
use crate::compile::{
    def_map::DefValue,
    hir::lower::{
        from_hir::{FromHirBlockCtx, FromHirCollectors},
        LowerResult,
    },
};

impl FromHirExpr for CodeAddress {
    fn from_hir_expr(
        collectors: &mut FromHirCollectors,
        ctx: &FromHirBlockCtx,
        expr: ExprId,
    ) -> LowerResult<Self> {
        let hir::Expr::NameRef(ref name) = ctx.expr(expr) else {
            return collectors.emit_diagnostic(
                expr.into(),
                format!("Expected a label, got {}", ctx.expr(expr).describe_ty()),
            );
        };

        match ctx.resolve_item(name) {
            None => collectors.emit_diagnostic(
                expr.into(),
                format!("Could not find the definition of `{}`", name),
            ),
            Some(DefValue::Value(_)) => {
                collectors.emit_diagnostic(expr.into(), format!("Expected a label, found an alias"))
            }
            Some(DefValue::Block(block)) => Ok(collectors.allocate_code_address(block)),
        }
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::CodeAddress;
    use crate::compile::{
        db::Database,
        def_map::{build_def_map, ResolveKind},
        hir::lower::{
            from_hir::{FromHirBlockCtx, FromHirCollectors},
            test_utils,
            test_utils::lower_hir_ok,
            CodeAddressCollector, FromHirExpr, HirDiagnosticCollector,
        },
        resolve::ResolveContext,
        MakeWithFile, Program,
    };

    #[test]
    fn from_hir() {
        let source = indoc! {r"
                HELLO BOBA, BABA, GEGE
                
                BOBA:
                BABA:
                GEGE:
            "};
        let expected = &[CodeAddress(0), CodeAddress(1), CodeAddress(2)];

        let db = Database::default();
        let db = &db;
        let (file, bodies) = lower_hir_ok(db, source);
        let block_ids = bodies.get_block_ids(db);

        let block_id = block_ids[0];
        let block_id_2 = block_ids[1];
        let block_id_2_with_file = block_id_2.in_file(file);
        let block = bodies.get_block(db, block_id).unwrap().clone();

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
            .map(|&expr_id| CodeAddress::from_hir_expr(&mut collectors, &ctx, expr_id))
            .collect::<Vec<_>>();

        if !diagnostics.is_empty() {
            panic!(
                "errors while lowering hir elements:\n{}",
                test_utils::diagnostic_collector_to_str(db, diagnostics)
            );
        }

        for (lowered, expected) in lowered_elements.iter().zip(expected) {
            assert_eq!(lowered.as_ref(), Ok(expected));
        }

        for block_id in code_address_collector.into_block_ids() {
            assert_eq!(block_id, block_id_2_with_file);
        }
    }
}
