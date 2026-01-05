//! Build script for autozig demo
//!
//! This demonstrates how to use autozig in a build.rs file

fn main() -> anyhow::Result<()> {
    // Scan src directory for autozig! macros and compile Zig code
    autozig_build::build("src")?;
    
    // Tell cargo to rerun if source files change
    println!("cargo:rerun-if-changed=src/main.rs");
    
    Ok(())
}