//! TypeScript/JavaScript bindings generator for WASM exports
//!
//! Automatically generates `.d.ts` type definitions and `.js` loader modules
//! for WASM64 exports, eliminating the need for manual bindings in HTML.

use std::fmt::Write;

/// Rust type representation for TypeScript mapping
#[derive(Debug, Clone, PartialEq)]
pub enum RustType {
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    Usize,
    Isize,
    F32,
    F64,
    Bool,
    Ptr, // *mut u8 / *const u8
    Void,
    Unknown(String),
}

impl RustType {
    /// Parse from Rust type string
    pub fn from_str(s: &str) -> Self {
        match s.trim() {
            "u8" => RustType::U8,
            "u16" => RustType::U16,
            "u32" => RustType::U32,
            "u64" => RustType::U64,
            "i8" => RustType::I8,
            "i16" => RustType::I16,
            "i32" => RustType::I32,
            "i64" => RustType::I64,
            "usize" => RustType::Usize,
            "isize" => RustType::Isize,
            "f32" => RustType::F32,
            "f64" => RustType::F64,
            "bool" => RustType::Bool,
            "()" | "" => RustType::Void,
            s if s.starts_with("*mut") || s.starts_with("*const") => RustType::Ptr,
            s => RustType::Unknown(s.to_string()),
        }
    }

    /// Convert to TypeScript type string
    pub fn to_typescript(&self, is_wasm64: bool) -> &'static str {
        match self {
            RustType::U8 | RustType::U16 | RustType::U32 => "number",
            RustType::I8 | RustType::I16 | RustType::I32 => "number",
            RustType::F32 | RustType::F64 => "number",
            // For wasm64, usize/isize/u64/i64 are 64-bit, need bigint
            RustType::U64 | RustType::I64 => "bigint",
            RustType::Usize | RustType::Isize => {
                if is_wasm64 {
                    "bigint"
                } else {
                    "number"
                }
            },
            RustType::Bool => "boolean",
            RustType::Ptr => "number", // Pointer as number address
            RustType::Void => "void",
            RustType::Unknown(_) => "unknown",
        }
    }

    /// Check if this type requires BigInt conversion in wasm64
    pub fn needs_bigint(&self, is_wasm64: bool) -> bool {
        match self {
            RustType::U64 | RustType::I64 => true,
            RustType::Usize | RustType::Isize => is_wasm64,
            _ => false,
        }
    }
}

/// Function signature for TypeScript generation
#[derive(Debug, Clone)]
pub struct FunctionSignature {
    /// Function name (e.g., "wasm64_get_memory_size")
    pub name: String,
    /// Parameters: (name, type)
    pub params: Vec<(String, RustType)>,
    /// Return type
    pub return_type: RustType,
    /// Optional documentation comment
    pub doc: Option<String>,
}

impl FunctionSignature {
    /// Parse from function declaration string
    /// Example: "fn get_memory_size() -> usize"
    pub fn parse(decl: &str) -> Option<Self> {
        let decl = decl.trim();

        // Remove leading attributes like #[autozig(...)]
        // Handle complex attributes with nested brackets/strings
        let decl = Self::strip_attributes(decl);
        let decl = decl.trim();

        // Must start with "fn " now
        let decl = decl.strip_prefix("fn ")?.trim();

        // Extract function name (until first '(')
        let paren_pos = decl.find('(')?;
        let name = decl[..paren_pos].trim().to_string();

        // Validate function name (should be alphanumeric + underscore)
        if !name.chars().all(|c| c.is_alphanumeric() || c == '_') || name.is_empty() {
            return None;
        }

        // Extract parameters
        let after_name = &decl[paren_pos + 1..];
        let close_paren = after_name.find(')')?;
        let params_str = &after_name[..close_paren];

        let params = Self::parse_params(params_str);

        // Extract return type
        let after_params = &after_name[close_paren + 1..];
        let return_type = if let Some(arrow_pos) = after_params.find("->") {
            let ret_str = after_params[arrow_pos + 2..].trim();
            // Remove trailing semicolon if present
            let ret_str = ret_str.trim_end_matches(';').trim();
            RustType::from_str(ret_str)
        } else {
            RustType::Void
        };

        Some(FunctionSignature { name, params, return_type, doc: None })
    }

    /// Strip all #[...] attributes from the beginning of a declaration
    fn strip_attributes(s: &str) -> &str {
        let mut s = s.trim();

        while s.starts_with('#') {
            // Find the matching closing bracket
            if let Some(bracket_start) = s.find('[') {
                let after_open = &s[bracket_start + 1..];
                let mut depth = 1;
                let mut in_string = false;
                let mut escape_next = false;

                for (i, c) in after_open.char_indices() {
                    if escape_next {
                        escape_next = false;
                        continue;
                    }

                    match c {
                        '\\' => escape_next = true,
                        '"' => in_string = !in_string,
                        '[' if !in_string => depth += 1,
                        ']' if !in_string => {
                            depth -= 1;
                            if depth == 0 {
                                // Found the closing bracket
                                let end_pos = bracket_start + 1 + i + 1;
                                s = s[end_pos..].trim();
                                break;
                            }
                        },
                        _ => {},
                    }
                }

                // If we couldn't find matching bracket, break to avoid infinite loop
                if depth > 0 {
                    break;
                }
            } else {
                break;
            }
        }

        s
    }

