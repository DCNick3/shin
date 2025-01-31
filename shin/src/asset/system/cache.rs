use std::{
    hash::Hash,
    sync::{Arc, Mutex, Weak},
};

use drop_bomb::DropBomb;
use indexmap::{map::Entry, IndexMap};
use once_cell::sync::OnceCell;

enum CacheEntry<V> {
    // NOTE: we should add support for some policies other than "if it's not used right now, yagni"
    // this is especially important for font glyphs (which the game uses a kind of generational LRU for)
    Loaded(Weak<V>),
    Loading(CacheLoadHandle<V>),
}

pub struct AssetCache<K, V> {
    cache: Mutex<IndexMap<K, CacheEntry<V>>>,
}

impl<K: Hash + Eq + Clone, V> AssetCache<K, V> {
    pub fn new() -> Self {
        AssetCache {
            cache: Mutex::new(IndexMap::new()),
        }
    }

    pub fn lookup(&self, key: K) -> CacheLookupResult<K, V> {
        let mut cache = self.cache.lock().unwrap();

        let key_clone = key.clone();
        let make_load_handle_pair = || {
            let handle = Arc::new(OnceCell::new());
            let load_handle = CacheLoadHandle {
                inner: handle.clone(),
            };
            let loader_handle = CacheLoaderHandle {
                key: key_clone,
                bomb: DropBomb::new("CacheLoaderHandle dropped without providing a loaded asset"),
                inner: handle,
            };
            (load_handle, loader_handle)
        };

        match cache.entry(key) {
            Entry::Occupied(mut o) => {
                match o.get() {
                    CacheEntry::Loaded(loaded) => {
                        if let Some(picture) = loaded.upgrade() {
                            CacheLookupResult::Loaded(picture)
                        } else {
                            // it was dropped already
                            let (load_handle, loader_handle) = make_load_handle_pair();
                            *o.get_mut() = CacheEntry::Loading(load_handle);
                            CacheLookupResult::LoadRequired(loader_handle)
                        }
                    }
                    CacheEntry::Loading(loading) => CacheLookupResult::Loading(loading.clone()),
                }
            }
            Entry::Vacant(e) => {
                let (load_handle, loader_handle) = make_load_handle_pair();
                e.insert(CacheEntry::Loading(load_handle));
                CacheLookupResult::LoadRequired(loader_handle)
            }
        }
    }

    pub fn finish_load(&self, mut handle: CacheLoaderHandle<K, V>, asset: Arc<V>) {
        let mut cache = self.cache.lock().unwrap();

        let entry = cache.get_mut(&handle.key).unwrap();
        let CacheEntry::Loading(_) = &entry else {
            panic!("BlockCacheEntry::Loading expected");
        };
        *entry = CacheEntry::Loaded(Arc::downgrade(&asset));
        let Ok(()) = handle.inner.set(asset) else {
            panic!("this picture block was loaded by someone else");
        };

        handle.bomb.defuse();
    }
}

/// If you have this type, you are now responsible for loading this asset!
#[must_use]
pub struct CacheLoaderHandle<K, V> {
    key: K,
    inner: Arc<OnceCell<Arc<V>>>,
    bomb: DropBomb,
}

impl<K, V> CacheLoaderHandle<K, V> {
    pub fn get_handle(&self) -> CacheHandle<V> {
        CacheHandle::Loading(CacheLoadHandle {
            inner: self.inner.clone(),
        })
    }
}

#[derive(Debug)]
pub struct CacheLoadHandle<V> {
    inner: Arc<OnceCell<Arc<V>>>,
}

impl<V> Clone for CacheLoadHandle<V> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<V> CacheLoadHandle<V> {
    pub fn wait(self) -> Arc<V> {
        self.inner.wait().clone()
    }

    pub fn wait_ref(&self) -> &V {
        self.inner.wait().as_ref()
    }
}

/// A result of a direct cache lookup.
///
/// NB: if the value is [`CacheLookupResult::LoadRequired`], you are responsible for loading the asset!
pub enum CacheLookupResult<K, V> {
    Loaded(Arc<V>),
    Loading(CacheLoadHandle<V>),
    LoadRequired(CacheLoaderHandle<K, V>),
}

impl<K, V> CacheLookupResult<K, V> {
    /// Try to convert this lookup result into a [`CacheHandle`]. Returns [`Err`] if loading is required.
    ///
    /// Can be used to implement a function loading an asset asynchronously:
    ///
    /// ```rust
    /// struct Context {
    ///     cache: Arc<AssetCache<i32, String>>,
    /// }
    ///
    /// impl Context {
    ///     fn load(&self, key: i32) -> CacheHandle<String> {
    ///         match self.cache.lookup(key).try_into_cache_handle() {
    ///             Ok(handle) => handle,
    ///             Err(load_required) => {
    ///                 let handle = load_required.get_handle();
    ///
    ///                 // asynchronously load the asset
    ///                 let cache_clone = self.cache.clone();
    ///                 std::thread::spawn(move || {
    ///                     cache_clone.finish_load(load_required, Arc::new("asset".to_string()))
    ///                 });
    ///
    ///                 // return a handle
    ///                 handle
    ///             }
    ///         }
    ///     }
    /// }
    /// ```
    pub fn try_into_cache_handle(self) -> Result<CacheHandle<V>, CacheLoaderHandle<K, V>> {
        match self {
            CacheLookupResult::Loaded(loaded) => Ok(CacheHandle::Loaded(loaded)),
            CacheLookupResult::Loading(loading) => Ok(CacheHandle::Loading(loading)),
            CacheLookupResult::LoadRequired(load_required) => Err(load_required),
        }
    }
}

/// A handle to a value present or to be present in cache with no responsibilities attached.
#[derive(Debug)]
pub enum CacheHandle<V> {
    Loaded(Arc<V>),
    Loading(CacheLoadHandle<V>),
    Tombstone,
}

impl<V> CacheHandle<V> {
    /// Get the asset value, but keep the token.
    ///
    /// When called the second time, there will be no waiting involved.
    pub fn wait_inplace(&mut self) -> &V {
        match self {
            CacheHandle::Loaded(loaded) => loaded,
            CacheHandle::Loading(_) => {
                let CacheHandle::Loading(loading) = std::mem::replace(self, CacheHandle::Tombstone)
                else {
                    unreachable!()
                };
                *self = CacheHandle::Loaded(loading.wait());

                self.wait_inplace()
            }
            CacheHandle::Tombstone => panic!("CacheHandle::Tombstone value observed"),
        }
    }

    /// Redeem the cache handle and get our hands on a loaded asset value (possibly shared)
    pub fn wait(self) -> Arc<V> {
        match self {
            CacheHandle::Loaded(loaded) => loaded,
            CacheHandle::Loading(loading) => loading.wait(),
            CacheHandle::Tombstone => panic!("CacheHandle::Tombstone value observed"),
        }
    }

    pub fn wait_ref(&self) -> &V {
        match self {
            CacheHandle::Loaded(loaded) => loaded,
            CacheHandle::Loading(loading) => loading.wait_ref(),
            CacheHandle::Tombstone => panic!("CacheHandle::Tombstone value observed"),
        }
    }
}

impl<V> Clone for CacheHandle<V> {
    fn clone(&self) -> Self {
        match self {
            CacheHandle::Loaded(loaded) => CacheHandle::Loaded(loaded.clone()),
            CacheHandle::Loading(loading) => CacheHandle::Loading(loading.clone()),
            CacheHandle::Tombstone => panic!("CacheHandle::Tombstone value observed"),
        }
    }
}
