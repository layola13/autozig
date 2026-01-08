//! Source code scanner to extract Zig code from autozig! macros using syn AST
//! parsing

use std::{
    collections::HashSet,
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
use syn::{
    visit::Visit,
    Macro,
};
use walkdir::WalkDir;

/// Compilation mode for Zig code
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompilationMode {
    /// Legacy mode: merge all Zig code into one file (default for backward
    /// compatibility)
    Merged,
    /// Modular mode with main module + @import (Solution 1)
    ModularImport,
    /// Modular mode with build.zig (Solution 2, recommended)
    ModularBuildZig,
}

impl Default for CompilationMode {
    fn default() -> Self {
        // Read from AUTOZIG_MODE environment variable
        // Valid values: "merged", "modular_import", "modular_buildzig"
        match std::env::var("AUTOZIG_MODE").as_deref() {
            Ok("merged") => CompilationMode::Merged,
            Ok("modular_import") => CompilationMode::ModularImport,
            Ok("modular_buildzig") => CompilationMode::ModularBuildZig,
            _ => {
                // Default to Merged if not set or invalid
                CompilationMode::Merged
            },
        }
    }
}

/// Result of scanning, containing either merged code or modular file
/// information
#[derive(Debug)]
pub enum ScanResult {
    /// Merged Zig code (legacy mode)
    Merged(String),
    /// Modular files with paths
    Modular {
        /// Embedded Zig code snippets from autozig! macros
        embedded_code: Vec<String>,
        /// External .zig file paths
        external_files: Vec<PathBuf>,
        /// All unique Zig files to be compiled
        all_zig_files: Vec<PathBuf>,
        /// C source files to be compiled and linked
        c_source_files: Vec<PathBuf>,
    },
}

/// Scanner for extracting Zig code from Rust source files
pub struct ZigCodeScanner {
    src_dir: std::path::PathBuf,
    manifest_dir: std::path::PathBuf,
    mode: CompilationMode,
}

impl ZigCodeScanner {
    pub fn new(src_dir: impl AsRef<Path>) -> Self {
        Self::with_mode(src_dir, CompilationMode::default())
    }

    /// Create scanner with specific compilation mode
    pub fn with_mode(src_dir: impl AsRef<Path>, mode: CompilationMode) -> Self {
        // Get manifest dir from environment or use src_dir parent
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
            .ok()
            .and_then(|d| std::path::PathBuf::from(d).canonicalize().ok())
            .unwrap_or_else(|| {
                src_dir
                    .as_ref()
                    .parent()
                    .unwrap_or(src_dir.as_ref())
                    .to_path_buf()
            });

        Self {
            src_dir: src_dir.as_ref().to_path_buf(),
            manifest_dir,
            mode,
        }
    }

    /// Get the compilation mode
    pub fn mode(&self) -> CompilationMode {
        self.mode
    }

    /// Scan all .rs files and extract Zig code using AST parsing
    /// Returns merged code string for backward compatibility
    pub fn scan(&self) -> Result<String> {
        match self.scan_modular()? {
            ScanResult::Merged(code) => Ok(code),
            ScanResult::Modular { embedded_code, .. } => {
                // Fallback: merge embedded code for compatibility
                Ok(embedded_code.join("\n"))
            },
        }
    }

    /// Scan with modular support - returns ScanResult based on mode
    pub fn scan_modular(&self) -> Result<ScanResult> {
        let mut embedded_code = Vec::new();
        let mut external_files = Vec::new();
        let mut all_zig_files = HashSet::new();
        let mut c_source_files = HashSet::new();

        // Scan all Rust files for autozig! macros
        for entry in WalkDir::new(&self.src_dir)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "rs") {
                let content = fs::read_to_string(path)
                    .with_context(|| format!("Failed to read {}", path.display()))?;

                // Parse the Rust file into an AST
                match syn::parse_file(&content) {
                    Ok(file) => {
                        let mut visitor = AutozigVisitor::default();
                        visitor.visit_file(&file);

                        // Collect embedded Zig code
                        embedded_code.extend(visitor.zig_code);

                        // Collect external Zig file paths
                        for external_file in visitor.external_files {
                            let external_path = self.manifest_dir.join(&external_file);
                            if external_path.exists() {
                                external_files.push(external_path.clone());
                                all_zig_files.insert(external_path);
                            } else {
                                eprintln!(
                                    "Warning: External Zig file not found: {}",
                                    external_path.display()
                                );
                            }
                        }
                    },
                    Err(e) => {
                        eprintln!("Warning: Failed to parse {}: {}", path.display(), e);
                    },
                }
            }
        }

        // Also scan for standalone .zig and .c files in src directory
        for entry in WalkDir::new(&self.src_dir)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if let Some(ext) = path.extension() {
                if ext == "zig" {
                    all_zig_files.insert(path.to_path_buf());
                } else if ext == "c" {
                    // Collect C source files for build.zig compilation
                    c_source_files.insert(path.to_path_buf());
                }
            }
        }

        // Return based on mode
        match self.mode {
            CompilationMode::Merged => {
                // Legacy mode: merge all code
                let merged = self.merge_code(&embedded_code, &external_files)?;
                Ok(ScanResult::Merged(merged))
            },
            CompilationMode::ModularImport | CompilationMode::ModularBuildZig => {
                // Modular modes: return file information
                Ok(ScanResult::Modular {
                    embedded_code,
                    external_files,
                    all_zig_files: all_zig_files.into_iter().collect(),
                    c_source_files: c_source_files.into_iter().collect(),
                })
            },
        }
    }

    /// Merge code for legacy mode
    fn merge_code(&self, embedded: &[String], external: &[PathBuf]) -> Result<String> {
        let mut consolidated_zig = String::new();
        let mut has_std_import = false;

        // Add embedded code
        for code in embedded {
            consolidated_zig.push_str(code);
            consolidated_zig.push('\n');
        }

        // Add external files
        for external_path in external {
            match fs::read_to_string(external_path) {
                Ok(external_content) => {
                    consolidated_zig.push_str(&format!(
                        "\n// From external file: {}\n",
                        external_path.display()
                    ));

                    let cleaned_content =
                        remove_duplicate_imports(&external_content, &mut has_std_import);
                    consolidated_zig.push_str(&cleaned_content);
                    consolidated_zig.push('\n');
                },
                Err(e) => {
                    eprintln!(
                        "Warning: Failed to read external Zig file {}: {}",
                        external_path.display(),
                        e
                    );
                },
            }
        }

        Ok(consolidated_zig)
    }
}