    fn parse_params(params_str: &str) -> Vec<(String, RustType)> {
        if params_str.trim().is_empty() {
            return Vec::new();
        }

        params_str
            .split(',')
            .filter_map(|param| {
                let param = param.trim();
                if param.is_empty() {
                    return None;
                }

                let parts: Vec<&str> = param.splitn(2, ':').collect();
                if parts.len() == 2 {
                    let name = parts[0].trim().to_string();
                    let ty = RustType::from_str(parts[1].trim());
                    Some((name, ty))
                } else {
                    None
                }
            })
            .collect()
    }
}

/// Configuration for TypeScript generation
#[derive(Debug, Clone)]
pub struct TsConfig {
    /// Is this for wasm64 target (affects type mappings)
    pub is_wasm64: bool,
    /// Module name for exports
    pub module_name: String,
    /// Generate ES Module format (true) or CommonJS (false)
    pub es_module: bool,
}

impl Default for TsConfig {
    fn default() -> Self {
        Self {
            is_wasm64: false,
            module_name: "autozig".to_string(),
            es_module: true,
        }
    }
}

/// TypeScript bindings generator
pub struct TsGenerator {
    functions: Vec<FunctionSignature>,
    config: TsConfig,
}

impl TsGenerator {
    /// Create a new generator with given functions
    pub fn new(functions: Vec<FunctionSignature>, config: TsConfig) -> Self {
        Self { functions, config }
    }

    /// Create from function declaration strings
    pub fn from_declarations(decls: &[String], config: TsConfig) -> Self {
        let functions = decls
            .iter()
            .filter_map(|d| FunctionSignature::parse(d))
            .collect();
        Self { functions, config }
    }

    /// Generate TypeScript declaration file (.d.ts)
    pub fn generate_dts(&self) -> String {
        let mut output = String::new();

        // Header
        writeln!(output, "/* Auto-generated by AutoZig - Do not edit */").unwrap();
        writeln!(output, "/* eslint-disable */").unwrap();
        writeln!(output).unwrap();

        // Exports interface
        writeln!(output, "export interface WasmExports {{").unwrap();

        for func in &self.functions {
            // Documentation
            if let Some(doc) = &func.doc {
                writeln!(output, "  /** {} */", doc).unwrap();
            }

            // Function signature
            let params = func
                .params
                .iter()
                .map(|(name, ty)| format!("{}: {}", name, ty.to_typescript(self.config.is_wasm64)))
                .collect::<Vec<_>>()
                .join(", ");

            let ret = func.return_type.to_typescript(self.config.is_wasm64);

            writeln!(output, "  {}({}): {};", func.name, params, ret).unwrap();
        }

        writeln!(output, "}}").unwrap();
        writeln!(output).unwrap();

        // AutoZigWasm interface
        writeln!(output, "export interface AutoZigWasm {{").unwrap();
        writeln!(output, "  exports: WasmExports;").unwrap();
        writeln!(output, "  memory: WebAssembly.Memory;").unwrap();
        writeln!(output, "  instance: WebAssembly.Instance;").unwrap();
        writeln!(output, "}}").unwrap();
        writeln!(output).unwrap();

        // Loader functions
        writeln!(output, "/** Load WASM module from URL */").unwrap();
        writeln!(output, "export function loadWasm(path: string): Promise<AutoZigWasm>;").unwrap();
        writeln!(output).unwrap();
        writeln!(output, "/** Load WASM module from ArrayBuffer */").unwrap();
        writeln!(
            output,
            "export function loadWasmSync(buffer: ArrayBuffer): Promise<AutoZigWasm>;"
        )
        .unwrap();

        output
    }

