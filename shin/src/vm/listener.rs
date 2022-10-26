use crate::vm::{VmImpl, VmState};
use shin_core::vm::command::{ready, AdvListener, ExitResult, LayerId, Ready, VLayerId};
use std::cell::RefCell;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, warn};

pub struct ListenerCtx<'vm, 'c>
where
// 's: 'vm,
// 'w: 'vm,
// 'vm: 's,
// 'vm: 'w,
{
    pub commands: &'c mut bevy::ecs::system::Commands<'c, 'c>,
    pub time: bevy::time::Time,
    pub vm_state: &'vm mut VmState,
}

impl AdvListener for VmImpl {
    type Context<'a> = ListenerCtx<'a, 'a>;

    type Exit = Ready<ExitResult>;
    fn exit(_ctx: &mut ListenerCtx, arg1: u8, arg2: i32) -> Self::Exit {
        todo!()
    }

    type SGet = Ready<i32>;
    fn sget(_ctx: &mut ListenerCtx, slot_number: i32) -> Self::SGet {
        warn!("TODO: SGET {}", slot_number);
        ready(0)
    }

    type SSet = Ready<()>;
    fn sset(_ctx: &mut ListenerCtx, slot_number: i32, value: i32) -> Self::SSet {
        warn!("TODO: SSET {} {}", slot_number, value);
        ready(())
    }

    type Wait = super::commands::Wait;
    fn wait(_ctx: &mut ListenerCtx, wait_kind: u8, wait_amount: i32) -> Self::Wait {
        assert_eq!(wait_kind, 0);
        debug!("WAIT {}", wait_amount);
        super::commands::Wait::new(Duration::from_millis(wait_amount.try_into().unwrap()))
    }

    type MsgInit = Ready<()>;
    fn msginit(_ctx: &mut ListenerCtx, arg: i32) -> Self::MsgInit {
        warn!("TODO: MSGINIT {}", arg);
        ready(())
    }

    type MsgSet = Ready<()>;
    fn msgset(_ctx: &mut ListenerCtx, msg_id: u32, text: &str) -> Self::MsgSet {
        todo!()
    }

    type MsgWait = Ready<()>;
    fn msgwait(_ctx: &mut ListenerCtx, arg: i32) -> Self::MsgWait {
        todo!()
    }

    type MsgSignal = Ready<()>;
    fn msgsignal(_ctx: &mut ListenerCtx) -> Self::MsgSignal {
        todo!()
    }

    type MsgSync = Ready<()>;
    fn msgsync(_ctx: &mut ListenerCtx, arg1: i32, arg2: i32) -> Self::MsgSync {
        todo!()
    }

    type MsgClose = Ready<()>;
    fn msgclose(_ctx: &mut ListenerCtx, arg: u8) -> Self::MsgClose {
        warn!("TODO: MSGCLOSE {}", arg);
        ready(())
    }

    type Select = Ready<i32>;
    fn select(
        _ctx: &mut ListenerCtx,
        choice_set_base: u16,
        choice_index: u16,
        arg4: i32,
        choice_title: &str,
        variants: &[&str],
    ) -> Self::Select {
        todo!()
    }

    type Wipe = Ready<()>;
    fn wipe(
        _ctx: &mut ListenerCtx,
        arg1: i32,
        arg2: i32,
        arg3: i32,
        params: &[i32; 8],
    ) -> Self::Wipe {
        warn!("TODO: WIPE {} {} {} {:?}", arg1, arg2, arg3, params);
        ready(())
    }

    type WipeWait = Ready<()>;
    fn wipewait(_ctx: &mut ListenerCtx) -> Self::WipeWait {
        todo!()
    }

    type BgmPlay = Ready<()>;
    fn bgmplay(
        _ctx: &mut ListenerCtx,
        arg1: i32,
        arg2: i32,
        arg3: i32,
        arg4: i32,
    ) -> Self::BgmPlay {
        todo!()
    }

    type BgmStop = Ready<()>;
    fn bgmstop(_ctx: &mut ListenerCtx, arg: i32) -> Self::BgmStop {
        todo!()
    }

    type BgmVol = Ready<()>;
    fn bgmvol(_ctx: &mut ListenerCtx, arg1: i32, arg2: i32) -> Self::BgmVol {
        todo!()
    }

    type BgmWait = Ready<()>;
    fn bgmwait(_ctx: &mut ListenerCtx, arg: i32) -> Self::BgmWait {
        todo!()
    }

    type BgmSync = Ready<()>;
    fn bgmsync(_ctx: &mut ListenerCtx, arg: i32) -> Self::BgmSync {
        todo!()
    }

    type SePlay = Ready<()>;
    fn seplay(
        _ctx: &mut ListenerCtx,
        arg1: i32,
        arg2: i32,
        arg3: i32,
        arg4: i32,
        arg5: i32,
        arg6: i32,
        arg7: i32,
    ) -> Self::SePlay {
        todo!()
    }

    type SeStop = Ready<()>;
    fn sestop(_ctx: &mut ListenerCtx, arg1: i32, arg2: i32) -> Self::SeStop {
        todo!()
    }

    type SeStopAll = Ready<()>;
    fn sestopall(_ctx: &mut ListenerCtx, arg: i32) -> Self::SeStopAll {
        todo!()
    }

    type SaveInfo = Ready<()>;
    fn saveinfo(ctx: &mut ListenerCtx, level: i32, info: &str) -> Self::SaveInfo {
        ctx.vm_state.save_info.set_save_info(level, info);
        ready(())
    }

