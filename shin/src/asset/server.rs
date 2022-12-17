use anyhow::{Context, Result};
use async_trait::async_trait;
use bevy_tasks::{AsyncComputeTaskPool, IoTaskPool};
use bevy_utils::HashMap;
use shin_core::format::rom::RomReader;
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock, Weak};

pub trait Asset: Send + Sync + Sized + 'static {
    fn load_from_bytes(data: Vec<u8>) -> Result<Self>;
}

struct AssetMap<T: Asset>(HashMap<String, Weak<T>>);

impl<T: Asset> Deref for AssetMap<T> {
    type Target = HashMap<String, Weak<T>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<T: Asset> DerefMut for AssetMap<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl<T: Asset> typemap::Key for AssetMap<T> {
    type Value = AssetMap<T>;
}

pub struct AssetServer<Io: AssetIo> {
    io: Io,
    loaded_assets: RwLock<typemap::ShareMap>,
}

impl<Io: AssetIo> AssetServer<Io> {
    pub fn new(io: Io) -> Self {
        Self {
            io,
            loaded_assets: RwLock::new(typemap::ShareMap::custom()),
        }
    }

    pub async fn load<T: Asset>(&self, path: &str) -> Result<Arc<T>> {
        if let Some(loaded) = self.loaded_assets.read().unwrap().get::<AssetMap<T>>() {
            if let Some(asset) = loaded.get(path) {
                if let Some(asset) = asset.upgrade() {
                    return Ok(asset);
                }
            }
        }

        // could not find the asset in the cache, load it
        let data = self
            .io
            .read_file(path)
            .await
            .with_context(|| format!("Reading asset {:?}", path))?;

        let asset = AsyncComputeTaskPool::get()
            .spawn(async move { T::load_from_bytes(data) })
            .await?;
        let asset = Arc::new(asset);

        self.loaded_assets
            .write()
            .unwrap()
            .entry::<AssetMap<T>>()
            .or_insert_with(|| AssetMap(HashMap::default()))
            .insert(path.to_string(), Arc::downgrade(&asset));

        Ok(asset)
    }
}

pub type AnyAssetServer = AssetServer<AnyAssetIo>;

impl AnyAssetServer {
    pub fn new_dir(root_path: PathBuf) -> Self {
        Self::new(AnyAssetIo::new_dir(root_path))
    }

    pub fn new_rom(rom_path: impl AsRef<Path>) -> Self {
        Self::new(AnyAssetIo::new_rom(rom_path))
    }
}

#[async_trait]
pub trait AssetIo {
    async fn read_file(&self, path: &str) -> Result<Vec<u8>>;
}

pub struct DirAssetIo {
    root_path: PathBuf,
}

impl DirAssetIo {
    pub fn new(root_path: PathBuf) -> Self {
        Self { root_path }
    }
}

#[async_trait]
impl AssetIo for DirAssetIo {
    async fn read_file(&self, path: &str) -> Result<Vec<u8>> {
        let full_path = self.root_path.join(path.trim_start_matches('/'));
        IoTaskPool::get()
            .spawn(async move { std::fs::read(full_path) })
            .await
            .with_context(|| {
                format!(
                    "Reading asset {:?} (root_path = {:?})",
                    path, self.root_path
                )
            })
    }
}

pub struct RomAssetIo<S: io::Read + io::Seek + Send + Sync + 'static> {
    rom: Arc<Mutex<RomReader<S>>>,
}

impl<S: io::Read + io::Seek + Send + Sync + 'static> RomAssetIo<S> {
    pub fn new(rom: RomReader<S>) -> Self {
        Self {
            rom: Arc::new(Mutex::new(rom)),
        }
    }
}

#[async_trait]
impl<S: io::Read + io::Seek + Send + Sync + 'static> AssetIo for RomAssetIo<S> {
    async fn read_file(&self, path: &str) -> Result<Vec<u8>> {
        let rom = self.rom.clone();
        let path = path.to_string();

        IoTaskPool::get()
            .spawn(async move {
                use io::Read;

                let mut rom = rom.lock().unwrap();
                let file = rom
                    .find_file(&path)
                    .with_context(|| format!("Finding asset {:?}", path))?;
                let mut file = rom
                    .open_file(file)
                    .with_context(|| format!("Opening asset {:?}", path))?;

                let mut data = Vec::new();
                file.read_to_end(&mut data)
                    .with_context(|| format!("Reading asset {:?}", path))?;

                Ok(data)
            })
            .await
    }
}

pub enum AnyAssetIo {
    Dir(DirAssetIo),
    RomFile(RomAssetIo<BufReader<File>>),
}

impl AnyAssetIo {
    pub fn new_dir(root_path: PathBuf) -> Self {
        Self::Dir(DirAssetIo::new(root_path))
    }

    pub fn new_rom(rom_path: impl AsRef<Path>) -> Self {
        let rom =
            RomReader::new(BufReader::new(File::open(rom_path).unwrap())).expect("Opening rom");
        Self::RomFile(RomAssetIo::new(rom))
    }
}

#[async_trait]
impl AssetIo for AnyAssetIo {
    async fn read_file(&self, path: &str) -> Result<Vec<u8>> {
        match self {
            Self::Dir(io) => io.read_file(path).await,
            Self::RomFile(io) => io.read_file(path).await,
        }
    }
}
