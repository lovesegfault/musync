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

"well there are some speeding tricks we can try still, like doing shifted writing with sequential reading instead of shifted reading with sequential writing, profiling etc. but I'm not up to it now, and can't really help not having the whole code."