fn main() {
    // Detect and report SIMD configuration
    let simd_config = autozig_build::detect_and_report();

    println!("cargo:warning=Detected SIMD: {}", simd_config.description);
    println!("cargo:warning=Zig will use: {}", simd_config.as_zig_flag());

    // Build Zig code
    autozig_build::build("src").expect("Failed to build Zig code");
}
