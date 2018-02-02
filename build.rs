extern crate pkg_config;

fn main() {
    pkg_config::probe_library("mad").expect("Failed to detect libmad");
    // TODO: Detect libmagic!
}
