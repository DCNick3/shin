use shin_core::format::scenario::instructions::Instruction;

use super::from_instr_args::FromInstrArgs;
use crate::compile::hir::lower::from_hir::{FromHirBlockCtx, FromHirCollectors};

pub trait InstrLowerFn<Dummy, Args: FromInstrArgs> {
    fn lower_instr(
        &self,
        collectors: &mut FromHirCollectors,
        ctx: &FromHirBlockCtx,
        instr_name: &str,
        args: Args,
    ) -> Option<Instruction>;
}

// TODO: refine the selection of the InstrLowerFn shapes

impl<
        Args: FromInstrArgs,
        F: Fn(&mut FromHirCollectors, &FromHirBlockCtx, &str, Args) -> Option<Instruction>,
    > InstrLowerFn<((), (), (), ()), Args> for F
{
    fn lower_instr(
        &self,
        collectors: &mut FromHirCollectors,
        ctx: &FromHirBlockCtx,
        instr_name: &str,
        args: Args,
    ) -> Option<Instruction> {
        self(collectors, ctx, instr_name, args)
    }
}

impl<
        Args: FromInstrArgs,
        F: Fn(&mut FromHirCollectors, &FromHirBlockCtx, Args) -> Option<Instruction>,
    > InstrLowerFn<((), (), ()), Args> for F
{
    fn lower_instr(
        &self,
        collectors: &mut FromHirCollectors,
        ctx: &FromHirBlockCtx,
        _instr_name: &str,
        args: Args,
    ) -> Option<Instruction> {
        self(collectors, ctx, args)
    }
}

impl<Args: FromInstrArgs, F: Fn(&str, Args) -> Option<Instruction>> InstrLowerFn<((), ()), Args>
    for F
{
    fn lower_instr(
        &self,
        _collectors: &mut FromHirCollectors,
        _ctx: &FromHirBlockCtx,
        instr_name: &str,
        args: Args,
    ) -> Option<Instruction> {
        self(instr_name, args)
    }
}

impl<Args: FromInstrArgs, F: Fn(Args) -> Option<Instruction>> InstrLowerFn<((),), Args> for F {
    fn lower_instr(
        &self,
        _collectors: &mut FromHirCollectors,
        _ctx: &FromHirBlockCtx,
        _instr_name: &str,
        args: Args,
    ) -> Option<Instruction> {
        self(args)
    }
}

impl<F: Fn() -> Option<Instruction>> InstrLowerFn<(), ()> for F {
    fn lower_instr(
        &self,
        _collectors: &mut FromHirCollectors,
        _ctx: &FromHirBlockCtx,
        _instr_name: &str,
        _args: (),
    ) -> Option<Instruction> {
        self()
    }
}
