use crate::asset::bustup::Bustup;
use crate::asset::gpu_picture::GpuImage;
use crate::layer::{Layer, LayerProperties};
use crate::render::GpuCommonResources;
use crate::render::Renderable;
use crate::update::{Updatable, UpdateContext};
use cgmath::Matrix4;
use std::sync::Arc;

pub struct BustupLayer {
    bustup: Arc<Bustup>,
    emotion: String,

    props: LayerProperties,
}

impl BustupLayer {
    pub fn new(resources: &GpuCommonResources, bustup: Arc<Bustup>, emotion: &str) -> Self {
        // ensure the picture is loaded to gpu
        bustup.base_gpu_image(resources);

        Self {
            bustup,
            emotion: emotion.to_owned(),
            props: LayerProperties::new(),
        }
    }
}

impl Renderable for BustupLayer {
    fn render<'enc>(
        &'enc self,
        resources: &'enc GpuCommonResources,
        render_pass: &mut wgpu::RenderPass<'enc>,
        transform: Matrix4<f32>,
    ) {
        let mut draw_image = |image: &'enc GpuImage| {
            // TODO: there should be a generic function to render a layer (from texture?)
            resources.draw_sprite(
                render_pass,
                image.vertex_buffer.vertex_source(),
                &image.bind_group,
                self.props.compute_transform(transform),
            );
        };

        let base_gpu_image = self.bustup.base_gpu_image(resources);
        draw_image(base_gpu_image);

        let emotion_gpu_image = self.bustup.face_gpu_image(resources, &self.emotion);
        draw_image(emotion_gpu_image);

        let mouth_gpu_image = self.bustup.mouth_gpu_image(resources, &self.emotion, 0.0);
        draw_image(mouth_gpu_image);
    }

    fn resize(&mut self, _resources: &GpuCommonResources) {
        // no internal buffers to resize
    }
}

impl Updatable for BustupLayer {
    fn update(&mut self, ctx: &UpdateContext) {
        self.props.update(ctx);
    }
}

impl Layer for BustupLayer {
    fn properties(&self) -> &LayerProperties {
        &self.props
    }

    fn properties_mut(&mut self) -> &mut LayerProperties {
        &mut self.props
    }
}
