use crate::format::scenario::instructions::MemoryAddress;
use tracing::debug;

#[must_use = "Don't forget to check whether the command is still pending"]
pub enum CommandPoll<T> {
    Ready(T),
    Pending,
}

impl CommandPoll<()> {
    pub fn map_continue(self) -> CommandPoll<ExitResult> {
        self.map(|_| ExitResult::Continue)
    }
}

impl<T> CommandPoll<T> {
    pub fn map<R, F: FnOnce(T) -> R>(self, f: F) -> CommandPoll<R> {
        match self {
            CommandPoll::Ready(t) => CommandPoll::Ready(f(t)),
            CommandPoll::Pending => CommandPoll::Pending,
        }
    }

    pub fn and_continue<F: FnOnce(T)>(self, f: F) -> CommandPoll<ExitResult> {
        self.map(|t| {
            f(t);
            ExitResult::Continue
        })
    }
}

/// A representation of an in-progress ADV command
///
/// The AdvCommand is similar to a future, though it doesn't have any callback stuff and is simply polled every update
pub trait AdvCommand<L> {
    type Output;
    fn poll(&mut self, listener: &mut L) -> CommandPoll<Self::Output>;
}

/*

#[brw(magic(0x00u8))]
    EXIT {
        /// This is encoded in the instruction
        /// If it's zero then the VM shuts down
        /// If it's nonzero then the VM treats it as a NOP
        /// Maybe it's a feature that is not used for umineko?
        arg1: u8,
        ///
        arg2: NumberSpec,
    },

    #[brw(magic(0x81u8))]
    SGET {
        dest: MemoryAddress,
        slot_number: NumberSpec,
    },
    #[brw(magic(0x82u8))]
    SSET { arg1: NumberSpec, arg2: NumberSpec },
    #[brw(magic(0x83u8))]
    WAIT { arg1: u8, arg2: NumberSpec },
    // 0x84 is unused
    #[brw(magic(0x85u8))]
    MSGINIT { arg: NumberSpec },
    #[brw(magic(0x86u8))]
    MSGSET { msg_id: u32, text: U16String }, // TODO: this string needs a fixup (see ShinDataUtil's OpcodeDefinitions.NeedsStringFixup)
    #[brw(magic(0x87u8))]
    MSGWAIT { arg: NumberSpec },
    #[brw(magic(0x88u8))]
    MSGSIGNAL {},
    #[brw(magic(0x89u8))]
    MSGSYNC { arg1: NumberSpec, arg2: NumberSpec },
    #[brw(magic(0x8au8))]
    MSGCLOSE { arg: u8 },

    #[brw(magic(0x8du8))]
    SELECT {
        choice_set_base: u16,
        choice_index: u16,
        dest: MemoryAddress,
        arg4: NumberSpec,
        choice_title: U16String,
        variants: StringArray,
    },
    #[brw(magic(0x8eu8))]
    WIPE {
        arg1: NumberSpec,
        arg2: NumberSpec,
        arg3: NumberSpec,
        params: BitmaskNumberArray,
    },
    #[brw(magic(0x8fu8))]
    WIPEWAIT {},
    #[brw(magic(0x90u8))]
    BGMPLAY {
        arg1: NumberSpec,
        arg2: NumberSpec,
        arg3: NumberSpec,
        arg4: NumberSpec,
    },
    #[brw(magic(0x91u8))]
    BGMSTOP { arg: NumberSpec },
    #[brw(magic(0x92u8))]
    BGMVOL { arg1: NumberSpec, arg2: NumberSpec },
    #[brw(magic(0x93u8))]
    BGMWAIT { arg: NumberSpec },
    #[brw(magic(0x94u8))]
    BGMSYNC { arg: NumberSpec },
    #[brw(magic(0x95u8))]
    SEPLAY {
        arg1: NumberSpec,
        arg2: NumberSpec,
        arg3: NumberSpec,
        arg4: NumberSpec,
        arg5: NumberSpec,
        arg6: NumberSpec,
        arg7: NumberSpec,
    },
    #[brw(magic(0x96u8))]
    SESTOP { arg1: NumberSpec, arg2: NumberSpec },
    #[brw(magic(0x97u8))]
    SESTOPALL { arg: NumberSpec },
    #[brw(magic(0x98u8))]
    SEVOL {
        arg1: NumberSpec,
        arg2: NumberSpec,
        arg3: NumberSpec,
    },
    #[brw(magic(0x99u8))]
    SEPAN {
        arg1: NumberSpec,
        arg2: NumberSpec,
        arg3: NumberSpec,
    },
    #[brw(magic(0x9au8))]
    SEWAIT { arg1: NumberSpec, arg2: NumberSpec },
    #[brw(magic(0x9bu8))]
    SEONCE {
        arg1: NumberSpec,
        arg2: NumberSpec,
        arg3: NumberSpec,
        arg4: NumberSpec,
        arg5: NumberSpec,
    },
    #[brw(magic(0x9cu8))]
    VOICEPLAY {
        name: U16String,
        arg1: NumberSpec,
        arg2: NumberSpec,
    },
    #[brw(magic(0x9du8))]
    VOICESTOP {},
    #[brw(magic(0x9eu8))]
    VOICEWAIT { arg: NumberSpec },
    #[brw(magic(0x9fu8))]
    SYSSE { arg1: NumberSpec, arg2: NumberSpec },

    #[brw(magic(0xa0u8))]
    SAVEINFO { level: NumberSpec, info: U16String }, // TODO: this string needs a fixup (see ShinDataUtil's OpcodeDefinitions.NeedsStringFixup)
    #[brw(magic(0xa1u8))]
    AUTOSAVE {},
    #[brw(magic(0xa2u8))]
    EVBEGIN { arg: NumberSpec },
    #[brw(magic(0xa3u8))]
    EVEND {},
    #[brw(magic(0xa4u8))]
    RESUMESET {},
    #[brw(magic(0xa5u8))]
    RESUME {},
    #[brw(magic(0xa6u8))]
    SYSCALL { arg1: NumberSpec, arg2: NumberSpec },

    #[brw(magic(0xb0u8))]
    TROPHY { arg: NumberSpec },
    #[brw(magic(0xb1u8))]
    UNLOCK {
        arg1: u8,
        arg2: U8SmallList<[NumberSpec; 6]>,
    },

    /// Reset property values to their initial state
    #[brw(magic(0xc0u8))]
    LAYERINIT { arg: NumberSpec },
    /// Load a layer resource or smth
    /// There are multiple layer types and they have different arguments
    #[brw(magic(0xc1u8))]
    LAYERLOAD {
        layer_id: NumberSpec,
        layer_type: NumberSpec,
        // TODO: what does this mean again?
        leave_uninitialized: NumberSpec,
        params: BitmaskNumberArray,
    },
    #[brw(magic(0xc2u8))]
    LAYERUNLOAD {
        layer_id: NumberSpec,
        delay_time: NumberSpec,
    },
    /// Change layer property, possibly through a transition.
    #[brw(magic(0xc3u8))]
    LAYERCTRL {
        layer_id: NumberSpec,
        property_id: NumberSpec,
        // in the params there are (always?) three numbers
        // ctrl_value, ctrl_time and ctrl_flags
        params: BitmaskNumberArray,
    },
    /// Wait for property transitions to finish.
    #[brw(magic(0xc4u8))]
    LAYERWAIT {
        layer_id: NumberSpec,
        wait_properties: U8SmallList<[NumberSpec; 6]>,
    },
    #[brw(magic(0xc5u8))]
    LAYERSWAP { arg1: NumberSpec, arg2: NumberSpec },
    /// Select a subset of layers to perform batch operations
    /// (TODO: fact check) These can be used as layer_id = -4
    #[brw(magic(0xc6u8))]
    LAYERSELECT {
        selection_start_id: NumberSpec,
        selection_end_id: NumberSpec,
    },
    #[brw(magic(0xc7u8))]
    MOVIEWAIT { arg1: NumberSpec, arg2: NumberSpec },
    // 0xc8 unused
    #[brw(magic(0xc9u8))]
    TRANSSET {
        arg1: NumberSpec,
        arg2: NumberSpec,
        arg3: NumberSpec,
        params: BitmaskNumberArray,
    },
    #[brw(magic(0xcau8))]
    TRANSWAIT { arg: NumberSpec },
    #[brw(magic(0xcbu8))]
    PAGEBACK {},
    #[brw(magic(0xccu8))]
    PLANESELECT { arg: NumberSpec },
    #[brw(magic(0xcdu8))]
    PLANECLEAR {},
    #[brw(magic(0xceu8))]
    MASKLOAD {
        arg1: NumberSpec,
        arg2: NumberSpec,
        arg3: NumberSpec,
    },
    #[brw(magic(0xcfu8))]
    MASKUNLOAD {},

    #[brw(magic(0xe0u8))]
    CHARS { arg1: NumberSpec, arg2: NumberSpec },
    #[brw(magic(0xe1u8))]
    TIPSGET { arg: U8SmallList<[NumberSpec; 6]> },
    #[brw(magic(0xe2u8))]
    QUIZ {
        dest: MemoryAddress,
        arg: NumberSpec,
    },
    #[brw(magic(0xe3u8))]
    SHOWCHARS {},
    #[brw(magic(0xe4u8))]
    NOTIFYSET { arg: NumberSpec },

    #[brw(magic(0xffu8))]
    DEBUGOUT {
        format: U8String,
        args: U8SmallList<[NumberSpec; 6]>,
    },

 */

