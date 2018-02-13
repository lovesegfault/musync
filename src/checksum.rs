use std::fmt;
use std::io;
use std::path::PathBuf;
use std::fmt::Display;
use std::fs::File;

use super::blake2::{Blake2b, Digest};
use super::byteorder::{ByteOrder, LittleEndian};
use super::magic::{Cookie, CookieFlags, MagicError};
use super::smallvec::SmallVec;
use super::simplemad::{Decoder, SimplemadError};
use super::simplemad_sys::MadMode;
use super::claxon;
use super::lewton::{inside_ogg, VorbisError};

//use self::rayon::prelude::*;

pub struct Checksum {
    checksum: [u8; 64],
}

impl Checksum {
    fn new(a: [u8; 64]) -> Checksum {
        Checksum { checksum: a }
    }

    fn new_xor<II: AsRef<[u8]>, I: IntoIterator<Item = II>>(slices: I) -> Self {
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

#[derive(PartialEq)]
pub enum Filetype {
    WAV,
    FLAC,
    MP3,
    Vorbis,
    Opus,
}

impl fmt::Debug for Filetype {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let filetype = match *self {
            Filetype::WAV => "wav",
            Filetype::FLAC => "flac",
            Filetype::MP3 => "mp3",
            Filetype::Vorbis => "ogg",
            Filetype::Opus => "opus",
        };
        write!(f, "{}", filetype)
    }
}

impl fmt::Display for Filetype {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let filetype = match *self {
            Filetype::WAV => "Wave",
            Filetype::FLAC => "FLAC",
            Filetype::MP3 => "MP3",
            Filetype::Vorbis => "Vorbis",
            Filetype::Opus => "Opus",
        };
        write!(f, "{}", filetype)
    }
}

pub enum CheckError {
    FError(String),
    FiletypeError(String),
    IOError(io::Error),
    MagicError(MagicError),
    ClaxonError(claxon::Error),
    SimplemadError(SimplemadError),
    VorbisError(VorbisError),
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
            CheckError::VorbisError(ref e) => write!(f, "Vorbis error: {:?}", e),
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

impl From<VorbisError> for CheckError {
    fn from(err: VorbisError) -> Self {
        CheckError::VorbisError(err)
    }
}

fn find_magic(cookie: &Cookie) -> Result<(), CheckError> {
    match cookie.load::<&str>(&[]) {
        Ok(_) => Ok(()),
        Err(_) => Err(CheckError::FError("Failed to locate libmagic\n".to_owned())),
    }
}

fn get_filetype(file_path: &PathBuf, cookie: &Cookie) -> Result<Filetype, CheckError> {
    if !file_path.exists() {
        let msg = format!("File '{:?}' failed to open.", file_path);
        return Err(CheckError::FError(msg));
    }

    let file_type = cookie.file(file_path)?;

    // TODO: Use magic::flags::MIME_TYPE

    if file_type.contains("FLAC") {
        Ok(Filetype::FLAC)
    } else if file_type.contains("MPEG") && file_type.contains("III") {
        Ok(Filetype::MP3)
    } else if file_type.contains("Vorbis") {
        Ok(Filetype::Vorbis)
    } else if file_type.contains("Opus") {
        Ok(Filetype::Opus)
    } else if file_type.contains("WAVE") {
        Ok(Filetype::WAV)
    } else {
        Err(CheckError::FiletypeError(format!(
            "Invalid filetype '{:?}'.",
            file_path.extension()
        )))
    }
}

#[inline]
unsafe fn as_u8_slice<T: Copy>(buf: &[T]) -> &[u8] {
    ::std::slice::from_raw_parts(
        buf.as_ptr() as *const u8,
        buf.len() * ::std::mem::size_of::<T>(),
    )
}

#[inline]
fn i32_as_u8_slice(buf: &[i32]) -> &[u8] {
    unsafe { as_u8_slice(buf) }
}

#[inline]
fn i16_as_u8_slice(buf: &[i16]) -> &[u8] {
    unsafe { as_u8_slice(buf) }
}

#[inline]
unsafe fn as_i32_slice<T: Copy>(buf: &mut [T]) -> &mut [i32] {
    debug_assert_eq!(::std::mem::size_of::<T>() % 4, 0);
    ::std::slice::from_raw_parts_mut(
        buf.as_ptr() as *mut i32,
        buf.len() * (::std::mem::size_of::<T>() / 4),
    )
}

#[inline]
fn madfixed_as_i32_slice(buf: &mut [::simplemad::MadFixed32]) -> &mut [i32] {
    unsafe { as_i32_slice(buf) }
}

fn flac_hash(file_path: &PathBuf) -> Result<Checksum, CheckError> {
    let mut reader = claxon::FlacReader::open(file_path)?;

    let channels = reader.streaminfo().channels as usize;

    let mut frame_reader = reader.blocks();
    let mut block_buffer: Vec<i32> = Vec::with_capacity(0x1_0000);

    // We use a SmallVec to allocate our hashers (up to 8, because if the audio
    // file has more than 8 channels then God save us) on the stack for faster
    // access. Excess hashers will spill over to heap causing slowdown.
    let mut hashers: SmallVec<[Blake2b; 8]> =
        ::std::iter::repeat(Blake2b::new()).take(channels).collect();

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
            hasher.input(i32_as_u8_slice(chunk));
        }
    }

    Ok(Checksum::new_xor(hashers.into_iter().map(|x| x.result())))
}

