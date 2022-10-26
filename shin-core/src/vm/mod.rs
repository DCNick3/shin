pub mod command;

use crate::format::scenario::instructions::{
    BinaryOperation, BinaryOperationType, CodeAddress, Command, Expression, ExpressionTerm,
    Instruction, JumpCond, JumpCondType, MemoryAddress, NumberSpec, UnaryOperation,
    UnaryOperationType,
};
use crate::format::scenario::{InstructionReader, Scenario};
use crate::vm::command::{
    AdvCommand, AdvListener, CommandContext, CommandPoll, CommandState, ExitResult, VLayerId,
};
use anyhow::Result;
use smallvec::SmallVec;
use tracing::{instrument, trace, warn};

// TODO: add a listener trait that can be used to get notified of commands
pub struct AdvVm<L: AdvListener> {
    /// Memory (aka registers I guess)
    memory: [i32; 0x1000],
    /// Call stack
    /// Stores the return address for each call instruction
    /// Also push instruction pushes here for some reason
    call_stack: Vec<CodeAddress>,
    /// Data stack
    /// Stores the arguments for each call instruction
    /// Can be addresses via MemoryAddress with addresses > 0x1000
    /// Also called mem3 in ShinDataUtil
    data_stack: Vec<i32>,
    /// PRNG state, updated on each instruction executed
    prng_state: u32,
    command_context: Option<CommandContext<L>>,
    instruction_reader: InstructionReader,
}

impl<L: AdvListener> AdvVm<L> {
    pub fn new(scenario: &Scenario, init_val: i32, random_seed: u32) -> Self {
        let mut memory = [0; 0x1000];
        memory[0] = init_val;

        Self {
            memory,
            call_stack: Vec::new(),
            data_stack: vec![0; 0x16], // Umineko scenario writes out of bounds of the stack so we add some extra space
            instruction_reader: scenario.instruction_reader(scenario.entrypoint_address()),
            prng_state: random_seed,
            command_context: None,
        }
    }

    #[inline]
    fn get_memory(&self, addr: MemoryAddress) -> i32 {
        if let Some(offset) = addr.as_stack_offset() {
            self.data_stack[self.data_stack.len() - 1 - (offset) as usize]
        } else {
            self.memory[addr.0 as usize]
        }
    }

    #[inline]
    fn set_memory(&mut self, addr: MemoryAddress, val: i32) {
        if let Some(offset) = addr.as_stack_offset() {
            let len = self.data_stack.len();
            // the top of the data stack is always the frame size
            // so we need to subtract 1 to get the actual top of the stack
            self.data_stack[len - 1 - (offset) as usize] = val;
        } else {
            self.memory[addr.0 as usize] = val;
        }
    }

    #[inline]
    fn get_number(&self, number: NumberSpec) -> i32 {
        match number {
            NumberSpec::Constant(c) => c,
            NumberSpec::Memory(addr) => self.get_memory(addr),
        }
    }

    fn compute_jump_condition(&self, cond: JumpCond, left: i32, right: i32) -> bool {
        let result = match cond.condition {
            JumpCondType::Equal => left == right,
            JumpCondType::NotEqual => left != right,
            JumpCondType::GreaterOrEqual => left >= right,
            JumpCondType::Greater => left > right,
            JumpCondType::LessOrEqual => left <= right,
            JumpCondType::Less => left < right,
            JumpCondType::BitwiseAndNotZero => (left & right) != 0,
            JumpCondType::BitSet => todo!(),
        };

        if cond.is_negated {
            !result
        } else {
            result
        }
    }

    fn push_code_stack(&mut self, addr: CodeAddress) {
        self.call_stack.push(addr);
    }

    fn pop_code_stack(&mut self) -> CodeAddress {
        self.call_stack.pop().unwrap()
    }

    fn push_data_stack_frame(&mut self, val: &[i32]) {
        for &v in val.iter().rev() {
            self.data_stack.push(v);
        }
        self.data_stack.push(val.len() as i32);
    }

    fn pop_data_stack_frame(&mut self) {
        let count = self.data_stack.pop().unwrap() as usize;
        for _ in 0..count {
            self.data_stack.pop().unwrap();
        }
    }

