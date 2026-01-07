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

pub use scanner::{CompilationMode, ScanResult, ZigCodeScanner};
pub use zig_compiler::ZigCompiler;

/// Main engine for processing autozig! macros during build
pub struct AutoZigEngine {
    /// Output directory (usually OUT_DIR from build.rs)
    out_dir: PathBuf,
    /// Source directory to scan
    src_dir: PathBuf,
    /// Compilation mode
    mode: CompilationMode,
}

impl AutoZigEngine {
    /// Create a new AutoZig engine with default mode
    pub fn new(src_dir: impl AsRef<Path>, out_dir: impl AsRef<Path>) -> Self {
        Self::with_mode(src_dir, out_dir, CompilationMode::default())
    }

    /// Create engine with specific compilation mode
    pub fn with_mode(
        src_dir: impl AsRef<Path>,
        out_dir: impl AsRef<Path>,
        mode: CompilationMode,
    ) -> Self {
        Self {
            src_dir: src_dir.as_ref().to_path_buf(),
            out_dir: out_dir.as_ref().to_path_buf(),
            mode,
        }
    }

    /// Run the complete build pipeline with incremental compilation
    pub fn build(&self) -> Result<BuildOutput> {
        match self.mode {
            CompilationMode::Merged => self.build_merged(),
            CompilationMode::ModularImport => self.build_modular_import(),
            CompilationMode::ModularBuildZig => self.build_modular_buildzig(),
        }
    }

    /// Legacy merged compilation mode
    fn build_merged(&self) -> Result<BuildOutput> {
        println!("cargo:rerun-if-changed={}", self.src_dir.display());
        println!("cargo:warning=Using MERGED compilation mode (legacy)");

        let scanner = ZigCodeScanner::with_mode(&self.src_dir, CompilationMode::Merged);
        let zig_code = scanner.scan()?;

        if zig_code.is_empty() {
            // No Zig code found, nothing to do
            return Ok(BuildOutput { lib_path: None });
        }

        let code_hash = format!("{:x}", Sha256::digest(&zig_code));
        let hash_file = self.out_dir.join(".zig_code_hash");
        let lib_path = self.out_dir.join("libautozig.a");

        if hash_file.exists() && lib_path.exists() {
            if let Ok(old_hash) = fs::read_to_string(&hash_file) {
                if old_hash == code_hash {
                    println!("cargo:warning=Zig code unchanged, skipping compilation");
                    self.link_library();
                    return Ok(BuildOutput { lib_path: Some(lib_path) });
                }
            }
        }

        let zig_file = self.out_dir.join("generated_autozig.zig");
        fs::write(&zig_file, &zig_code).context("Failed to write Zig source file")?;

        let rust_target = env::var("TARGET").unwrap_or_else(|_| "native".to_string());
        let zig_target = rust_to_zig_target(&rust_target);

        let compiler = ZigCompiler::new();
        compiler.compile_with_target_and_src(&zig_file, &lib_path, zig_target, &self.src_dir)?;

        fs::write(&hash_file, &code_hash).context("Failed to write hash file")?;
        self.link_library();

        Ok(BuildOutput { lib_path: Some(lib_path) })
    }

    /// Modular compilation with main module + @import
    fn build_modular_import(&self) -> Result<BuildOutput> {
        println!("cargo:rerun-if-changed={}", self.src_dir.display());
        println!("cargo:warning=Using MODULAR_IMPORT compilation mode");

        let scanner = ZigCodeScanner::with_mode(&self.src_dir, CompilationMode::ModularImport);
        let scan_result = scanner.scan_modular()?;

        let (embedded_code, external_files, all_zig_files) = match scan_result {
            ScanResult::Modular {
                embedded_code,
                external_files,
                all_zig_files,
                c_source_files: _,
            } => (embedded_code, external_files, all_zig_files),
            _ => return Err(anyhow::anyhow!("Expected modular scan result")),
        };

        if embedded_code.is_empty() && external_files.is_empty() {
            return Ok(BuildOutput { lib_path: None });
        }

        // Copy external .zig files to output directory with their original names
        let mut copied_files = Vec::new();
        for file in &external_files {
            if let Some(file_name) = file.file_name() {
                let dest = self.out_dir.join(file_name);
                fs::copy(file, &dest)
                    .with_context(|| format!("Failed to copy {}", file.display()))?;
                copied_files.push(dest);
            }
        }

        // Generate main module with @import statements using actual copied file names
        let main_zig = self.generate_main_module_with_files(&embedded_code, &copied_files)?;
        let main_file = self.out_dir.join("generated_main.zig");
        fs::write(&main_file, &main_zig).context("Failed to write main module")?;

        // Compile main module
        let lib_path = self.out_dir.join("libautozig.a");
        let rust_target = env::var("TARGET").unwrap_or_else(|_| "native".to_string());
        let zig_target = rust_to_zig_target(&rust_target);

        let compiler = ZigCompiler::new();
        compiler.compile_with_target_and_src(&main_file, &lib_path, zig_target, &self.src_dir)?;

        self.link_library();
        Ok(BuildOutput { lib_path: Some(lib_path) })
    }