fn vorbis_hash(file_path: &PathBuf) -> Result<Checksum, CheckError> {
    let f = File::open(file_path)?;
    let mut decoder = inside_ogg::OggStreamReader::new(f)?;

    let channels = decoder.ident_hdr.audio_channels as usize;

    // We use a SmallVec to allocate our hashers (up to 8, because if the audio
    // file has more than 8 channels then God save us) on the stack for faster
    // access. Excess hashers will spill over to heap causing slowdown.
    let mut hashers: SmallVec<[Blake2b; 8]> =
        ::std::iter::repeat(Blake2b::new()).take(channels).collect();

    while let Some(mut block) = decoder.read_dec_packet_itl()? {
        LittleEndian::from_slice_i16(&mut block);

        for (hasher, chunk) in hashers.iter_mut().zip(block.chunks(block.len())) {
            hasher.input(i16_as_u8_slice(chunk));
        }
    }

    Ok(Checksum::new_xor(hashers.into_iter().map(|x| x.result())))
}

fn mp3_hash(file_path: &PathBuf) -> Result<Checksum, CheckError> {
    let f = File::open(file_path)?;
    let decoder = Decoder::decode(f)?;

    // There are just 2 channels at max, so just allocate two hashers up front
    let mut hashers: SmallVec<[Blake2b; 2]> = SmallVec::from_buf(Default::default());
    let mut channels = 0;

    // Skip errored-out frames, as simplemad tries to parse metadata as audio too
    for mut frame in decoder.filter_map(|f| f.ok()) {
        channels = match frame.mode {
            MadMode::SingleChannel => 1,
            _ => 2,
        };
        for (ch, hasher) in frame
            .samples
            .iter_mut()
            .take(channels)
            .zip(hashers.iter_mut())
        {
            let frame_buffer = madfixed_as_i32_slice(ch);
            LittleEndian::from_slice_i32(frame_buffer);
            hasher.input(i32_as_u8_slice(frame_buffer));
        }
    }

    Ok(Checksum::new_xor(
        hashers.into_iter().take(channels).map(|x| x.result()),
    ))
}

pub fn hash_audio(file_path: &PathBuf) -> Result<Checksum, CheckError> {
    let cookie = Cookie::open(CookieFlags::default()).unwrap();
    find_magic(&cookie)?;
    let file_type = get_filetype(file_path, &cookie)?;
    match file_type {
        Filetype::FLAC => Ok(flac_hash(file_path)?),
        Filetype::MP3 => Ok(mp3_hash(file_path)?),
        Filetype::Vorbis => Ok(vorbis_hash(file_path)?),
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

    fn get_config(file_type: &Filetype) -> Config {
        let cfg_path = PathBuf::from("./data/hashes.toml");
        let mut input = String::new();
        File::open(cfg_path)
            .and_then(|mut f| f.read_to_string(&mut input))
            .unwrap();
        let mut cfg: HashMap<String, Config> = toml::from_str(&input).unwrap();
        match *file_type {
            Filetype::FLAC => return cfg.remove("flac").unwrap(),
            Filetype::MP3 => return cfg.remove("mp3").unwrap(),
            Filetype::Opus => return cfg.remove("opus").unwrap(),
            Filetype::Vorbis => return cfg.remove("vorbis").unwrap(),
            Filetype::WAV => return cfg.remove("wav").unwrap(),
        }
    }

    fn make_path(filename: String, extension: &Filetype) -> PathBuf {
        PathBuf::from("./data/test.flac")
            .with_file_name(filename)
            .with_extension(format!("{:?}", extension))
    }

    impl<'a> From<&'a String> for Checksum {
        fn from(s: &String) -> Self {
            let arr: [u8; 64] = FromHex::from_hex(s).unwrap();
            Checksum::new(arr)
        }
    }

    impl Filetype {
        fn iterator() -> Vec<Filetype> {
            vec![
                Filetype::FLAC,
                Filetype::MP3,
                // Filetype::Vorbis,
                // Filetype::Opus,
                // Filetype::WAV,
            ]
        }
    }

    #[test]
    fn test_magic() {
        let cookie = Cookie::open(CookieFlags::default()).unwrap();
        find_magic(&cookie).unwrap();
        for format in &Filetype::iterator() {
            let cfg = get_config(&format).into_iter();
            for pair in cfg {
                let file_path = make_path(pair.0, format);
                assert_eq!(get_filetype(&file_path, &cookie).unwrap(), *format);
            }
        }
    }

    #[test]
    fn test_flac_hash() {
        let cfg = get_config(&Filetype::FLAC);
        for pair in cfg.into_iter() {
            let file_path = make_path(pair.0, &Filetype::FLAC);
            let check = flac_hash(&file_path).unwrap();
            assert_eq!(check, Checksum::from(&pair.1));
        }
    }

    #[test]
    fn test_mp3_hash() {
        let cfg = get_config(&Filetype::MP3);
        for pair in cfg.into_iter() {
            let file_path = make_path(pair.0, &Filetype::MP3);
            let check = mp3_hash(&file_path).unwrap();
            assert_eq!(check, Checksum::from(&pair.1));
        }
    }

    #[test]
    fn test_vorbis_hash() {
        let cfg = get_config(&Filetype::Vorbis);
        for pair in cfg.into_iter() {
            let file_path = make_path(pair.0, &Filetype::Vorbis);
            let check = vorbis_hash(&file_path).unwrap();
            assert_eq!(check, Checksum::from(&pair.1));
        }
    }
}
