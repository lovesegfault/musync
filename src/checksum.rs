extern crate blake2;
extern crate byteorder;
extern crate claxon;
extern crate magic;
//extern crate rayon;
extern crate simplemad;

use std::fmt;
use std::io;
use std::path::PathBuf;
use self::blake2::{Blake2b, Digest};
use self::byteorder::{ByteOrder, LittleEndian};
use self::magic::{Cookie, CookieFlags};
//use self::simplemad::{Decoder, Frame};
//use self::rayon::prelude::*;

pub struct Checksum {
    checksum: Vec<u8>,
}

impl fmt::Display for Checksum {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut res = String::new();
        for s in &self.checksum {
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
    cookie.load(&vec!["/usr/share/file/misc/magic"])?;
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

fn xor_slices(slices: &[&[u8]]) -> Vec<u8> {
    let res = vec![0u8; slices[0].len()];

    slices.iter().fold(res, |mut acc, sl| {
        for (a, b) in acc.iter_mut().zip(sl.iter()) {
            *a ^= b;
        }
        acc
    })
}

fn as_u8_slice(buf: &[i32]) -> &[u8] {
    let b: &[u8] = unsafe {
        ::std::slice::from_raw_parts(buf.as_ptr() as *const u8, buf.len() * 4)
    };
    b
}

// TODO: Make fast
fn flac_check(fpath: PathBuf) -> Result<Checksum, CheckError> {
    let mut reader = claxon::FlacReader::open(fpath)?;

    let channels = reader.streaminfo().channels as usize;

    let mut frame_reader = reader.blocks();
    let mut block_buffer: Vec<i32> = Vec::with_capacity(65536);

    let mut hashers: Vec<Blake2b> = vec![Blake2b::new(); channels];

    loop {
        let block = match frame_reader.read_next_or_eof(block_buffer) {
            Ok(Some(next_block)) => next_block,
            Ok(None) => break, // EOF.
            Err(error) => return Err(CheckError::ClaxonError(error)),
        };

        let duration = block.duration() as usize;
        block_buffer = block.into_buffer();

        LittleEndian::from_slice_i32(&mut block_buffer);

        for (hasher, chunk) in hashers.iter_mut().zip(block_buffer.chunks(duration)) {
            hasher.input(as_u8_slice(chunk));
        }
    }

    let res: Vec<_> = hashers.into_iter().map(|x| x.result()).collect();
    let res: Vec<_> = res.iter().map(|y| y.as_slice()).collect();

    // Extract slices, XOR, return
    Ok(Checksum {
        checksum: xor_slices(&res),
    })
}

/*
fn mp3_check(fpath: PathBuf) -> Result<String, CheckError> {
    let mut hasher = Blake2b::new();

    Ok("Foo".to_owned())
}
*/

pub fn check_file(fpath: PathBuf) -> Result<Checksum, CheckError> {
    let ftype = get_filetype(&fpath)?;
    match ftype {
        Filetype::FLAC => Ok(flac_check(fpath)?),
        Filetype::MP3 => unimplemented!(),
        _ => unimplemented!(),
    }
}
