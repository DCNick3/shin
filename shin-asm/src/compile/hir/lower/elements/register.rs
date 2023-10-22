use super::prelude::*;
use shin_core::format::scenario::instruction_elements::Register;

impl FromHirExpr for Register {
    fn from_hir_expr(
        diagnostics: &mut HirDiagnosticCollectorWithBlock,
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
    use super::super::check_from_hir_ok;
    use super::Register;

    #[test]
    fn from_hir() {
        // TODO: test register aliases

        check_from_hir_ok(
            "HELLO $v0, $v1, $a0",
            &["$v0", "$v1", "$a0"].map(|s| s.parse::<Register>().unwrap()),
        );
    }
}
