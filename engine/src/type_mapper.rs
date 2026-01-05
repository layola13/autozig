//! Type mapping between Zig and Rust types

use std::collections::HashMap;

/// Maps Zig types to Rust types for FFI
pub struct TypeMapper {
    mappings: HashMap<&'static str, &'static str>,
}

impl TypeMapper {
    pub fn new() -> Self {
        let mut mappings = HashMap::new();

        // Integer types
        mappings.insert("i8", "i8");
        mappings.insert("i16", "i16");
        mappings.insert("i32", "i32");
        mappings.insert("i64", "i64");
        mappings.insert("i128", "i128");
        mappings.insert("isize", "isize");

        mappings.insert("u8", "u8");
        mappings.insert("u16", "u16");
        mappings.insert("u32", "u32");
        mappings.insert("u64", "u64");
        mappings.insert("u128", "u128");
        mappings.insert("usize", "usize");

        // Floating point
        mappings.insert("f32", "f32");
        mappings.insert("f64", "f64");

        // Boolean (Zig bool is u8 in C ABI)
        mappings.insert("bool", "u8");

        // Void
        mappings.insert("void", "()");

        // Pointer types
        mappings.insert("[*]const u8", "*const u8");
        mappings.insert("[*]u8", "*mut u8");

        Self { mappings }
    }

    /// Map a Zig type to its Rust equivalent
    pub fn map_type(&self, zig_type: &str) -> Option<&str> {
        self.mappings.get(zig_type).copied()
    }

    /// Check if a type is a pointer that needs special handling
    pub fn is_slice_type(&self, zig_type: &str) -> bool {
        zig_type.starts_with("[*]")
    }
}

impl Default for TypeMapper {
    fn default() -> Self {
        Self::new()
    }
}

/// Conversion strategy for function parameters
#[derive(Debug, Clone, PartialEq)]
pub enum ParamConversion {
    /// Direct pass-through (primitive types)
    Direct,
    /// Convert &[T] to (ptr, len)
    SliceToPtrLen,
    /// Convert &str to (ptr, len)
    StrToPtrLen,
}

/// Analyze a Rust type and determine conversion strategy
pub fn analyze_param_type(ty: &syn::Type) -> ParamConversion {
    match ty {
        syn::Type::Reference(type_ref) => {
            // Check if it's &[T] or &str
            match &*type_ref.elem {
                syn::Type::Slice(_) => ParamConversion::SliceToPtrLen,
                syn::Type::Path(type_path) => {
                    if let Some(segment) = type_path.path.segments.last() {
                        if segment.ident == "str" {
                            return ParamConversion::StrToPtrLen;
                        }
                    }
                    ParamConversion::Direct
                },
                _ => ParamConversion::Direct,
            }
        },
        _ => ParamConversion::Direct,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_mapper() {
        let mapper = TypeMapper::new();

        assert_eq!(mapper.map_type("i32"), Some("i32"));
        assert_eq!(mapper.map_type("u64"), Some("u64"));
        assert_eq!(mapper.map_type("f32"), Some("f32"));
        assert_eq!(mapper.map_type("bool"), Some("u8"));
        assert_eq!(mapper.map_type("void"), Some("()"));
    }

    #[test]
    fn test_slice_detection() {
        let mapper = TypeMapper::new();

        assert!(mapper.is_slice_type("[*]const u8"));
        assert!(mapper.is_slice_type("[*]u8"));
        assert!(!mapper.is_slice_type("i32"));
    }
}
