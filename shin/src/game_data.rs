use std::path::PathBuf;

pub struct GameData {
    data_directory: PathBuf,
}

impl GameData {
    pub fn new(data_directory: PathBuf) -> Self {
        Self { data_directory }
    }

    // TODO: this should be async
    pub fn read_file(&self, path: &str) -> Vec<u8> {
        let real_path = self.data_directory.join(path.trim_start_matches('/'));
        std::fs::read(&real_path).unwrap_or_else(|e| {
            panic!(
                "Failed to read file: {:?} (data_directory = {}): {:?}",
                path,
                self.data_directory.display(),
                e
            )
        })
    }
}
