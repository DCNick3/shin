use shin_core::format::scenario::instructions::Instruction;
use typenum::{B0, B1};

use super::from_instr_args::FromInstrArgs;
use crate::compile::hir::lower::{
    from_hir::{FromHirBlockCtx, FromHirCollectors},
    instruction::into_instruction::IntoInstructionResult,
    LowerResult,
};

// Dummy type parameter is a hacky way to work around current rustc limitations
// rustc can't guarantee that all the different `T: Fn(...)` variants we use are disjoint types,
// so we have to implement technically different traits for each of them (they differ by the Dummy type parameter).
// Currently, we use tuples of the following shape for this:
// (ReturnType, HasCollectors, HasCtx, HasInstrName, HasArgs) (where the Has* types are typenum::B0 or typenum::B1)
pub trait InstrLowerFn<Dummy, Result, Args: FromInstrArgs> {
    fn lower_instr(
        &self,
        collectors: &mut FromHirCollectors,
        ctx: &FromHirBlockCtx,
        instr_name: &str,
        args: Args,
    ) -> LowerResult<Instruction>;
}

// TODO: refine the selection of the InstrLowerFn shapes

impl<
        Args: FromInstrArgs,
        Result: IntoInstructionResult,
        F: Fn(&mut FromHirCollectors, &FromHirBlockCtx, &str, Args) -> Result,
    > InstrLowerFn<(B1, B1, B1, B1), Result, Args> for F
{
    fn lower_instr(
        &self,
        collectors: &mut FromHirCollectors,
        ctx: &FromHirBlockCtx,
        instr_name: &str,
        args: Args,
    ) -> LowerResult<Instruction> {
        self(collectors, ctx, instr_name, args).into_instruction_result()
    }
}

impl<
        Args: FromInstrArgs,
        Result: IntoInstructionResult,
        F: Fn(&mut FromHirCollectors, &FromHirBlockCtx, Args) -> Result,
    > InstrLowerFn<(B1, B1, B0, B1), Result, Args> for F
{
    fn lower_instr(
        &self,
        collectors: &mut FromHirCollectors,
        ctx: &FromHirBlockCtx,
        _instr_name: &str,
        args: Args,
    ) -> LowerResult<Instruction> {
        self(collectors, ctx, args).into_instruction_result()
    }
}

impl<Args: FromInstrArgs, Result: IntoInstructionResult, F: Fn(&str, Args) -> Result>
    InstrLowerFn<(B0, B0, B1, B1), Result, Args> for F
{
    fn lower_instr(
        &self,
        _collectors: &mut FromHirCollectors,
        _ctx: &FromHirBlockCtx,
        instr_name: &str,
        args: Args,
    ) -> LowerResult<Instruction> {
        self(instr_name, args).into_instruction_result()
    }
}

impl<Args: FromInstrArgs, Result: IntoInstructionResult, F: Fn(Args) -> Result>
    InstrLowerFn<(B0, B0, B0, B1), Result, Args> for F
{
    fn lower_instr(
        &self,
        _collectors: &mut FromHirCollectors,
        _ctx: &FromHirBlockCtx,
        _instr_name: &str,
        args: Args,
    ) -> LowerResult<Instruction> {
        self(args).into_instruction_result()
    }
}

impl<Result: IntoInstructionResult, F: Fn() -> Result> InstrLowerFn<(B0, B0, B0, B0), Result, ()>
    for F
{
    fn lower_instr(
        &self,
        _collectors: &mut FromHirCollectors,
        _ctx: &FromHirBlockCtx,
        _instr_name: &str,
        _args: (),
    ) -> LowerResult<Instruction> {
        self().into_instruction_result()
    }
}
