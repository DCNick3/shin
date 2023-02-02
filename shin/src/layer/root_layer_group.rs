use crate::layer::{Layer, LayerGroup, LayerProperties, MessageLayer};
use crate::render::{GpuCommonResources, RenderTarget, Renderable, SpriteVertexBuffer};
use crate::update::{Updatable, UpdateContext};
use glam::Mat4;

type ScreenLayer = LayerGroup;

pub struct RootLayerGroup {
    screen_layer: ScreenLayer,
    message_layer: MessageLayer,
    render_target: RenderTarget,
    properties: LayerProperties,
    vertices: SpriteVertexBuffer,
}

impl RootLayerGroup {
    pub fn new(
        resources: &GpuCommonResources,
        screen_layer: ScreenLayer,
        message_layer: MessageLayer,
    ) -> Self {
        let render_target = RenderTarget::new(
            resources,
            resources.current_render_buffer_size(),
            Some("LayerGroup RenderTarget"),
        );
        let vertices = SpriteVertexBuffer::new_fullscreen(resources);

        Self {
            screen_layer,
            message_layer,
            render_target,
            properties: LayerProperties::new(),
            vertices,
        }
    }

    #[allow(unused)]
    pub fn screen_layer(&self) -> &ScreenLayer {
        &self.screen_layer
    }

    pub fn screen_layer_mut(&mut self) -> &mut ScreenLayer {
        &mut self.screen_layer
    }

    pub fn message_layer(&self) -> &MessageLayer {
        &self.message_layer
    }

    pub fn message_layer_mut(&mut self) -> &mut MessageLayer {
        &mut self.message_layer
    }
}

impl Updatable for RootLayerGroup {
    fn update(&mut self, context: &UpdateContext) {
        self.properties.update(context);
        self.screen_layer.update(context);
        self.message_layer.update(context);
    }
}

impl Renderable for RootLayerGroup {
    fn render<'enc>(
        &'enc self,
        resources: &'enc GpuCommonResources,
        render_pass: &mut wgpu::RenderPass<'enc>,
        transform: Mat4,
    ) {
        {
            let mut encoder = resources.start_encoder();
            let mut render_pass = self
                .render_target
                .begin_render_pass(&mut encoder, Some("RootLayerGroup RenderPass"));

            render_pass.push_debug_group("ScreenLayer");
            self.screen_layer.render(
                resources,
                &mut render_pass,
                self.properties.compute_transform(transform),
            );
            render_pass.pop_debug_group();

            render_pass.push_debug_group("MessageLayer");
            self.message_layer.render(
                resources,
                &mut render_pass,
                self.properties.compute_transform(transform),
            );
            render_pass.pop_debug_group();
        }

        render_pass.push_debug_group("RootLayerGroup Render");
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

impl Layer for RootLayerGroup {
    fn properties(&self) -> &LayerProperties {
        &self.properties
    }

    fn properties_mut(&mut self) -> &mut LayerProperties {
        &mut self.properties
    }
}
