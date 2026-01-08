fn main() {
    println!("cargo:rerun-if-changed=src/");
    println!("cargo:rerun-if-changed=build.rs");

    // Use modular_buildzig mode
    std::env::set_var("AUTOZIG_MODE", "modular_buildzig");

    autozig_build::build("src").expect("Failed to build Zig code");
}