/// AST visitor to extract autozig! and include_zig! macro contents
#[derive(Default)]
struct AutozigVisitor {
    zig_code: Vec<String>,
    external_files: Vec<String>,
}

impl<'ast> Visit<'ast> for AutozigVisitor {
    fn visit_macro(&mut self, node: &'ast Macro) {
        // Check if this is an autozig! macro
        if node.path.is_ident("autozig") {
            // Extract the token stream and convert to string
            let tokens = node.tokens.to_string();

            // The tokens will be in the format: { ... }
            // We need to extract the content and split by ---
            if let Some(zig_code) = extract_zig_from_tokens(&tokens) {
                self.zig_code.push(zig_code);
            }
        }
        // Check if this is an include_zig! macro
        else if node.path.is_ident("include_zig") {
            // Extract file path from tokens
            // Format: include_zig!("path/to/file.zig", { ... })
            let tokens = node.tokens.to_string();
            if let Some(file_path) = extract_file_path_from_tokens(&tokens) {
                self.external_files.push(file_path);
            }
        }

        // Continue visiting nested items
        syn::visit::visit_macro(self, node);
    }

    fn visit_item_macro(&mut self, node: &'ast syn::ItemMacro) {
        // Visit the macro itself
        self.visit_macro(&node.mac);
    }
}

/// Extract file path from include_zig! macro tokens
/// Expected format: ("path/to/file.zig", { ... }) or just ("path/to/file.zig")
fn extract_file_path_from_tokens(tokens: &str) -> Option<String> {
    let content = tokens.trim();

    // Remove outer parentheses if present
    let content = if content.starts_with('(') && content.ends_with(')') {
        &content[1..content.len() - 1]
    } else {
        content
    };

    // Find the first string literal (file path)
    // Look for quoted strings
    if let Some(start) = content.find('"') {
        if let Some(end) = content[start + 1..].find('"') {
            let file_path = &content[start + 1..start + 1 + end];
            return Some(file_path.to_string());
        }
    }

    None
}

