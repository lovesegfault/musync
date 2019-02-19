use musync;

use std::path::PathBuf;

fn main() {
    let songs = vec![
        PathBuf::from("./data/mono-sweep-1Hz-96KHz.flac"),
        PathBuf::from("./data/stereo-sweep-1Hz-96KHz.flac"),
        PathBuf::from("./data/stereo-sweep-1Hz-96KHz-2.flac"),
    ];

    musync::bench_checksum(
        "Performing (fast) file hashes: ".to_owned(),
        &songs,
        &musync::checksum::hash_file,
    );

    musync::bench_checksum(
        "Performing (slow) audio hashes: ".to_owned(),
        &songs,
        &musync::checksum::hash_audio,
    );
}
