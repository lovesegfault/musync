use std::fmt;
use std::io;
use std::path::PathBuf;
use std::fmt::Display;
use std::fs::File;

use super::blake2::{Blake2b, Digest};
use super::byteorder::{ByteOrder, LittleEndian};
use super::magic::{Cookie, CookieFlags, MagicError};
use super::smallvec::SmallVec;
use super::simplemad::{Decoder, Frame, SimplemadError};
use super::simplemad_sys::MadMode;
use super::claxon;

//use self::rayon::prelude::*;

pub struct Checksum {
    checksum: [u8; 64],
}

impl Checksum {
    fn new(a: [u8; 64]) -> Checksum {
        Checksum { checksum: a }
    }

    fn new_xor<II: AsRef<[u8]>, I: IntoIterator<Item=II>>(slices: I) -> Self {
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
    fn default() -> Self {
        Checksum::new([0u8; 64])
    }
}

impl PartialEq for Checksum {
    fn eq(&self, other: &Checksum) -> bool {
        self.checksum.iter().eq(other.checksum.iter())
    }
}

impl fmt::Debug for Checksum {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(self, f)
    }
}

impl fmt::Display for Checksum {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut res = String::new();
        for s in self.checksum.iter() {
            res += &format!("{:02x}", s);
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
    MagicError(MagicError),
    ClaxonError(claxon::Error),
    FiletypeError(String),
    IOError(io::Error),
    SimplemadError(SimplemadError),
}

impl fmt::Display for CheckError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CheckError::FError(ref e) => write!(f, "File error: {}", e),
            CheckError::MagicError(ref e) => write!(f, "Magic error: {}", e),
            CheckError::ClaxonError(ref e) => write!(f, "Claxon error: {}", e),
            CheckError::FiletypeError(ref e) => write!(f, "Filetype error: {}", e),
            CheckError::IOError(ref e) => write!(f, "IO error: {}", e),
            CheckError::SimplemadError(ref e) => write!(f, "Simplemad error: {:?}", e),
        }
    }
}

impl fmt::Debug for CheckError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(self, f)
    }
}

impl From<SimplemadError> for CheckError {
    fn from(err: SimplemadError) -> Self {
        CheckError::SimplemadError(err)
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

impl From<MagicError> for CheckError {
    fn from(err: MagicError) -> Self {
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

fn as_u8_slice<T: Copy>(buf: &[T]) -> &[u8] {
    let b: &[u8] = unsafe {
        ::std::slice::from_raw_parts(
            buf.as_ptr() as *const u8,
            buf.len() * ::std::mem::size_of::<T>(),
        )
    };
    b
}

fn flac_hash(file_path: &PathBuf) -> Result<Checksum, CheckError> {
    let mut reader = claxon::FlacReader::open(file_path)?;

    let channels = reader.streaminfo().channels as usize;

    let mut frame_reader = reader.blocks();
    let mut block_buffer: Vec<i32> = Vec::with_capacity(0x1_0000);

    // We use a SmallVec to allocate our hashers (up to 8, because if the audio
    // file has more than 8 channels then God save us) on the stack for faster
    // access. Excess hashers will spill over to heap causing slowdown.
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

    Ok(Checksum::new_xor(hashers.into_iter().map(|x| x.result())))
}

fn mp3_hash(file_path: &PathBuf) -> Result<Checksum, CheckError> {
    let f = File::open(file_path)?;
    let mut decoder = Decoder::decode(f)?;

    let initial: Frame;
    'skip_md: loop {
        match decoder.get_frame() {
            Ok(frame) => {
                initial = frame;
                break 'skip_md;
            }
            // libmad is moronic and reads metadata as audio, so we ignore errors ¯\_(ツ)_/¯
            Err(_) => {
                continue;
            }
        }
    }

    // Initial setup
    // We do it this way because Decoder does not provide the necessary information about the file
    // for hasher allocation et al
    let channels = match initial.mode {
        MadMode::SingleChannel => 1,
        _ => 2,
    };

    // We use a SmallVec to allocate our hashers (at most 2 because that's the
    // limit of the MP3 format) on the stack for faster access. Excess hashers
    // will spill over to heap causing slowdown.
    let mut hashers = SmallVec::<[Blake2b; 2]>::from(vec![Blake2b::new(); channels]);
    let mut frame_buffer: Vec<i32> = Vec::with_capacity(2000);

    for ch in 0..channels {
        // TODO: This can be done more effectively. We can just map Frame.samples from the start
        // and then get to use .into_iter() instead of .iter(), which is good since we don't need
        // the samples after we cast them to i32
        frame_buffer = initial.samples[ch].iter().map(|x| x.to_i32()).collect::<Vec<i32>>();
        LittleEndian::from_slice_i32(&mut frame_buffer);
        hashers[ch].input(as_u8_slice(&frame_buffer));
    }

    for result in decoder {
        let frame = match result {
            Ok(fr) => fr,
            _ => continue,
        };
        for ch in 0..channels {
            frame_buffer = frame.samples[ch].iter().map(|x| x.to_i32()).collect::<Vec<i32>>();
            LittleEndian::from_slice_i32(&mut frame_buffer);
            hashers[ch].input(as_u8_slice(&frame_buffer));
        }
    }

    // Iterate over frames
    // Copy flac_check() logic for speed
    Ok(Checksum::new_xor(hashers.into_iter().map(|x| x.result())))
}

pub fn hash_audio(file_path: &PathBuf) -> Result<Checksum, CheckError> {
    let file_type = get_filetype(file_path)?;
    match file_type {
        Filetype::FLAC => Ok(flac_hash(file_path)?),
        Filetype::MP3 => Ok(mp3_hash(file_path)?),
        _ => unimplemented!(),
    }
}

#[cfg(test)]
mod tests {
    extern crate hex;
    extern crate toml;

    use super::*;
    use self::hex::FromHex;

    use std::collections::HashMap;
    use std::fs::File;
    use std::io::Read;

    type Config = HashMap<String, String>;

    fn get_config(file_type: Filetype) -> Config {
        let cfg_path = PathBuf::from("./data/hashes.toml");
        let mut input = String::new();
        File::open(cfg_path)
            .and_then(|mut f| f.read_to_string(&mut input))
            .unwrap();
        let mut cfg: HashMap<String, Config> = toml::from_str(&input).unwrap();
        match file_type {
            Filetype::FLAC => return cfg.remove("flac").unwrap(),
            Filetype::MP3 => return cfg.remove("mp3").unwrap(),
            Filetype::Opus => return cfg.remove("opus").unwrap(),
            Filetype::Vorbis => return cfg.remove("vorbis").unwrap(),
            Filetype::WAV => return cfg.remove("wav").unwrap(),
        }
    }

    impl<'a> From<&'a String> for Checksum {
        fn from(s: &String) -> Self {
            let arr: [u8; 64] = FromHex::from_hex(s).unwrap();
            Checksum::new(arr)
        }
    }

    #[test]
    fn test_flac_hash() {
        let cfg = get_config(Filetype::FLAC);
        for pair in cfg.iter() {
            let fpath = PathBuf::from("./data/test.flac")
                .with_file_name(pair.0)
                .with_extension("flac");
            let check = flac_hash(&fpath).unwrap();
            assert_eq!(check, Checksum::from(pair.1));
        }
    }

    #[test]
    fn test_mp3_hash() {
        ()
    }
}
