use std::sync::Arc;

use drop_bomb::DropBomb;

pub struct RenderCloneCtx<'d> {
    pub device: &'d Arc<wgpu::Device>,
    pub encoder: wgpu::CommandEncoder,
    bomb: DropBomb,
}

impl<'d> RenderCloneCtx<'d> {
    pub fn new(device: &'d Arc<wgpu::Device>) -> Self {
        Self {
            device,
            encoder: device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("RenderCloneCtx"),
            }),
            bomb: DropBomb::new("RenderCloneCtx not finished"),
        }
    }

    pub fn finish(self, queue: &wgpu::Queue) {
        let Self {
            device: _,
            encoder,
            mut bomb,
        } = self;

        queue.submit([encoder.finish()]);
        bomb.defuse();
    }
}

pub use shin_derive::RenderClone;
/// Like [`Clone`], but allows access to wgpu device and encoder to perform the copies
///
/// Used to make "deep" copies of GPU resources like buffers and textures
pub trait RenderClone {
    fn render_clone(&self, ctx: &mut RenderCloneCtx) -> Self;
}

// implementations for containers/wrappers
impl<T: RenderClone> RenderClone for Vec<T> {
    fn render_clone(&self, ctx: &mut RenderCloneCtx) -> Self {
        self.iter().map(|v| v.render_clone(ctx)).collect()
    }
}

impl<T: RenderClone> RenderClone for Option<T> {
    fn render_clone(&self, ctx: &mut RenderCloneCtx) -> Self {
        self.as_ref().map(|v| v.render_clone(ctx))
    }
}

impl<T: RenderClone> RenderClone for Box<T> {
    fn render_clone(&self, ctx: &mut RenderCloneCtx) -> Self {
        Box::new(self.as_ref().render_clone(ctx))
    }
}

impl<T> RenderClone for Arc<T> {
    fn render_clone(&self, _: &mut RenderCloneCtx) -> Self {
        Arc::clone(self)
    }
}

// implementations for wgpu types
impl RenderClone for wgpu::Buffer {
    fn render_clone(&self, ctx: &mut RenderCloneCtx) -> Self {
        let cloned_buffer = ctx.device.create_buffer(&wgpu::BufferDescriptor {
            // AFAIK, there's no API to get label out of a wgpu object
            // we might want to not implement [`RenderClone`] directly on wgpu objects because of this...
            label: None,
            size: self.size(),
            usage: self.usage(),
            mapped_at_creation: false,
        });

        ctx.encoder
            .copy_buffer_to_buffer(self, 0, &cloned_buffer, 0, self.size());

        cloned_buffer
    }
}
