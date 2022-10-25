use crate::vm::{VmImpl, VmState};
use shin_core::vm::command::{ready, AdvListener, ExitResult, LayerId, Ready, VLayerId};
use std::cell::RefCell;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, warn};

pub struct ListenerCtx<'a> {
    pub commands: Arc<RefCell<bevy::ecs::system::Commands<'a, 'a>>>,
    pub time: bevy::time::Time,
}

impl AdvListener for VmImpl {
    type Context<'a> = ListenerCtx<'a>;

    type Exit = Ready<ExitResult>;
    fn exit(&mut self, _ctx: &ListenerCtx, arg1: u8, arg2: i32) -> Self::Exit {
        todo!()
    }

    type SGet = Ready<i32>;
    fn sget(&mut self, _ctx: &ListenerCtx, slot_number: i32) -> Self::SGet {
        warn!("TODO: SGET {}", slot_number);
        ready(0)
    }

    type SSet = Ready<()>;
    fn sset(&mut self, _ctx: &ListenerCtx, slot_number: i32, value: i32) -> Self::SSet {
        warn!("TODO: SSET {} {}", slot_number, value);
        ready(())
    }

    type Wait = super::commands::Wait;
    fn wait(&mut self, _ctx: &ListenerCtx, wait_kind: u8, wait_amount: i32) -> Self::Wait {
        assert_eq!(wait_kind, 0);
        debug!("WAIT {}", wait_amount);
        super::commands::Wait::new(Duration::from_millis(wait_amount.try_into().unwrap()))
    }

    type MsgInit = Ready<()>;
    fn msginit(&mut self, _ctx: &ListenerCtx, arg: i32) -> Self::MsgInit {
        warn!("TODO: MSGINIT {}", arg);
        ready(())
    }

    type MsgSet = Ready<()>;
    fn msgset(&mut self, _ctx: &ListenerCtx, msg_id: u32, text: &str) -> Self::MsgSet {
        todo!()
    }

    type MsgWait = Ready<()>;
    fn msgwait(&mut self, _ctx: &ListenerCtx, arg: i32) -> Self::MsgWait {
        todo!()
    }

    type MsgSignal = Ready<()>;
    fn msgsignal(&mut self, _ctx: &ListenerCtx) -> Self::MsgSignal {
        todo!()
    }

    type MsgSync = Ready<()>;
    fn msgsync(&mut self, _ctx: &ListenerCtx, arg1: i32, arg2: i32) -> Self::MsgSync {
        todo!()
    }

    type MsgClose = Ready<()>;
    fn msgclose(&mut self, _ctx: &ListenerCtx, arg: u8) -> Self::MsgClose {
        warn!("TODO: MSGCLOSE {}", arg);
        ready(())
    }

    type Select = Ready<i32>;
    fn select(
        &mut self,
        _ctx: &ListenerCtx,
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
        &mut self,
        _ctx: &ListenerCtx,
        arg1: i32,
        arg2: i32,
        arg3: i32,
        params: &[i32; 8],
    ) -> Self::Wipe {
        warn!("TODO: WIPE {} {} {} {:?}", arg1, arg2, arg3, params);
        ready(())
    }

    type WipeWait = Ready<()>;
    fn wipewait(&mut self, _ctx: &ListenerCtx) -> Self::WipeWait {
        todo!()
    }

    type BgmPlay = Ready<()>;
    fn bgmplay(
        &mut self,
        _ctx: &ListenerCtx,
        arg1: i32,
        arg2: i32,
        arg3: i32,
        arg4: i32,
    ) -> Self::BgmPlay {
        todo!()
    }

    type BgmStop = Ready<()>;
    fn bgmstop(&mut self, _ctx: &ListenerCtx, arg: i32) -> Self::BgmStop {
        todo!()
    }

    type BgmVol = Ready<()>;
    fn bgmvol(&mut self, _ctx: &ListenerCtx, arg1: i32, arg2: i32) -> Self::BgmVol {
        todo!()
    }

    type BgmWait = Ready<()>;
    fn bgmwait(&mut self, _ctx: &ListenerCtx, arg: i32) -> Self::BgmWait {
        todo!()
    }

    type BgmSync = Ready<()>;
    fn bgmsync(&mut self, _ctx: &ListenerCtx, arg: i32) -> Self::BgmSync {
        todo!()
    }