    fn evaluate_expression(&mut self, expr: &Expression) -> i32 {
        let mut stack = SmallVec::<[i32; 16]>::new();
        for term in expr.0.iter() {
            match term {
                &ExpressionTerm::Push(v) => stack.push(self.get_number(v)),
                ExpressionTerm::Add => {
                    let left = stack.pop().unwrap();
                    let right = stack.pop().unwrap();
                    stack.push(left + right);
                }
                ExpressionTerm::Subtract => {
                    let left = stack.pop().unwrap();
                    let right = stack.pop().unwrap();
                    stack.push(left - right);
                }
                ExpressionTerm::Multiply => {
                    let left = stack.pop().unwrap();
                    let right = stack.pop().unwrap();
                    stack.push(left * right);
                }
                ExpressionTerm::Divide => {
                    let left = stack.pop().unwrap();
                    let right = stack.pop().unwrap();
                    stack.push(if right != 0 { left / right } else { 0 });
                }
                ExpressionTerm::Remainder => {
                    let left = stack.pop().unwrap();
                    let right = stack.pop().unwrap();
                    let div = if right != 0 { left / right } else { 0 };
                    stack.push(left - div * right);
                }
                ExpressionTerm::MultiplyReal => {
                    let left = stack.pop().unwrap();
                    let right = stack.pop().unwrap();
                    assert!(left >= 0 && right >= 0); // not sure if this will behave correctly otherwise
                    stack.push(left * right / 1000);
                }
                ExpressionTerm::Min => {
                    let left = stack.pop().unwrap();
                    let right = stack.pop().unwrap();
                    stack.push(left.min(right));
                }
                ExpressionTerm::Max => {
                    let left = stack.pop().unwrap();
                    let right = stack.pop().unwrap();
                    stack.push(left.max(right));
                }
            }
        }
        if stack.len() != 1 {
            warn!("Expression did not evaluate to a single value");
        }

        stack.pop().unwrap()
    }

    fn update_prng(&mut self) {
        self.prng_state = self.prng_state.wrapping_mul(0x343fd).wrapping_add(0x269ec3);
    }

    fn run_prng(&self, a: i32, b: i32) -> i32 {
        let state = self.prng_state;

        if a == b {
            a
        } else {
            let useful_state = (state >> 8 & 0xffff) as i32;
            let interval_size = (b - a).abs() + 1;
            let lower_bound = a.min(b);

            let amplitude = (useful_state * interval_size) >> 0x10;

            lower_bound + amplitude
        }
    }

