use super::prelude::*;
use crate::compile::from_hir::CodeAddressCollector;
use shin_core::format::scenario::instruction_elements::Register;

impl FromHirExpr for Register {
    fn from_hir_expr(
        diagnostics: &mut HirDiagnosticCollectorWithBlock,
        _code_address_collector: &mut CodeAddressCollector,
        resolve_ctx: &ResolveContext,
        block: &HirBlockBody,
        expr: ExprId,
    ) -> Option<Self> {
        let hir::Expr::RegisterRef(register) = &block.exprs[expr] else {
            diagnostics.emit(
                expr.into(),
                format!(
                    "Expected a register, but got {}",
                    block.exprs[expr].describe_ty()
                ),
            );
            return None;
        };

        let register = register.as_ref()?;
        match resolve_ctx.resolve_register(register) {
            Some(register) => Some(register),
            None => {
                let ast::RegisterIdentKind::Alias(alias) = register else {
                    unreachable!("BUG: a regular register should always resolve");
                };
                diagnostics.emit(
                    expr.into(),
                    format!("Unresolved register alias: `${}`", alias),
                );
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::check_from_hir_ok;
    use super::Register;
    use indoc::indoc;

    #[test]
    fn from_hir() {
        check_from_hir_ok(
            "HELLO $v0, $v1, $a0",
            &["$v0", "$v1", "$a0"].map(|s| s.parse::<Register>().unwrap()),
        );
        check_from_hir_ok(
            indoc! {r"
                def $BIBA = $v0
                
                HELLO $v0, $BIBA, $a0
            "},
            &["$v0", "$v0", "$a0"].map(|s| s.parse::<Register>().unwrap()),
        );
    }
}
