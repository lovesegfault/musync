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
use self::byteorder::{LittleEndian, WriteBytesExt};
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

// TODO: Make fast
fn flac_check(fpath: PathBuf) -> Result<Checksum, CheckError> {
    let mut reader = claxon::FlacReader::open(fpath)?;

    let channels = reader.streaminfo().channels;

    let mut frame_reader = reader.blocks();
    let mut block = claxon::Block::empty();
    let mut buffer = vec![];

    let mut hashers: Vec<Blake2b> = Vec::with_capacity(channels as usize);

    loop {
        match frame_reader.read_next_or_eof(block.into_buffer()) {
            Ok(Some(next_block)) => block = next_block,
            Ok(None) => break, // EOF.
            Err(error) => return Err(CheckError::ClaxonError(error)),
        }

        for c in 0..channels {
            for s in 0..block.duration() {
                buffer.write_i32::<LittleEndian>(block.sample(c, s))?;
            }
            hashers[c as usize].input(&buffer);
            buffer.clear()
        }
    }

    let _res: Vec<_> = hashers.into_iter().map(|x| x.result()).collect();
    let _res: Vec<_> = _res.iter().map(|y| y.as_slice()).collect();
    let foo = xor_slices(_res);

    // Extract slices, XOR, return
    Ok(Checksum {
        checksum: vec![1, 2, 3],
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
