fn main() {
    println!("cargo:rerun-if-changed=src/zig/memory.zig");
    // Ensure we are using the modular build mode
    std::env::set_var("AUTOZIG_MODE", "modular_buildzig");
    autozig_build::build("src").expect("Failed to build Zig code");
}