    /// Modular compilation with build.zig (recommended)
    fn build_modular_buildzig(&self) -> Result<BuildOutput> {
        println!("cargo:rerun-if-changed={}", self.src_dir.display());
        println!("cargo:warning=Using MODULAR_BUILDZIG compilation mode (recommended)");

        let scanner = ZigCodeScanner::with_mode(&self.src_dir, CompilationMode::ModularBuildZig);
        let scan_result = scanner.scan_modular()?;

        let (embedded_code, external_files, _all_zig_files, c_source_files) = match scan_result {
            ScanResult::Modular {
                embedded_code,
                external_files,
                all_zig_files,
                c_source_files,
            } => (embedded_code, external_files, all_zig_files, c_source_files),
            _ => return Err(anyhow::anyhow!("Expected modular scan result")),
        };

        if embedded_code.is_empty() && external_files.is_empty() {
            return Ok(BuildOutput { lib_path: None });
        }

        // CRITICAL: Copy external .zig files FIRST and track their output paths
        // because main module will reference these files via @import
        let mut copied_files = Vec::new();
        for file in &external_files {
            let file_name = file.file_name().unwrap_or_default();
            let dest = self.out_dir.join(file_name);
            fs::copy(file, &dest)
                .with_context(|| format!("Failed to copy {}", file.display()))?;
            copied_files.push(dest);
        }

        // Copy C source files to output directory
        let mut copied_c_files = Vec::new();
        for file in &c_source_files {
            let file_name = file.file_name().unwrap_or_default();
            let dest = self.out_dir.join(file_name);
            fs::copy(file, &dest)
                .with_context(|| format!("Failed to copy C file {}", file.display()))?;
            copied_c_files.push(dest);
        }

        // Generate main module using copied file paths (now files are in place)
        let main_zig = self.generate_main_module_with_files(&embedded_code, &copied_files)?;
        let main_file = self.out_dir.join("generated_main.zig");
        fs::write(&main_file, &main_zig).context("Failed to write main module")?;

        // Generate build.zig file with C file support
        let build_zig = self.generate_build_zig_with_c(&embedded_code, &copied_files, &copied_c_files)?;
        let build_file = self.out_dir.join("build.zig");
        fs::write(&build_file, &build_zig).context("Failed to write build.zig")?;

        // Compile using build.zig
        let lib_path = self.out_dir.join("libautozig.a");
        let compiler = ZigCompiler::new();
        compiler.compile_with_buildzig(&build_file, &self.out_dir, &lib_path)?;

        self.link_library();
        Ok(BuildOutput { lib_path: Some(lib_path) })
    }

    /// Generate main module with @import statements
    fn generate_main_module(
        &self,
        embedded_code: &[String],
        all_zig_files: &[PathBuf],
    ) -> Result<String> {
        self.generate_main_module_with_files(embedded_code, all_zig_files)
    }

    /// Generate main module with @import statements using specific file list
    fn generate_main_module_with_files(
        &self,
        embedded_code: &[String],
        zig_files: &[PathBuf],
    ) -> Result<String> {
        let mut main = String::new();
        
        // Check if embedded code already contains std import to avoid duplication
        let has_std_import = embedded_code.iter().any(|code| {
            code.contains("const std = @import") || code.contains("const std=@import")
        });
        
        if !has_std_import {
            main.push_str("const std = @import(\"std\");\n\n");
            
            // Global allocator (defined once to avoid duplication)
            main.push_str("// Global allocator - defined once\n");
            main.push_str("pub var g_allocator: std.mem.Allocator = undefined;\n\n");
        }

        // Import external modules and force export of their symbols
        // This ensures that export functions in imported modules are included in the final binary
        for (idx, file) in zig_files.iter().enumerate() {
            if let Some(file_name) = file.file_name() {
                let module_name = format!("mod_{}", idx);
                main.push_str(&format!(
                    "pub const {} = @import(\"{}\");\n",
                    module_name,
                    file_name.to_string_lossy()
                ));
            }
        }
        if !zig_files.is_empty() {
            main.push_str("\n");
            main.push_str("// Force exported symbols from imported modules to be included\n");
            main.push_str("comptime {\n");
            for (idx, _) in zig_files.iter().enumerate() {
                main.push_str(&format!("    _ = mod_{};\n", idx));
            }
            main.push_str("}\n\n");
        }

        // Add embedded code
        if !embedded_code.is_empty() {
            main.push_str("// Embedded code from autozig! macros\n");
            for code in embedded_code {
                main.push_str(code);
                main.push_str("\n\n");
            }
        }

        Ok(main)
    }

