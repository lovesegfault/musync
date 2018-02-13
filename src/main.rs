extern crate musync;

use std::path::PathBuf;

fn main() {
    let songs = vec![
        PathBuf::from("./data/mono-sweep-1Hz-96KHz.flac"),
        PathBuf::from("./data/stereo-sweep-1Hz-96KHz.flac"),
    ];
    musync::check_file(&songs);
    musync::check_audio(&songs);
}
