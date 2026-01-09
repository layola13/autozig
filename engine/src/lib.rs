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

pub use scanner::{
    CompilationMode,
    ScanResult,
    ZigCodeScanner,
};
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

        // Generate ABI lowering wrappers and modify original code
        let (modified_code, abi_wrappers) =
            self.generate_abi_lowering_with_modified_code(&[zig_code.clone()]);

        // Combine modified code with ABI wrappers
        let mut complete_code = if modified_code.is_empty() {
            zig_code.clone()
        } else {
            modified_code
        };

        if !abi_wrappers.is_empty() {
            complete_code.push_str("\n\n");
            complete_code.push_str("// ABI Lowering: Pointer-based wrappers for struct returns\n");
            complete_code.push_str("// These wrappers ensure cross-platform ABI compatibility\n");
            complete_code.push_str(&abi_wrappers);
        }

        let code_hash = format!("{:x}", Sha256::digest(&complete_code));
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
        fs::write(&zig_file, &complete_code).context("Failed to write Zig source file")?;

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
            fs::copy(file, &dest).with_context(|| format!("Failed to copy {}", file.display()))?;
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
        let build_zig =
            self.generate_build_zig_with_c(&embedded_code, &copied_files, &copied_c_files)?;
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
        let has_std_import = embedded_code
            .iter()
            .any(|code| code.contains("const std = @import") || code.contains("const std=@import"));

        if !has_std_import {
            main.push_str("const std = @import(\"std\");\n\n");

            // Global allocator (defined once to avoid duplication)
            main.push_str("// Global allocator - defined once\n");
            main.push_str("pub var g_allocator: std.mem.Allocator = undefined;\n\n");
        }

        // Import external modules and force export of their symbols
        // This ensures that export functions in imported modules are included in the
        // final binary
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

        // Generate ABI lowering wrappers for struct returns
        let abi_wrappers = self.generate_abi_lowering_wrappers(embedded_code);
        if !abi_wrappers.is_empty() {
            main.push_str("// ABI Lowering: Pointer-based wrappers for struct returns\n");
            main.push_str("// These wrappers ensure cross-platform ABI compatibility\n");
            main.push_str(&abi_wrappers);
            main.push_str("\n");
        }

        Ok(main)
    }

    /// Generate ABI lowering wrappers for functions returning structs
    /// Transforms: export fn foo() -> StructType
    /// Into: export fn foo__autozig_ptr() -> *const StructType
    fn generate_abi_lowering_wrappers(&self, embedded_code: &[String]) -> String {
        let mut wrappers = String::new();

        for code in embedded_code {
            // Extract all export functions that return non-primitive types
            let export_fns = extract_export_functions(code);

            for export_fn in export_fns {
                if needs_abi_wrapper(&export_fn.return_type) {
                    let wrapper = generate_ptr_wrapper(&export_fn);
                    wrappers.push_str(&wrapper);
                    wrappers.push_str("\n");
                }
            }
        }

        wrappers
    }

    /// Generate ABI lowering wrappers with correct handling for arrays vs
    /// structs Returns (modified_code, wrappers)
    /// CRITICAL FIX:
    /// - Arrays: Rename impl to _impl, generate export wrapper returning
    ///   pointer (macro expects this)
    /// - Structs: Keep export AND add __autozig_ptr wrapper (dual export for
    ///   compatibility)
    fn generate_abi_lowering_with_modified_code(
        &self,
        embedded_code: &[String],
    ) -> (String, String) {
        let mut wrappers = String::new();
        let mut modified_code = String::new();

        for code in embedded_code {
            // Extract all export functions that return non-primitive types
            let export_fns = extract_export_functions(code);
            let mut functions_to_rename = Vec::new();

            for export_fn in export_fns {
                if needs_abi_wrapper(&export_fn.return_type) {
                    if must_use_wrapper(&export_fn.return_type) {
                        // Arrays: rename to _impl, generate pointer-returning export with original
                        // name
                        functions_to_rename.push(export_fn.name.clone());
                        let wrapper = generate_array_pointer_wrapper(&export_fn);
                        wrappers.push_str(&wrapper);
                        wrappers.push_str("\n");
                    } else {
                        // Structs: keep export, add __autozig_ptr wrapper
                        let wrapper = generate_ptr_wrapper(&export_fn);
                        wrappers.push_str(&wrapper);
                        wrappers.push_str("\n");
                    }
                }
            }

            // Rename array-returning functions to _impl variants
            if functions_to_rename.is_empty() {
                modified_code = code.clone();
            } else {
                modified_code = rename_functions_to_impl(code, &functions_to_rename);
            }
        }

        (modified_code, wrappers)
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
                        "    lib.addCSourceFile(.{{ .file = b.path(\"{}\"), .flags = \
                         &.{{\"-fno-sanitize=undefined\"}} }});\n",
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
        "wasm64-unknown-unknown" => "wasm64-freestanding",
        "wasm64-wasi" => "wasm64-wasi",

        // Default to native
        _ => "native",
    }
}

