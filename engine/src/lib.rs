//! Core engine for AutoZig code generation
//!
//! This engine handles:
//! 1. Scanning Rust source files for autozig! invocations
//! 2. Extracting Zig code
//! 3. Compiling Zig code to static libraries with incremental optimization
//! 4. Target triple mapping for cross-compilation

#![forbid(unsafe_code)]

use std::{
    env,
    fs,
    path::{
        Path,
        PathBuf,
    },
};

use anyhow::{
    Context,
    Result,
};
use sha2::{
    Digest,
    Sha256,
};

pub mod scanner;
pub mod type_mapper;
pub mod zig_compiler;

pub use scanner::ZigCodeScanner;
pub use zig_compiler::ZigCompiler;

/// Main engine for processing autozig! macros during build
pub struct AutoZigEngine {
    /// Output directory (usually OUT_DIR from build.rs)
    out_dir: PathBuf,
    /// Source directory to scan
    src_dir: PathBuf,
}

impl AutoZigEngine {
    /// Create a new AutoZig engine
    pub fn new(src_dir: impl AsRef<Path>, out_dir: impl AsRef<Path>) -> Self {
        Self {
            src_dir: src_dir.as_ref().to_path_buf(),
            out_dir: out_dir.as_ref().to_path_buf(),
        }
    }

    /// Run the complete build pipeline with incremental compilation
    pub fn build(&self) -> Result<BuildOutput> {
        // Step 1: Scan source files for autozig! macros
        println!("cargo:rerun-if-changed={}", self.src_dir.display());

        let scanner = ZigCodeScanner::new(&self.src_dir);
        let zig_code = scanner.scan()?;

        if zig_code.is_empty() {
            // No Zig code found, nothing to do
            return Ok(BuildOutput { lib_path: None });
        }

        // Step 2: Check if Zig code changed using hash
        let code_hash = format!("{:x}", Sha256::digest(&zig_code));
        let hash_file = self.out_dir.join(".zig_code_hash");
        let lib_path = self.out_dir.join("libautozig.a");

        // Incremental compilation: skip if hash unchanged and lib exists
        if hash_file.exists() && lib_path.exists() {
            if let Ok(old_hash) = fs::read_to_string(&hash_file) {
                if old_hash == code_hash {
                    println!("cargo:warning=Zig code unchanged, skipping compilation");
                    println!("cargo:rustc-link-search=native={}", self.out_dir.display());
                    println!("cargo:rustc-link-lib=static=autozig");
                    return Ok(BuildOutput { lib_path: Some(lib_path) });
                }
            }
        }

        // Step 3: Write consolidated Zig code
        let zig_file = self.out_dir.join("generated_autozig.zig");
        fs::write(&zig_file, &zig_code).context("Failed to write Zig source file")?;

        // Step 4: Get target triple for cross-compilation
        let rust_target = env::var("TARGET").unwrap_or_else(|_| "native".to_string());
        let zig_target = rust_to_zig_target(&rust_target);

        // Step 5: Compile Zig code
        let compiler = ZigCompiler::new();
        compiler.compile_with_target(&zig_file, &lib_path, zig_target)?;

        // Step 6: Save hash for incremental compilation
        fs::write(&hash_file, &code_hash).context("Failed to write hash file")?;

        // Step 7: Link the static library
        println!("cargo:rustc-link-search=native={}", self.out_dir.display());
        println!("cargo:rustc-link-lib=static=autozig");

        Ok(BuildOutput { lib_path: Some(lib_path) })
    }
}

/// Map Rust target triple to Zig target
fn rust_to_zig_target(rust_target: &str) -> &str {
    match rust_target {
        // Linux targets
        "x86_64-unknown-linux-gnu" => "x86_64-linux-gnu",
        "x86_64-unknown-linux-musl" => "x86_64-linux-musl",
        "aarch64-unknown-linux-gnu" => "aarch64-linux-gnu",
        "aarch64-unknown-linux-musl" => "aarch64-linux-musl",
        "arm-unknown-linux-gnueabihf" => "arm-linux-gnueabihf",
        "i686-unknown-linux-gnu" => "i386-linux-gnu",

        // macOS targets
        "x86_64-apple-darwin" => "x86_64-macos",
        "aarch64-apple-darwin" => "aarch64-macos",

        // Windows targets
        "x86_64-pc-windows-msvc" => "x86_64-windows",
        "x86_64-pc-windows-gnu" => "x86_64-windows-gnu",
        "i686-pc-windows-msvc" => "i386-windows",
        "aarch64-pc-windows-msvc" => "aarch64-windows",

        // WebAssembly
        "wasm32-unknown-unknown" => "wasm32-freestanding",
        "wasm32-wasi" => "wasm32-wasi",

        // Default to native
        _ => "native",
    }
}

/// Output from the build process
#[derive(Debug)]
pub struct BuildOutput {
    /// Path to the generated static library
    pub lib_path: Option<PathBuf>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_creation() {
        let engine = AutoZigEngine::new("src", "target");
        assert_eq!(engine.src_dir, PathBuf::from("src"));
        assert_eq!(engine.out_dir, PathBuf::from("target"));
    }

    #[test]
    fn test_target_mapping() {
        assert_eq!(rust_to_zig_target("x86_64-unknown-linux-gnu"), "x86_64-linux-gnu");
        assert_eq!(rust_to_zig_target("aarch64-apple-darwin"), "aarch64-macos");
        assert_eq!(rust_to_zig_target("x86_64-pc-windows-msvc"), "x86_64-windows");
        assert_eq!(rust_to_zig_target("wasm32-wasi"), "wasm32-wasi");
        assert_eq!(rust_to_zig_target("unknown-target"), "native");
    }
}