    type AutoSave = Ready<()>;
    fn autosave(_ctx: &mut ListenerCtx) -> Self::AutoSave {
        warn!("TODO: AUTOSAVE");
        ready(())
    }

    type LayerInit = Ready<()>;
    fn layerinit(_ctx: &mut ListenerCtx, layer_id: VLayerId) -> Self::LayerInit {
        if let Some(layer_id) = layer_id.to_layer_id() {
            todo!("LayerInit {:?}", layer_id)
        } else {
            warn!("TODO: LAYERINIT: handle virtual layers");
        }
        ready(())
    }

    type LayerLoad = Ready<()>;
    fn layerload(
        ctx: &mut ListenerCtx,
        layer_id: VLayerId,
        layer_type: i32,
        leave_uninitialized: i32,
        params: &[i32; 8],
    ) -> Self::LayerLoad {
        if let Some(layer_id) = layer_id.to_layer_id() {
            if let Some(layerbank_id) = ctx
                .vm_state
                .layerbank_info
                .get_or_allocate_layerbank_id(layer_id.into())
            {
                todo!(
                    "LayerLoad {:?} {} {} {:?}",
                    layer_id,
                    layer_type,
                    leave_uninitialized,
                    params
                )
            } else {
                panic!("Out of layerbanks");
            }
        } else {
            todo!("LayerLoad of virtual layers")
        }
    }

    type LayerUnload = Ready<()>;
    fn layerunload(
        ctx: &mut ListenerCtx,
        layer_id: VLayerId,
        delay_time: i32,
    ) -> Self::LayerUnload {
        if let Some(layer_id) = layer_id.to_layer_id() {
            if let Some(layerbank_id) = ctx
                .vm_state
                .layerbank_info
                .get_layerbank_id(layer_id.into())
            {
                todo!("LayerUnload {:?}, bank = {:?}", layer_id, layerbank_id)
            }
        } else {
            todo!("TODO: LAYERINIT: handle virtual layers");
        }
        ready(())
    }

    type LayerCtrl = Ready<()>;
    fn layerctrl(
        _ctx: &mut ListenerCtx,
        layer_id: VLayerId,
        property_id: i32,
        params: &[i32; 8],
    ) -> Self::LayerCtrl {
        todo!()
    }

    type LayerWait = Ready<()>;
    fn layerwait(
        _ctx: &mut ListenerCtx,
        layer_id: VLayerId,
        wait_properties: &[i32],
    ) -> Self::LayerWait {
        todo!()
    }

    type LayerSwap = Ready<()>;
    fn layerswap(
        _ctx: &mut ListenerCtx,
        layer_id1: LayerId,
        layer_id2: LayerId,
    ) -> Self::LayerSwap {
        todo!()
    }

    type LayerSelect = Ready<()>;
    fn layerselect(
        _ctx: &mut ListenerCtx,
        selection_start_id: LayerId,
        selection_end_id: LayerId,
    ) -> Self::LayerSelect {
        todo!()
    }

    type MovieWait = Ready<()>;
    fn moviewait(_ctx: &mut ListenerCtx, arg: i32, arg2: i32) -> Self::MovieWait {
        todo!()
    }

    type TransSet = Ready<()>;
    fn transset(
        _ctx: &mut ListenerCtx,
        arg: i32,
        arg2: i32,
        arg3: i32,
        params: &[i32; 8],
    ) -> Self::TransSet {
        todo!()
    }

    type TransWait = Ready<()>;
    fn transwait(_ctx: &mut ListenerCtx, arg: i32) -> Self::TransWait {
        todo!()
    }

    type PageBack = Ready<()>;
    fn pageback(_ctx: &mut ListenerCtx) -> Self::PageBack {
        warn!("TODO: PAGEBACK");
        ready(())
    }

    type PlaneSelect = Ready<()>;
    fn planeselect(_ctx: &mut ListenerCtx, arg: i32) -> Self::PlaneSelect {
        todo!()
    }

    type PlaneClear = Ready<()>;
    fn planeclear(_ctx: &mut ListenerCtx) -> Self::PlaneClear {
        todo!()
    }

    type MaskLoad = Ready<()>;
    fn maskload(_ctx: &mut ListenerCtx, arg1: i32, arg2: i32, arg3: i32) -> Self::MaskLoad {
        todo!()
    }

    type MaskUnload = Ready<()>;
    fn maskunload(_ctx: &mut ListenerCtx) -> Self::MaskUnload {
        todo!()
    }

    type Chars = Ready<()>;
    fn chars(_ctx: &mut ListenerCtx, arg1: i32, arg2: i32) -> Self::Chars {
        todo!()
    }

    type TipsGet = Ready<()>;
    fn tipsget(_ctx: &mut ListenerCtx, arg: &[i32]) -> Self::TipsGet {
        todo!()
    }

    type Quiz = Ready<i32>;
    fn quiz(_ctx: &mut ListenerCtx, arg: i32) -> Self::Quiz {
        todo!()
    }

    type ShowChars = Ready<()>;
    fn showchars(_ctx: &mut ListenerCtx) -> Self::ShowChars {
        todo!()
    }

    type NotifySet = Ready<()>;
    fn notifyset(_ctx: &mut ListenerCtx, arg: i32) -> Self::NotifySet {
        todo!()
    }

    type DebugOut = Ready<()>;
    fn debugout(_ctx: &mut ListenerCtx, format: &str, args: &[i32]) -> Self::DebugOut {
        todo!()
    }
}
