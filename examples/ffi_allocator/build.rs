//! FFI Allocator Example
//!
//! Demonstrates the correct pattern for memory allocation in Zig FFI functions.
//!
//! Key points:
//! 1. Use local GeneralPurposeAllocator in each Zig file
//! 2. Return non-optional pointers with `catch unreachable`
//! 3. Use raw pointers in Rust FFI (not Option<*mut T>)

fn main() {
    println!("cargo:rerun-if-changed=src/");
    println!("cargo:rerun-if-changed=build.rs");

    // Clean potentially corrupted files before build
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let lib_path = std::path::Path::new(&out_dir).join("libautozig.a");
    if lib_path.exists() {
        let _ = std::fs::remove_file(&lib_path);
    }

    // Use modular_buildzig mode (recommended)
    std::env::set_var("AUTOZIG_MODE", "modular_buildzig");

    autozig_build::build("src").expect("Failed to build Zig code");
}
