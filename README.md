# Musync \[WIP\]

Synchronizing made easy

Musync makes keeping you music library and your devices easy. It's a simple CLI
for transcoding your library while keeping your metadata and folder structure
intact. Musync is built with Rust for safety and speed.

## Features

* Incremental changes
* Metadata preservation
* Aggressive multithreading
* Fast and memory safe

## TODO
- [ ] Hashing / UUIDs
  - [ ] Audio hashes
    - [X] FLAC
    - [X] MP3
    - [X] Vorbis
    - [ ] Opus
    - [ ] Wave
  - [ ] Remove libmagic dependency, do filetype ID ourselves
  - [ ] Remove libmad dependency, make a pure-rust MP3 dec/enc
  - [ ] Currently stereo hashes of dual-mono arrangements cause the hash to be 0..0 because of the xoring strategy. Fix that.
  - [ ] Implement fast (unsafe) hashes using xxHash
    * The idea is that the fast hashes work as a soft UUID that we can use to match the files. Iff one of them is missing, it means the file either got deleted, or that the file was changed, in which case we can use the audio-stream real UUID to find its new location, etc.
- [ ] Database
  - [ ] Try using sled, if that doesn't work go to sqlite3 bindings
  - [ ] Server-side
    - [ ] Path
    - [ ] File hash (soft UUID)
    - [ ] Audio hash (true UUID)
  - [ ] Client-side
    - [ ] Client Path
    - [ ] Server Path
    - [ ] Server soft UUID
    - [ ] Server true UUID
    - [ ] Client soft UUID
    - [ ] Client true UUID
- [ ] Synchronizing
    - [ ] ?
- [ ] Decoding / Encoding
    - [ ] ?
