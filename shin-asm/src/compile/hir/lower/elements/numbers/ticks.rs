use shin_core::{format::scenario::instruction_elements::NumberSpec, time::Ticks};

use super::{super::prelude::*, try_number_spec};
use crate::compile::hir::lower::LowerResult;

impl FromHirExpr for NumberSpec<Ticks> {
    fn from_hir_expr(
        collectors: &mut FromHirCollectors,
        ctx: &FromHirBlockCtx,
        expr: ExprId,
    ) -> LowerResult<Self> {
        if let Some(number) = try_number_spec(collectors, ctx, expr)? {
            // TODO: warn if an integer literal is used?
            // it's kinda not nice to use a literal if a symbolic name is available
            Ok(number)
        } else {
            collectors.emit_unexpected_type(ctx, "an amount of ticks", expr)
        }
    }
}