/// Layer id, allowing for the special values -1, -2, -3, -4, -5
pub struct LayerId(pub i32);
/// Layer id, but allowing only "real" layers
pub struct RealLayerId(pub u32);

pub(super) struct CommandContext<L: AdvListener> {
    pub span: tracing::Span,
    pub command_state: CommandState<L>,
}

// TODO: maybe macro for this
pub(super) enum CommandState<L: AdvListener> {
    Exit(L::Exit),
    SGet(MemoryAddress, L::SGet),
    SSet(L::SSet),
    MsgInit(L::MsgInit),
    MsgSet(L::MsgSet),
    MsgSignal(L::MsgSignal),
    MsgSync(L::MsgSync),
    MsgClose(L::MsgClose),
    SaveInfo(L::SaveInfo),
    AutoSave(L::AutoSave),
    LayerInit(L::LayerInit),
    LayerLoad(L::LayerLoad),
    LayerUnload(L::LayerUnload),
    LayerCtrl(L::LayerCtrl),
    LayerWait(L::LayerWait),
    LayerSwap(L::LayerSwap),
    LayerSelect(L::LayerSelect),
    DebugOut(L::DebugOut),
}

// TODO: use macros?
pub trait AdvListener: Sized {
    type Exit: AdvCommand<Self, Output = ExitResult>;
    fn exit(&mut self, arg1: u8, arg2: i32) -> Self::Exit;

