use crate::format::scenario::instructions::{MemoryAddress, NumberSpec};
use std::fmt::{Display, Formatter};
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
pub trait AdvCommand<L: AdvListener> {
    type Output;
    fn poll(&mut self, ctx: &L::Context<'_>, listener: &mut L) -> CommandPoll<Self::Output>;
}

/// Layer id, allowing for the special values -1, -2, -3, -4, -5
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VLayerId(pub i32);
/// Layer id, but allowing only "real" layers
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LayerId(pub u32);

impl VLayerId {
    pub fn to_layer_id(self) -> Option<LayerId> {
        if self.0 < 0 {
            None
        } else {
            Some(LayerId(self.0 as u32))
        }
    }
}

impl Display for VLayerId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Display for LayerId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub(super) struct CommandContext<L: AdvListener> {
    pub span: tracing::Span,
    pub command_state: CommandState<L>,
}

// TODO: maybe macro for this
pub(super) enum CommandState<L: AdvListener> {
    Exit(L::Exit),
    SGet(MemoryAddress, L::SGet),
    SSet(L::SSet),
    Wait(L::Wait),
    MsgInit(L::MsgInit),
    MsgSet(L::MsgSet),
    MsgWait(L::MsgWait),
    MsgSignal(L::MsgSignal),
    MsgSync(L::MsgSync),
    MsgClose(L::MsgClose),
    Select(MemoryAddress, L::Select),
    Wipe(L::Wipe),
    WipeWait(L::WipeWait),
    BgmPlay(L::BgmPlay),
    BgmStop(L::BgmStop),
    BgmVol(L::BgmVol),
    BgmWait(L::BgmWait),
    BgmSync(L::BgmSync),
    SePlay(L::SePlay),
    SeStop(L::SeStop),
    SeStopAll(L::SeStopAll),

    SaveInfo(L::SaveInfo),
    AutoSave(L::AutoSave),
    LayerInit(L::LayerInit),
    LayerLoad(L::LayerLoad),
    LayerUnload(L::LayerUnload),
    LayerCtrl(L::LayerCtrl),
    LayerWait(L::LayerWait),
    LayerSwap(L::LayerSwap),
    LayerSelect(L::LayerSelect),
    MovieWait(L::MovieWait),
    TransSet(L::TransSet),
    TransWait(L::TransWait),
    PageBack(L::PageBack),
    PlaneSelect(L::PlaneSelect),
    PlaneClear(L::PlaneClear),
    MaskLoad(L::MaskLoad),
    MaskUnload(L::MaskUnload),

    Chars(L::Chars),
    TipsGet(L::TipsGet),
    Quiz(MemoryAddress, L::Quiz),
    ShowChars(L::ShowChars),
    NotifySet(L::NotifySet),

    DebugOut(L::DebugOut),
}

// TODO: use macros?
pub trait AdvListener: Sized {
    type Context<'a>: 'a;