    fn begin_command(
        &mut self,
        command: Command,
        pc: CodeAddress,
        ctx: &mut L::Context<'_>,
    ) -> CommandContext<L> {
        use tracing::debug_span;

        // TODO: we probably can manage spans in a generalized way
        let (span, command_state) = match command {
            Command::EXIT { arg1, arg2 } => {
                let arg2 = self.get_number(arg2);
                (
                    debug_span!("EXIT", ?pc, ?arg1, ?arg2).entered(),
                    CommandState::Exit(L::exit(ctx, arg1, arg2)),
                )
            }
            Command::SGET { dest, slot_number } => {
                let slot_number = self.get_number(slot_number);
                (
                    debug_span!("SGET", ?pc, ?dest, ?slot_number).entered(),
                    CommandState::SGet(dest, L::sget(ctx, slot_number)),
                )
            }
            Command::SSET { slot_number, value } => {
                let slot_number = self.get_number(slot_number);
                let value = self.get_number(value);
                (
                    debug_span!("SSET", ?pc, ?slot_number, ?value).entered(),
                    CommandState::SSet(L::sset(ctx, slot_number, value)),
                )
            }
            Command::WAIT {
                wait_kind,
                wait_amount,
            } => {
                let wait_amount = self.get_number(wait_amount);
                (
                    debug_span!("WAIT", ?pc, ?wait_kind, ?wait_amount).entered(),
                    CommandState::Wait(L::wait(ctx, wait_kind, wait_amount)),
                )
            }
            Command::MSGINIT { arg } => {
                let arg = self.get_number(arg);
                let span = debug_span!("MSGINIT", ?pc, ?arg).entered();
                (span, CommandState::MsgInit(L::msginit(ctx, arg)))
            }
            Command::MSGSET { msg_id, text } => {
                let text = text.as_str();
                let span = debug_span!("MSGSET", ?pc, ?msg_id, ?text).entered();
                (span, CommandState::MsgSet(L::msgset(ctx, msg_id, text)))
            }
            Command::MSGWAIT { arg } => {
                let arg = self.get_number(arg);
                let span = debug_span!("MSGWAIT", ?pc, ?arg).entered();
                (span, CommandState::MsgWait(L::msgwait(ctx, arg)))
            }
            Command::MSGSIGNAL {} => {
                let span = debug_span!("MSGSIGNAL", ?pc).entered();
                (span, CommandState::MsgSignal(L::msgsignal(ctx)))
            }
            Command::MSGSYNC { arg1, arg2 } => {
                let arg1 = self.get_number(arg1);
                let arg2 = self.get_number(arg2);
                let span = debug_span!("MSGSYNC", ?pc, ?arg1, ?arg2).entered();
                (span, CommandState::MsgSync(L::msgsync(ctx, arg1, arg2)))
            }
            Command::MSGCLOSE { arg } => {
                let span = debug_span!("MSGCLOSE", ?pc, ?arg).entered();
                (span, CommandState::MsgClose(L::msgclose(ctx, arg)))
            }
            Command::SELECT {
                choice_set_base,
                choice_index,
                dest,
                arg4,
                choice_title,
                variants,
            } => {
                let choice_set_base = choice_set_base;
                let choice_index = choice_index;
                let arg4 = self.get_number(arg4);
                let choice_title = choice_title.as_str();
                let variants = variants
                    .0
                    .iter()
                    .map(|v| v.as_str())
                    .collect::<SmallVec<[&str; 4]>>();
                let span = debug_span!(
                    "SELECT",
                    ?pc,
                    ?choice_set_base,
                    ?choice_index,
                    ?dest,
                    ?arg4,
                    ?choice_title,
                    ?variants
                )
                .entered();
                (
                    span,
                    CommandState::Select(
                        dest,
                        L::select(
                            ctx,
                            choice_set_base,
                            choice_index,
                            arg4,
                            choice_title,
                            &variants,
                        ),
                    ),
                )
            }
            Command::WIPE {
                arg1,
                arg2,
                arg3,
                params,
            } => {
                let arg1 = self.get_number(arg1);
                let arg2 = self.get_number(arg2);
                let arg3 = self.get_number(arg3);
                let params = params.0.map(|p| self.get_number(p));
                let span = debug_span!("WIPE", ?pc, ?arg1, ?arg2, ?arg3, ?params).entered();
                (
                    span,
                    CommandState::Wipe(L::wipe(ctx, arg1, arg2, arg3, &params)),
                )
            }
            Command::WIPEWAIT {} => {
                let span = debug_span!("WIPEWAIT", ?pc).entered();
                (span, CommandState::WipeWait(L::wipewait(ctx)))
            }
            Command::BGMPLAY {
                arg1,
                arg2,
                arg3,
                arg4,
            } => {
                let arg1 = self.get_number(arg1);
                let arg2 = self.get_number(arg2);
                let arg3 = self.get_number(arg3);
                let arg4 = self.get_number(arg4);
                let span = debug_span!("BGMPLAY", ?pc, ?arg1, ?arg2, ?arg3, ?arg4).entered();
                (
                    span,
                    CommandState::BgmPlay(L::bgmplay(ctx, arg1, arg2, arg3, arg4)),
                )
            }
            Command::BGMSTOP { arg } => {
                let arg = self.get_number(arg);
                let span = debug_span!("BGMSTOP", ?pc, ?arg).entered();
                (span, CommandState::BgmStop(L::bgmstop(ctx, arg)))
            }
            Command::BGMVOL { arg1, arg2 } => {
                let arg1 = self.get_number(arg1);
                let arg2 = self.get_number(arg2);
                let span = debug_span!("BGMVOL", ?pc, ?arg1, ?arg2).entered();
                (span, CommandState::BgmVol(L::bgmvol(ctx, arg1, arg2)))
            }
            Command::BGMWAIT { arg } => {
                let arg = self.get_number(arg);
                let span = debug_span!("BGMWAIT", ?pc, ?arg).entered();
                (span, CommandState::BgmWait(L::bgmwait(ctx, arg)))
            }
            Command::BGMSYNC { arg } => {
                let arg = self.get_number(arg);
                let span = debug_span!("BGMSYNC", ?pc, ?arg).entered();
                (span, CommandState::BgmSync(L::bgmsync(ctx, arg)))
            }
            Command::SEPLAY {
                arg1,
                arg2,
                arg3,
                arg4,
                arg5,
                arg6,
                arg7,
            } => {
                let arg1 = self.get_number(arg1);
                let arg2 = self.get_number(arg2);
                let arg3 = self.get_number(arg3);
                let arg4 = self.get_number(arg4);
                let arg5 = self.get_number(arg5);
                let arg6 = self.get_number(arg6);
                let arg7 = self.get_number(arg7);
                let span = debug_span!(
                    "SEPLAY",
                    ?pc,
                    ?arg1,
                    ?arg2,
                    ?arg3,
                    ?arg4,
                    ?arg5,
                    ?arg6,
                    ?arg7
                )
                .entered();
                (
                    span,
                    CommandState::SePlay(L::seplay(ctx, arg1, arg2, arg3, arg4, arg5, arg6, arg7)),
                )
            }
            Command::SESTOP { arg1, arg2 } => {
                let arg1 = self.get_number(arg1);
                let arg2 = self.get_number(arg2);
                let span = debug_span!("SESTOP", ?pc, ?arg1, ?arg2).entered();
                (span, CommandState::SeStop(L::sestop(ctx, arg1, arg2)))
            }
            Command::SESTOPALL { arg } => {
                let arg = self.get_number(arg);
                let span = debug_span!("SESTOPALL", ?pc, ?arg).entered();
                (span, CommandState::SeStopAll(L::sestopall(ctx, arg)))
            }

            // GAP
            Command::SAVEINFO { level, info } => {
                let level = self.get_number(level);
                let info = info.as_str();
                let span = debug_span!("SAVEINFO", ?pc, ?level, ?info).entered();
                (span, CommandState::SaveInfo(L::saveinfo(ctx, level, info)))
            }
            Command::AUTOSAVE {} => {
                let span = debug_span!("AUTOSAVE", ?pc).entered();
                (span, CommandState::AutoSave(L::autosave(ctx)))
            }

            Command::LAYERINIT { arg: layer_id } => {
                let arg = VLayerId(self.get_number(layer_id));
                let span = debug_span!("LAYERINIT", ?pc, ?arg).entered();
                (span, CommandState::LayerInit(L::layerinit(ctx, arg)))
            }
            Command::LAYERLOAD {
                layer_id,
                layer_type,
                leave_uninitialized,
                params,
            } => {
                let layer_id = VLayerId(self.get_number(layer_id));
                let layer_type = self.get_number(layer_type);
                let leave_uninitialized = self.get_number(leave_uninitialized);
                let params = params.0.map(|v| self.get_number(v));
                let span = debug_span!(
                    "LAYERLOAD",
                    ?pc,
                    ?layer_id,
                    ?layer_type,
                    ?leave_uninitialized,
                    ?params
                )
                .entered();
                (
                    span,
                    CommandState::LayerLoad(L::layerload(
                        ctx,
                        layer_id,
                        layer_type,
                        leave_uninitialized,
                        &params,
                    )),
                )
            }
            Command::LAYERUNLOAD {
                layer_id,
                delay_time,
            } => {
                let layer_id = VLayerId(self.get_number(layer_id));
                let delay_time = self.get_number(delay_time);
                let span = debug_span!("LAYERUNLOAD", ?pc, ?layer_id, ?delay_time).entered();
                (
                    span,
                    CommandState::LayerUnload(L::layerunload(ctx, layer_id, delay_time)),
                )
            }
            Command::LAYERCTRL {
                layer_id,
                property_id,
                params,
            } => {
                let layer_id = VLayerId(self.get_number(layer_id));
                let property_id = self.get_number(property_id);
                let params = params.0.map(|v| self.get_number(v));
                let span =
                    debug_span!("LAYERCTRL", ?pc, ?layer_id, ?property_id, ?params).entered();
                (
                    span,
                    CommandState::LayerCtrl(L::layerctrl(ctx, layer_id, property_id, &params)),
                )
            }
            Command::LAYERWAIT {
                layer_id,
                wait_properties,
            } => {
                let layer_id = VLayerId(self.get_number(layer_id));
                let wait_properties = wait_properties
                    .0
                    .into_iter()
                    .map(|v| self.get_number(v))
                    .collect::<SmallVec<[i32; 6]>>();
                let span = debug_span!("LAYERWAIT", ?pc, ?layer_id, ?wait_properties).entered();
                (
                    span,
                    CommandState::LayerWait(L::layerwait(ctx, layer_id, &wait_properties)),
                )
            }
            Command::TRANSSET {
                arg1,
                arg2,
                arg3,
                params,
            } => {
                let arg1 = self.get_number(arg1);
                let arg2 = self.get_number(arg2);
                let arg3 = self.get_number(arg3);
                let params = params.0.map(|v| self.get_number(v));
                let span = debug_span!("TRANSSET", ?pc, ?arg1, ?arg2, ?arg3, ?params).entered();
                (
                    span,
                    CommandState::TransSet(L::transset(ctx, arg1, arg2, arg3, &params)),
                )
            }
            Command::TRANSWAIT { arg } => {
                let arg = self.get_number(arg);
                let span = debug_span!("TRANSWAIT", ?pc, ?arg).entered();
                (span, CommandState::TransWait(L::transwait(ctx, arg)))
            }
            Command::PAGEBACK {} => {
                let span = debug_span!("PAGEBACK", ?pc).entered();
                (span, CommandState::PageBack(L::pageback(ctx)))
            }
            Command::PLANESELECT { arg } => {
                let arg = self.get_number(arg);
                let span = debug_span!("PLANESELECT", ?pc, ?arg).entered();
                (span, CommandState::PlaneSelect(L::planeselect(ctx, arg)))
            }
            Command::MASKLOAD { arg1, arg2, arg3 } => {
                let arg1 = self.get_number(arg1);
                let arg2 = self.get_number(arg2);
                let arg3 = self.get_number(arg3);
                let span = debug_span!("MASKLOAD", ?pc, ?arg1, ?arg2, ?arg3).entered();
                (
                    span,
                    CommandState::MaskLoad(L::maskload(ctx, arg1, arg2, arg3)),
                )
            }
            Command::MASKUNLOAD {} => {
                let span = debug_span!("MASKUNLOAD", ?pc).entered();
                (span, CommandState::MaskUnload(L::maskunload(ctx)))
            }
            Command::CHARS { arg1, arg2 } => {
                let arg1 = self.get_number(arg1);
                let arg2 = self.get_number(arg2);
                let span = debug_span!("CHARS", ?pc, ?arg1, ?arg2).entered();
                (span, CommandState::Chars(L::chars(ctx, arg1, arg2)))
            }
            Command::TIPSGET { arg } => {
                let arg = arg
                    .0
                    .iter()
                    .map(|v| self.get_number(*v))
                    .collect::<SmallVec<[i32; 6]>>();
                let span = debug_span!("TIPSGET", ?pc, ?arg).entered();
                (span, CommandState::TipsGet(L::tipsget(ctx, &arg)))
            }
            Command::QUIZ { dest, arg } => {
                let arg = self.get_number(arg);
                let span = debug_span!("QUIZ", ?pc, ?dest, ?arg).entered();
                (span, CommandState::Quiz(dest, L::quiz(ctx, arg)))
            }
            Command::SHOWCHARS {} => {
                let span = debug_span!("SHOWCHARS", ?pc).entered();
                (span, CommandState::ShowChars(L::showchars(ctx)))
            }
            Command::NOTIFYSET { arg } => {
                let arg = self.get_number(arg);
                let span = debug_span!("NOTIFYSET", ?pc, ?arg).entered();
                (span, CommandState::NotifySet(L::notifyset(ctx, arg)))
            }

            Command::DEBUGOUT { format, args } => {
                let args = args
                    .0
                    .into_iter()
                    .map(|v| self.get_number(v))
                    .collect::<SmallVec<[i32; 6]>>();

                (
                    debug_span!("DEBUGOUT", ?pc, ?format, ?args).entered(),
                    CommandState::DebugOut(L::debugout(ctx, format.as_str(), &args)),
                )
            }
            _ => {
                warn!(?pc, ?command, "unimplemented command");
                todo!("unimplemented command: {:?}", command)
            }
        };

        CommandContext {
            span: span.exit(),
            command_state,
        }
    }

