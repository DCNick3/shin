use super::prelude::*;
use shin_core::time::{Easing, Tween};

impl StartableCommand for command::runtime::LAYERCTRL {
    fn apply_state(&self, state: &mut VmState) {
        let [target_value, _time, _flags, _, _, _, _, _] = self.params;

        state.layers.for_each_vlayer_mut(self.layer_id, |layer| {
            layer
                .properties
                .set_property(self.property_id, target_value);
        });
    }

    fn start(
        self,
        _context: &UpdateContext,
        _scenario: &Arc<Scenario>,
        vm_state: &VmState,
        adv_state: &mut AdvState,
    ) -> CommandStartResult {
        let [target_value, time, flags, _, _, _, _, _] = self.params;
        let duration = Ticks::from_i32(time);

        if flags != 0 {
            warn!("LAYERCTRL: flags are not supported yet (flags={})", flags);
        }

        let mut changed = false;
        adv_state.for_each_vlayer_mut(vm_state, self.layer_id, |mut layer| {
            if layer.properties().get_property(self.property_id) != target_value as f32 {
                changed = true;
            }

            layer.properties_mut().set_property(
                self.property_id,
                target_value as f32,
                Tween {
                    duration,
                    easing: Easing::Linear,
                },
            );
        });

        if !self.property_id.is_implemented() && changed {
            warn!(
                "LAYERCTRL: property is not implemented yet (property_id={:?}, value={})",
                self.property_id, target_value
            );
        }

        self.token.finish().into()
    }
}