/// Extract Zig code from macro tokens
/// This preserves the original formatting to avoid breaking Zig syntax like
/// @import
fn extract_zig_from_tokens(tokens: &str) -> Option<String> {
    // Remove outer braces if present, but preserve internal spacing
    let content = tokens.trim();
    let content = if content.starts_with('{') && content.ends_with('}') {
        &content[1..content.len() - 1]
    } else {
        content
    };

    // Split by --- separator (Zig code comes before ---)
    // Only take the first part (before ---)
    let zig_section = if let Some(separator_pos) = content.find("---") {
        content[..separator_pos].trim()
    } else {
        // No separator, take all content
        content.trim()
    };

    if zig_section.is_empty() {
        None
    } else {
        // Fix TokenStream formatting issues: remove spaces after @ symbol
        // We need a comprehensive fix for all @ builtins
        let fixed = zig_section.to_string();

        // Use regex-like pattern matching to fix all "@ word" to "@word"
        // This handles @import, @floatFromInt, @sqrt, etc.
        let mut result = String::with_capacity(fixed.len());
        let mut chars = fixed.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '@' {
                result.push(ch);
                // Skip any whitespace after @
                while let Some(&next_ch) = chars.peek() {
                    if next_ch.is_whitespace() {
                        chars.next();
                    } else {
                        break;
                    }
                }
            } else {
                result.push(ch);
            }
        }

        let fixed = result
            // Fix array syntax spacing
            .replace("[ * ]", "[*]")
            .replace("[ ]", "[]")
            // Fix range syntax
            .replace("[ 0 .. len ]", "[0..len]")
            .replace(".. ", "..");

        // Transform struct -> extern struct for C ABI compatibility
        // Zig's export fn requires extern struct for C calling convention
        let fixed = convert_to_extern_struct(&fixed);

        Some(fixed)
    }
}

/// Remove duplicate imports from external Zig files
/// This prevents "duplicate struct member name" errors when merging multiple
/// files
fn remove_duplicate_imports(content: &str, has_std_import: &mut bool) -> String {
    let mut result = String::new();

    for line in content.lines() {
        let trimmed = line.trim();

        // Check if this is a std import line
        if trimmed.starts_with("const std") && trimmed.contains("@import(\"std\")") {
            // Only include the first std import
            if !*has_std_import {
                result.push_str(line);
                result.push('\n');
                *has_std_import = true;
            }
            // Skip subsequent std imports
            continue;
        }

        // Keep all other lines
        result.push_str(line);
        result.push('\n');
    }

    result
}

/// Convert struct declarations to extern struct for C ABI compatibility
/// This is necessary because Zig's export fn requires extern struct for C
/// calling convention
///
/// Transforms patterns like:
///   `pub const Color = struct {` -> `pub const Color = extern struct {`
///   `const Vec3 = struct {` -> `const Vec3 = extern struct {`
///
/// Skips:
///   - Already `extern struct`
///   - `packed struct`
///   - Anonymous structs (inside other declarations)
fn convert_to_extern_struct(code: &str) -> String {
    let mut result = String::with_capacity(code.len() + 100);
    let mut remaining = code;

    while !remaining.is_empty() {
        // Look for "= struct {" pattern (with possible whitespace variations)
        // This matches named struct declarations like `pub const Name = struct {`
        if let Some(pos) = find_struct_declaration(remaining) {
            // Copy everything before the match
            result.push_str(&remaining[..pos]);

            // Check if this is already extern struct or packed struct
            let before_match = &remaining[..pos];
            let trimmed = before_match.trim_end();

            // Skip if the previous word is "extern" or "packed"
            if trimmed.ends_with("extern") || trimmed.ends_with("packed") {
                // Already extern or packed, copy "= struct" as-is
                let struct_end = pos + find_struct_keyword_end(&remaining[pos..]);
                result.push_str(&remaining[pos..struct_end]);
                remaining = &remaining[struct_end..];
            } else {
                // Need to convert: "= struct" -> "= extern struct"
                let struct_keyword_end = pos + find_struct_keyword_end(&remaining[pos..]);
                let struct_pattern = &remaining[pos..struct_keyword_end];

                // Replace "= struct" with "= extern struct" (preserve spacing)
                let converted = struct_pattern
                    .replace("= struct", "= extern struct")
                    .replace("=struct", "= extern struct");
                result.push_str(&converted);
                remaining = &remaining[struct_keyword_end..];
            }
        } else {
            // No more struct declarations, copy the rest
            result.push_str(remaining);
            break;
        }
    }

    result
}

