// Syncs directories
use std::io::Result;
use std::result;
use std::path::{Path, PathBuf, StripPrefixError};
use std::fs;
use probe;

/// Syncs all files in source to dst.
/// WARNING: Deletes anything previously in dst
pub fn copy(src: &Path, dst: &Path, depth: Option<u32>)-> Result<Vec<PathBuf>> {
    clear_dir(dst)?;
    let mut synced_objects: Vec<PathBuf> = Vec::new();

    let source_dirs = probe::directories(src, depth)?;
    for dir in source_dirs{
        fs::create_dir_all(make_path(src, dir.as_path(), dst)?)?;
        synced_objects.push(dir);
    }

    let source_files = probe::files(src, depth)?;
    for file in source_files{
        fs::copy(&file, make_path(src, file.as_path(), dst)?)?;
        synced_objects.push(file);
    }


    synced_objects.sort();
    Ok(synced_objects)
}

/// Corrects a path for copying
/// https://stackoverflow.com/questions/44419890/replacing-path-parts-in-rust
fn make_path(src: &Path, file: &Path, dst: &Path) -> Result<PathBuf> {
    Ok(dst.join(file.strip_prefix(src)?))
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
