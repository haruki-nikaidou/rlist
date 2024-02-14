use std::collections::HashMap;
use std::sync::Arc;
use crate::vfs::{VfsBasicMeta, VfsDir, VfsEntry, VfsFile};
use rand::Rng;
use std::marker::Send;

#[derive(Clone)]
pub struct CombinableVfsFile {
    _links: Vec<String>,
    _name: String,
    _size: u64,
    _last_modified: std::time::SystemTime,
    _on_download: Arc<dyn Send + Fn() -> String>,
}

#[derive(Clone)]
pub struct CombinableVfsDir {
    _name: String,
    _sub_dirs: Vec<CombinableVfsDir>,
    _files: Vec<CombinableVfsFile>,
    _size: u64,
}

impl CombinableVfsDir {
    pub fn new(name: String, sub_dirs: Vec<CombinableVfsDir>, files: Vec<CombinableVfsFile>, size: u64) -> Self {
        CombinableVfsDir {
            _name: name,
            _sub_dirs: sub_dirs,
            _files: files,
            _size: size,
        }
    }
}

unsafe impl Send for CombinableVfsFile {}
unsafe impl Sync for CombinableVfsFile {}
unsafe impl Sync for CombinableVfsDir {}
unsafe impl Send for CombinableVfsDir {}

impl CombinableVfsFile {
    fn possible_on_download(&self) -> Vec<String> {
        self._links.clone()
    }
    pub fn new(links: Vec<String>, name: String, size: u64, last_modified: std::time::SystemTime) -> Self {
        let on_download = get_random_selector(links.len(), links.clone());
        CombinableVfsFile {
            _links: links,
            _name: name,
            _size: size,
            _last_modified: last_modified,
            _on_download: Arc::new(on_download),
        }
    }
}

impl VfsBasicMeta for CombinableVfsFile {
    fn name(&self) -> &str {
        &self._name
    }
    fn size(&self) -> u64 {
        self._size
    }
    fn last_modified(&self) -> std::time::SystemTime {
        self._last_modified
    }
}

impl VfsFile for CombinableVfsFile {
    fn on_download(&self) -> String {
        (self._on_download)()
    }
}

fn combine_vfs_files(files: Vec<CombinableVfsFile>) -> CombinableVfsFile {
    let maybe_files: Vec<String> = files.iter()
        .map(|file| file.possible_on_download())
        .flatten().collect();
    let on_download = get_random_selector(maybe_files.len(), maybe_files.clone());
    return CombinableVfsFile {
        _links: maybe_files,
        _name: files[0].name().to_owned(),
        _size: files[0].size(),
        _last_modified: files.iter().map(|file| file.last_modified()).max().unwrap(),
        _on_download: Arc::new(on_download),
    };
}

fn get_random_selector<T: Clone>(n: usize, possibles: Vec<T>) -> impl Fn() -> T {
    move || {
        let index = rand::thread_rng().gen_range(0..n);
        possibles[index].clone()
    }
}

impl VfsBasicMeta for CombinableVfsDir {
    fn name(&self) -> &str {
        &self._name
    }
    fn size(&self) -> u64 {
        self._size
    }
    fn last_modified(&self) -> std::time::SystemTime {
        self._files.iter().map(|file| file.last_modified()).max().unwrap()
    }
}

impl VfsDir<CombinableVfsFile> for CombinableVfsDir {
    fn list(&self) -> Vec<VfsEntry<CombinableVfsFile,CombinableVfsDir>> {
        let mut entries: Vec<VfsEntry<_,_>> = self._sub_dirs.iter()
            .map(|dir| VfsEntry::Dir(dir.clone())).collect();
        entries.extend(self._files.iter()
            .map(|file| VfsEntry::File(file.clone())));
        entries
    }
}

impl CombinableVfsDir {
    fn destruct(self) -> (Vec<CombinableVfsDir>, Vec<CombinableVfsFile>, u64, String) {
        let sub_dirs = self._sub_dirs;
        let files = self._files;
        let size = self._size;
        let name = self._name;
        (sub_dirs, files, size, name)
    }
}

pub fn combine_vfs_dirs(dirs: Vec<CombinableVfsDir>) -> CombinableVfsDir {
    let dirs = dirs.into_iter().map(|dir| dir.destruct()).collect::<Vec<_>>();
    let name = dirs[0].3.clone();
    let dirs: (Vec<Vec<CombinableVfsDir>>, Vec<Vec<CombinableVfsFile>>) = dirs.into_iter().map(|(sub_dirs, files, _, _)| {
        (sub_dirs, files)
    }).unzip();
    let sub_dirs = dirs.0.into_iter().flatten().collect();
    let files = dirs.1.into_iter().flatten().collect();
    let sub_dirs = separate_by_name(sub_dirs);
    let files = separate_by_name(files);
    let sub_dirs: Vec<CombinableVfsDir> = sub_dirs.into_iter().map(|(_, dirs)| {
        combine_vfs_dirs(dirs)
    }).collect();
    let files: Vec<CombinableVfsFile> = files.into_iter().map(|(_, files)| {
        combine_vfs_files(files)
    }).collect();
    let size: u64 =
        files.iter()
            .map(|file| file.size()).sum::<u64>()
            +
            sub_dirs.iter()
                .map(|dir| dir.size()).sum::<u64>();
    CombinableVfsDir {
        _name: name,
        _sub_dirs: sub_dirs,
        _files: files,
        _size: size,
    }
}

fn separate_by_name<T: VfsBasicMeta>(flat: Vec<T>) -> HashMap<String, Vec<T>> {
    let mut map: HashMap<String, Vec<T>> = HashMap::new();
    for entry in flat {
        let name = entry.name().to_owned();
        if map.contains_key(&name) {
            map.get_mut(&name).unwrap().push(entry);
        } else {
            map.insert(name, vec![entry]);
        }
    }
    map
}