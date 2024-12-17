use shin_core::format::text::{
    string::{StringFixup, StringLengthDesc},
    SJisString,
};

use super::prelude::*;
use crate::compile::hir::lower::LowerResult;

impl<L: StringLengthDesc, F: StringFixup + 'static> FromHirExpr for SJisString<L, F> {
    fn from_hir_expr(
        collectors: &mut FromHirCollectors,
        ctx: &FromHirBlockCtx,
        expr: ExprId,
    ) -> LowerResult<Self> {
        let hir::Expr::Literal(hir::Literal::String(s)) = ctx.expr(expr) else {
            return collectors.emit_unexpected_type(ctx, "a string", expr);
        };

        Ok(SJisString::new(s.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use shin_core::format::text::U16FixupString;

    use super::super::check_from_hir_ok;

    #[test]
    fn from_hir() {
        check_from_hir_ok(
            // TODO: support & test string escapes
            r#"HELLO "biba", "BoBa", """#,
            &["biba", "BoBa", ""].map(U16FixupString::new),
        );
    }
}
