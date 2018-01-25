extern crate sled;
extern crate time;

mod db;
mod checksum;

use std::path::PathBuf;

fn main() {
    //println!("Hello world!");
    //let srv_db = db::DB::open("./serv.db".to_owned());
    let songs = vec![
        PathBuf::from("/home/bemeurer/src/musync/data/sweep-1Hz-96KHz.flac"),
        //PathBuf::from("/home/bemeurer/src/musync/data/sweep-1Hz-96KHz.mp3"),
        //PathBuf::from("/home/bemeurer/src/musync/data/sweep-1Hz-96KHz.ogg"),
        //PathBuf::from("/home/bemeurer/src/musync/data/sweep-1Hz-96KHz.opus"),
        //PathBuf::from("/home/bemeurer/src/musync/data/sweep-1Hz-96KHz.wav"),
    ];

    for song in songs {
        let start = time::PreciseTime::now();
        checksum::check_file(song);
        let end = time::PreciseTime::now();
        println!("Took: {}", start.to(end));
    }
}