/// Representation of an exported Zig function
#[derive(Debug, Clone)]
struct ExportFunction {
    name: String,
    params: String,
    return_type: String,
}

/// Extract export function declarations from Zig code
fn extract_export_functions(zig_code: &str) -> Vec<ExportFunction> {
    let mut functions = Vec::new();

    // Scanner removes newlines, so code is all on one line
    // Search for all occurrences of "export fn"
    let mut pos = 0;
    while let Some(start) = zig_code[pos..].find("export fn ") {
        let actual_start = pos + start;
        // Find the portion from "export fn" onwards
        let remainder = &zig_code[actual_start..];

        if let Some(func) = parse_export_function(remainder, &[], 0) {
            functions.push(func);
        }

        // Move past this occurrence
        pos = actual_start + 10; // length of "export fn "
    }

    functions
}

/// Parse a single export function from Zig code
fn parse_export_function(line: &str, _lines: &[&str], _idx: usize) -> Option<ExportFunction> {
    // Pattern: export fn name(params) ReturnType {
    let line = line.trim();

    if !line.starts_with("export fn ") {
        return None;
    }

    // Extract function name
    let after_fn = line.strip_prefix("export fn ")?;
    let paren_pos = after_fn.find('(')?;
    let name = after_fn[..paren_pos].trim().to_string();

    // Extract parameters (everything between ( and ))
    let after_paren_start = &after_fn[paren_pos + 1..];
    let mut paren_count = 1;
    let mut params_end = 0;

    for (i, ch) in after_paren_start.chars().enumerate() {
        match ch {
            '(' => paren_count += 1,
            ')' => {
                paren_count -= 1;
                if paren_count == 0 {
                    params_end = i;
                    break;
                }
            },
            _ => {},
        }
    }

    let params = after_paren_start[..params_end].trim().to_string();

    // Extract return type (between ) and {)
    let after_params = &after_paren_start[params_end + 1..];
    let brace_pos = after_params.find('{')?;
    let return_type = after_params[..brace_pos].trim().to_string();

    Some(ExportFunction { name, params, return_type })
}

/// Check if a Zig type needs ABI wrapper (not a primitive)
/// All non-primitive types (structs, enums, etc.) need ABI wrappers for
/// cross-platform compatibility
fn needs_abi_wrapper(zig_type: &str) -> bool {
    let zig_type = zig_type.trim();

    // Check for array types [N]T - these always need wrappers
    if zig_type.starts_with('[') && zig_type.contains(']') {
        return true;
    }

    // Whitelist of safe primitive types - only these can be returned by value
    // All other types (structs, enums, etc.) need ABI wrappers
    if matches!(
        zig_type,
        "void"
            | "i8"
            | "i16"
            | "i32"
            | "i64"
            | "i128"
            | "u8"
            | "u16"
            | "u32"
            | "u64"
            | "u128"
            | "usize"
            | "isize"
            | "f32"
            | "f64"
            | "bool"
            | "c_int"
            | "c_uint"
            | "c_void"
    ) {
        return false;
    }

    // All other types (structs, enums, custom types) need ABI wrappers
    // This ensures the engine generates wrappers that the macro expects
    true
}

/// Check if a type MUST use wrapper (cannot be exported directly due to Zig ABI
/// restrictions) CRITICAL: Arrays violate Zig's C ABI calling convention and
/// cause compilation errors Structs CAN be exported (though ABI may be
/// unstable), so we allow dual export
fn must_use_wrapper(zig_type: &str) -> bool {
    let zig_type = zig_type.trim();

    // Arrays MUST use wrappers - Zig refuses to compile export functions returning
    // arrays Error: "return type '[N]T' not allowed in function with calling
    // convention 'x86_64_sysv'"
    if zig_type.starts_with('[') && zig_type.contains(']') {
        return true;
    }

    // Structs CAN be exported (return false here to keep dual export)
    // Even large structs like Sprite, TextureAtlas are allowed by Zig compiler
    // The wrapper provides ABI-safe alternative, but original export is kept for
    // compatibility
    false
}