/// Find position of "= struct" or "=struct" pattern (start of the "=" sign)
fn find_struct_declaration(code: &str) -> Option<usize> {
    let patterns = ["= struct", "=struct"];

    let mut earliest_pos = None;

    for pattern in patterns {
        if let Some(pos) = code.find(pattern) {
            match earliest_pos {
                None => earliest_pos = Some(pos),
                Some(current) if pos < current => earliest_pos = Some(pos),
                _ => {},
            }
        }
    }

    earliest_pos
}

/// Find the end of "= struct" or "=struct" keyword (including the space or
/// brace after)
fn find_struct_keyword_end(code: &str) -> usize {
    // Find "struct" and return position after it
    if let Some(struct_pos) = code.find("struct") {
        struct_pos + "struct".len()
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_zig_from_tokens() {
        let tokens = r#"{
            const std = @import("std");
            export fn add(a: i32, b: i32) i32 {
                return a + b;
            }
            ---
            fn add(a: i32, b: i32) -> i32;
        }"#;

        let result = extract_zig_from_tokens(tokens).unwrap();
        // Should contain Zig code
        assert!(result.contains("export fn add"));
        assert!(result.contains("const std"));
        // Should NOT contain separator or Rust signatures (look for -> which is
        // Rust-specific)
        assert!(!result.contains("---"));
        assert!(!result.contains("-> i32;")); // Rust return type syntax
    }

    #[test]
    fn test_extract_without_separator() {
        let tokens = r#"{
            const std = @import("std");
            export fn multiply(a: i32, b: i32) i32 {
                return a * b;
            }
        }"#;

        let result = extract_zig_from_tokens(tokens).unwrap();
        assert!(result.contains("export fn multiply"));
    }

    #[test]
    fn test_remove_duplicate_imports() {
        let content = r#"const std = @import("std");

export fn test() void {}
"#;
        let mut has_std = false;
        let result1 = remove_duplicate_imports(content, &mut has_std);
        assert!(result1.contains("const std"));
        assert!(has_std);

        // Second call should remove the import
        let result2 = remove_duplicate_imports(content, &mut has_std);
        assert!(!result2.contains("const std"));
        assert!(result2.contains("export fn test"));
    }

    #[test]
    fn test_convert_to_extern_struct_basic() {
        let code = "pub const Color = struct { r: f32, g: f32, b: f32, };";
        let result = convert_to_extern_struct(code);
        assert!(result.contains("= extern struct"));
        assert!(!result.contains("= struct {") || result.contains("extern struct"));
    }

    #[test]
    fn test_convert_to_extern_struct_already_extern() {
        let code = "pub const Vec3 = extern struct { x: f32, y: f32, z: f32, };";
        let result = convert_to_extern_struct(code);
        // Should remain unchanged - only one "extern struct"
        assert_eq!(result.matches("extern struct").count(), 1);
        assert!(!result.contains("extern extern"));
    }

    #[test]
    fn test_convert_to_extern_struct_packed() {
        let code = "pub const PackedData = packed struct { a: u8, b: u8, };";
        let result = convert_to_extern_struct(code);
        // Should remain packed struct, not converted
        assert!(result.contains("packed struct"));
        assert!(!result.contains("extern struct"));
    }

    #[test]
    fn test_convert_to_extern_struct_multiple() {
        let code = r#"
            pub const Color = struct { r: f32, g: f32, b: f32, };
            pub const Vec3 = struct { x: f32, y: f32, z: f32, };
        "#;
        let result = convert_to_extern_struct(code);
        // Both should be converted
        assert_eq!(result.matches("extern struct").count(), 2);
    }

    #[test]
    fn test_convert_to_extern_struct_with_export_fn() {
        let code = r#"
            pub const Color = struct { r: f32, g: f32, b: f32, };
            export fn create_color(r: f32, g: f32, b: f32) Color {
                return Color{ .r = r, .g = g, .b = b };
            }
        "#;
        let result = convert_to_extern_struct(code);
        assert!(result.contains("extern struct"));
        assert!(result.contains("export fn create_color"));
    }
}
