use crate::syntax;
use rustc_hash::FxHashMap;
use std::collections::hash_map::Entry;
use std::fmt;
use std::num::NonZeroU16;

mod in_file;

pub use in_file::InFile;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FileId(NonZeroU16);

pub struct File {
    path: String,
    contents: String,
}

impl fmt::Debug for File {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // exclude contents from debug output
        f.debug_struct("File").field("path", &self.path).finish()
    }
}

impl fmt::Display for File {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "File@{}", self.path)
    }
}

#[derive(Default)]
pub struct FileDb {
    files: FxHashMap<FileId, File>,
    files_by_path: FxHashMap<String, FileId>,
    seq_id: u16,
}

impl FileDb {
    pub fn new() -> Self {
        FileDb {
            files: FxHashMap::default(),
            files_by_path: FxHashMap::default(),
            seq_id: 0,
        }
    }

    pub fn single_file(path: String, contents: String) -> Self {
        let mut db = Self::new();
        db.add_file(path, contents);
        db
    }

    pub fn file(&self, id: FileId) -> &File {
        self.files.get(&id).unwrap()
    }

    pub fn file_id_by_path(&self, path: &str) -> Option<FileId> {
        self.files_by_path.get(path).copied()
    }

    pub fn files(&self) -> impl Iterator<Item = FileId> + '_ {
        self.files.keys().copied()
    }

    pub fn parse(&self, file_id: FileId) -> syntax::Parse<syntax::SourceFile> {
        let file = self.file(file_id);
        // TODO: use salsa or smth
        syntax::SourceFile::parse(&file.contents)
    }

    pub fn add_file(&mut self, path: String, contents: String) -> FileId {
        self.seq_id = self.seq_id.checked_add(1).unwrap();
        let id = FileId(NonZeroU16::new(self.seq_id).unwrap());
        match self.files_by_path.entry(path.clone()) {
            Entry::Occupied(_) => {
                todo!("handle file already exists")
            }
            Entry::Vacant(entry) => {
                entry.insert(id);

                self.files.insert(id, File { path, contents });

                id
            }
        }
    }
}
