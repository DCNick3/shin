use crate::Plugin;
use bevy::app::App;
use bevy::asset::{AssetLoader, LoadContext, LoadedAsset};
use bevy::math::Vec2;
use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::utils::BoxedFuture;
use shin_core::format::picture::SimpleMergedPicture;
use tracing::trace;

#[derive(Debug, Copy, Clone, TypeUuid)]
#[uuid = "6f1a853e-249e-4373-90bd-8c571a330884"]
pub struct PictureOrigin(pub Vec2);

// #[derive(Debug, Clone, TypeUuid)]
// #[uuid = "800d5b93-78cb-41c2-baa7-b40ec94b70b6"]
// pub struct Picture {
//     chunks: Image,
//     effective_width: usize,
//     effective_height: usize,
//     origin_x: usize,
//     origin_y: usize,
// }

// pub struct GpuPictureChunk {
//     position: Vec2,
//     texture: Texture,
//     texture_view: TextureView,
//     sampler: Sampler,
// }
//
// pub struct GpuPicture {
//     chunks: Vec<GpuPictureChunk>,
//     effective_width: usize,
//     effective_height: usize,
//     origin_x: usize,
//     origin_y: usize,
// }

pub struct PictureLoader;

impl AssetLoader for PictureLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, anyhow::Result<(), bevy::asset::Error>> {
        Box::pin(async move {
            trace!("Loading PIC {}", load_context.path().display());
            // even though the original game splits the picture into chunks and keeps it so during the rendering,
            // this is not really efficient for modern GPUs
            // so we merge the chunks into a single texture
            // as a bonus, we can re-use the bevy's sprite engine (and a pipeline) instead of writing our own
            let pic = shin_core::format::picture::read_picture::<SimpleMergedPicture>(bytes, ())?;
            let origin = Vec2::new(pic.origin_x as f32, pic.origin_y as f32);
            let pic = Image::new(
                Extent3d {
                    width: pic.image.width() as u32,
                    height: pic.image.height() as u32,
                    depth_or_array_layers: 1,
                },
                TextureDimension::D2,
                pic.image.into_raw(),
                TextureFormat::Rgba8UnormSrgb,
            );

            load_context.set_default_asset(LoadedAsset::new(pic));
            // I think this is kinda a shitty way to pass this information...
            // I need to find a better way to pass origin information to the renderer
            load_context.set_labeled_asset("origin", LoadedAsset::new(PictureOrigin(origin)));

            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["pic"]
    }
}

// impl RenderAsset for Picture {
//     type ExtractedAsset = Picture;
//     type PreparedAsset = GpuPicture;
//     type Param = (
//         SRes<RenderDevice>,
//         SRes<RenderQueue>,
//         SRes<DefaultImageSampler>,
//     );
//
//     fn extract_asset(&self) -> Self::ExtractedAsset {
//         self.clone()
//     }
//
//     fn prepare_asset(
//         extracted_asset: Self::ExtractedAsset,
//         (render_device, render_queue, default_sampler): &mut SystemParamItem<Self::Param>,
//     ) -> Result<Self::PreparedAsset, PrepareAssetError<Self::ExtractedAsset>> {
//         trace!("Preparing PIC asset for rendering");
//
//         let mut chunks = Vec::new();
//         for (position, chunk) in extracted_asset.chunks {
//             let format = TextureFormat::bevy_default();
//
//             let size = Extent3d {
//                 width: chunk.data.width(),
//                 height: chunk.data.height(),
//                 depth_or_array_layers: 1,
//             };
//             let texture = render_device.create_texture(&TextureDescriptor {
//                 size,
//                 format,
//                 dimension: TextureDimension::D2,
//                 label: Some(&format!("PIC_CHUNK({}, {})", position.x, position.y)),
//                 mip_level_count: 1,
//                 sample_count: 1,
//                 usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
//             });
//
//             let format_size = format.pixel_size();
//             render_queue.write_texture(
//                 ImageCopyTexture {
//                     texture: &texture,
//                     mip_level: 0,
//                     origin: Origin3d::ZERO,
//                     aspect: TextureAspect::All,
//                 },
//                 &chunk.data,
//                 ImageDataLayout {
//                     offset: 0,
//                     bytes_per_row: Some(
//                         std::num::NonZeroU32::new(chunk.data.width() * format_size as u32).unwrap(),
//                     ),
//                     rows_per_image: None,
//                 },
//                 size,
//             );
//
//             let texture_view = texture.create_view(&Default::default());
//             let sampler = (***default_sampler).clone();
//
//             chunks.push(GpuPictureChunk {
//                 position,
//                 texture,
//                 texture_view,
//                 sampler,
//             });
//         }
//
//         Ok(GpuPicture {
//             chunks,
//             effective_width: extracted_asset.effective_width,
//             effective_height: extracted_asset.effective_height,
//             origin_x: extracted_asset.origin_x,
//             origin_y: extracted_asset.origin_y,
//         })
//     }
// }
//
// // TODO: this is a GREAT simplification
// #[derive(Component, Clone)]
// pub struct PictureLayer {
//     pub picture: Handle<Picture>,
// }
//
// #[derive(Bundle, Clone)]
// pub struct PictureLayerBundle {
//     pub picture_layer: PictureLayer,
//     pub transform: Transform,
//     pub global_transform: GlobalTransform,
//     /// User indication of whether an entity is visible
//     pub visibility: Visibility,
//     /// Algorithmically-computed indication of whether an entity is visible and should be extracted for rendering
//     pub computed_visibility: ComputedVisibility,
// }
//
// #[derive(Component, Clone, Copy)]
// pub struct ExtractedPictureLayer {
//     pub entity: Entity,
//     pub transform: GlobalTransform,
//     pub color: Color,
//     /// Handle to the `Picture` of this sprite
//     /// PERF: storing a `HandleId` instead of `Handle<Picture>` enables some optimizations (`ExtractedPictureLayer` becomes `Copy` and doesn't need to be dropped)
//     pub picture_handle_id: HandleId,
// }
//
// #[derive(Default)]
// pub struct ExtractedPictureLayers {
//     pub layers: Vec<ExtractedPictureLayer>,
// }
//
// fn extract_picture_layers(
//     mut extracted_picture_layers: ResMut<ExtractedPictureLayers>,
//     picture_layers_query: Extract<
//         Query<(Entity, &ComputedVisibility, &PictureLayer, &GlobalTransform)>,
//     >,
// ) {
//     extracted_picture_layers.layers.clear();
//     for (entity, visibility, picture_layer, transform) in picture_layers_query.iter() {
//         if !visibility.is_visible() {
//             continue;
//         }
//         // PERF: we don't check in this function that the `Picture` asset is ready, since it should be in most cases and hashing the handle is expensive
//         extracted_picture_layers
//             .layers
//             .alloc()
//             .init(ExtractedPictureLayer {
//                 entity,
//                 color: Color::WHITE,
//                 transform: *transform,
//                 picture_handle_id: picture_layer.picture.id,
//             });
//     }
// }
//
// fn queue_picture_layers(
//     mut commands: Commands,
//     // TODO: this needs A LOT of figuring out
//     // mut view_entities: Local<FixedBitSet>,
//     // draw_functions: Res<DrawFunctions<Transparent2d>>,
//     // render_device: Res<RenderDevice>,
//     // render_queue: Res<RenderQueue>,
//     // mut sprite_meta: ResMut<SpriteMeta>,
//     // view_uniforms: Res<ViewUniforms>,
//     // sprite_pipeline: Res<SpritePipeline>,
//     // mut pipelines: ResMut<SpecializedRenderPipelines<SpritePipeline>>,
//     // mut pipeline_cache: ResMut<PipelineCache>,
//     // mut image_bind_groups: ResMut<ImageBindGroups>,
//     // gpu_images: Res<RenderAssets<Image>>,
//     // msaa: Res<Msaa>,
//     mut extracted_picture_layers: ResMut<ExtractedPictureLayers>,
//     // mut views: Query<(&VisibleEntities, &mut RenderPhase<Transparent2d>)>,
//     // events: Res<SpriteAssetEvents>,
// ) {
// }

pub struct PicturePlugin;
impl Plugin for PicturePlugin {
    fn build(&self, app: &mut App) {
        app.add_asset::<PictureOrigin>()
            .add_asset_loader(PictureLoader);

        // app.add_plugin(RenderAssetPlugin::<Picture>::with_prepare_asset_label(
        //     PrepareAssetLabel::PreAssetPrepare, // TODO: this is what image loader uses. is it correct? why does it matter at all?
        // ));

        // if let Ok(render_app) = app.get_sub_app_mut(RenderApp) {
        //     render_app
        //         .init_resource::<ExtractedPictureLayers>()
        //         // .init_resource::<SpriteAssetEvents>()
        //         // .add_render_command::<Transparent2d, DrawSprite>()
        //         .add_system_to_stage(
        //             RenderStage::Extract,
        //             extract_picture_layers, //.label(SpriteSystem::ExtractSprites),
        //         )
        //         // .add_system_to_stage(RenderStage::Extract, render::extract_sprite_events)
        //         .add_system_to_stage(RenderStage::Queue, queue_picture_layers);
        // };
    }
}
