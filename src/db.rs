use sled;

use std::path::PathBuf;

pub struct DB {
    tree: sled::Tree,
}

impl DB {
    pub fn open(path: String) -> Self {
        let db_path = PathBuf::from(&path);
        let cfg = sled::Config::default()
            .path(path.to_owned())
            .use_compression(true);
        if db_path.is_file() {
            return DB { tree: cfg.tree() };
        } else {
            return DB { tree: sled::Tree::new(cfg) };

        }
    }
}
