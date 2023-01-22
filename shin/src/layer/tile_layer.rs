use crate::layer::{Layer, LayerProperties};
use crate::render::Renderable;
use crate::render::{GpuCommonResources, PosVertexBuffer};
use crate::update::{Updatable, UpdateContext};
use cgmath::{Matrix4, Vector4};
use std::fmt::Debug;
use std::sync::Arc;

pub struct TileLayer {
    vertex_color: Vector4<f32>,
    vertex_buffer: Arc<PosVertexBuffer>,

    props: LayerProperties,
}

impl TileLayer {
    #[allow(clippy::identity_op)]
    pub fn new(
        resources: &GpuCommonResources,
        tile_color: i32,
        offset_x: i32,
        offset_y: i32,
        width: i32,
        height: i32,
    ) -> Self {
        // tile_color stores the value as 0xARGB — 4 bits for one channel
        let alpha = ((tile_color & 0xf000) >> 12) as u8;
        let red = ((tile_color & 0x0f00) >> 8) as u8;
        let green = ((tile_color & 0x00f0) >> 4) as u8;
        let blue = ((tile_color & 0x000f) >> 0) as u8;

        let vertex_color = Vector4 {
            x: (red as f32) / (0xf as f32),
            y: (green as f32) / (0xf as f32),
            z: (blue as f32) / (0xf as f32),
            w: (alpha as f32) / (0xf as f32),
        };

        let rect = (
            offset_x as f32,
            offset_y as f32,
            (offset_x + width) as f32,
            (offset_y + height) as f32,
        );

        let vertex_buffer = PosVertexBuffer::new(resources, rect);

        Self {
            vertex_color,
            vertex_buffer: Arc::new(vertex_buffer),

            props: LayerProperties::new(),
        }
    }
}

impl Renderable for TileLayer {
    fn render<'enc>(
        &'enc self,
        resources: &'enc GpuCommonResources,
        render_pass: &mut wgpu::RenderPass<'enc>,
        transform: Matrix4<f32>,
    ) {
        resources.draw_fill(
            render_pass,
            self.vertex_buffer.vertex_source(),
            self.props.compute_transform(transform),
            self.vertex_color,
        );
    }

    fn resize(&mut self, _resources: &GpuCommonResources) {
        // no internal buffers to resize
    }
}

impl Updatable for TileLayer {
    fn update(&mut self, ctx: &UpdateContext) {
        self.props.update(ctx);
    }
}

impl Debug for TileLayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let color = self.vertex_color.map(|v| (v * 255.0) as u8);
        let color = format!(
            "#{:02x}{:02x}{:02x}{:02x}",
            color.x, color.y, color.z, color.w
        );

        f.debug_tuple("TileLayer").field(&color).finish()
    }
}

impl Layer for TileLayer {
    fn properties(&self) -> &LayerProperties {
        &self.props
    }

    fn properties_mut(&mut self) -> &mut LayerProperties {
        &mut self.props
    }
}
