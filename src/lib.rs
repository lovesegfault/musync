extern crate blake2;
extern crate byteorder;
extern crate claxon;
extern crate hex;
extern crate lewton;
extern crate magic;
extern crate simplemad;
extern crate simplemad_sys;
extern crate smallvec;
extern crate time;
extern crate twox_hash;

mod checksum;

use std::path::PathBuf;

pub fn check_file(songs: &Vec<PathBuf>) {
    println!("Performing (fast) file hashes: ");
    for song in songs {
        let mut sample_times = [0i64; 32];
        let mut file_hashes: Vec<checksum::Checksum> = Vec::with_capacity(sample_times.len());

        println!("File: {:?}", song);
        for sample_time in &mut sample_times {
            let start = time::PreciseTime::now();
            let check = match checksum::hash_file(song) {
                Err(why) => panic!("{}", why),
                Ok(check) => check,
            };
            let end = time::PreciseTime::now();
            *sample_time = start.to(end).num_nanoseconds().unwrap();
            file_hashes.push(check);
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
        file_hashes.dedup();
        println!("---- Hash: {:?}", file_hashes);
        if file_hashes.len() != 1 {
            panic!("Bug! Inconsistent hash calculation.")
        }
        println!(
            "---- Min: {}\n---- Avg: {}\n---- Max: {}\n---- Stddev: {}",
            min, avg, max, stddev
        );
    }
}

pub fn check_audio(songs: &Vec<PathBuf>) {
    println!("Performing (slow) audio hashes: ");
    for song in songs {
        let mut sample_times = [0i64; 32];
        let mut audio_hashes: Vec<checksum::Checksum> = Vec::with_capacity(sample_times.len());

        println!("File: {:?}", song);
        for sample_time in &mut sample_times {
            let start = time::PreciseTime::now();
            let check = match checksum::hash_audio(song) {
                Err(why) => panic!("{}", why),
                Ok(check) => check,
            };
            let end = time::PreciseTime::now();
            *sample_time = start.to(end).num_nanoseconds().unwrap();
            audio_hashes.push(check);
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
        audio_hashes.dedup();
        println!("---- Hash: {:?}", audio_hashes);
        if audio_hashes.len() != 1 {
            panic!("Bug! Inconsistent hash calculation.")
        }
        println!(
            "---- Min: {}\n---- Avg: {}\n---- Max: {}\n---- Stddev: {}",
            min, avg, max, stddev
        );
    }
}