    fn poll_command(&mut self, ctx: &mut L::Context<'_>) -> CommandPoll<ExitResult> {
        // hack to get IntelliJ rust plugin to stop complaining about poll being undefined
        #[allow(unused_imports)]
        use std::future::Future;

        const CONTINUE: CommandPoll<ExitResult> = CommandPoll::Ready(ExitResult::Continue);

        let result = match &mut self.command_context {
            None => CONTINUE,
            Some(CommandContext {
                span,
                command_state,
            }) => {
                let span = span.clone(); // make the borrow checker happy
                let _guard = span.enter();
                match command_state {
                    CommandState::Exit(cmd) => cmd.poll(ctx),
                    &mut CommandState::SGet(dest, ref mut cmd) => cmd
                        .poll(ctx)
                        .and_continue(|result| self.set_memory(dest, result)),
                    CommandState::SSet(cmd) => cmd.poll(ctx).map_continue(),
                    CommandState::Wait(cmd) => cmd.poll(ctx).map_continue(),
                    CommandState::MsgInit(cmd) => cmd.poll(ctx).map_continue(),
                    CommandState::MsgSet(cmd) => cmd.poll(ctx).map_continue(),
                    CommandState::MsgWait(cmd) => cmd.poll(ctx).map_continue(),
                    CommandState::MsgSignal(cmd) => cmd.poll(ctx).map_continue(),
                    CommandState::MsgSync(cmd) => cmd.poll(ctx).map_continue(),
                    CommandState::MsgClose(cmd) => cmd.poll(ctx).map_continue(),
                    &mut CommandState::Select(dest, ref mut cmd) => cmd
                        .poll(ctx)
                        .and_continue(|result| self.set_memory(dest, result)),
                    CommandState::Wipe(cmd) => cmd.poll(ctx).map_continue(),
                    CommandState::WipeWait(cmd) => cmd.poll(ctx).map_continue(),
                    CommandState::BgmPlay(cmd) => cmd.poll(ctx).map_continue(),
                    CommandState::BgmStop(cmd) => cmd.poll(ctx).map_continue(),
                    CommandState::BgmVol(cmd) => cmd.poll(ctx).map_continue(),
                    CommandState::BgmWait(cmd) => cmd.poll(ctx).map_continue(),
                    CommandState::BgmSync(cmd) => cmd.poll(ctx).map_continue(),
                    CommandState::SePlay(cmd) => cmd.poll(ctx).map_continue(),
                    CommandState::SeStop(cmd) => cmd.poll(ctx).map_continue(),
                    CommandState::SeStopAll(cmd) => cmd.poll(ctx).map_continue(),

                    CommandState::SaveInfo(cmd) => cmd.poll(ctx).map_continue(),
                    CommandState::AutoSave(cmd) => cmd.poll(ctx).map_continue(),
                    CommandState::LayerInit(cmd) => cmd.poll(ctx).map_continue(),
                    CommandState::LayerLoad(cmd) => cmd.poll(ctx).map_continue(),
                    CommandState::LayerUnload(cmd) => cmd.poll(ctx).map_continue(),
                    CommandState::LayerCtrl(cmd) => cmd.poll(ctx).map_continue(),
                    CommandState::LayerWait(cmd) => cmd.poll(ctx).map_continue(),
                    CommandState::LayerSwap(cmd) => cmd.poll(ctx).map_continue(),
                    CommandState::LayerSelect(cmd) => cmd.poll(ctx).map_continue(),
                    CommandState::MovieWait(cmd) => cmd.poll(ctx).map_continue(),
                    CommandState::TransSet(cmd) => cmd.poll(ctx).map_continue(),
                    CommandState::TransWait(cmd) => cmd.poll(ctx).map_continue(),
                    CommandState::PageBack(cmd) => cmd.poll(ctx).map_continue(),
                    CommandState::PlaneSelect(cmd) => cmd.poll(ctx).map_continue(),
                    CommandState::PlaneClear(cmd) => cmd.poll(ctx).map_continue(),
                    CommandState::MaskLoad(cmd) => cmd.poll(ctx).map_continue(),
                    CommandState::MaskUnload(cmd) => cmd.poll(ctx).map_continue(),

                    CommandState::Chars(cmd) => cmd.poll(ctx).map_continue(),
                    CommandState::TipsGet(cmd) => cmd.poll(ctx).map_continue(),
                    &mut CommandState::Quiz(dest, ref mut cmd) => cmd
                        .poll(ctx)
                        .and_continue(|result| self.set_memory(dest, result)),
                    CommandState::ShowChars(cmd) => cmd.poll(ctx).map_continue(),
                    CommandState::NotifySet(cmd) => cmd.poll(ctx).map_continue(),

                    CommandState::DebugOut(cmd) => cmd.poll(ctx).map_continue(),
                }
            }
        };

        if let CommandPoll::Ready(_) = &result {
            self.command_context = None;
        }

        result
    }

