extern crate filetime;

use filetime::FileTime;
use std::collections::{HashSet, HashMap};
use std::path::{Path, PathBuf};
use std::fs;

enum Changed {
    Newer,
    Older,
    None,
};

type PathBufSet = HashSet<PathBuf>;
type ChangedMap = HashMap<PathBuf, Changed>;