    type SGet: AdvCommand<Self, Output = i32>;
    fn sget(&mut self, slot_number: i32) -> Self::SGet;

    type SSet: AdvCommand<Self, Output = ()>;
    fn sset(&mut self, slot_number: i32, value: i32) -> Self::SSet;

    type MsgInit: AdvCommand<Self, Output = ()>;
    fn msginit(&mut self, arg: i32) -> Self::MsgInit;

    type MsgSet: AdvCommand<Self, Output = ()>;
    fn msgset(&mut self, msg_id: u32, text: &str) -> Self::MsgSet;

    type MsgSignal: AdvCommand<Self, Output = ()>;
    fn msgsignal(&mut self) -> Self::MsgSignal;

    type MsgSync: AdvCommand<Self, Output = ()>;
    fn msgsync(&mut self, arg1: i32, arg2: i32) -> Self::MsgSync;

    type MsgClose: AdvCommand<Self, Output = ()>;
    fn msgclose(&mut self, arg: u8) -> Self::MsgClose;

    // GAP

    type SaveInfo: AdvCommand<Self, Output = ()>;
    fn saveinfo(&mut self, level: i32, info: &str) -> Self::SaveInfo;

    type AutoSave: AdvCommand<Self, Output = ()>;
    fn autosave(&mut self) -> Self::AutoSave;

    // GAP

    type LayerInit: AdvCommand<Self, Output = ()>;
    fn layerinit(&mut self, layer_id: LayerId) -> Self::LayerInit;

    type LayerLoad: AdvCommand<Self, Output = ()>;
    fn layerload(
        &mut self,
        layer_id: LayerId,
        layer_type: i32,
        leave_uninitialized: i32,
        params: &[i32],
    ) -> Self::LayerLoad;

    type LayerUnload: AdvCommand<Self, Output = ()>;
    fn layerunload(&mut self, layer_id: LayerId, delay_time: i32) -> Self::LayerUnload;

    type LayerCtrl: AdvCommand<Self, Output = ()>;
    fn layerctrl(&mut self, layer_id: LayerId, property_id: i32, params: &[i32])
        -> Self::LayerCtrl;

