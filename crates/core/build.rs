//! Build script for squirrel-core
//! 
//! This script generates build-time information including version, authors, and other metadata
//! that is used by the crate.

use std::io;

fn main() -> Result<(), io::Error> {
    built::write_built_file()?;
    Ok(())
} 