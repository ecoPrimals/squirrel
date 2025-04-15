use std::fs;
use std::io::{self, Read, Write};

fn main() -> io::Result<()> {
    // Read the file
    let mut file = fs::File::open("crates/mcp/src/error.rs")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    
    // Replace the problematic line
    let fixed_contents = contents.replace("impl From<e> for MCPError", "impl From<Error> for MCPError");
    
    // Write the fixed content back
    let mut output_file = fs::File::create("crates/mcp/src/error.rs")?;
    output_file.write_all(fixed_contents.as_bytes())?;
    
    println!("File fixed successfully");
    Ok(())
} 