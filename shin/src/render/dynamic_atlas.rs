use crate::render::{GpuCommonResources, TextureBindGroup};
use bevy_utils::{Entry, HashMap};
use std::num::NonZeroU32;

pub trait ImageProvider {
    const IMAGE_FORMAT: wgpu::TextureFormat;
    const MIPMAP_LEVELS: u32;
    type Id: Copy + Eq + std::hash::Hash;

    fn get_image(&self, id: Self::Id) -> (Vec<Vec<u8>>, (u32, u32));
}

struct AtlasAllocation {
    allocation: etagere::Allocation,
    ref_count: usize,
}

impl AtlasAllocation {
    pub fn as_atlas_image(&self) -> AtlasImage {
        let pos = self.allocation.rectangle.min;
        let size = self.allocation.rectangle.size();

        AtlasImage {
            x: pos.x.try_into().unwrap(),
            y: pos.y.try_into().unwrap(),
            width: size.width.try_into().unwrap(),
            height: size.height.try_into().unwrap(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AtlasImage {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

/// Dynamic texture atlas, (for now) used for text rendering.
pub struct DynamicAtlas<P: ImageProvider> {
    image_provider: P,

    // TODO: support multiple atlas pages
    texture: wgpu::Texture,
    texture_view: wgpu::TextureView,
    texture_bind_group: TextureBindGroup,
    texture_size: (u32, u32),

    allocator: etagere::BucketedAtlasAllocator,

    /// These are the images that are currently in the atlas and cannot be evicted.
    active_allocations: HashMap<P::Id, AtlasAllocation>,
    /// These are images still in the atlas, but can be evicted.
    eviction_ready: HashMap<P::Id, etagere::Allocation>,
}

impl<P: ImageProvider> DynamicAtlas<P> {
    pub fn new(
        resources: &GpuCommonResources,
        image_provider: P,
        texture_size: (u32, u32),
        label: Option<&str>,
    ) -> Self {
        let label = label
            .map(|s| format!("{} DynamicAtlas", s))
            .unwrap_or_else(|| "DynamicAtlas".to_string());

        let texture = resources.device.create_texture(&wgpu::TextureDescriptor {
            label: Some(&format!("{} Texture", label)),
            size: wgpu::Extent3d {
                width: texture_size.0,
                height: texture_size.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: P::IMAGE_FORMAT,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        });
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        // TODO: make sampler configurable
        let texture_sampler = resources.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some(&format!("{} Sampler", label)),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let texture_bind_group = TextureBindGroup::new(
            resources,
            &texture_view,
            &texture_sampler,
            Some(&format!("{} TextureBindGroup", label)),
        );

        let allocator = etagere::BucketedAtlasAllocator::with_options(
            etagere::Size::new(
                texture_size.0.try_into().unwrap(),
                texture_size.1.try_into().unwrap(),
            ),
            &etagere::AllocatorOptions {
                alignment: etagere::Size::new(8, 8), // TODO: make this configurable
                vertical_shelves: false,
                num_columns: 1,
            },
        );

        Self {
            image_provider,
            texture,
            texture_view,
            texture_bind_group,
            texture_size,
            allocator,
            active_allocations: HashMap::default(),
            eviction_ready: HashMap::default(),
        }
    }

    pub fn texture_bind_group(&self) -> &TextureBindGroup {
        &self.texture_bind_group
    }

    pub fn texture_size(&self) -> (u32, u32) {
        self.texture_size
    }

    /// Gets an image from the atlas, or adds it if it's not already there.
    /// Increases the ref count of the image.
    pub fn get_image(&mut self, resources: &GpuCommonResources, id: P::Id) -> Option<AtlasImage> {
        let entry = self.active_allocations.entry(id);

        let allocation: &AtlasAllocation = match entry {
            Entry::Occupied(entry) => {
                let allocation = entry.into_mut();
                allocation.ref_count += 1;
                allocation
            }
            Entry::Vacant(entry) => {
                if let Some(allocation) = self.eviction_ready.remove(&id) {
                    // The image is already allocated, but not in use, so we can restore it
                    entry.insert(AtlasAllocation {
                        allocation,
                        ref_count: 1,
                    })
                } else {
                    // The image is not in atlas. We need to actually upload it to GPU
                    let (mip_data, (width, height)) = self.image_provider.get_image(id);

                    assert_eq!(mip_data.len(), P::MIPMAP_LEVELS as usize);

                    // First, find a place to put it
                    let format = P::IMAGE_FORMAT.describe();
                    // no compressed textures support for now
                    assert_eq!(format.block_dimensions, (1, 1));

                    let allocation = if let Some(alloc) = self.allocator.allocate(
                        etagere::Size::new(width.try_into().unwrap(), height.try_into().unwrap()),
                    ) {
                        alloc
                    } else {
                        // seems like we are out of space
                        // we can evict unused images to make space
                        todo!("evict some images");
                    };

                    let x: u32 = allocation.rectangle.min.x.try_into().unwrap();
                    let y: u32 = allocation.rectangle.min.y.try_into().unwrap();

                    for (mip_level, data) in (0..P::MIPMAP_LEVELS).zip(mip_data) {
                        let mip_scale = 1 << mip_level;

                        assert_eq!(
                            data.len(),
                            (width * height / mip_scale / mip_scale) as usize
                                * format.block_size as usize
                        );

                        // Upload the image to the atlas
                        let texture_copy_view = wgpu::ImageCopyTexture {
                            texture: &self.texture,
                            mip_level,
                            origin: wgpu::Origin3d {
                                x: x / mip_scale,
                                y: y / mip_scale,
                                z: 0,
                            },
                            aspect: Default::default(),
                        };

                        resources.queue.write_texture(
                            texture_copy_view,
                            &data,
                            wgpu::ImageDataLayout {
                                offset: 0,
                                bytes_per_row: Some(
                                    NonZeroU32::new(width * format.block_size as u32).unwrap(),
                                ),
                                rows_per_image: Some(NonZeroU32::new(height / mip_scale).unwrap()),
                            },
                            wgpu::Extent3d {
                                width: width / mip_scale,
                                height: height / mip_scale,
                                depth_or_array_layers: 1,
                            },
                        );
                    }

                    entry.insert(AtlasAllocation {
                        allocation,
                        ref_count: 1,
                    })
                }
            }
        };

        Some(allocation.as_atlas_image())
    }

    pub fn peek_image(&mut self, id: P::Id) -> Option<AtlasImage> {
        Some(self.active_allocations.get(&id)?.as_atlas_image())
    }

    pub fn free_image(&mut self, id: P::Id) {
        let allocation = self
            .active_allocations
            .get_mut(&id)
            .expect("Attempt to free an image not in atlas");

        allocation.ref_count -= 1;

        if allocation.ref_count == 0 {
            self.eviction_ready.insert(id, allocation.allocation);
            self.active_allocations.remove(&id);
        }
    }
}
