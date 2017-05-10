// Syncs directories
use std::io::Result;
use std::path::{Path, PathBuf};
use std::fs;
use probe;

/// Syncs all files in source to dst.
/// WARNING: Deletes anything previously in dst
pub fn copy(src: &Path, dst: &Path, depth: Option<u32>)-> Result<Vec<PathBuf>> {
    clear_dir(dst)?;
    let mut synced_files: Vec<PathBuf> = Vec::new();
    let source_files = probe::files(src, depth)?;
    for file in source_files{
        fs::copy(&file, make_path(file.as_path(), dst))?;
        synced_files.push(file);
    }
    Ok(synced_files)
}

fn make_path(orig: &Path, dst: &Path) -> PathBuf{
    let mut parts = orig.components();
    let _ = parts.next();
    dst.join(parts.as_path())
}

fn clear_dir(src: &Path)-> Result<()>{
    let file_list = probe::files(src, None)?;
    for file in file_list{
        fs::remove_file(file.as_path())?;
    }
    let subdir_list = probe::files(src, None)?;
    for dir in subdir_list{
        fs::remove_dir_all(dir.as_path())?;
    }
Ok(())
}
