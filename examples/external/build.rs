fn main() {
    // Build the main Zig library
    autozig_build::build("src").expect("Failed to build Zig code");
    
    // Build Zig tests from zig/ directory
    let test_exes = autozig_build::build_tests("zig").expect("Failed to build Zig tests");
    
    // Print test executable paths for reference
    for test_exe in &test_exes {
        println!("cargo:warning=Test executable built: {}", test_exe.display());
    }
}