/// Generate pointer-based wrapper for a function returning struct
fn generate_ptr_wrapper(func: &ExportFunction) -> String {
    let wrapper_name = format!("{}__autozig_ptr", func.name);
    let return_ptr_type = format!("*const {}", func.return_type);

    // Convert struct parameters to pointers
    let (wrapper_params, forwarding_args) = convert_params_to_ptrs(&func.params);

    // Generate static storage for the return value
    format!(
        "export fn {}({}) {} {{\n    // ABI-safe wrapper: returns pointer instead of struct by \
         value\n    const static = struct {{\n        var result: {} = undefined;\n    }};\n    \
         static.result = {}({});\n    return &static.result;\n}}",
        wrapper_name, wrapper_params, return_ptr_type, func.return_type, func.name, forwarding_args
    )
}

/// Convert struct parameters to pointer parameters
/// Returns (wrapper_params, forwarding_args)
fn convert_params_to_ptrs(params: &str) -> (String, String) {
    if params.trim().is_empty() {
        return (String::new(), String::new());
    }

    let mut wrapper_params = Vec::new();
    let mut forwarding_args = Vec::new();

    for param in params.split(',') {
        let param = param.trim();
        if param.is_empty() {
            continue;
        }

        // Pattern: "name : Type"
        if let Some((name, type_part)) = param.split_once(':') {
            let name = name.trim();
            let param_type = type_part.trim();

            // Check if parameter type needs ABI wrapping (is a struct)
            if needs_abi_wrapper(param_type) {
                // Convert to pointer: "name: Type" -> "name: *const Type"
                wrapper_params.push(format!("{} : *const {}", name, param_type));
                // Dereference when forwarding: "name" -> "name.*"
                forwarding_args.push(format!("{}.*", name));
            } else {
                // Keep primitive types as-is
                wrapper_params.push(format!("{} : {}", name, param_type));
                forwarding_args.push(name.to_string());
            }
        }
    }

    (wrapper_params.join(" , "), forwarding_args.join(", "))
}

/// Extract parameter names from parameter list for forwarding
fn extract_param_names(params: &str) -> String {
    if params.trim().is_empty() {
        return String::new();
    }

    params
        .split(',')
        .filter_map(|param| {
            let param = param.trim();
            // Pattern: "name: Type" -> extract "name"
            param.split(':').next().map(|s| s.trim())
        })
        .collect::<Vec<_>>()
        .join(", ")
}
/// Rename functions to _impl variants (for array-returning functions)
/// Pattern: "export fn function_name(" -> "fn function_name_impl("
fn rename_functions_to_impl(code: &str, function_names: &[String]) -> String {
    let mut result = code.to_string();

    for fn_name in function_names {
        // Remove export and rename to _impl
        let pattern_with_space = format!("export fn {} (", fn_name);
        let pattern_no_space = format!("export fn {}(", fn_name);
        let replacement_with_space = format!("fn {}_impl (", fn_name);
        let replacement_no_space = format!("fn {}_impl(", fn_name);

        if result.contains(&pattern_with_space) {
            result = result.replace(&pattern_with_space, &replacement_with_space);
        } else {
            result = result.replace(&pattern_no_space, &replacement_no_space);
        }
    }

    result
}

/// Generate pointer-returning export wrapper for array-returning functions
/// This creates an export function with original name that calls the _impl
/// version and returns a pointer (matching what macro expects)
fn generate_array_pointer_wrapper(func: &ExportFunction) -> String {
    let impl_name = format!("{}_impl", func.name);
    let return_ptr_type = format!("*const {}", func.return_type);

    // Convert struct parameters to pointers
    let (wrapper_params, forwarding_args) = convert_params_to_ptrs(&func.params);

    // Generate wrapper that calls _impl and returns pointer
    format!(
        "export fn {}({}) {} {{\n    // Macro expects pointer return for array types\n    const \
         static = struct {{\n        var result: {} = undefined;\n    }};\n    static.result = \
         {}({});\n    return &static.result;\n}}",
        func.name, wrapper_params, return_ptr_type, func.return_type, impl_name, forwarding_args
    )
}

/// Remove export keyword from specified functions in Zig code
fn remove_export_from_functions(code: &str, function_names: &[String]) -> String {
    let mut result = code.to_string();

    for fn_name in function_names {
        // Pattern: "export fn function_name(" -> "fn function_name("
        // Note: Zig code may have spaces compressed, so match both with/without space
        let pattern_with_space = format!("export fn {} (", fn_name);
        let pattern_no_space = format!("export fn {}(", fn_name);
        let replacement_with_space = format!("fn {} (", fn_name);
        let replacement_no_space = format!("fn {}(", fn_name);

        if result.contains(&pattern_with_space) {
            result = result.replace(&pattern_with_space, &replacement_with_space);
        } else {
            result = result.replace(&pattern_no_space, &replacement_no_space);
        }
    }

    result
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
