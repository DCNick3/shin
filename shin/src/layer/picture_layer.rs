use crate::asset::picture::Picture;
use crate::layer::{Layer, LayerProperties};
use crate::render::GpuCommonResources;
use crate::render::Renderable;
use crate::update::{Updatable, UpdateContext};
use cgmath::Matrix4;
use std::sync::Arc;

pub struct PictureLayer {
    picture: Arc<Picture>,

    props: LayerProperties,
}

impl PictureLayer {
    pub fn new(resources: &GpuCommonResources, picture: Arc<Picture>) -> Self {
        // ensure the picture is loaded to gpu
        picture.gpu_image(resources);

        Self {
            picture,
            props: LayerProperties::new(),
        }
    }
}

impl Renderable for PictureLayer {
    fn render<'enc>(
        &'enc self,
        resources: &'enc GpuCommonResources,
        render_pass: &mut wgpu::RenderPass<'enc>,
        transform: Matrix4<f32>,
    ) {
        // TODO: there should be a generic function to render a layer (from texture?)
        let gpu_picture = self.picture.gpu_image(resources);
        resources.draw_sprite(
            render_pass,
            gpu_picture.vertex_buffer.vertex_source(),
            &gpu_picture.bind_group,
            self.props.compute_transform(transform),
        );
    }

    fn resize(&mut self, _resources: &GpuCommonResources) {
        // no internal buffers to resize
    }
}

impl Updatable for PictureLayer {
    fn update(&mut self, ctx: &UpdateContext) {
        self.props.update(ctx);
    }
}

impl Layer for PictureLayer {
    fn properties(&self) -> &LayerProperties {
        &self.props
    }

    fn properties_mut(&mut self) -> &mut LayerProperties {
        &mut self.props
    }
}
