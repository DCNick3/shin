use shin_core::format::scenario::instruction_elements::Register;

use super::prelude::*;
use crate::compile::hir::lower::{
    from_hir::{FromHirBlockCtx, FromHirCollectors},
    LowerResult,
};

impl FromHirExpr for Register {
    fn from_hir_expr(
        collectors: &mut FromHirCollectors,
        ctx: &FromHirBlockCtx,
        expr: ExprId,
    ) -> LowerResult<Self> {
        let hir::Expr::RegisterRef(register) = ctx.expr(expr) else {
            return collectors.emit_diagnostic(
                expr.into(),
                format!(
                    "Expected a register, but got {}",
                    ctx.expr(expr).describe_ty()
                ),
            );
        };

        let register = match *register {
            Ok(ref register) => register,
            Err(e) => return Err(e),
        };
        match ctx.resolve_register(register) {
            Some(register) => Ok(register),
            None => {
                let ast::RegisterIdentKind::Alias(alias) = register else {
                    unreachable!("BUG: a regular register should always resolve");
                };
                collectors.emit_diagnostic(
                    expr.into(),
                    format!("Unresolved register alias: `${}`", alias),
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::{super::check_from_hir_ok, Register};

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
