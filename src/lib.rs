#![allow(unused_imports)]
extern crate blake2;
extern crate byteorder;
extern crate claxon;
extern crate lewton;
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
        let mut sample_times = [0i64; 32];
        let mut hashes: Vec<checksum::Checksum> = Vec::with_capacity(sample_times.len());

        println!("---- song: {:?}", song);
        for sample_time in &mut sample_times {
            let start = time::PreciseTime::now();
            let check = match checksum::hash_audio(&song) {
                Err(why) => panic!("{}", why),
                Ok(check) => check,
            };
            let end = time::PreciseTime::now();
            *sample_time = start.to(end).num_nanoseconds().unwrap();
            hashes.push(check);
        }
        let min = sample_times.iter().min().unwrap();
        let max = sample_times.iter().max().unwrap();
        let avg: i64 = sample_times.iter().sum::<i64>() / sample_times.len() as i64;
        let stddev = (sample_times
            .iter()
            .map(|v| (v - avg) as f64)
            .map(|v| v * v)
            .sum::<f64>() / (sample_times.len() - 1) as f64)
            .sqrt() as i64;
        hashes.dedup();
        println!("hash: {:?}", hashes);
        if hashes.len() != 1 {
            panic!("Bug! Inconsistent hash calculation.")
        }
        println!(
            "---- min: {}; avg: {}; max: {}; stddev: {}",
            min, avg, max, stddev
        );
    }
}