    /// Generate build.zig file with C source file support
    fn generate_build_zig_with_c(
        &self,
        _embedded_code: &[String],
        all_zig_files: &[PathBuf],
        c_source_files: &[PathBuf],
    ) -> Result<String> {
        let rust_target = env::var("TARGET").unwrap_or_else(|_| "native".to_string());
        let zig_target = rust_to_zig_target(&rust_target);
        let is_wasm = zig_target.contains("wasm32");

        let mut build = String::new();
        build.push_str("const std = @import(\"std\");\n\n");
        build.push_str("pub fn build(b: *std.Build) void {\n");
        
        // Target configuration with BASELINE CPU to match zig build-lib behavior
        // This fixes the "incompatible with elf64-x86-64" linking error
        build.push_str("    // Force baseline CPU model to match Rust's expectations\n");
        build.push_str("    const target = b.resolveTargetQuery(.{\n");
        build.push_str("        .cpu_model = .baseline,  // Critical: use baseline, not native\n");
        
        if is_wasm {
            build.push_str("        .cpu_arch = .wasm32,\n");
            build.push_str("        .os_tag = .freestanding,\n");
        } else if zig_target.contains("x86_64") {
            build.push_str("        .cpu_arch = .x86_64,\n");
            if zig_target.contains("linux") {
                build.push_str("        .os_tag = .linux,\n");
                if zig_target.contains("musl") {
                    build.push_str("        .abi = .musl,\n");
                } else {
                    build.push_str("        .abi = .gnu,\n");
                }
            } else if zig_target.contains("macos") {
                build.push_str("        .os_tag = .macos,\n");
            } else if zig_target.contains("windows") {
                build.push_str("        .os_tag = .windows,\n");
                if zig_target.contains("gnu") {
                    build.push_str("        .abi = .gnu,\n");
                } else {
                    build.push_str("        .abi = .msvc,\n");
                }
            }
        } else if zig_target.contains("aarch64") {
            build.push_str("        .cpu_arch = .aarch64,\n");
            if zig_target.contains("linux") {
                build.push_str("        .os_tag = .linux,\n");
                build.push_str("        .abi = .gnu,\n");
            } else if zig_target.contains("macos") {
                build.push_str("        .os_tag = .macos,\n");
            }
        }
        
        build.push_str("    });\n");
        build.push_str("    const optimize = b.standardOptimizeOption(.{});\n\n");

        // Create module first (required by Zig 0.15.2 API)
        build.push_str("    const mod = b.addModule(\"autozig\", .{\n");
        build.push_str("        .root_source_file = b.path(\"generated_main.zig\"),\n");
        build.push_str("        .target = target,\n");
        build.push_str("        .optimize = optimize,\n");
        build.push_str("    });\n\n");

        // Create static library using addLibrary (Zig 0.15.2 API)
        build.push_str("    const lib = b.addLibrary(.{\n");
        build.push_str("        .name = \"autozig\",\n");
        build.push_str("        .root_module = mod,\n");
        build.push_str("        .linkage = .static,\n");
        build.push_str("    });\n\n");

        // Enable PIC (Position Independent Code) for compatibility with Rust
        if !is_wasm {
            build.push_str("    // Enable PIC for Rust FFI compatibility\n");
            build.push_str("    lib.root_module.pic = true;\n\n");
        }

        // WASM-specific configuration
        if is_wasm {
            build.push_str("    // WASM-specific configuration\n");
            build.push_str("    lib.root_module.stack_protector = false;\n");
            build.push_str("    lib.root_module.red_zone = false;\n");
        } else {
            build.push_str("    // Link with libc\n");
            build.push_str("    lib.linkLibC();\n");
        }

        // Add C source files if present
        if !c_source_files.is_empty() {
            build.push_str("\n    // Add C source files\n");
            for c_file in c_source_files {
                if let Some(file_name) = c_file.file_name() {
                    build.push_str(&format!(
                        "    lib.addCSourceFile(.{{ .file = b.path(\"{}\"), .flags = &.{{\"-fno-sanitize=undefined\"}} }});\n",
                        file_name.to_string_lossy()
                    ));
                }
            }
        }

        build.push_str("\n    b.installArtifact(lib);\n");
        build.push_str("}\n");

        Ok(build)
    }

    /// Generate build.zig file for modular compilation (Zig 0.15.2 compatible)
    /// Legacy version without C file support
    fn generate_build_zig(
        &self,
        embedded_code: &[String],
        all_zig_files: &[PathBuf],
    ) -> Result<String> {
        // Delegate to version with empty C files
        self.generate_build_zig_with_c(embedded_code, all_zig_files, &[])
    }

    /// Link the static library
    fn link_library(&self) {
        println!("cargo:rustc-link-search=native={}", self.out_dir.display());
        println!("cargo:rustc-link-lib=static=autozig");
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
