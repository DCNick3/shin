use std::{
    marker::PhantomData,
    sync::{Arc, Mutex, Weak},
};

use anyhow::{Context, Result};
use bevy_utils::HashMap;
use glam::{vec2, Vec2};
use indexmap::IndexMap;
use once_cell::sync::OnceCell;
use shin_core::format::{
    bustup::{
        BustupBlockId, BustupBlockPromise, BustupBlockPromiseToken, BustupBuilder, BustupId,
        BustupSkeleton,
    },
    picture::PicBlock,
};

use crate::asset::{
    picture::{GpuPictureBlock, GpuTextureBuilderContext},
    system::{
        cache::{AssetCache, CacheLoadHandle, CacheLoaderHandle, CacheLookupResult},
        Asset, AssetDataAccessor, AssetLoadContext,
    },
};

type BlockCache = AssetCache<BustupBlockId, GpuPictureBlock>;
type BlockCacheLoadHandle = CacheLoadHandle<GpuPictureBlock>;
type BlockCacheLoaderHandle = CacheLoaderHandle<BustupBlockId, GpuPictureBlock>;

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct CharacterId(i32);

impl CharacterId {
    pub fn new(id: i32) -> Self {
        Self(id)
    }
}

struct GpuBustupBuilder<'b> {
    phantom: PhantomData<&'b ()>,
}

struct GpuBustupBuilderArgs<'a> {
    context: GpuTextureBuilderContext<'a>,
    cache: &'a BlockCache,
    label: String,
    expression: String,
    character_id: CharacterId,
    disable_animations: bool,
}

struct GpuBustupSkeleton<'a> {
    pub origin_x: i16,
    pub origin_y: i16,
    pub effective_width: u16,
    pub effective_height: u16,
    pub bustup_id: BustupId,

    pub base_blocks: Vec<GpuBustupBlockPromise<'a>>,
    pub face1: Option<GpuBustupBlockPromise<'a>>,
    pub face2: Option<GpuBustupBlockPromise<'a>>,
    pub mouth_blocks: Vec<GpuBustupBlockPromise<'a>>,
    pub eye_blocks: Vec<GpuBustupBlockPromise<'a>>,
}

enum GpuBustupBlockPromise<'a> {
    Loaded(Arc<GpuPictureBlock>),
    Loading(BlockCacheLoadHandle),
    LoadRequired(BlockCacheLoaderHandle, BustupBlockPromise<'a>),
}

pub struct Bustup {
    pub origin_x: i16,
    pub origin_y: i16,
    pub effective_width: u16,
    pub effective_height: u16,
    pub bustup_id: BustupId,

    pub base_blocks: Vec<Arc<GpuPictureBlock>>,
    pub face1: Option<Arc<GpuPictureBlock>>,
    pub face2: Option<Arc<GpuPictureBlock>>,
    pub mouth_blocks: Vec<Arc<GpuPictureBlock>>,
    pub eye_blocks: Vec<Arc<GpuPictureBlock>>,
}

impl<'a> GpuBustupBlockPromise<'a> {
    pub fn new(cache: &BlockCache, promise: BustupBlockPromise<'a>) -> Self {
        match cache.lookup(promise.get_id()) {
            CacheLookupResult::Loaded(block) => Self::Loaded(block),
            CacheLookupResult::Loading(load_handle) => Self::Loading(load_handle),
            CacheLookupResult::LoadRequired(loader_handle) => {
                Self::LoadRequired(loader_handle, promise)
            }
        }
    }

    pub fn materialize(
        self,
        cache: &BlockCache,
        token: &BustupBlockPromiseToken<Arc<GpuPictureBlock>>,
    ) -> Arc<GpuPictureBlock> {
        match self {
            GpuBustupBlockPromise::Loaded(block) => block,
            GpuBustupBlockPromise::Loading(load_handle) => load_handle.wait(),
            GpuBustupBlockPromise::LoadRequired(loader_handle, promise) => {
                let block = promise.get(token).clone();
                cache.finish_load(loader_handle, block.clone());

                block
            }
        }
    }
}