    type SePlay = Ready<()>;
    fn seplay(
        &mut self,
        _ctx: &ListenerCtx,
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
    fn sestop(&mut self, _ctx: &ListenerCtx, arg1: i32, arg2: i32) -> Self::SeStop {
        todo!()
    }

    type SeStopAll = Ready<()>;
    fn sestopall(&mut self, _ctx: &ListenerCtx, arg: i32) -> Self::SeStopAll {
        todo!()
    }

    type SaveInfo = Ready<()>;
    fn saveinfo(&mut self, _ctx: &ListenerCtx, level: i32, info: &str) -> Self::SaveInfo {
        self.state.save_info.set_save_info(level, info);
        ready(())
    }

    type AutoSave = Ready<()>;
    fn autosave(&mut self, _ctx: &ListenerCtx) -> Self::AutoSave {
        warn!("TODO: AUTOSAVE");
        ready(())
    }

    type LayerInit = Ready<()>;
    fn layerinit(&mut self, _ctx: &ListenerCtx, layer_id: VLayerId) -> Self::LayerInit {
        if let Some(layer_id) = layer_id.to_layer_id() {
            todo!("LayerInit {:?}", layer_id)
        } else {
            warn!("TODO: LAYERINIT: handle virtual layers");
        }
        ready(())
    }

    type LayerLoad = Ready<()>;
    fn layerload(
        &mut self,
        _ctx: &ListenerCtx,
        layer_id: VLayerId,
        layer_type: i32,
        leave_uninitialized: i32,
        params: &[i32; 8],
    ) -> Self::LayerLoad {
        if let Some(layer_id) = layer_id.to_layer_id() {
            if let Some(layerbank_id) = self
                .state
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
        &mut self,
        _ctx: &ListenerCtx,
        layer_id: VLayerId,
        delay_time: i32,
    ) -> Self::LayerUnload {
        if let Some(layer_id) = layer_id.to_layer_id() {
            if let Some(layerbank_id) = self.state.layerbank_info.get_layerbank_id(layer_id.into())
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
        &mut self,
        _ctx: &ListenerCtx,
        layer_id: VLayerId,
        property_id: i32,
        params: &[i32; 8],
    ) -> Self::LayerCtrl {
        todo!()
    }

    type LayerWait = Ready<()>;
    fn layerwait(
        &mut self,
        _ctx: &ListenerCtx,
        layer_id: VLayerId,
        wait_properties: &[i32],
    ) -> Self::LayerWait {
        todo!()
    }

    type LayerSwap = Ready<()>;
    fn layerswap(
        &mut self,
        _ctx: &ListenerCtx,
        layer_id1: LayerId,
        layer_id2: LayerId,
    ) -> Self::LayerSwap {
        todo!()
    }

    type LayerSelect = Ready<()>;
    fn layerselect(
        &mut self,
        _ctx: &ListenerCtx,
        selection_start_id: LayerId,
        selection_end_id: LayerId,
    ) -> Self::LayerSelect {
        todo!()
    }

    type MovieWait = Ready<()>;
    fn moviewait(&mut self, _ctx: &ListenerCtx, arg: i32, arg2: i32) -> Self::MovieWait {
        todo!()
    }

    type TransSet = Ready<()>;
    fn transset(
        &mut self,
        _ctx: &ListenerCtx,
        arg: i32,
        arg2: i32,
        arg3: i32,
        params: &[i32; 8],
    ) -> Self::TransSet {
        todo!()
    }

    type TransWait = Ready<()>;
    fn transwait(&mut self, _ctx: &ListenerCtx, arg: i32) -> Self::TransWait {
        todo!()
    }

    type PageBack = Ready<()>;
    fn pageback(&mut self, _ctx: &ListenerCtx) -> Self::PageBack {
        warn!("TODO: PAGEBACK");
        ready(())
    }

    type PlaneSelect = Ready<()>;
    fn planeselect(&mut self, _ctx: &ListenerCtx, arg: i32) -> Self::PlaneSelect {
        todo!()
    }

    type PlaneClear = Ready<()>;
    fn planeclear(&mut self, _ctx: &ListenerCtx) -> Self::PlaneClear {
        todo!()
    }

    type MaskLoad = Ready<()>;
    fn maskload(&mut self, _ctx: &ListenerCtx, arg1: i32, arg2: i32, arg3: i32) -> Self::MaskLoad {
        todo!()
    }

    type MaskUnload = Ready<()>;
    fn maskunload(&mut self, _ctx: &ListenerCtx) -> Self::MaskUnload {
        todo!()
    }

    type Chars = Ready<()>;
    fn chars(&mut self, _ctx: &ListenerCtx, arg1: i32, arg2: i32) -> Self::Chars {
        todo!()
    }

    type TipsGet = Ready<()>;
    fn tipsget(&mut self, _ctx: &ListenerCtx, arg: &[i32]) -> Self::TipsGet {
        todo!()
    }

    type Quiz = Ready<i32>;
    fn quiz(&mut self, _ctx: &ListenerCtx, arg: i32) -> Self::Quiz {
        todo!()
    }

    type ShowChars = Ready<()>;
    fn showchars(&mut self, _ctx: &ListenerCtx) -> Self::ShowChars {
        todo!()
    }

    type NotifySet = Ready<()>;
    fn notifyset(&mut self, _ctx: &ListenerCtx, arg: i32) -> Self::NotifySet {
        todo!()
    }

    type DebugOut = Ready<()>;
    fn debugout(&mut self, _ctx: &ListenerCtx, format: &str, args: &[i32]) -> Self::DebugOut {
        todo!()
    }
}