    /// Generate JavaScript loader module (.js)
    pub fn generate_js_loader(&self) -> String {
        let mut output = String::new();

        // Header
        writeln!(output, "/* Auto-generated by AutoZig - Do not edit */").unwrap();
        writeln!(output).unwrap();

        // Loader function
        writeln!(output, "/**").unwrap();
        writeln!(output, " * Load WASM module from URL").unwrap();
        writeln!(output, " * @param {{string}} path - Path to .wasm file").unwrap();
        writeln!(output, " * @returns {{Promise<AutoZigWasm>}}").unwrap();
        writeln!(output, " */").unwrap();
        writeln!(output, "export async function loadWasm(path) {{").unwrap();
        writeln!(output, "  const response = await fetch(path);").unwrap();
        writeln!(output, "  const buffer = await response.arrayBuffer();").unwrap();
        writeln!(output, "  return loadWasmSync(buffer);").unwrap();
        writeln!(output, "}}").unwrap();
        writeln!(output).unwrap();

        // Sync loader
        writeln!(output, "/**").unwrap();
        writeln!(output, " * Load WASM module from ArrayBuffer").unwrap();
        writeln!(output, " * @param {{ArrayBuffer}} buffer").unwrap();
        writeln!(output, " * @returns {{Promise<AutoZigWasm>}}").unwrap();
        writeln!(output, " */").unwrap();
        writeln!(output, "export async function loadWasmSync(buffer) {{").unwrap();
        writeln!(output, "  const result = await WebAssembly.instantiate(buffer, {{}});").unwrap();
        writeln!(output, "  const instance = result.instance;").unwrap();
        writeln!(output, "  const raw = instance.exports;").unwrap();
        writeln!(output).unwrap();
        writeln!(output, "  return {{").unwrap();
        writeln!(output, "    exports: wrapExports(raw),").unwrap();
        writeln!(output, "    memory: raw.memory,").unwrap();
        writeln!(output, "    instance: instance").unwrap();
        writeln!(output, "  }};").unwrap();
        writeln!(output, "}}").unwrap();
        writeln!(output).unwrap();

        // Wrapper function
        writeln!(output, "/**").unwrap();
        writeln!(output, " * Wrap raw WASM exports with type conversions").unwrap();
        writeln!(output, " * @param {{object}} raw - Raw WebAssembly exports").unwrap();
        writeln!(output, " * @returns {{WasmExports}}").unwrap();
        writeln!(output, " */").unwrap();
        writeln!(output, "function wrapExports(raw) {{").unwrap();
        writeln!(output, "  return {{").unwrap();

        for (i, func) in self.functions.iter().enumerate() {
            let trailing_comma = if i < self.functions.len() - 1 {
                ","
            } else {
                ""
            };

            // Check if any params need BigInt conversion
            let needs_conversion = func
                .params
                .iter()
                .any(|(_, ty)| ty.needs_bigint(self.config.is_wasm64));

            if needs_conversion {
                // Generate wrapper with conversion
                let params: Vec<_> = func.params.iter().map(|(n, _)| n.as_str()).collect();
                let params_str = params.join(", ");

                let converted_args: Vec<_> = func
                    .params
                    .iter()
                    .map(|(name, ty)| {
                        if ty.needs_bigint(self.config.is_wasm64) {
                            format!("BigInt({})", name)
                        } else {
                            name.clone()
                        }
                    })
                    .collect();
                let args_str = converted_args.join(", ");

                // Handle return type conversion (BigInt -> number if needed)
                let ret_needs_bigint = func.return_type.needs_bigint(self.config.is_wasm64);

                if ret_needs_bigint {
                    writeln!(
                        output,
                        "    {}: ({}) => raw.{}({}){}",
                        func.name, params_str, func.name, args_str, trailing_comma
                    )
                    .unwrap();
                } else {
                    writeln!(
                        output,
                        "    {}: ({}) => raw.{}({}){}",
                        func.name, params_str, func.name, args_str, trailing_comma
                    )
                    .unwrap();
                }
            } else {
                // Direct passthrough
                writeln!(output, "    {}: raw.{}{}", func.name, func.name, trailing_comma).unwrap();
            }
        }

        writeln!(output, "  }};").unwrap();
        writeln!(output, "}}").unwrap();

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_function_simple() {
        let sig = FunctionSignature::parse("fn get_memory_size() -> usize").unwrap();
        assert_eq!(sig.name, "get_memory_size");
        assert!(sig.params.is_empty());
        assert_eq!(sig.return_type, RustType::Usize);
    }

    #[test]
    fn test_parse_function_with_params() {
        let sig = FunctionSignature::parse("fn write_buffer(offset: usize, value: u8)").unwrap();
        assert_eq!(sig.name, "write_buffer");
        assert_eq!(sig.params.len(), 2);
        assert_eq!(sig.params[0], ("offset".to_string(), RustType::Usize));
        assert_eq!(sig.params[1], ("value".to_string(), RustType::U8));
        assert_eq!(sig.return_type, RustType::Void);
    }

    #[test]
    fn test_parse_with_autozig_attr() {
        let sig =
            FunctionSignature::parse(r#"#[autozig(strategy = "dual")] fn get_arch_info() -> u32"#)
                .unwrap();
        assert_eq!(sig.name, "get_arch_info");
        assert_eq!(sig.return_type, RustType::U32);
    }

    #[test]
    fn test_type_to_typescript() {
        assert_eq!(RustType::U32.to_typescript(false), "number");
        assert_eq!(RustType::Usize.to_typescript(false), "number");
        assert_eq!(RustType::Usize.to_typescript(true), "bigint");
        assert_eq!(RustType::U64.to_typescript(true), "bigint");
    }
}
