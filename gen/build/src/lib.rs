//! Build script support for autozig
//!
//! This crate provides helper functions for use in build.rs scripts

#![forbid(unsafe_code)]

use std::{
    env,
    path::PathBuf,
};

use anyhow::Result;
use autozig_engine::{
    AutoZigEngine,
    BuildOutput,
};

pub mod simd;

// Re-export CompilationMode for user convenience
pub use autozig_engine::CompilationMode;
pub use simd::{
    detect_and_report,
    SimdConfig,
};

/// Builder for autozig in build.rs
pub struct Builder {
    src_dir: PathBuf,
    mode: CompilationMode,
}

impl Builder {
    /// Create a new builder with default mode (ModularBuildZig)
    ///
    /// # Arguments
    /// * `src_dir` - The source directory to scan for autozig! macros (usually
    ///   "src")
    pub fn new(src_dir: impl Into<PathBuf>) -> Self {
        Self {
            src_dir: src_dir.into(),
            mode: CompilationMode::default(),
        }
    }

    /// Set compilation mode
    ///
    /// # Arguments
    /// * `mode` - The compilation mode to use:
    ///   - `CompilationMode::Merged` - Legacy mode (merge all files)
    ///   - `CompilationMode::ModularImport` - Modular with @import
    ///   - `CompilationMode::ModularBuildZig` - Modular with build.zig
    ///     (recommended, default)
    ///
    /// # Example
    /// ```rust,no_run
    /// use autozig_build::{
    ///     Builder,
    ///     CompilationMode,
    /// };
    ///
    /// Builder::new("src")
    ///     .mode(CompilationMode::ModularBuildZig)
    ///     .build()
    ///     .expect("Build failed");
    /// ```
    pub fn mode(mut self, mode: CompilationMode) -> Self {
        self.mode = mode;
        self
    }

    /// Run the build process
    ///
    /// This will:
    /// 1. Scan source files for autozig! macros
    /// 2. Extract and compile Zig code
    /// 3. Generate FFI bindings
    /// 4. Configure cargo to link the generated library
    pub fn build(&self) -> Result<BuildOutput> {
        // Get OUT_DIR from environment
        let out_dir = env::var("OUT_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("target/debug/build"));

        // Create and run engine with specified mode
        let engine = AutoZigEngine::with_mode(&self.src_dir, &out_dir, self.mode);
        engine.build()
    }
}

/// Convenience function for simple build scripts (uses default ModularBuildZig
/// mode)
///
/// # Example
///
/// ```rust,no_run
/// // In build.rs:
/// fn main() -> anyhow::Result<()> {
///     autozig_build::build("src")?;
///     Ok(())
/// }
/// ```
pub fn build(src_dir: impl Into<PathBuf>) -> Result<BuildOutput> {
    Builder::new(src_dir).build()
}

/// Build with specific compilation mode
///
/// # Example
///
/// ```rust,no_run
/// use autozig_build::CompilationMode;
///
/// fn main() -> anyhow::Result<()> {
///     // Use modular build.zig mode (recommended)
///     autozig_build::build_with_mode("src", CompilationMode::ModularBuildZig)?;
///     Ok(())
/// }
/// ```
pub fn build_with_mode(src_dir: impl Into<PathBuf>, mode: CompilationMode) -> Result<BuildOutput> {
    Builder::new(src_dir).mode(mode).build()
}

/// Compile Zig test executables from .zig files in a directory
///
/// This will find all .zig files in the specified directory and compile their
/// tests. Test executables will be placed in OUT_DIR with the naming pattern:
/// test_{filename}
///
/// # Arguments
/// * `zig_dir` - Directory containing .zig files with test blocks
///
/// # Example
///
/// ```rust,no_run
/// // In build.rs:
/// fn main() -> anyhow::Result<()> {
///     autozig_build::build("src")?;
///     autozig_build::build_tests("zig")?; // Compile tests from zig/ directory
///     Ok(())
/// }
/// ```
pub fn build_tests(zig_dir: impl Into<PathBuf>) -> Result<Vec<PathBuf>> {
    use std::fs;

    use autozig_engine::ZigCompiler;

    let zig_dir = zig_dir.into();
    let out_dir = env::var("OUT_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("target/debug/build"));

    let compiler = ZigCompiler::new();
    let mut test_executables = Vec::new();

    // Find all .zig files
    if !zig_dir.exists() {
        println!("cargo:warning=Zig directory not found: {}", zig_dir.display());
        return Ok(test_executables);
    }

    for entry in fs::read_dir(&zig_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("zig") {
            let file_stem = path.file_stem().unwrap().to_str().unwrap();
            let test_exe = out_dir.join(format!("test_{}", file_stem));

            println!("cargo:warning=Building Zig tests for: {}", path.display());

            // Compile tests
            compiler.compile_tests(&path, &test_exe, "native")?;

            test_executables.push(test_exe);

            println!("cargo:rerun-if-changed={}", path.display());
        }
    }

    println!("cargo:warning=Built {} Zig test executables", test_executables.len());

    Ok(test_executables)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_creation() {
        let builder = Builder::new("src");
        assert_eq!(builder.src_dir, PathBuf::from("src"));
    }
}
