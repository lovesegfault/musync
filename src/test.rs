use std::io::{Result, Write};
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

pub fn mk_test(dst: &Path, depth: u32, size: u32) -> Result<Vec<PathBuf>> {
    let mut created: Vec<PathBuf> = Vec::new();
    clear_test(&dst)?;
    let mut counter = 0;
    for c in ASCII_LOWER.iter() {
        if counter >= size {
            break;
        }
        let create = dst.join(c.to_string());
        fs::create_dir_all(&create)?;
        mk_test_files(&create, size)?;
        created.push(create.to_path_buf());
        if depth != 0 {
            let mut inner = mk_test(dst, depth - 1, size)?;
            created.append(&mut inner);
        }
        counter += 1;
    }

    Ok(created)
}

fn mk_test_files(dst: &Path, size: u32) -> Result<Vec<PathBuf>> {
    let mut created: Vec<PathBuf> = Vec::new();
    for c in 1..(size + 1) {
        let path = dst.join(format!("{:?}.txt", c));
        let mut file = fs::File::create(&path)?;
        file.write_all(b"test")?;
        file.sync_data()?;

        created.push(path);
    }
    Ok(created)
}
