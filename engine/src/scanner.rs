//! Source code scanner to extract Zig code from autozig! macros using syn AST parsing

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use syn::{visit::Visit, Macro};
use walkdir::WalkDir;

/// Scanner for extracting Zig code from Rust source files
pub struct ZigCodeScanner {
    src_dir: std::path::PathBuf,
    manifest_dir: std::path::PathBuf,
}

impl ZigCodeScanner {
    pub fn new(src_dir: impl AsRef<Path>) -> Self {
        // Get manifest dir from environment or use src_dir parent
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
            .ok()
            .and_then(|d| std::path::PathBuf::from(d).canonicalize().ok())
            .unwrap_or_else(|| {
                src_dir.as_ref()
                    .parent()
                    .unwrap_or(src_dir.as_ref())
                    .to_path_buf()
            });
        
        Self {
            src_dir: src_dir.as_ref().to_path_buf(),
            manifest_dir,
        }
    }
    
    /// Scan all .rs files and extract Zig code using AST parsing
    pub fn scan(&self) -> Result<String> {
        let mut consolidated_zig = String::new();
        let mut has_std_import = false;
        
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
                        
                        // Process embedded Zig code
                        for zig_code in visitor.zig_code {
                            consolidated_zig.push_str(&zig_code);
                            consolidated_zig.push('\n');
                        }
                        
                        // Process external Zig files
                        for external_file in visitor.external_files {
                            let external_path = self.manifest_dir.join(&external_file);
                            match fs::read_to_string(&external_path) {
                                Ok(external_content) => {
                                    consolidated_zig.push_str(&format!("\n// From external file: {}\n", external_file));
                                    
                                    // Remove duplicate std import and other common imports
                                    let cleaned_content = remove_duplicate_imports(&external_content, &mut has_std_import);
                                    consolidated_zig.push_str(&cleaned_content);
                                    consolidated_zig.push('\n');
                                }
                                Err(e) => {
                                    eprintln!("Warning: Failed to read external Zig file {}: {}", external_path.display(), e);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Warning: Failed to parse {}: {}", path.display(), e);
                        // Continue scanning other files
                    }
                }
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
        &content[1..content.len()-1]
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
/// This preserves the original formatting to avoid breaking Zig syntax like @import
fn extract_zig_from_tokens(tokens: &str) -> Option<String> {
    // Remove outer braces if present, but preserve internal spacing
    let content = tokens.trim();
    let content = if content.starts_with('{') && content.ends_with('}') {
        &content[1..content.len()-1]
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
        
        Some(fixed)
    }
}

/// Remove duplicate imports from external Zig files
/// This prevents "duplicate struct member name" errors when merging multiple files
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
        // Should NOT contain separator or Rust signatures (look for -> which is Rust-specific)
        assert!(!result.contains("---"));
        assert!(!result.contains("-> i32;"));  // Rust return type syntax
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
}