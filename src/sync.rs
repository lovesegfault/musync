// Syncs directories
use std::error::Error;
use std::io;
use std::fmt;
use std::path::{Path, PathBuf, StripPrefixError};
use std::fs;
use probe;

#[derive(Debug)]
pub enum SyncError {
    IO(io::Error),
    Prefix(StripPrefixError),
}

impl From<io::Error> for SyncError {
    fn from(error: io::Error) -> Self {
        SyncError::IO(error)
    }
}

impl From<StripPrefixError> for SyncError {
    fn from(error: StripPrefixError) -> Self {
        SyncError::Prefix(error)
    }
}

impl fmt::Display for SyncError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            SyncError::IO(ref e) => write!(f, "An IO error occurred while syncing: {}", e),
            SyncError::Prefix(ref e) => write!(f, "A prefix error occurred while syncing: {}", e),
        }
    }
}

impl Error for SyncError {
    fn description(&self) -> &str {
        "Error occured during syncing"
    }
}

/// Syncs all files in source to dst.
/// WARNING: Deletes anything previously in dst
pub fn copy(src: &Path, dst: &Path, depth: Option<u32>) -> Result<Vec<PathBuf>, SyncError> {
    clear_dir(dst)?;
    let mut synced_objects: Vec<PathBuf> = Vec::new();

    let source_dirs = probe::directories(src, depth)?;
    for dir in source_dirs {
        fs::create_dir_all(make_path(src, dir.as_path(), dst)?)?;
        synced_objects.push(dir);
    }

    let source_files = probe::files(src, depth)?;
    for file in source_files {
        fs::copy(&file, make_path(src, file.as_path(), dst)?)?;
        synced_objects.push(file);
    }


    synced_objects.sort();
    Ok(synced_objects)
}

/// Corrects a path for copying
/// https://stackoverflow.com/questions/44419890/replacing-path-parts-in-rust
fn make_path(src: &Path, file: &Path, dst: &Path) -> Result<PathBuf, SyncError> {
    Ok(dst.join(file.strip_prefix(src)?))
}

fn clear_dir(src: &Path) -> Result<(), SyncError> {
    let file_list = probe::files(src, None)?;
    for file in file_list {
        fs::remove_file(file.as_path())?;
    }
    let subdir_list = probe::directories(src, None)?;
    for dir in subdir_list {
        fs::remove_dir_all(dir.as_path())?;
    }
    Ok(())
}
