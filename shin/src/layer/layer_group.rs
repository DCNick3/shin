use bevy_utils::hashbrown::HashMap;
use itertools::Itertools;
use shin_core::vm::command::layer::LayerId;

use crate::layer::{LayerProperties, UserLayer};
use crate::render::GpuCommonResources;
use crate::render::{RenderTarget, Renderable};
use crate::update::{Updatable, UpdateContext};

pub struct LayerGroup {
    layers: HashMap<LayerId, UserLayer>,
    render_target: RenderTarget,
    properties: LayerProperties,
}

impl Updatable for LayerGroup {
    fn update(&mut self, context: &UpdateContext) {
        self.properties.update(context);
        for layer in self.layers.values_mut() {
            layer.update(context);
        }
    }
}

impl Renderable for LayerGroup {
    fn render<'enc>(
        &'enc self,
        resources: &'enc GpuCommonResources,
        render_pass: &mut wgpu::RenderPass<'enc>,
    ) {
        {
            let mut encoder = resources.start_encoder();
            let mut render_pass = self
                .render_target
                .begin_render_pass(&mut encoder, Some("LayerGroup::render"));

            let ordered_layers = self
                .layers
                .iter()
                .sorted_by_key(|&(id, l)| {
                    // TODO: use render order property
                    *id
                })
                .map(|(_, l)| l)
                .collect::<Vec<_>>();
            for l in ordered_layers {
                l.render(resources, &mut render_pass);
            }
        }

        // TODO use layer pseudo-pipeline
        // resources.draw_sprite(
        todo!("Render the render target to the screen");
    }

    fn resize(&mut self, resources: &GpuCommonResources, size: (u32, u32)) {
        self.render_target.resize(resources, size);
    }
}
