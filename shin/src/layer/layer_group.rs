use bevy_utils::hashbrown::HashMap;
use cgmath::Matrix4;
use itertools::Itertools;
use shin_core::vm::command::layer::{LayerId, VLayerId};

use crate::layer::{Layer, LayerProperties, UserLayer};
use crate::render::{GpuCommonResources, SpriteVertexBuffer};
use crate::render::{RenderTarget, Renderable};
use crate::update::{Updatable, UpdateContext};

pub struct LayerGroup {
    layers: HashMap<LayerId, UserLayer>,
    render_target: RenderTarget,
    properties: LayerProperties,
    vertices: SpriteVertexBuffer,
}

impl LayerGroup {
    pub fn new(resources: &GpuCommonResources) -> Self {
        let render_target = RenderTarget::new(
            resources,
            resources.current_render_buffer_size(),
            Some("LayerGroup RenderTarget"),
        );
        let vertices = SpriteVertexBuffer::new_fullscreen(resources);

        Self {
            layers: HashMap::new(),
            render_target,
            properties: LayerProperties::new(),
            vertices,
        }
    }

    pub fn add_layer(&mut self, id: LayerId, layer: UserLayer) {
        self.layers.insert(id, layer);
    }

    pub fn remove_layer(&mut self, id: LayerId) {
        self.layers.remove(&id);
    }

    pub fn get_layer(&self, id: LayerId) -> Option<&UserLayer> {
        self.layers.get(&id)
    }

    pub fn get_layer_mut(&mut self, id: LayerId) -> Option<&mut UserLayer> {
        self.layers.get_mut(&id)
    }

    pub fn get_vlayer(&self, id: VLayerId) -> impl Iterator<Item = &UserLayer> {
        std::iter::once(todo!())
        // self.layers
        //     .iter()
        //     .filter(move |(_, layer)| layer.properties().vlayer_id() == id)
    }

    pub fn get_vlayer_mut(&mut self, id: VLayerId) -> impl Iterator<Item = &mut UserLayer> {
        std::iter::once(todo!())
        // self.layers
        //     .iter_mut()
        //     .filter(move |(_, layer)| layer.properties().vlayer_id() == id)
    }
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
        transform: Matrix4<f32>,
    ) {
        {
            let mut encoder = resources.start_encoder();
            let mut render_pass = self
                .render_target
                .begin_render_pass(&mut encoder, Some("LayerGroup RenderPass"));

            let ordered_layers = self
                .layers
                .iter()
                .sorted_by_key(|&(id, _)| {
                    // TODO: use render order property
                    *id
                })
                .collect::<Vec<_>>();
            for (id, l) in ordered_layers {
                render_pass.push_debug_group(&format!("Layer {:?}", id));
                l.render(
                    resources,
                    &mut render_pass,
                    self.properties.compute_transform(transform),
                );
                render_pass.pop_debug_group();
            }
        }

        render_pass.push_debug_group("LayerGroup Render");
        // TODO use layer pseudo-pipeline
        resources.draw_sprite(
            render_pass,
            self.vertices.vertex_source(),
            self.render_target.bind_group(),
            transform,
        );
        render_pass.pop_debug_group();
    }

    fn resize(&mut self, resources: &GpuCommonResources) {
        self.render_target
            .resize(resources, resources.current_render_buffer_size());
    }
}

impl Layer for LayerGroup {
    fn properties(&self) -> &LayerProperties {
        &self.properties
    }

    fn properties_mut(&mut self) -> &mut LayerProperties {
        &mut self.properties
    }
}
