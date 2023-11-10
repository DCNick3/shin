mod messagebox_style;

use shin_core::format::scenario::instruction_elements::{NumberSpec, UntypedNumberSpec};

use super::prelude::*;
use crate::compile::{
    constexpr::ConstexprValue,
    hir::lower::{LowerError, LowerResult},
};

fn try_lit_i32(
    collectors: &mut FromHirCollectors,
    ctx: &FromHirBlockCtx,
    expr: ExprId,
) -> Option<LowerResult<ConstexprValue>> {
    match *ctx.expr(expr) {
        hir::Expr::Literal(hir::Literal::IntNumber(lit)) => Some(Ok(ConstexprValue::constant(lit))),
        hir::Expr::Literal(hir::Literal::RationalNumber(lit)) => {
            Some(Ok(ConstexprValue::constant(lit.into_raw())))
        }
        hir::Expr::UnaryOp {
            op: ast::UnaryOp::Negate,
            expr,
        } => match *ctx.expr(expr) {
            hir::Expr::Literal(hir::Literal::IntNumber(lit)) => {
                Some(Ok(ConstexprValue::constant(-lit)))
            }
            hir::Expr::Literal(hir::Literal::RationalNumber(lit)) => {
                Some(Ok(ConstexprValue::constant(-lit.into_raw())))
            }
            _ => None,
        },
        hir::Expr::NameRef(ref name) => match ctx.resolve_item(name) {
            None => Some(collectors.emit_diagnostic(
                expr.into(),
                format!("Could not find the definition of `{}`", name),
            )),
            Some(DefValue::Block(_)) => Some(collectors.emit_diagnostic(
                expr.into(),
                format!("Expected a number, found a code reference"),
            )),
            Some(DefValue::Value(value)) => Some(value),
        },
        _ => None,
    }
}

fn try_number_spec<T>(
    collectors: &mut FromHirCollectors,
    ctx: &FromHirBlockCtx,
    expr: ExprId,
) -> LowerResult<Option<NumberSpec<T>>> {
    if let Some(lit) = try_lit_i32(collectors, ctx, expr) {
        let Ok(lit) = lit else {
            return Err(LowerError);
        };

        Ok(Some(NumberSpec::new(UntypedNumberSpec::Constant(
            lit.value(),
        ))))
    } else if let hir::Expr::RegisterRef(register) = &ctx.expr(expr) {
        let Ok(register) = register else {
            return Err(LowerError);
        };

        let register = ctx.resolve_register(register).ok_or(()).or_else(|()| {
            let ast::RegisterIdentKind::Alias(register) = register else {
                unreachable!()
            };

            collectors.emit_diagnostic(
                expr.into(),
                format!("Could not find the definition of `${}`", register),
            )
        })?;

        Ok(Some(NumberSpec::new(UntypedNumberSpec::Register(register))))
    } else {
        Ok(None)
    }
}

impl FromHirExpr for i32 {
    fn from_hir_expr(
        collectors: &mut FromHirCollectors,
        ctx: &FromHirBlockCtx,
        expr: ExprId,
    ) -> LowerResult<Self> {
        let lit = try_lit_i32(collectors, ctx, expr);

        let Some(lit) = lit else {
            return collectors.emit_diagnostic(expr.into(), "Expected a number".into());
        };

        lit.map(ConstexprValue::value)
    }
}

// TODO: typed numbers
// we probably want to allow symbolic names for constants if the type is an enum
impl FromHirExpr for NumberSpec {
    fn from_hir_expr(
        collectors: &mut FromHirCollectors,
        ctx: &FromHirBlockCtx,
        expr: ExprId,
    ) -> LowerResult<Self> {
        try_number_spec(collectors, ctx, expr)?
            .ok_or(())
            .or_else(|()| {
                collectors.emit_diagnostic(
                    expr.into(),
                    format!(
                        "Expected either a number or a register, found {}",
                        ctx.expr(expr).describe_ty()
                    ),
                )
            })
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;
    use shin_core::format::scenario::instruction_elements::NumberSpec;

    use super::super::check_from_hir_ok;

    // TODO: test diagnostics

    #[test]
    fn i32_from_hir() {
        check_from_hir_ok::<i32>("HELLO 1, -2, 10.0", &[1, -2, 10_000]);
        check_from_hir_ok::<i32>(
            indoc! {r"
            def ALIAS = -2
            def ALIAS_RATIONAL = 10.0

            HELLO 1, ALIAS, ALIAS_RATIONAL
        "},
            &[1, -2, 10_000],
        );
    }

    #[test]
    fn number_spec_from_hir() {
        check_from_hir_ok::<NumberSpec>(
            "HELLO 1, -2, 10.0, $a1, $v0",
            &[
                NumberSpec::constant(1),
                NumberSpec::constant(-2),
                NumberSpec::constant(10_000),
                NumberSpec::register("$a1".parse().unwrap()),
                NumberSpec::register("$v0".parse().unwrap()),
            ],
        );
    }
}
