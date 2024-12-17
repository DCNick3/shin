mod accessor;
mod context;

use std::{
    fmt::Debug,
    fs::File,
    future::Future,
    ops::{Deref, DerefMut},
    path::{Path, PathBuf},
    sync::{Arc, RwLock, Weak},
};

use anyhow::{anyhow, bail, Context, Result};
use bevy_utils::HashMap;
use derive_more::From;
use shin_core::{
    format::rom::{RomFileReader, RomReader},
    primitives::stateless_reader::StatelessFile,
};
use shin_tasks::IoTaskPool;
use tracing::debug;

pub use self::{
    accessor::{AssetDataAccessor, AssetDataCursor},
    context::AssetLoadContext,
};

pub trait Asset: Send + Sync + Sized + 'static {
    /// Load an asset from the provided data accessor.
    ///
    /// The future returned by this function will be spawned on the IO task pool.
    /// CPU-intensive work should be offloaded to the compute task pool.
    fn load(
        context: &AssetLoadContext,
        data: AssetDataAccessor,
    ) -> impl Future<Output = Result<Self>> + Send;
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

pub struct AssetServer {
    io: AssetIo,
    context: Arc<AssetLoadContext>,
    loaded_assets: RwLock<anymap::Map<dyn core::any::Any + Send + Sync>>,
}

impl AssetServer {
    pub fn new(io: AssetIo, context: AssetLoadContext) -> Self {
        Self {
            io,
            context: Arc::new(context),
            loaded_assets: RwLock::new(anymap::Map::new()),
        }
    }

    pub async fn load<T: Asset, P: AsRef<str>>(&self, path: P) -> Result<Arc<T>> {
        let path = path.as_ref();

        if let Some(loaded) = self.loaded_assets.read().unwrap().get::<AssetMap<T>>() {
            if let Some(asset) = loaded.get(path) {
                if let Some(asset) = asset.upgrade() {
                    debug!("Loaded asset from cache: {}", path);
                    return Ok(asset);
                }
            }
        }

        debug!("Loading asset: {}", path);

        // could not find the asset in the cache, load it
        let data = self
            .io
            .read_file(path)
            .with_context(|| format!("Reading asset {:?}", path))?;

        let context = self.context.clone();

        // spawn tasks on IO task pool because they can be blocking
        // they should take care off-load CPU-intensive work to the compute task pool
        let asset = IoTaskPool::get()
            .spawn(async move { T::load(&context, data).await })
            .await
            .with_context(|| format!("Loading asset {:?}", path))?;
        let asset = Arc::new(asset);

        self.loaded_assets
            .write()
            .unwrap()
            .entry::<AssetMap<T>>()
            .or_insert_with(|| AssetMap(HashMap::default()))
            .insert(path.to_string(), Arc::downgrade(&asset));

        Ok(asset)
    }

    /// Load an asset synchronously. This is useful for assets not requiring much CPU time to load.
    /// Though it might cause lockups if the loading is not blazing fast (tm).
    ///
    /// Ideally I want to get rid of all uses of this function
    pub fn load_sync<T: Asset>(&self, path: impl AsRef<str>) -> Result<Arc<T>> {
        shin_tasks::block_on(self.load(path))
    }
}

#[derive(Debug, From)]
pub enum AssetIo {
    Dir(DirAssetIo),
    RomFile(RomAssetIo),
    Layered(LayeredAssetIo),
}

impl AssetIo {
    pub fn new_dir(dir_path: impl AsRef<Path>) -> Result<Self> {
        let dir_path = dir_path.as_ref();
        let meta = std::fs::metadata(dir_path).with_context(|| {
            format!(
                "Failed to get metadata for {:?}, cannot use as asset directory",
                dir_path
            )
        })?;
        if !meta.is_dir() {
            bail!(
                "{:?} is not a directory, cannot use as asset directory",
                dir_path
            );
        }

        Ok(AssetIo::Dir(DirAssetIo::new(dir_path.to_path_buf())))
    }

    pub fn new_rom(rom_path: impl AsRef<Path>) -> Result<Self> {
        let rom_path = rom_path.as_ref();
        let rom = File::open(rom_path)
            .context("Opening ROM file")
            .and_then(|rom| StatelessFile::new(rom).context("Creating stateless file"))
            .and_then(RomReader::new)
            .with_context(|| format!("Failed to open {:?}, cannot use as asset ROM", rom_path))?;

        Ok(AssetIo::RomFile(RomAssetIo::new(
            rom,
            Some(&rom_path.display().to_string()),
        )))
    }

    fn read_file(&self, path: &str) -> Result<AssetDataAccessor> {
        match self {
            AssetIo::Dir(io) => io.read_file(path),
            AssetIo::RomFile(io) => io.read_file(path),
            AssetIo::Layered(io) => io.read_file(path),
        }
    }
}

#[derive(Debug)]
pub struct DirAssetIo {
    root_path: PathBuf,
}

impl DirAssetIo {
    pub fn new(root_path: PathBuf) -> Self {
        Self { root_path }
    }

    fn read_file(&self, path: &str) -> Result<AssetDataAccessor> {
        // 1. check if the file exists
        let full_path = self.root_path.join(path.trim_start_matches('/'));
        if !full_path.exists() {
            bail!("Asset {:?} not found", path);
        }
        Ok(AssetDataAccessor::from_file(full_path))
    }
}

pub struct RomAssetIo {
    rom: Arc<RomReader<StatelessFile>>,
    label: Option<String>,
}

impl Debug for RomAssetIo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("RomAssetIo")
            .field(&self.label.as_deref().unwrap_or("unnamed"))
            .finish()
    }
}

impl RomAssetIo {
    pub fn new(rom: RomReader<StatelessFile>, label: Option<&str>) -> Self {
        Self {
            rom: Arc::new(rom),
            label: label.map(|s| s.to_string()),
        }
    }

    fn read_file(&self, path: &str) -> Result<AssetDataAccessor> {
        // 1. find the file in the ROM
        let file = self
            .rom
            .find_file(path)
            .with_context(|| format!("Finding asset {:?}", path))?;
        let file = RomFileReader::new(self.rom.clone(), file);
        Ok(AssetDataAccessor::from_rom_file(file))
    }
}

#[derive(Debug, Default)]
pub struct LayeredAssetIo {
    io: Vec<AssetIo>,
}

impl LayeredAssetIo {
    pub fn new() -> Self {
        Self { io: Vec::new() }
    }

    pub fn is_empty(&self) -> bool {
        self.io.is_empty()
    }

    pub fn with(&mut self, io: AssetIo) {
        self.io.push(io);
    }

    pub fn try_with_dir(&mut self, dir_path: impl AsRef<Path>) -> Result<()> {
        self.with(AssetIo::new_dir(dir_path)?);
        Ok(())
    }

    pub fn try_with_rom(&mut self, rom_path: impl AsRef<Path>) -> Result<()> {
        self.with(AssetIo::new_rom(rom_path)?);
        Ok(())
    }

    fn read_file(&self, path: &str) -> Result<AssetDataAccessor> {
        let mut errors = Vec::new();

        for io in &self.io {
            match io.read_file(path) {
                Ok(data) => return Ok(data),
                Err(err) => errors.push(err),
            }
        }

        Err(anyhow!(
            "Failed to read asset {:?} from all layers: {:?}",
            path,
            errors
        ))
    }
}
