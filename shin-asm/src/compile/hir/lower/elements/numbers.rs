use shin_core::format::scenario::instruction_elements::{NumberSpec, UntypedNumberSpec};

use super::prelude::*;
use crate::compile::{
    constexpr::ConstexprValue,
    def_map::DefValue,
    hir::lower::from_hir::{FromHirBlockCtx, FromHirCollectors},
};

fn try_lit_i32(
    collectors: &mut FromHirCollectors,
    ctx: &FromHirBlockCtx,
    expr: ExprId,
) -> Option<ConstexprValue> {
    match *ctx.expr(expr) {
        hir::Expr::Literal(hir::Literal::IntNumber(lit)) => Some(ConstexprValue::constant(lit)),
        hir::Expr::Literal(hir::Literal::RationalNumber(lit)) => {
            Some(ConstexprValue::constant(lit.into_raw()))
        }
        hir::Expr::UnaryOp {
            op: ast::UnaryOp::Negate,
            expr,
        } => match *ctx.expr(expr) {
            hir::Expr::Literal(hir::Literal::IntNumber(lit)) => {
                Some(ConstexprValue::constant(-lit))
            }
            hir::Expr::Literal(hir::Literal::RationalNumber(lit)) => {
                Some(ConstexprValue::constant(-lit.into_raw()))
            }
            _ => None,
        },
        hir::Expr::NameRef(ref name) => match ctx.resolve_item(name) {
            None => {
                collectors.emit_diagnostic(
                    expr.into(),
                    format!("Could not find the definition of `{}`", name),
                );
                Some(ConstexprValue::dummy())
            }
            Some(DefValue::Block(_)) => {
                collectors.emit_diagnostic(
                    expr.into(),
                    format!("Expected a number, found a code reference"),
                );
                Some(ConstexprValue::dummy())
            }
            Some(DefValue::Value(value)) => Some(value),
        },
        _ => None,
    }
}

impl FromHirExpr for i32 {
    fn from_hir_expr(
        collectors: &mut FromHirCollectors,
        ctx: &FromHirBlockCtx,
        expr: ExprId,
    ) -> Option<Self> {
        let lit = try_lit_i32(collectors, ctx, expr);

        let Some(lit) = lit else {
            collectors.emit_diagnostic(expr.into(), "Expected a number".into());
            return None;
        };

        lit.unwrap()
    }
}

// TODO: typed numbers
// we probably want to allow symbolic names for constants if the type is an enum
impl FromHirExpr for NumberSpec {
    fn from_hir_expr(
        collectors: &mut FromHirCollectors,
        ctx: &FromHirBlockCtx,
        expr: ExprId,
    ) -> Option<Self> {
        let untyped = (|| {
            if let Some(lit) = try_lit_i32(collectors, ctx, expr) {
                Some(UntypedNumberSpec::Constant(lit.unwrap()?))
            } else if let hir::Expr::RegisterRef(register) = &ctx.expr(expr) {
                let register = ctx.resolve_register(register.as_ref()?)?;

                Some(UntypedNumberSpec::Register(register))
            } else {
                collectors.emit_diagnostic(
                    expr.into(),
                    format!(
                        "Expected either a number or a register, found {}",
                        ctx.expr(expr).describe_ty()
                    ),
                );
                None
            }
        })();

        untyped.map(NumberSpec::new)
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
