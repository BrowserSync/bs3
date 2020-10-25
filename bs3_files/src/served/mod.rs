use std::path::{PathBuf, Path};
use std::collections::HashSet;

#[derive(Default, Debug)]
pub struct ServedFiles {
    pub items: HashSet<ServedFile>
}

impl ServedFiles {
    pub fn add(&mut self, sf: ServedFile) {
        self.items.insert(sf);
    }
    pub fn add_from_path(&mut self, path: PathBuf, web_path: PathBuf) {
        self.items.insert(ServedFile { path: path.into(), web_path: web_path.into() });
    }
}

#[derive(Default, Debug, Eq, Hash, PartialEq)]
pub struct ServedFile {
    pub path: PathBuf,
    pub web_path: PathBuf,
}

