use shin_render::shaders::types::RenderCloneCtx;

use super::prelude::*;

impl StartableCommand for command::runtime::PAGEBACK {
    type StateInfo = bool;
    fn apply_state(&self, state: &mut VmState) -> bool {
        if state.layers.is_page_back_started {
            false
        } else {
            state.layers.is_page_back_started = true;

            true
        }
    }

    fn start(
        self,
        context: &UpdateContext,
        _scenario: &Arc<Scenario>,
        _vm_state: &VmState,
        needs_page_back: bool,
        adv_state: &mut AdvState,
    ) -> CommandStartResult {
        if !needs_page_back {
            return self.token.finish().into();
        }

        let mut clone_ctx = RenderCloneCtx::new(context.pre_render.device);

        adv_state.create_back_layer_group_if_needed(&mut clone_ctx);
        adv_state.screen_layer_mut().pageback(&mut clone_ctx, false);

        clone_ctx.finish(context.pre_render.queue);

        self.token.finish().into()
    }
}