impl<'b> BustupBuilder for GpuBustupBuilder<'b> {
    type Args = GpuBustupBuilderArgs<'b>;
    type Skeleton<'a> = GpuBustupSkeleton<'a>;
    type BlockType = Arc<GpuPictureBlock>;
    type Output = Bustup;

    fn new<'a>(args: &Self::Args, skeleton: BustupSkeleton<'a>) -> Self::Skeleton<'a> {
        let BustupSkeleton {
            origin_x,
            origin_y,
            effective_width,
            effective_height,
            bustup_id,
            base_blocks,
            mut expressions,
        } = skeleton;

        let lower_block =
            |block: BustupBlockPromise<'a>| GpuBustupBlockPromise::new(args.cache, block);

        let base_blocks = base_blocks.into_iter().map(lower_block).collect::<Vec<_>>();

        let mut face1 = None;
        let mut face2 = None;
        let mut mouth_blocks = Vec::new();
        let mut eye_blocks = Vec::new();

        if let Some(expression) = expressions.swap_remove(&args.expression) {
            face1 = expression.face1.map(lower_block);
            face2 = expression.face2.map(lower_block);
            mouth_blocks = expression
                .mouth_blocks
                .into_iter()
                .map(lower_block)
                .collect();
            eye_blocks = expression.eye_blocks.into_iter().map(lower_block).collect();
        }

        GpuBustupSkeleton {
            origin_x,
            origin_y,
            effective_width,
            effective_height,
            bustup_id,
            base_blocks,
            face1,
            face2,
            mouth_blocks,
            eye_blocks,
        }
    }

    fn new_block(args: &Self::Args, data_offset: u32, block: PicBlock) -> Result<Self::BlockType> {
        Ok(Arc::new(GpuPictureBlock::new(
            args.context,
            block,
            &format!("{}/{}", args.label, data_offset),
        )))
    }

    fn build(
        args: &Self::Args,
        skeleton: Self::Skeleton<'_>,
        token: BustupBlockPromiseToken<Self::BlockType>,
    ) -> Result<Self::Output> {
        let GpuBustupSkeleton {
            origin_x,
            origin_y,
            effective_width,
            effective_height,
            bustup_id,
            base_blocks,
            face1,
            face2,
            mouth_blocks,
            eye_blocks,
        } = skeleton;

        let lower_block = |block: GpuBustupBlockPromise| block.materialize(args.cache, &token);

        let base_blocks = base_blocks.into_iter().map(lower_block).collect::<Vec<_>>();
        let face1 = face1.map(|block| lower_block(block));
        let face2 = face2.map(|block| lower_block(block));
        let mouth_blocks = mouth_blocks
            .into_iter()
            .map(lower_block)
            .collect::<Vec<_>>();
        let eye_blocks = eye_blocks.into_iter().map(lower_block).collect::<Vec<_>>();

        Ok(Bustup {
            origin_x,
            origin_y,
            effective_width,
            effective_height,
            bustup_id,
            base_blocks,
            face1,
            face2,
            mouth_blocks,
            eye_blocks,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct BustupArgs {
    pub expression: String,
    pub character_id: CharacterId,
    pub disable_animations: bool,
}

impl Asset for Bustup {
    type Args = BustupArgs;

    async fn load(
        context: &AssetLoadContext,
        args: BustupArgs,
        name: &str,
        data: AssetDataAccessor,
    ) -> Result<Self> {
        let data = data.read_all().await;

        let info = shin_core::format::bustup::read_bustup_header(&data)?;

        let args = GpuBustupBuilderArgs {
            context: GpuTextureBuilderContext {
                wgpu_device: &context.wgpu_device,
                wgpu_queue: &context.wgpu_queue,
            },
            cache: &context.bustup_cache,
            label: format!("{}[{}]", name, args.expression),
            expression: args.expression,
            character_id: args.character_id,
            disable_animations: args.disable_animations,
        };

        shin_core::format::bustup::read_bustup::<GpuBustupBuilder>(&data, args)
    }
}
