use autozig_build::CompilationMode;

fn main() {
    // Note: ModularBuildZig mode has linking issues with zig build output
    // Using Merged mode for now, which works correctly
    autozig_build::build_with_mode("src", CompilationMode::Merged)
        .expect("Failed to build Zig code");
}