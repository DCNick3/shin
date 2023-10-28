use super::prelude::*;
use crate::compile::def_map::DefValue;
use crate::compile::hir::lower::CodeAddressCollector;
use shin_core::format::scenario::instruction_elements::CodeAddress;

impl FromHirExpr for CodeAddress {
    fn from_hir_expr(
        diagnostics: &mut HirDiagnosticCollectorWithBlock,
        code_address_collector: &mut CodeAddressCollector,
        resolve_ctx: &ResolveContext,
        block: &HirBlockBody,
        expr: ExprId,
    ) -> Option<Self> {
        let hir::Expr::NameRef(ref name) = &block.exprs[expr] else {
            diagnostics.emit(
                expr.into(),
                format!("Expected a label, got {}", block.exprs[expr].describe_ty()),
            );
            return None;
        };

        match resolve_ctx.resolve_item(name) {
            None => {
                diagnostics.emit(
                    expr.into(),
                    format!("Could not find the definition of `{}`", name),
                );
                None
            }
            Some(DefValue::Value(_)) => {
                diagnostics.emit(expr.into(), format!("Expected a label, found an alias"));
                None
            }
            Some(DefValue::Block(block)) => Some(code_address_collector.allocate(block)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::CodeAddress;
    use crate::compile::hir::lower::{CodeAddressCollector, FromHirExpr, HirDiagnosticCollector};
    use crate::compile::{
        db::Database, def_map::build_def_map, def_map::ResolveKind, hir::lower::test_utils,
        hir::lower::test_utils::lower_hir_ok, resolve::ResolveContext, MakeWithFile, Program,
    };
    use indoc::indoc;

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

        let lowered_elements = args
            .iter()
            .map(|&expr_id| {
                CodeAddress::from_hir_expr(
                    &mut diagnostics.with_file(file).with_block(block_id.into()),
                    &mut code_address_collector,
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

        for block_id in code_address_collector.into_block_ids() {
            assert_eq!(block_id, block_id_2_with_file);
        }
    }
}
