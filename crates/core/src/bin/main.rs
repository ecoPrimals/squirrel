//! Entry point for the Squirrel Core binary.

#[cfg(not(feature = "di-tests"))]
use squirrel_core::error::Result;

// The `app` module has been moved to squirrel-app crate
#[cfg(not(feature = "di-tests"))]
fn main() -> Result<()> {
    println!("Squirrel Core");
    println!("Note: Application code moved to squirrel-app crate");
    
    Ok(())
}

#[cfg(feature = "di-tests")]
fn main() {
    println!("Di-tests mode active");
    println!("Note: Application code moved to squirrel-app crate");
} 