    type LayerWait: AdvCommand<Self, Output = ()>;
    fn layerwait(&mut self, layer_id: LayerId, wait_properties: &[i32]) -> Self::LayerWait;

    type LayerSwap: AdvCommand<Self, Output = ()>;
    fn layerswap(&mut self, layer_id1: RealLayerId, layer_id2: RealLayerId) -> Self::LayerSwap;

    type LayerSelect: AdvCommand<Self, Output = ()>;
    fn layerselect(
        &mut self,
        selection_start_id: RealLayerId,
        selection_end_id: RealLayerId,
    ) -> Self::LayerSelect;

    type DebugOut: AdvCommand<Self, Output = ()>;
    fn debugout(&mut self, format: &str, args: &[i32]) -> Self::DebugOut;
}

pub enum ExitResult {
    Exit(i32),
    Continue,
}

pub struct Ready<T>(Option<T>);
impl<T, L> AdvCommand<L> for Ready<T> {
    type Output = T;

    fn poll(&mut self, _listener: &mut L) -> CommandPoll<Self::Output> {
        CommandPoll::Ready(self.0.take().expect("`Ready` polled after completion"))
    }
}

pub fn ready<R>(result: R) -> Ready<R> {
    Ready(Some(result))
}

pub struct DummyAdvListener;

impl AdvListener for DummyAdvListener {
    type Exit = Ready<ExitResult>;
    fn exit(&mut self, arg1: u8, arg2: i32) -> Self::Exit {
        todo!()
    }

    type SGet = Ready<i32>;
    fn sget(&mut self, slot_number: i32) -> Self::SGet {
        debug!("SGET {}", slot_number);
        ready(0)
    }

    type SSet = Ready<()>;
    fn sset(&mut self, slot_number: i32, value: i32) -> Self::SSet {
        debug!("SSET {} {}", slot_number, value);
        ready(())
    }

    type MsgInit = Ready<()>;
    fn msginit(&mut self, arg: i32) -> Self::MsgInit {
        debug!("MSGINIT {}", arg);
        ready(())
    }

    type MsgSet = Ready<()>;
    fn msgset(&mut self, msg_id: u32, text: &str) -> Self::MsgSet {
        todo!()
    }

    type MsgSignal = Ready<()>;
    fn msgsignal(&mut self) -> Self::MsgSignal {
        todo!()
    }

    type MsgSync = Ready<()>;
    fn msgsync(&mut self, arg1: i32, arg2: i32) -> Self::MsgSync {
        todo!()
    }

    type MsgClose = Ready<()>;
    fn msgclose(&mut self, arg: u8) -> Self::MsgClose {
        todo!()
    }

    type SaveInfo = Ready<()>;
    fn saveinfo(&mut self, level: i32, info: &str) -> Self::SaveInfo {
        todo!()
    }

    type AutoSave = Ready<()>;
    fn autosave(&mut self) -> Self::AutoSave {
        todo!()
    }

    type LayerInit = Ready<()>;
    fn layerinit(&mut self, layer_id: LayerId) -> Self::LayerInit {
        todo!()
    }

    type LayerLoad = Ready<()>;
    fn layerload(
        &mut self,
        layer_id: LayerId,
        layer_type: i32,
        leave_uninitialized: i32,
        params: &[i32],
    ) -> Self::LayerLoad {
        todo!()
    }

    type LayerUnload = Ready<()>;
    fn layerunload(&mut self, layer_id: LayerId, delay_time: i32) -> Self::LayerUnload {
        todo!()
    }

    type LayerCtrl = Ready<()>;
    fn layerctrl(
        &mut self,
        layer_id: LayerId,
        property_id: i32,
        params: &[i32],
    ) -> Self::LayerCtrl {
        todo!()
    }

    type LayerWait = Ready<()>;
    fn layerwait(&mut self, layer_id: LayerId, wait_properties: &[i32]) -> Self::LayerWait {
        todo!()
    }

    type LayerSwap = Ready<()>;
    fn layerswap(&mut self, layer_id1: RealLayerId, layer_id2: RealLayerId) -> Self::LayerSwap {
        todo!()
    }

    type LayerSelect = Ready<()>;
    fn layerselect(
        &mut self,
        selection_start_id: RealLayerId,
        selection_end_id: RealLayerId,
    ) -> Self::LayerSelect {
        todo!()
    }

    type DebugOut = Ready<()>;
    fn debugout(&mut self, format: &str, args: &[i32]) -> Self::DebugOut {
        todo!()
    }
}
