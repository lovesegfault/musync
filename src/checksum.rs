extern crate blake2;
extern crate byteorder;
extern crate claxon;
extern crate magic;
extern crate smallvec;
//extern crate rayon;
extern crate simplemad;

use std::fmt;
use std::io;
use std::path::PathBuf;
use self::blake2::{Blake2b, Digest};
use self::byteorder::{ByteOrder, LittleEndian};
use self::magic::{Cookie, CookieFlags};
use self::smallvec::SmallVec;
//use self::simplemad::{Decoder, Frame};
//use self::rayon::prelude::*;

pub struct Checksum {
    checksum: [u8; 64],
}

impl Checksum {
    fn new() -> Checksum {
        Checksum {
            checksum: [0u8; 64],
        }
    }

    fn new_xored<II: AsRef<[u8]>, I: IntoIterator<Item = II>>(slices: I) -> Self {
        let mut res = Checksum::default();
        {
            let acc = &mut res.checksum;
            for sl in slices {
                let sl = sl.as_ref();
                debug_assert_eq!(sl.len(), acc.len());

                for (a, b) in acc.iter_mut().zip(sl.iter()) {
                    *a ^= *b;
                }
            }
        }
        res
    }
}

impl Default for Checksum {
    fn default() -> Checksum {
        Checksum::new()
    }
}

impl fmt::Display for Checksum {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut res = String::new();
        for s in self.checksum.iter() {
            res += &format!("{:x}", s);
        }
        write!(f, "{}", res)
    }
}

pub enum Filetype {
    WAV,
    FLAC,
    MP3,
    Vorbis,
    Opus,
}

impl fmt::Display for Filetype {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let ftype = match *self {
            Filetype::WAV => "Wave",
            Filetype::FLAC => "FLAC",
            Filetype::MP3 => "MP3",
            Filetype::Vorbis => "Vorbis",
            Filetype::Opus => "Opus",
        };
        write!(f, "{}", ftype)
    }
}

pub enum CheckError {
    FError(String),
    MagicError(magic::MagicError),
    ClaxonError(claxon::Error),
    FiletypeError(String),
    IOError(io::Error),
}

impl fmt::Display for CheckError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CheckError::FError(ref e) => write!(f, "File error: {}", e),
            CheckError::MagicError(ref e) => write!(f, "Magic error: {}", e),
            CheckError::ClaxonError(ref e) => write!(f, "Claxon error: {}", e),
            CheckError::FiletypeError(ref e) => write!(f, "Filetype error: {}", e),
            CheckError::IOError(ref e) => write!(f, "{}", e),
        }
    }
}

impl From<io::Error> for CheckError {
    fn from(err: io::Error) -> Self {
        CheckError::IOError(err)
    }
}

impl From<claxon::Error> for CheckError {
    fn from(err: claxon::Error) -> Self {
        CheckError::ClaxonError(err)
    }
}

impl From<String> for CheckError {
    fn from(err: String) -> Self {
        CheckError::FError(err)
    }
}

impl From<magic::MagicError> for CheckError {
    fn from(err: magic::MagicError) -> Self {
        CheckError::MagicError(err)
    }
}

fn get_filetype(fpath: &PathBuf) -> Result<Filetype, CheckError> {
    if !fpath.exists() {
        let msg = format!("File '{:?}' failed to open.", fpath);
        return Err(CheckError::FError(msg));
    }

    let cookie = Cookie::open(CookieFlags::default()).unwrap();
    cookie.load(&["/usr/share/file/misc/magic"])?;
    let ftype = cookie.file(fpath).unwrap();

    if ftype.contains("FLAC") {
        Ok(Filetype::FLAC)
    } else if ftype.contains("MPEG") && ftype.contains("III") {
        Ok(Filetype::MP3)
    } else if ftype.contains("Vorbis") {
        Ok(Filetype::Vorbis)
    } else if ftype.contains("Opus") {
        Ok(Filetype::Opus)
    } else if ftype.contains("WAVE") {
        Ok(Filetype::WAV)
    } else {
        Err(CheckError::FiletypeError(format!(
            "Invalid filetype '{:?}'.",
            fpath.extension()
        )))
    }
}

fn as_u8_slice(buf: &[i32]) -> &[u8] {
    let b: &[u8] =
        unsafe { ::std::slice::from_raw_parts(buf.as_ptr() as *const u8, buf.len() * 4) };
    b
}

fn flac_check(fpath: PathBuf) -> Result<Checksum, CheckError> {
    let mut reader = claxon::FlacReader::open(fpath)?;

    let channels = reader.streaminfo().channels as usize;

    let mut frame_reader = reader.blocks();
    let mut block_buffer: Vec<i32> = Vec::with_capacity(0x1_0000);

    //let mut hashers: Vec<Blake2b> = vec![Blake2b::new(); channels];
    let mut hashers = SmallVec::<[Blake2b; 8]>::from(vec![Blake2b::new(); channels]);

    while let Some(block) = frame_reader.read_next_or_eof(block_buffer)? {
        let duration = block.duration() as usize;
        block_buffer = block.into_buffer();

        LittleEndian::from_slice_i32(&mut block_buffer);

        // This relies on block_buffer containing `channel` * `duration` samples
        // for each channel in succession, which is claxon implementation detail,
        // and so might be broken on claxon update.
        // Instead it could be rewritten with block.channel() and LittleEndian making a copy,
        // but I'm too lazy to check how much that would be slower.
        for (hasher, chunk) in hashers.iter_mut().zip(block_buffer.chunks(duration)) {
            hasher.input(as_u8_slice(chunk));
        }
    }

    Ok(Checksum::new_xored(hashers.into_iter().map(|x| x.result())))
}

pub fn check_file(fpath: PathBuf) -> Result<Checksum, CheckError> {
    let ftype = get_filetype(&fpath)?;
    match ftype {
        Filetype::FLAC => Ok(flac_check(fpath)?),
        Filetype::MP3 => unimplemented!(),
        _ => unimplemented!(),
    }
}

#[cfg(tests)]
mod tests {
    use super::*;

    #[test]
    fn test_flac_check() {}
}