    /// Execute one instruction
    /// pc is the program counter before the instruction was read
    #[instrument(skip(self, ctx), level = "trace")]
    fn run_instruction(
        &mut self,
        instruction: Instruction,
        pc: CodeAddress,
        ctx: &mut L::Context<'_>,
    ) -> CommandPoll<ExitResult> {
        self.update_prng();

        match instruction {
            Instruction::uo(UnaryOperation {
                ty,
                destination,
                source,
            }) => {
                let source = self.get_number(source);
                let result = match ty {
                    UnaryOperationType::Zero => 0,
                    UnaryOperationType::Negate => -source,
                    _ => todo!(),
                };

                trace!(?pc, ?ty, ?destination, ?source, ?result, "uo");

                self.set_memory(destination, result);
            }
            Instruction::bo(BinaryOperation {
                ty,
                left,
                right,
                destination,
            }) => {
                let left = self.get_number(left);
                let right = self.get_number(right);
                let result = match ty {
                    BinaryOperationType::MovRight => right,
                    BinaryOperationType::Zero => 0,
                    BinaryOperationType::Add => left + right,
                    BinaryOperationType::Subtract => left - right,
                    BinaryOperationType::Multiply => left * right,
                    BinaryOperationType::Divide => {
                        if right != 0 {
                            left / right
                        } else {
                            0
                        }
                    }
                    BinaryOperationType::Remainder => {
                        let div = if right != 0 { left / right } else { 0 };
                        left - div * right
                    }
                    BinaryOperationType::BitwiseAnd => left & right,
                    BinaryOperationType::BitwiseOr => left | right,
                    BinaryOperationType::BitwiseXor => left ^ right,
                    BinaryOperationType::LeftShift => left << right,
                    BinaryOperationType::RightShift => left >> right,
                    BinaryOperationType::MultiplyReal => todo!(),
                    BinaryOperationType::DivideReal => todo!(),
                };

                trace!(?pc, ?ty, ?destination, ?left, ?right, ?result, "bo");

                self.set_memory(destination, result);
            }

            Instruction::exp { dest, expr } => {
                let result = self.evaluate_expression(&expr);
                trace!(?pc, ?dest, ?result, ?expr, "exp");
                self.set_memory(dest, result);
            }
            Instruction::gt { dest, value, table } => {
                let value = self.get_number(value);

                let result = if value >= 0 && value < table.0.len() as i32 {
                    self.get_number(table.0[value as usize])
                } else {
                    0
                };
                trace!(?pc, ?value, ?result, ?dest, table_len = ?table.0.len(), "gt");
                self.set_memory(dest, result);
            }
            Instruction::jc {
                cond,
                left,
                right,
                target,
            } => {
                let left = self.get_number(left);
                let right = self.get_number(right);
                let cond = self.compute_jump_condition(cond, left, right);

                trace!(?pc, ?cond, ?left, ?right, ?target, "jc");
                if cond {
                    self.instruction_reader.set_position(target);
                }
            }
            Instruction::j { target } => {
                trace!(?pc, ?target, "j");
                self.instruction_reader.set_position(target);
            }
            Instruction::gosub { target } => {
                trace!(?pc, ?target, "gosub");
                self.push_code_stack(self.instruction_reader.position());
                self.instruction_reader.set_position(target);
            }
            Instruction::retsub {} => {
                let target = self.pop_code_stack();
                trace!(?pc, ?target, "retsub");
                self.instruction_reader.set_position(target);
            }
            Instruction::jt { value, table } => {
                let value = self.get_number(value);

                trace!(?pc, ?value, table_len = ?table.0.len(), "jt");

                // if value < 0 {
                //     panic!("jump table command with negative value");
                // }
                if value >= 0 && value < table.0.len() as i32 {
                    self.instruction_reader
                        .set_position(table.0[value as usize]);
                }
            }
            Instruction::rnd { dest, min, max } => {
                let min = self.get_number(min);
                let max = self.get_number(max);
                let result = self.run_prng(min, max);
                trace!(?pc, ?dest, ?min, ?max, ?result, prng_state = ?self.prng_state, "rnd");
                self.set_memory(dest, result);
            }
            Instruction::call { target, args } => {
                let args = args
                    .0
                    .into_iter()
                    .map(|v| self.get_number(v))
                    .collect::<SmallVec<[i32; 6]>>();
                trace!(?pc, ?target, ?args, "call");

                self.push_code_stack(self.instruction_reader.position());
                self.push_data_stack_frame(&args);
                self.instruction_reader.set_position(target);
            }
            Instruction::push { values } => {
                // unfortunately the game uses the call stack for both code addresses and sometimes data...
                // we just cast the data provided to CodeOffset and hope for the best
                // what could go wrong?
                let values = values
                    .0
                    .into_iter()
                    .map(|v| CodeAddress(self.get_number(v).try_into().unwrap()))
                    .collect::<SmallVec<[CodeAddress; 6]>>();
                trace!(?pc, ?values, "push");

                for value in values {
                    self.push_code_stack(value)
                }
            }
            Instruction::pop { dest } => {
                let values = (0..dest.0.len())
                    .map(|_| self.pop_code_stack().0.try_into().unwrap())
                    .collect::<SmallVec<[i32; 6]>>();
                trace!(?pc, ?values, "pop");

                for (dest, value) in dest.0.iter().zip(values) {
                    self.set_memory(*dest, value);
                }
            }
            Instruction::r#return {} => {
                self.pop_data_stack_frame();
                let target = self.pop_code_stack();
                trace!(?pc, ?target, "return");
                self.instruction_reader.set_position(target);
            }
            Instruction::Command(command) => {
                debug_assert!(matches!(self.command_context, None));
                self.command_context = Some(self.begin_command(command, pc, ctx));
                return self.poll_command(ctx);
            }
        }

        CommandPoll::Ready(ExitResult::Continue)
    }

    pub fn run<'a>(&mut self, ctx: &'a L::Context<'a>) -> Result<CommandPoll<i32>> {
        match self.poll_command(ctx) {
            CommandPoll::Ready(ExitResult::Continue) => {}
            CommandPoll::Ready(ExitResult::Exit(result)) => return Ok(CommandPoll::Ready(result)),
            CommandPoll::Pending => {
                return Ok(CommandPoll::Pending);
            }
        }

        loop {
            let pc = self.instruction_reader.position();
            let instruction = self.instruction_reader.read()?;
            match self.run_instruction(instruction, pc, ctx) {
                CommandPoll::Ready(ExitResult::Continue) => {}
                CommandPoll::Ready(ExitResult::Exit(result)) => {
                    return Ok(CommandPoll::Ready(result))
                }
                CommandPoll::Pending => {
                    return Ok(CommandPoll::Pending);
                }
            }
        }
    }
}
