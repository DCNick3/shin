use std::{fmt::Debug, sync::Arc};

use glam::{Mat4, vec3};
use parking_lot::Mutex;
use shin_audio::AudioManager;
use shin_core::{
    format::scenario::info::{MovieTransparencyMode, MovieVolumeSource},
    primitives::update::{FrameId, UpdateTracker},
    vm::command::types::Volume,
};
use shin_render::{
    LayerBlendType, PassKind, RenderProgramWithArguments, RenderRequestBuilder,
    render_pass::RenderPass,
    shaders::types::{
        RenderClone, RenderCloneCtx,
        texture::{DepthStencilTarget, TextureTarget},
    },
};
use shin_video::VideoPlayerHandle;
use tracing::warn;

use crate::{
    asset::{movie::Movie, picture::Picture},
    layer::{
        DrawableLayer, Layer, NewDrawableLayer, NewDrawableLayerWrapper,
        new_drawable_layer::{
            NewDrawableLayerFastForward, NewDrawableLayerNeedsSeparatePass, NewDrawableLayerState,
        },
        properties::LayerProperties,
        render_params::{DrawableClipMode, DrawableClipParams, DrawableParams, TransformParams},
        user::PictureLayer,
    },
    render::PreRenderContext,
    update::{AdvUpdatable, AdvUpdateContext, Updatable, UpdateContext},
};

struct Shared {
    update_tracker: UpdateTracker,
    volume_source: MovieVolumeSource,
    local_volume: Volume,
}

impl Shared {
    fn new(volume_source: MovieVolumeSource, local_volume: Volume) -> Self {
        Self {
            update_tracker: UpdateTracker::new(),
            volume_source,
            local_volume,
        }
    }

    fn update(&mut self, frame_id: FrameId, handle: &VideoPlayerHandle) {
        if !self.update_tracker.update(frame_id) {
            return;
        }

        // TODO: restart the video if it's looping and it's finished

        // TODO: need settings handle to read volume
        let settings_volume = Volume(1.0);
        let final_volume = self.local_volume * settings_volume;

        handle.set_volume(final_volume);
    }
}

#[derive(RenderClone)]
pub struct MovieLayerImpl {
    movie_label: String,
    video_player: Arc<VideoPlayerHandle>,
    still_picture: Option<Arc<Picture>>,
    shared: Arc<Mutex<Shared>>,
    transparency: MovieTransparencyMode,
    repeat: bool,
}

pub struct MovieArgs {
    pub volume_source: MovieVolumeSource,
    pub transparency: MovieTransparencyMode,
    pub local_volume: Volume,
    pub repeat: bool,
}

struct MovieBackendArgs {
    pub is_bgm: bool,
    pub transparency: MovieTransparencyMode,
    pub repeat: bool,
}

pub type MovieLayer = NewDrawableLayerWrapper<MovieLayerImpl>;

impl MovieLayer {
    pub fn new(
        device: &wgpu::Device,
        audio_manager: &AudioManager,
        movie: Arc<Movie>,
        MovieArgs {
            volume_source,
            transparency,
            local_volume,
            repeat,
        }: MovieArgs,
        // NB: the original engine uses PictureLayer here
        // we have decoupled asset type from the layer type though
        still_picture: Option<Arc<Picture>>,
    ) -> Self {
        if transparency != MovieTransparencyMode::Opaque {
            warn!("Movie transparency mode {:?} not supported", transparency);
        }
        if repeat {
            warn!("Movie repeat mode not supported");
        }

        NewDrawableLayerWrapper::from_inner(MovieLayerImpl {
            movie_label: movie.label().to_string(),
            video_player: Arc::new(
                movie
                    .play(device, audio_manager)
                    .expect("Failed to play movie"),
            ),
            still_picture,
            shared: Arc::new(Mutex::new(Shared::new(volume_source, local_volume))),
            transparency,
            repeat,
        })
    }

    pub fn is_finished(&self) -> bool {
        self.inner_ref().video_player.is_finished()
    }
}

impl NewDrawableLayerNeedsSeparatePass for MovieLayerImpl {
    fn needs_separate_pass(&self, props: &LayerProperties) -> bool {
        // NB: this if is not present in the original implementation
        // instead, it always forces an additional render pass to convert YUV to RGB
        // we try to be better and optimistically render it in one pass

        // In actuality, I think we can do even better, by adding support of some of these effects to the movie shader
        // but that's probably not very useful (?)
        if props.get_clip_mode() != DrawableClipMode::None
            || props.is_fragment_shader_nontrivial()
            || props.is_blending_nontrivial()
        {
            return true;
        }

        if !self.video_player.is_finished() {
            return false;
        }

        false
        // TODO: add support for still pictures & indirect rendering needed for them (?)
        // self.still_picture.is_some()
    }
}

impl NewDrawableLayer for MovieLayerImpl {
    fn render_drawable_indirect(
        &mut self,
        context: &mut PreRenderContext,
        props: &LayerProperties,
        target: TextureTarget,
        depth_stencil: DepthStencilTarget,
        transform: &TransformParams,
    ) -> PassKind {
        todo!()
    }

    fn render_drawable_direct(
        &self,
        pass: &mut RenderPass,
        transform: &TransformParams,
        drawable: &DrawableParams,
        _clip: &DrawableClipParams,
        stencil_ref: u8,
        pass_kind: PassKind,
    ) {
        // TODO: I think some of these conditions conflict with the needs_separate_pass logic
        // (aka they will never be true)
        let target_pass = if self.transparency != MovieTransparencyMode::Opaque
            || drawable.blend_type != LayerBlendType::Type1
            || drawable.color_multiplier.a < 1.0
        {
            PassKind::Transparent
        } else {
            PassKind::Opaque
        };

        if pass_kind != target_pass {
            return;
        }

        let Some(frame) = self.video_player.get_frame() else {
            return;
        };

        // NB: the original engine uses a generic layer shader here, because it does YUV->RGB conversion in a separate pass
        // we try to do better, so we do the conversion in the main pass

        let transform =
            transform.compute_final_transform() * Mat4::from_translation(vec3(-960.0, -540.0, 0.0));

        frame.render(
            pass,
            RenderRequestBuilder::new().depth_stencil_shorthand(stencil_ref, false, false),
            transform,
        );
    }
}

impl NewDrawableLayerFastForward for MovieLayerImpl {
    fn fast_forward(&mut self) {
        // TODO: fast forward the movie
    }
}

impl AdvUpdatable for MovieLayerImpl {
    fn update(&mut self, ctx: &AdvUpdateContext) {
        self.video_player
            .update(ctx.frame_id, ctx.delta_ticks, ctx.queue);
    }
}

impl Debug for MovieLayerImpl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("MovieLayer")
            .field(&self.movie_label)
            .finish()
    }
}
