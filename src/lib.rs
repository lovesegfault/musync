pub mod checksum;

use crate::checksum::Result;
use pretty_toa::ThousandsSep;
use std::io::Write;
use std::path::PathBuf;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

pub fn print_res(samples: &[i64; 32]) {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);

    let min = samples.iter().min().unwrap();
    let max = samples.iter().max().unwrap();
    let avg: i64 = samples.iter().sum::<i64>() / samples.len() as i64;
    let stddev = (samples
        .iter()
        .map(|v| (v - avg) as f64)
        .map(|v| v * v)
        .sum::<f64>()
        / (samples.len() - 1) as f64)
        .sqrt() as i64;
    stdout
        .set_color(ColorSpec::new().set_fg(Some(Color::White)))
        .unwrap();
    write!(&mut stdout, "---- ").unwrap();
    stdout
        .set_color(ColorSpec::new().set_fg(Some(Color::Green)))
        .unwrap();
    write!(&mut stdout, "Min: {}", min.thousands_sep()).unwrap();
    stdout
        .set_color(ColorSpec::new().set_fg(Some(Color::White)))
        .unwrap();
    write!(&mut stdout, " | ").unwrap();
    stdout
        .set_color(ColorSpec::new().set_fg(Some(Color::Red)))
        .unwrap();
    write!(&mut stdout, "Max: {}", max.thousands_sep()).unwrap();
    stdout
        .set_color(ColorSpec::new().set_fg(Some(Color::White)))
        .unwrap();
    write!(&mut stdout, " | ").unwrap();
    stdout
        .set_color(ColorSpec::new().set_fg(Some(Color::Yellow)))
        .unwrap();
    writeln!(&mut stdout, "Avg: {}", avg.thousands_sep()).unwrap();
    stdout
        .set_color(ColorSpec::new().set_fg(Some(Color::White)))
        .unwrap();
    write!(&mut stdout, "---- ").unwrap();
    stdout
        .set_color(ColorSpec::new().set_fg(Some(Color::Magenta)))
        .unwrap();
    writeln!(&mut stdout, "Std. Deviation: {} ", stddev.thousands_sep()).unwrap();
}

pub fn bench_checksum(msg: String, songs: &Vec<PathBuf>, f: &dyn Fn(&PathBuf) -> Result) {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    stdout
        .set_color(ColorSpec::new().set_fg(Some(Color::White)))
        .unwrap();
    writeln!(&mut stdout, "{}", msg).unwrap();

    for song in songs {
        let mut sample_times = [0i64; 32];
        let mut hashes: Vec<checksum::Checksum> = Vec::with_capacity(sample_times.len());

        stdout
            .set_color(ColorSpec::new().set_fg(Some(Color::White)))
            .unwrap();
        writeln!(&mut stdout, "-- File: {:?}", song).unwrap();

        for sample_time in &mut sample_times {
            let start = time::PreciseTime::now();
            let check = match f(song) {
                Err(why) => panic!("{}", why),
                Ok(check) => check,
            };
            let end = time::PreciseTime::now();
            *sample_time = start.to(end).num_nanoseconds().unwrap();
            hashes.push(check);
        }
        hashes.dedup();
        if hashes.len() != 1 {
            panic!("Bug! Inconsistent hash calculation.\n Hashes: {:?}", hashes)
        }
        stdout
            .set_color(ColorSpec::new().set_fg(Some(Color::White)))
            .unwrap();
        write!(&mut stdout, "---- ").unwrap();
        stdout
            .set_color(ColorSpec::new().set_fg(Some(Color::Blue)))
            .unwrap();
        writeln!(&mut stdout, "Hash: {}", hashes[0]).unwrap();
        print_res(&sample_times);
    }
}
