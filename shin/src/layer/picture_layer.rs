use crate::asset::picture::GpuPicture;
use crate::layer::{Layer, LayerProperties};
use crate::render::Renderable;
use crate::render::{GpuCommonResources, SpriteVertexBuffer};
use crate::update::{Updatable, UpdateContext};
use cgmath::{Matrix4, Vector3, Vector4};

pub struct PictureLayer {
    picture: GpuPicture,
    vertices: SpriteVertexBuffer,

    props: LayerProperties,
}

impl PictureLayer {
    pub fn new(resources: &GpuCommonResources, picture: GpuPicture) -> Self {
        let origin_translate = -Vector3::new(picture.origin_x as f32, picture.origin_y as f32, 0.0);

        let color = Vector4::new(1.0, 1.0, 1.0, 1.0);

        let vertices = SpriteVertexBuffer::new(
            resources,
            (
                origin_translate.x,
                origin_translate.y,
                origin_translate.x + picture.width as f32,
                origin_translate.y + picture.height as f32,
            ),
            color,
        );

        Self {
            picture,
            vertices,
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
        resources.draw_sprite(
            render_pass,
            self.vertices.vertex_source(),
            &self.picture.bind_group,
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
