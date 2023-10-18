use crate::compile::{
    hir, hir::ExprId, FromHirExpr, HirBlockBody, HirDiagnosticCollector, ResolveContext,
};
use crate::syntax::ast;
use shin_core::format::scenario::instruction_elements::Register;

impl FromHirExpr for Register {
    fn from_hir_expr(
        diagnostics: &mut HirDiagnosticCollector,
        resolve_ctx: &ResolveContext,
        block: &HirBlockBody,
        expr: ExprId,
    ) -> Option<Self> {
        let hir::Expr::RegisterRef(register) = &block.exprs[expr] else {
            diagnostics.emit(expr.into(), "Expected a register reference".into());
            return None;
        };

        let register = register.as_ref()?;
        match resolve_ctx.resolve_register(register) {
            Some(register) => Some(register),
            None => {
                let ast::RegisterIdentKind::Alias(alias) = register else {
                    unreachable!("BUG: a regular register should always resolve");
                };
                diagnostics.emit(expr.into(), format!("Unknown register alias: `${}`", alias));
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Register;
    use crate::compile::db::Database;
    use crate::compile::diagnostics::{HirDiagnosticAccumulator, SourceDiagnosticAccumulator};
    use crate::compile::{hir, File, FromHirExpr, HirDiagnosticCollector, ResolveContext};

    #[test]
    fn from_hir() {
        let db = Database::default();
        let db = &db;
        let file = File::new(
            db,
            "test.sal".to_string(),
            r#"
            HELLO $v0, $v1, $a0
        "#
            .to_string(),
        );

        // TODO: test register aliases

        let registers = ["$v0", "$v1", "$a0"]
            .map(|s| s.parse::<Register>().unwrap())
            .into_iter();

        let bodies = hir::collect_file_bodies(db, file);

        let hir_errors =
            hir::collect_file_bodies::accumulated::<HirDiagnosticAccumulator>(db, file);
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

        assert_eq!(block.exprs.len(), registers.len());

        for ((expr_id, _), expected) in block.exprs.iter().zip(registers) {
            let register = Register::from_hir_expr(&mut diagnostics, &resolve_ctx, &block, expr_id);
            assert!(diagnostics.is_empty());

            assert_eq!(register, Some(expected));
        }
    }
}
