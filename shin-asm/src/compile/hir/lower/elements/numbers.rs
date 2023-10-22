use super::prelude::*;
use shin_core::format::scenario::instruction_elements::{NumberSpec, UntypedNumberSpec};

fn try_lit_i32(block: &HirBlockBody, expr: ExprId) -> Option<i32> {
    match block.exprs[expr] {
        hir::Expr::Literal(hir::Literal::IntNumber(lit)) => Some(lit),
        hir::Expr::Literal(hir::Literal::RationalNumber(lit)) => Some(lit.into_raw()),
        hir::Expr::UnaryOp {
            op: ast::UnaryOp::Negate,
            expr,
        } => match block.exprs[expr] {
            hir::Expr::Literal(hir::Literal::IntNumber(lit)) => Some(-lit),
            hir::Expr::Literal(hir::Literal::RationalNumber(lit)) => Some(-lit.into_raw()),
            _ => None,
        },
        _ => None,
    }
}

impl FromHirExpr for i32 {
    fn from_hir_expr(
        diagnostics: &mut HirDiagnosticCollectorWithBlock,
        _resolve_ctx: &ResolveContext,
        block: &HirBlockBody,
        expr: ExprId,
    ) -> Option<Self> {
        let lit = try_lit_i32(block, expr);

        let Some(lit) = lit else {
            diagnostics.emit(expr.into(), "Expected a number literal".into());
            return None;
        };

        Some(lit)
    }
}

// TODO: typed numbers
// we probably want to allow symbolic names for constants if the type is an enum
impl FromHirExpr for NumberSpec {
    fn from_hir_expr(
        diagnostics: &mut HirDiagnosticCollectorWithBlock,
        resolve_ctx: &ResolveContext,
        block: &HirBlockBody,
        expr: ExprId,
    ) -> Option<Self> {
        let untyped = (|| {
            if let Some(lit) = try_lit_i32(block, expr) {
                Some(UntypedNumberSpec::Constant(lit))
            } else if let hir::Expr::RegisterRef(register) = &block.exprs[expr] {
                let register = resolve_ctx.resolve_register(register.as_ref()?)?;

                Some(UntypedNumberSpec::Register(register))
            } else {
                diagnostics.emit(
                    expr.into(),
                    "Expected either a number literal or a register reference".into(),
                );
                None
            }
        })();

        untyped.map(NumberSpec::new)
    }
}

#[cfg(test)]
mod tests {
    use super::super::check_from_hir_ok;
    use shin_core::format::scenario::instruction_elements::{NumberSpec, UntypedNumberSpec};

    // TODO: test diagnostics

    #[test]
    fn i32_from_hir() {
        check_from_hir_ok::<i32>("HELLO 1, -2, 10.0", &[1, -2, 10_000]);
    }

    #[test]
    fn number_spec_from_hir() {
        check_from_hir_ok::<NumberSpec>(
            "HELLO 1, -2, 10.0, $a1, $v0",
            &[
                NumberSpec::new(UntypedNumberSpec::Constant(1)),
                NumberSpec::new(UntypedNumberSpec::Constant(-2)),
                NumberSpec::new(UntypedNumberSpec::Constant(10_000)),
                NumberSpec::new(UntypedNumberSpec::Register("$a1".parse().unwrap())),
                NumberSpec::new(UntypedNumberSpec::Register("$v0".parse().unwrap())),
            ],
        );
    }
}
