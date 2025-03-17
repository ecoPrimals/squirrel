//! Build script for squirrel-core
//! 
//! This script generates build-time information including version, authors, and other metadata
//! that is used by the crate.

fn main() {
    built::write_built_file().expect("Failed to acquire build-time information");
} 