#![allow(unused_imports)]
extern crate blake2;
extern crate byteorder;
extern crate claxon;
extern crate magic;
#[macro_use]
extern crate serde_derive;
extern crate simplemad;
extern crate simplemad_sys;
extern crate smallvec;
extern crate time;

mod checksum;

use std::path::PathBuf;

pub fn check_songs() {
    let songs = vec![
        PathBuf::from("./data/mono-sweep-1Hz-96KHz.mp3"),
        PathBuf::from("./data/stereo-sweep-1Hz-96KHz.mp3"),
    ];

    for song in songs {
        let start = time::PreciseTime::now();
        let check = match checksum::hash_audio(&song) {
            Err(why) => panic!("{}", why),
            Ok(check) => check,
        };
        let end = time::PreciseTime::now();
        println!("{}", check);
        println!("---- {:?} took: {:?}", song, start.to(end));
    }
}
