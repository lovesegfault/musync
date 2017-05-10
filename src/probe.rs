/// Probes the filesystem for files, directories, ...
use std::io::Result;
use std::path::{Path, PathBuf};
use std::fs;

fn is_newer(src: &fs::Metadata, dst: &fs::Metadata) -> Result<bool> {
    let src_time = src.modified()?;
    let dst_time = dst.modified()?;
    Ok(src_time > dst_time)
}

/// Yields all objects (files, directories, etc) in a given path (dir)
/// Passing None means unlimited depth
pub fn objects(dir: &Path, depth: Option<u32>) -> Result<Vec<PathBuf>> {
    let path_list = fs::read_dir(dir)?; // Get paths in dir
    let mut object_list: Vec<PathBuf> = Vec::new(); // Result vector
    for object in path_list {
        let object_path = object?.path(); // Unwrap object, we only need it's path
        let metadata = fs::metadata(&object_path)?; // Get metadata
        if metadata.is_dir() && depth != Some(0) {
            object_list.extend(objects(object_path.as_path(), depth.map(|d| d - 1))?); // Recursion
        }
        object_list.push(object_path); // Add object to list
    }
    object_list.sort(); // Sort result
    Ok(object_list)
}


/// Yields all files in a given path up to depth
/// Passing None means unlimited depth
pub fn files(dir: &Path, depth: Option<u32>) -> Result<Vec<PathBuf>> {
    let path_list = fs::read_dir(dir)?; // Get paths in dir
    let mut file_list: Vec<PathBuf> = Vec::new(); // Generate result vector
    for object in path_list {
        let object_path = object?.path(); // Unwrap object, we only need it's path
        let metadata = fs::metadata(&object_path)?; // Get metadata
        if metadata.is_dir() && depth != Some(0) {
            // Recursion for directories
            file_list.extend(files(object_path.as_path(), depth.map(|d| d - 1))?);
        } else {
            file_list.push(object_path.to_path_buf()); // Add path to result vector
        }
    }
    file_list.sort(); // read_dir doesn't order things, so we do that here.
    Ok(file_list)
}

/// Yields all directories in path up to depth
/// Passing None means unlimited depth
pub fn directories(dir: &Path, depth: Option<u32>) -> Result<Vec<PathBuf>> {
    let path_list = fs::read_dir(dir)?; // Get paths in dir
    let mut dir_list: Vec<PathBuf> = Vec::new(); // Result vector
    for object in path_list {
        let object_path = object?.path(); // Unwrap object, we only need it's path
        let metadata = fs::metadata(&object_path)?; // Get metadata
        if metadata.is_dir() {
            dir_list.push(object_path.to_path_buf()); // Add dir to result
            if depth != Some(0) {
                // Recursion
                dir_list.extend(directories(object_path.as_path(), depth.map(|d| d - 1))?)
            }
        }
    }
    dir_list.sort(); // Sort result
    Ok(dir_list)
}

/// Yields all new files in src; i.e. files in src _not_ in dst.
/// Passing None means unlimited depth
pub fn created(src: &Path, dst: &Path, depth: Option<u32>) -> Result<Vec<PathBuf>> {
    let mut new_files: Vec<PathBuf> = Vec::new();
    let src_files = files(src, depth)?;
    let dst_files = files(dst, depth)?;
    'outer: for orig in src_files {
        for copy in &dst_files {
            if orig.file_stem() == copy.file_stem() {
                continue 'outer;
            }
        }
        new_files.push(orig);
    }
    Ok(new_files)
}

/// Yields all files in dst _not_ in src, i.e. deleted files from source.
/// Passing None mean unlimited depth
pub fn deleted(src: &Path, dst: &Path, depth: Option<u32>) -> Result<Vec<PathBuf>> {
    let mut deleted_files: Vec<PathBuf> = Vec::new();
    let src_files = files(src, depth)?;
    let dst_files = files(dst, depth)?;
    'outer: for copy in dst_files {
        for orig in &src_files {
            if copy.file_stem() == orig.file_stem() {
                continue 'outer;
            }
        }
        deleted_files.push(copy);
    }
    Ok(deleted_files)
}

/// Yields all files in src which are also in dst but whose last modification date is newer.
/// Passing None means unlimited depth
/// Does not include files in src which are _not_ in dst, nor the contrary.
/// Check `probe::created` and `probe::deleted` for that.
pub fn changed(src: &Path, dst: &Path, depth: Option<u32>) -> Result<Vec<PathBuf>> {
    let mut changed_files: Vec<PathBuf> = Vec::new();
    let src_files = files(src, depth)?;
    let dst_files = files(dst, depth)?;
    'outer: for copy in dst_files {
        let copy_md = fs::metadata(&copy)?;
        for orig in &src_files {
            let orig_md = fs::metadata(&orig)?;
            if orig.file_stem() == copy.file_stem() && is_newer(&orig_md, &copy_md)? {
                changed_files.push(orig.to_path_buf());
                continue 'outer;
            }

        }
    }
    Ok(changed_files)
}
