use std::io::Result;
use std::path::{Path, PathBuf};
use std::fs;

use probe;

static ASCII_LOWER: [char; 26] = [
    'a',
    'b',
    'c',
    'd',
    'e',
    'f',
    'g',
    'h',
    'i',
    'j',
    'k',
    'l',
    'm',
    'n',
    'o',
    'p',
    'q',
    'r',
    's',
    't',
    'u',
    'v',
    'w',
    'x',
    'y',
    'z',
];


fn clear_test(src: &Path) -> Result<()> {
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

pub fn mk_test_dirs(dst: &Path, depth: Option<u32>, size: u32) -> Result<Vec<PathBuf>> {
    let mut created: Vec<PathBuf> = Vec::new();
    clear_test(&dst)?;
    for c in ASCII_LOWER.iter() {
        let create = dst.join(c.to_string());
        fs::create_dir_all(&create)?;
        created.push(create.to_path_buf());
        println!("{:?}", create);
    }

    Ok(created)
}
