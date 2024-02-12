use std::collections::HashMap;
use std::sync::Arc;
use crate::vfs::{VfsBasicMeta, VfsDir, VfsEntry, VfsFile};

pub struct IndexedVfs{
    root: Arc<dyn VfsDir>,
    compressed_path: HashMap<String, VfsEntry>
}

pub enum TryPathResult {
    NotFound,
    Dir(Arc<dyn VfsDir>),
    File(Arc<dyn VfsFile>)
}

impl IndexedVfs {
    pub fn new(root: Arc<dyn VfsDir>) -> IndexedVfs {
        let mut compressed_path = HashMap::new();
        IndexedVfs::compress_path(root.clone(), &mut compressed_path, "");
        IndexedVfs {
            root,
            compressed_path
        }
    }

    fn compress_path(dir: Arc<dyn VfsDir>, compressed_path: &mut HashMap<String, VfsEntry>, path: &str) {
        for entry in dir.list() {
            let entry_path = format!("{}/{}", path, entry.name());
            compressed_path.insert(entry_path.clone(), entry.clone());
            match entry {
                VfsEntry::Dir(dir) => {
                    IndexedVfs::compress_path(dir, compressed_path, &entry_path);
                },
                VfsEntry::File(_) => {}
            }
        }
    }

    pub fn try_path(&self, path: &str) -> TryPathResult {
        match self.compressed_path.get(path) {
            None => TryPathResult::NotFound,
            Some(entry) => {
                match entry {
                    VfsEntry::Dir(dir) => TryPathResult::Dir(dir.clone()),
                    VfsEntry::File(file) => TryPathResult::File(file.clone())
                }
            }
        }
    }
}