fn main() {
    // Build Zig code for zero-copy buffer example
    autozig_build::build("src").expect("Failed to build Zig code");

    // Optionally detect SIMD for optimization
    let _simd_config = autozig_build::detect_and_report();
}