    type Exit: AdvCommand<Self, Output = ExitResult>;
    fn exit(&mut self, ctx: &Self::Context<'_>, arg1: u8, arg2: i32) -> Self::Exit;

    type SGet: AdvCommand<Self, Output = i32>;
    fn sget(&mut self, ctx: &Self::Context<'_>, slot_number: i32) -> Self::SGet;

    type SSet: AdvCommand<Self, Output = ()>;
    fn sset(&mut self, ctx: &Self::Context<'_>, slot_number: i32, value: i32) -> Self::SSet;

    type Wait: AdvCommand<Self, Output = ()>;
    fn wait(&mut self, ctx: &Self::Context<'_>, wait_kind: u8, wait_amount: i32) -> Self::Wait;

    type MsgInit: AdvCommand<Self, Output = ()>;
    fn msginit(&mut self, ctx: &Self::Context<'_>, arg: i32) -> Self::MsgInit;

    type MsgSet: AdvCommand<Self, Output = ()>;
    fn msgset(&mut self, ctx: &Self::Context<'_>, msg_id: u32, text: &str) -> Self::MsgSet;

    type MsgWait: AdvCommand<Self, Output = ()>;
    fn msgwait(&mut self, ctx: &Self::Context<'_>, arg: i32) -> Self::MsgWait;

    type MsgSignal: AdvCommand<Self, Output = ()>;
    fn msgsignal(&mut self, ctx: &Self::Context<'_>) -> Self::MsgSignal;

    type MsgSync: AdvCommand<Self, Output = ()>;
    fn msgsync(&mut self, ctx: &Self::Context<'_>, arg1: i32, arg2: i32) -> Self::MsgSync;

    type MsgClose: AdvCommand<Self, Output = ()>;
    fn msgclose(&mut self, ctx: &Self::Context<'_>, arg: u8) -> Self::MsgClose;

    type Select: AdvCommand<Self, Output = i32>;
    fn select(
        &mut self,
        ctx: &Self::Context<'_>,
        choice_set_base: u16,
        choice_index: u16,
        arg4: i32,
        choice_title: &str,
        variants: &[&str],
    ) -> Self::Select;

    type Wipe: AdvCommand<Self, Output = ()>;
    fn wipe(
        &mut self,
        ctx: &Self::Context<'_>,
        arg1: i32,
        arg2: i32,
        arg3: i32,
        params: &[i32; 8],
    ) -> Self::Wipe;

    type WipeWait: AdvCommand<Self, Output = ()>;
    fn wipewait(&mut self, ctx: &Self::Context<'_>) -> Self::WipeWait;

    type BgmPlay: AdvCommand<Self, Output = ()>;
    fn bgmplay(
        &mut self,
        ctx: &Self::Context<'_>,
        arg1: i32,
        arg2: i32,
        arg3: i32,
        arg4: i32,
    ) -> Self::BgmPlay;

    type BgmStop: AdvCommand<Self, Output = ()>;
    fn bgmstop(&mut self, ctx: &Self::Context<'_>, arg: i32) -> Self::BgmStop;

    type BgmVol: AdvCommand<Self, Output = ()>;
    fn bgmvol(&mut self, ctx: &Self::Context<'_>, arg1: i32, arg2: i32) -> Self::BgmVol;

    type BgmWait: AdvCommand<Self, Output = ()>;
    fn bgmwait(&mut self, ctx: &Self::Context<'_>, arg: i32) -> Self::BgmWait;

    type BgmSync: AdvCommand<Self, Output = ()>;
    fn bgmsync(&mut self, ctx: &Self::Context<'_>, arg: i32) -> Self::BgmSync;

    type SePlay: AdvCommand<Self, Output = ()>;
    #[allow(clippy::too_many_arguments)]
    fn seplay(
        &mut self,
        ctx: &Self::Context<'_>,
        arg1: i32,
        arg2: i32,
        arg3: i32,
        arg4: i32,
        arg5: i32,
        arg6: i32,
        arg7: i32,
    ) -> Self::SePlay;

    type SeStop: AdvCommand<Self, Output = ()>;
    fn sestop(&mut self, ctx: &Self::Context<'_>, arg1: i32, arg2: i32) -> Self::SeStop;

    type SeStopAll: AdvCommand<Self, Output = ()>;
    fn sestopall(&mut self, ctx: &Self::Context<'_>, arg: i32) -> Self::SeStopAll;

    // GAP

    type SaveInfo: AdvCommand<Self, Output = ()>;
    fn saveinfo(&mut self, ctx: &Self::Context<'_>, level: i32, info: &str) -> Self::SaveInfo;

    type AutoSave: AdvCommand<Self, Output = ()>;
    fn autosave(&mut self, ctx: &Self::Context<'_>) -> Self::AutoSave;

    // GAP

    type LayerInit: AdvCommand<Self, Output = ()>;
    fn layerinit(&mut self, ctx: &Self::Context<'_>, layer_id: VLayerId) -> Self::LayerInit;

    type LayerLoad: AdvCommand<Self, Output = ()>;
    fn layerload(
        &mut self,
        ctx: &Self::Context<'_>,
        layer_id: VLayerId,
        layer_type: i32,
        leave_uninitialized: i32,
        params: &[i32; 8],
    ) -> Self::LayerLoad;

    type LayerUnload: AdvCommand<Self, Output = ()>;
    fn layerunload(
        &mut self,
        ctx: &Self::Context<'_>,
        layer_id: VLayerId,
        delay_time: i32,
    ) -> Self::LayerUnload;

    type LayerCtrl: AdvCommand<Self, Output = ()>;
    fn layerctrl(
        &mut self,
        ctx: &Self::Context<'_>,
        layer_id: VLayerId,
        property_id: i32,
        params: &[i32; 8],
    ) -> Self::LayerCtrl;

    type LayerWait: AdvCommand<Self, Output = ()>;
    fn layerwait(
        &mut self,
        ctx: &Self::Context<'_>,
        layer_id: VLayerId,
        wait_properties: &[i32],
    ) -> Self::LayerWait;

    type LayerSwap: AdvCommand<Self, Output = ()>;
    fn layerswap(
        &mut self,
        ctx: &Self::Context<'_>,
        layer_id1: LayerId,
        layer_id2: LayerId,
    ) -> Self::LayerSwap;

    type LayerSelect: AdvCommand<Self, Output = ()>;
    fn layerselect(
        &mut self,
        ctx: &Self::Context<'_>,
        selection_start_id: LayerId,
        selection_end_id: LayerId,
    ) -> Self::LayerSelect;

    type MovieWait: AdvCommand<Self, Output = ()>;
    fn moviewait(&mut self, ctx: &Self::Context<'_>, arg: i32, arg2: i32) -> Self::MovieWait;

    type TransSet: AdvCommand<Self, Output = ()>;
    fn transset(
        &mut self,
        ctx: &Self::Context<'_>,
        arg: i32,
        arg2: i32,
        arg3: i32,
        params: &[i32; 8],
    ) -> Self::TransSet;

    type TransWait: AdvCommand<Self, Output = ()>;
    fn transwait(&mut self, ctx: &Self::Context<'_>, arg: i32) -> Self::TransWait;

    type PageBack: AdvCommand<Self, Output = ()>;
    fn pageback(&mut self, ctx: &Self::Context<'_>) -> Self::PageBack;

    type PlaneSelect: AdvCommand<Self, Output = ()>;
    fn planeselect(&mut self, ctx: &Self::Context<'_>, arg: i32) -> Self::PlaneSelect;

    type PlaneClear: AdvCommand<Self, Output = ()>;
    fn planeclear(&mut self, ctx: &Self::Context<'_>) -> Self::PlaneClear;

    type MaskLoad: AdvCommand<Self, Output = ()>;
    fn maskload(
        &mut self,
        ctx: &Self::Context<'_>,
        arg1: i32,
        arg2: i32,
        arg3: i32,
    ) -> Self::MaskLoad;

    type MaskUnload: AdvCommand<Self, Output = ()>;
    fn maskunload(&mut self, ctx: &Self::Context<'_>) -> Self::MaskUnload;

    type Chars: AdvCommand<Self, Output = ()>;
    fn chars(&mut self, ctx: &Self::Context<'_>, arg1: i32, arg2: i32) -> Self::Chars;

    type TipsGet: AdvCommand<Self, Output = ()>;
    fn tipsget(&mut self, ctx: &Self::Context<'_>, arg: &[i32]) -> Self::TipsGet;

    type Quiz: AdvCommand<Self, Output = i32>;
    fn quiz(&mut self, ctx: &Self::Context<'_>, arg: i32) -> Self::Quiz;

    type ShowChars: AdvCommand<Self, Output = ()>;
    fn showchars(&mut self, ctx: &Self::Context<'_>) -> Self::ShowChars;

    type NotifySet: AdvCommand<Self, Output = ()>;
    fn notifyset(&mut self, ctx: &Self::Context<'_>, arg: i32) -> Self::NotifySet;

    type DebugOut: AdvCommand<Self, Output = ()>;
    fn debugout(&mut self, ctx: &Self::Context<'_>, format: &str, args: &[i32]) -> Self::DebugOut;
}

pub enum ExitResult {
    Exit(i32),
    Continue,
}

pub struct Ready<T>(Option<T>);
impl<T, L: AdvListener> AdvCommand<L> for Ready<T> {
    type Output = T;

    #[inline]
    fn poll(&mut self, _ctx: &L::Context<'_>, _listener: &mut L) -> CommandPoll<Self::Output> {
        CommandPoll::Ready(self.0.take().expect("`Ready` polled after completion"))
    }
}

#[inline]
pub fn ready<R>(result: R) -> Ready<R> {
    Ready(Some(result))
}

pub struct DummyAdvListener;

// TODO: this is boilerplaty, at least we can add an immediate mode adapter
// in it the AdvCallbacks will return immediately, without returning a command object
impl AdvListener for DummyAdvListener {
    type Context<'a> = ();

    type Exit = Ready<ExitResult>;
    fn exit(&mut self, _: &(), arg1: u8, arg2: i32) -> Self::Exit {
        todo!()
    }

    type SGet = Ready<i32>;
    fn sget(&mut self, _: &(), slot_number: i32) -> Self::SGet {
        debug!("SGET {}", slot_number);
        ready(0)
    }

    type SSet = Ready<()>;
    fn sset(&mut self, _: &(), slot_number: i32, value: i32) -> Self::SSet {
        debug!("SSET {} {}", slot_number, value);
        ready(())
    }

    type Wait = Ready<()>;
    fn wait(&mut self, _: &(), wait_kind: u8, wait_amount: i32) -> Self::Wait {
        // assert_eq!(wait_kind, 0);
        debug!("WAIT {} {}", wait_kind, wait_amount);
        ready(())
    }

    type MsgInit = Ready<()>;
    fn msginit(&mut self, _: &(), arg: i32) -> Self::MsgInit {
        debug!("MSGINIT {}", arg);
        ready(())
    }

    type MsgSet = Ready<()>;
    fn msgset(&mut self, _: &(), msg_id: u32, text: &str) -> Self::MsgSet {
        debug!("MSGSET {} {}", msg_id, text);
        ready(())
    }

    type MsgWait = Ready<()>;
    fn msgwait(&mut self, _: &(), arg: i32) -> Self::MsgWait {
        debug!("MSGWAIT {}", arg);
        ready(())
    }

    type MsgSignal = Ready<()>;
    fn msgsignal(&mut self, _: &()) -> Self::MsgSignal {
        todo!()
    }

    type MsgSync = Ready<()>;
    fn msgsync(&mut self, _: &(), arg1: i32, arg2: i32) -> Self::MsgSync {
        todo!()
    }

    type MsgClose = Ready<()>;
    fn msgclose(&mut self, _: &(), arg: u8) -> Self::MsgClose {
        debug!("MSGCLOSE {}", arg);
        ready(())
    }

    type Select = Ready<i32>;
    fn select(
        &mut self,
        _: &(),
        choice_set_base: u16,
        choice_index: u16,
        arg4: i32,
        choice_title: &str,
        variants: &[&str],
    ) -> Self::Select {
        debug!(
            "SELECT {} {} {} {}, {:?}",
            choice_set_base, choice_index, arg4, choice_title, variants
        );
        ready(0)
    }

    type Wipe = Ready<()>;
    fn wipe(&mut self, _: &(), arg1: i32, arg2: i32, arg3: i32, params: &[i32; 8]) -> Self::Wipe {
        debug!("WIPE {} {} {} {:?}", arg1, arg2, arg3, params);
        ready(())
    }

    type WipeWait = Ready<()>;
    fn wipewait(&mut self, _: &()) -> Self::WipeWait {
        debug!("WIPEWAIT");
        ready(())
    }

    type BgmPlay = Ready<()>;
    fn bgmplay(&mut self, _: &(), arg1: i32, arg2: i32, arg3: i32, arg4: i32) -> Self::BgmPlay {
        debug!("BGMPLAY {} {} {} {}", arg1, arg2, arg3, arg4);
        ready(())
    }

    type BgmStop = Ready<()>;
    fn bgmstop(&mut self, _: &(), arg: i32) -> Self::BgmStop {
        debug!("BGMSTOP {}", arg);
        ready(())
    }

    type BgmVol = Ready<()>;
    fn bgmvol(&mut self, _: &(), arg1: i32, arg2: i32) -> Self::BgmVol {
        debug!("BGMVOL {} {}", arg1, arg2);
        ready(())
    }

    type BgmWait = Ready<()>;
    fn bgmwait(&mut self, _: &(), arg: i32) -> Self::BgmWait {
        debug!("BGMWAIT {}", arg);
        ready(())
    }

    type BgmSync = Ready<()>;
    fn bgmsync(&mut self, _: &(), arg: i32) -> Self::BgmSync {
        debug!("BGMSYNC {}", arg);
        ready(())
    }

    type SePlay = Ready<()>;
    fn seplay(
        &mut self,
        _: &(),
        arg1: i32,
        arg2: i32,
        arg3: i32,
        arg4: i32,
        arg5: i32,
        arg6: i32,
        arg7: i32,
    ) -> Self::SePlay {
        debug!(
            "SEPLAY {} {} {} {} {} {} {}",
            arg1, arg2, arg3, arg4, arg5, arg6, arg7
        );
        ready(())
    }

    type SeStop = Ready<()>;
    fn sestop(&mut self, _: &(), arg1: i32, arg2: i32) -> Self::SeStop {
        debug!("SESTOP {} {}", arg1, arg2);
        ready(())
    }

    type SeStopAll = Ready<()>;
    fn sestopall(&mut self, _: &(), arg: i32) -> Self::SeStopAll {
        debug!("SESTOPALL {}", arg);
        ready(())
    }

    type SaveInfo = Ready<()>;
    fn saveinfo(&mut self, _: &(), level: i32, info: &str) -> Self::SaveInfo {
        debug!("SAVEINFO {} {}", level, info);
        ready(())
    }

    type AutoSave = Ready<()>;
    fn autosave(&mut self, _: &()) -> Self::AutoSave {
        debug!("AUTOSAVE");
        ready(())
    }

    type LayerInit = Ready<()>;
    fn layerinit(&mut self, _: &(), layer_id: VLayerId) -> Self::LayerInit {
        debug!("LAYERINIT {}", layer_id);
        ready(())
    }

    type LayerLoad = Ready<()>;
    fn layerload(
        &mut self,
        _: &(),
        layer_id: VLayerId,
        layer_type: i32,
        leave_uninitialized: i32,
        params: &[i32; 8],
    ) -> Self::LayerLoad {
        debug!(
            "LAYERLOAD {} {} {} {:?}",
            layer_id, layer_type, leave_uninitialized, params
        );
        ready(())
    }

    type LayerUnload = Ready<()>;
    fn layerunload(&mut self, _: &(), layer_id: VLayerId, delay_time: i32) -> Self::LayerUnload {
        debug!("LAYERUNLOAD {} {}", layer_id, delay_time);
        ready(())
    }

    type LayerCtrl = Ready<()>;
    fn layerctrl(
        &mut self,
        _: &(),
        layer_id: VLayerId,
        property_id: i32,
        params: &[i32; 8],
    ) -> Self::LayerCtrl {
        debug!("LAYERCTRL {} {} {:?}", layer_id, property_id, params);
        ready(())
    }

    type LayerWait = Ready<()>;
    fn layerwait(
        &mut self,
        _: &(),
        layer_id: VLayerId,
        wait_properties: &[i32],
    ) -> Self::LayerWait {
        debug!("LAYERWAIT {} {:?}", layer_id, wait_properties);
        ready(())
    }

    type LayerSwap = Ready<()>;
    fn layerswap(&mut self, _: &(), layer_id1: LayerId, layer_id2: LayerId) -> Self::LayerSwap {
        debug!("LAYERSWAP {} {}", layer_id1, layer_id2);
        ready(())
    }

    type LayerSelect = Ready<()>;
    fn layerselect(
        &mut self,
        _: &(),
        selection_start_id: LayerId,
        selection_end_id: LayerId,
    ) -> Self::LayerSelect {
        debug!("LAYERSELECT {} {}", selection_start_id, selection_end_id);
        ready(())
    }

    type MovieWait = Ready<()>;
    fn moviewait(&mut self, _: &(), arg: i32, arg2: i32) -> Self::MovieWait {
        debug!("MOVIEWAIT {} {}", arg, arg2);
        ready(())
    }

    type TransSet = Ready<()>;
    fn transset(
        &mut self,
        _: &(),
        arg: i32,
        arg2: i32,
        arg3: i32,
        params: &[i32; 8],
    ) -> Self::TransSet {
        debug!("TRANSSET {} {} {} {:?}", arg, arg2, arg3, params);
        ready(())
    }

    type TransWait = Ready<()>;
    fn transwait(&mut self, _: &(), arg: i32) -> Self::TransWait {
        debug!("TRANSWAIT {}", arg);
        ready(())
    }

    type PageBack = Ready<()>;
    fn pageback(&mut self, _: &()) -> Self::PageBack {
        debug!("PAGEBACK");
        ready(())
    }

    type PlaneSelect = Ready<()>;
    fn planeselect(&mut self, _: &(), arg: i32) -> Self::PlaneSelect {
        debug!("PLANESELECT {}", arg);
        ready(())
    }

    type PlaneClear = Ready<()>;
    fn planeclear(&mut self, _: &()) -> Self::PlaneClear {
        debug!("PLANECLEAR");
        ready(())
    }

    type MaskLoad = Ready<()>;
    fn maskload(&mut self, _: &(), arg1: i32, arg2: i32, arg3: i32) -> Self::MaskLoad {
        debug!("MASKLOAD {} {} {}", arg1, arg2, arg3);
        ready(())
    }

    type MaskUnload = Ready<()>;
    fn maskunload(&mut self, _: &()) -> Self::MaskUnload {
        debug!("MASKUNLOAD");
        ready(())
    }

    type Chars = Ready<()>;
    fn chars(&mut self, _: &(), arg1: i32, arg2: i32) -> Self::Chars {
        debug!("CHARS {} {}", arg1, arg2);
        ready(())
    }

    type TipsGet = Ready<()>;
    fn tipsget(&mut self, _: &(), arg: &[i32]) -> Self::TipsGet {
        debug!("TIPSGET {:?}", arg);
        ready(())
    }

    type Quiz = Ready<i32>;
    fn quiz(&mut self, _: &(), arg: i32) -> Self::Quiz {
        debug!("QUIZ {}", arg);
        ready(0)
    }

    type ShowChars = Ready<()>;
    fn showchars(&mut self, _: &()) -> Self::ShowChars {
        debug!("SHOWCHARS");
        ready(())
    }

    type NotifySet = Ready<()>;
    fn notifyset(&mut self, _: &(), arg: i32) -> Self::NotifySet {
        debug!("NOTIFYSET {}", arg);
        ready(())
    }

    type DebugOut = Ready<()>;
    fn debugout(&mut self, _: &(), format: &str, args: &[i32]) -> Self::DebugOut {
        todo!()
    }
}
