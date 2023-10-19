use super::prelude::*;

impl FromHirExpr for i32 {
    fn from_hir_expr(
        diagnostics: &mut HirDiagnosticCollector,
        _resolve_ctx: &ResolveContext,
        block: &HirBlockBody,
        expr: ExprId,
    ) -> Option<Self> {
        let lit = match block.exprs[expr] {
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
        };

        let Some(lit) = lit else {
            diagnostics.emit(expr.into(), "Expected a number literal".into());
            return None;
        };

        Some(lit)
    }
}

#[cfg(test)]
mod tests {
    use super::super::check_from_hir_ok;

    #[test]
    fn from_hir() {
        check_from_hir_ok::<i32>("HELLO 1, -2, 10.0", &[1, -2, 10_000]);
    }
}
