use std::sync::{Arc, Mutex, Weak};

use indexmap::{map::Entry, IndexMap};
use once_cell::sync::OnceCell;
use shin_core::format::bustup::BustupBlockId;

use crate::asset::picture::GpuPictureBlock;

enum BlockCacheEntry {
    Loaded(Weak<GpuPictureBlock>),
    Loading(BlockCacheLoadHandle),
}

pub struct BlockCache {
    cache: Mutex<IndexMap<BustupBlockId, BlockCacheEntry>>,
}

impl BlockCache {
    pub fn new() -> Self {
        BlockCache {
            cache: Mutex::new(IndexMap::new()),
        }
    }

    pub fn get(&self, block_id: BustupBlockId) -> BlockCacheResult {
        let mut cache = self.cache.lock().unwrap();

        let make_load_handle_pair = || {
            let handle = Arc::new(OnceCell::new());
            let load_handle = BlockCacheLoadHandle {
                inner: handle.clone(),
            };
            let loader_handle = BlockCacheLoaderHandle {
                block_id,
                inner: handle,
            };
            (load_handle, loader_handle)
        };

        match cache.entry(block_id) {
            Entry::Occupied(mut o) => {
                match o.get() {
                    BlockCacheEntry::Loaded(loaded) => {
                        if let Some(picture) = loaded.upgrade() {
                            BlockCacheResult::Loaded(picture)
                        } else {
                            // it was dropped already
                            let (load_handle, loader_handle) = make_load_handle_pair();
                            *o.get_mut() = BlockCacheEntry::Loading(load_handle);
                            BlockCacheResult::LoadRequired(loader_handle)
                        }
                    }
                    BlockCacheEntry::Loading(loading) => BlockCacheResult::Loading(loading.clone()),
                }
            }
            Entry::Vacant(e) => {
                let (load_handle, loader_handle) = make_load_handle_pair();
                e.insert(BlockCacheEntry::Loading(load_handle));
                BlockCacheResult::LoadRequired(loader_handle)
            }
        }
    }

    pub fn finish_load(&self, handle: BlockCacheLoaderHandle, picture: Arc<GpuPictureBlock>) {
        let mut cache = self.cache.lock().unwrap();

        let entry = cache.get_mut(&handle.block_id).unwrap();
        let BlockCacheEntry::Loading(_) = &entry else {
            panic!("BlockCacheEntry::Loading expected");
        };
        *entry = BlockCacheEntry::Loaded(Arc::downgrade(&picture));
        let Ok(()) = handle.inner.set(picture) else {
            panic!("this picture block was loaded by someone else");
        };
    }
}

#[must_use]
pub struct BlockCacheLoaderHandle {
    block_id: BustupBlockId,
    inner: Arc<OnceCell<Arc<GpuPictureBlock>>>,
}

#[derive(Clone)]
pub struct BlockCacheLoadHandle {
    inner: Arc<OnceCell<Arc<GpuPictureBlock>>>,
}

impl BlockCacheLoadHandle {
    pub fn wait(self) -> Arc<GpuPictureBlock> {
        self.inner.wait().clone()
    }
}

pub enum BlockCacheResult {
    Loaded(Arc<GpuPictureBlock>),
    Loading(BlockCacheLoadHandle),
    LoadRequired(BlockCacheLoaderHandle),
}
