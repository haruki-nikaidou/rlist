mod path_compress;
pub mod combine;

use std::sync::Arc;

pub trait VfsBasicMeta {
    fn name(&self) -> &str;
    fn size(&self) -> u64;  // in bytes
    fn last_modified(&self) -> std::time::SystemTime;
}

pub trait VfsFile: VfsBasicMeta {
    fn on_download(&self) -> String;
}

#[derive(Clone)]
pub enum VfsEntry {
    File(Arc<dyn VfsFile>),
    Dir(Arc<dyn VfsDir>),
}

impl VfsBasicMeta for VfsEntry {
    fn name(&self) -> &str {
        match self {
            VfsEntry::File(file) => file.name(),
            VfsEntry::Dir(dir) => dir.name()
        }
    }

    fn size(&self) -> u64 {
        match self {
            VfsEntry::File(file) => file.size(),
            VfsEntry::Dir(dir) => dir.size()
        }
    }
    fn last_modified(&self) -> std::time::SystemTime {
        match self {
            VfsEntry::File(file) => file.last_modified(),
            VfsEntry::Dir(dir) => dir.last_modified()
        }
    }
}

pub trait VfsDir: VfsBasicMeta {
    fn list(&self) -> Vec<VfsEntry>;
}