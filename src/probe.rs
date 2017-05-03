/// Probes the filesystem for files, directories, ...
use std::io::Result;
use std::path::{Path, PathBuf};
use std::fs;

pub fn objects(dir: &Path, depth: i32) -> Result<Vec<PathBuf>> {
    let path_list = fs::read_dir(dir)?;
    let mut object_list : Vec<PathBuf> = Vec::new();
    for object in path_list{
        let object_path = object?.path();
        let metadata = fs::metadata(&object_path)?;
        if metadata.is_dir() && depth != 0 {
            object_list.extend(objects(object_path.as_path(), depth -1)?);
        }
        object_list.push(object_path);
    }
    object_list.sort();
    Ok(object_list)
}


/// Yields all files in a given path up to depth
/// Passing a negative value means "unlimited" depth. Up to the limit of i32
pub fn files(dir: &Path, depth: i32) -> Result<Vec<PathBuf>> {
    let path_list = fs::read_dir(dir)?;
    let mut file_list: Vec<PathBuf> = Vec::new();
    for object in path_list {
        let object_path = object?.path();
        let metadata = fs::metadata(&object_path)?;
        if metadata.is_dir() && depth != 0 {
            file_list.extend(files(object_path.as_path(), depth - 1)?);
        } else {
            file_list.push(object_path.to_path_buf());
        }
    }
    file_list.sort();
    Ok(file_list)
}

/// Yields all directories in path up to depth
/// Passing a negative value means "unlimited" depth. Up to the limit of i32
pub fn directories(dir: &Path, depth: i32) -> Result<Vec<PathBuf>> {
    let path_list = fs::read_dir(dir)?;
    let mut dir_list: Vec<PathBuf> = Vec::new();

    for object in path_list {
        let object_path = object?.path();
        let metadata = fs::metadata(&object_path)?;
        if metadata.is_dir() {
            dir_list.push(object_path.to_path_buf());
            if depth != 0 {
                dir_list.extend(directories(object_path.as_path(), depth - 1)?)
            }
        }
    }
    dir_list.sort();
    Ok(dir_list)
}
