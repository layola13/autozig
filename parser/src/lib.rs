//! Parser for autozig! macro input
//!
//! This module parses the mixed Zig/Rust syntax within autozig! blocks

#![forbid(unsafe_code)]

use proc_macro2::TokenStream;
use syn::{
    parse::{
        Parse,
        ParseStream,
    },
    ItemEnum,
    ItemImpl,
    ItemStruct,
    Result as ParseResult,
    Signature,
};

/// Configuration parsed from autozig! macro
#[derive(Debug, Clone)]
pub struct AutoZigConfig {
    /// Raw Zig code to be compiled (for embedded mode)
    pub zig_code: String,
    /// External Zig file path (for include mode)
    pub external_file: Option<String>,
    /// Rust function signatures for safe wrappers
    pub rust_signatures: Vec<RustFunctionSignature>,
    /// Rust struct definitions for FFI types
    pub rust_structs: Vec<RustStructDefinition>,
    /// Rust enum definitions for FFI types
    pub rust_enums: Vec<RustEnumDefinition>,
    /// Rust trait implementations (Phase 1: stateless traits)
    pub rust_trait_impls: Vec<RustTraitImpl>,
}

/// Generic parameter definition (Phase 3)
#[derive(Debug, Clone)]
pub struct GenericParam {
    /// Parameter name (e.g., "T")
    pub name: String,
    /// Type bounds (e.g., Copy, Clone)
    pub bounds: Vec<String>,
}

/// A Rust function signature that will have a safe wrapper generated
#[derive(Clone)]
pub struct RustFunctionSignature {
    pub sig: Signature,
    /// Generic parameters (Phase 3: Generics support)
    pub generic_params: Vec<GenericParam>,
    /// Whether this is an async function (Phase 3: Async support)
    pub is_async: bool,
    /// Monomorphization attribute types (e.g., #[monomorphize(i32, f64)])
    pub monomorphize_types: Vec<String>,
}

/// A Rust struct definition for FFI types
#[derive(Clone)]
pub struct RustStructDefinition {
    pub item: ItemStruct,
}

/// A Rust enum definition for FFI types
#[derive(Clone)]
pub struct RustEnumDefinition {
    pub item: ItemEnum,
}

/// A Rust trait implementation (impl Trait for Type)
#[derive(Clone)]
pub struct RustTraitImpl {
    /// The trait being implemented (e.g., "Calculator")
    pub trait_name: String,
    /// The type implementing the trait (e.g., "ZigCalculator")
    pub target_type: String,
    /// Methods in this trait implementation
    pub methods: Vec<TraitMethod>,
    /// Whether the target type is a zero-sized type (stateless)
    pub is_zst: bool,
    /// Whether the target type is an opaque pointer (stateful) - Phase 2
    pub is_opaque: bool,
    /// Constructor method for opaque types - Phase 2
    pub constructor: Option<TraitMethod>,
    /// Destructor method for opaque types - Phase 2
    pub destructor: Option<TraitMethod>,
}

/// A method within a trait implementation
#[derive(Clone)]
pub struct TraitMethod {
    /// Method name (e.g., "add")
    pub name: String,
    /// Method signature
    pub sig: Signature,
    /// Zig function name that this method calls (e.g., "zig_add")
    pub zig_function: String,
    /// Original method body (for complex wrapper logic)
    pub body: Option<syn::Block>,
    /// Zig function's actual return type (extracted from Zig code)
    pub zig_return_type: Option<syn::ReturnType>,
    /// Whether this is a constructor (#[constructor]) - Phase 2
    pub is_constructor: bool,
    /// Whether this is a destructor (#[destructor]) - Phase 2
    pub is_destructor: bool,
}

impl std::fmt::Debug for RustStructDefinition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RustStructDefinition")
            .field("ident", &self.item.ident.to_string())
            .finish()
    }
}

impl std::fmt::Debug for RustEnumDefinition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RustEnumDefinition")
            .field("ident", &self.item.ident.to_string())
            .finish()
    }
}

impl std::fmt::Debug for RustTraitImpl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RustTraitImpl")
            .field("trait_name", &self.trait_name)
            .field("target_type", &self.target_type)
            .field("methods", &self.methods.len())
            .field("is_zst", &self.is_zst)
            .field("is_opaque", &self.is_opaque)
            .finish()
    }
}

impl std::fmt::Debug for TraitMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TraitMethod")
            .field("name", &self.name)
            .field("zig_function", &self.zig_function)
            .finish()
    }
}

impl std::fmt::Debug for RustFunctionSignature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RustFunctionSignature")
            .field("sig", &self.sig.ident.to_string())
            .finish()
    }
}

impl Parse for AutoZigConfig {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        // Strategy: Parse everything as a token stream, then split by "---" separator
        let tokens: TokenStream = input.parse()?;
        let token_str = tokens.to_string();


        // TokenStream.to_string() may add spaces: "---" becomes "- - -"
        // Try multiple separator patterns
        let separators = ["---", "- - -", "-- -", "- --"];
        let mut parts: Vec<&str> = vec![&token_str];
        for sep in &separators {
            let test_split: Vec<&str> = token_str.split(sep).collect();
            if test_split.len() > 1 {
                parts = test_split;
                break;
            }
        }


        if parts.len() == 1 {
            // No separator, treat entire input as Zig code
            Ok(AutoZigConfig {
                zig_code: parts[0].trim().to_string(),
                external_file: None,
                rust_signatures: Vec::new(),
                rust_structs: Vec::new(),
                rust_enums: Vec::new(),
                rust_trait_impls: Vec::new(),
            })
        } else if parts.len() >= 2 {
            // Has separator: first part is Zig, second is Rust definitions
            let zig_code = parts[0].trim().to_string();


            // Parse Rust definitions (enums, structs, function signatures, and trait impls)
            // from second part
            let (rust_enums, rust_structs, rust_signatures, rust_trait_impls) =
                parse_rust_definitions(parts[1])?;


            Ok(AutoZigConfig {
                zig_code,
                external_file: None,
                rust_signatures,
                rust_structs,
                rust_enums,
                rust_trait_impls,
            })
        } else {
            Err(syn::Error::new(input.span(), "autozig! macro parsing error"))
        }
    }
}

/// Parse Rust definitions (enums, structs, function signatures, and trait
/// impls) from a string
fn parse_rust_definitions(
    input: &str,
) -> ParseResult<(
    Vec<RustEnumDefinition>,
    Vec<RustStructDefinition>,
    Vec<RustFunctionSignature>,
    Vec<RustTraitImpl>,
)> {
    let mut enums = Vec::new();
    let mut structs = Vec::new();
    let mut signatures = Vec::new();
    let mut trait_impls = Vec::new();
    let mut trait_impl_types = std::collections::HashSet::new();


    // First, try to parse the entire input as a token stream
    // This handles struct definitions better
    let input_normalized = input.replace(['\n', '\r'], " ");
    let input_normalized = input_normalized
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");

    // Check if input is already wrapped in braces (from include_zig! macro)
    let input_content =
        if input_normalized.trim().starts_with('{') && input_normalized.trim().ends_with('}') {
            // Remove outer braces
            let trimmed = input_normalized.trim();
            &trimmed[1..trimmed.len() - 1]
        } else {
            input_normalized.as_str()
        };

    // Try to parse as a file (multiple items)
    let file_str = format!("mod temp {{ {} }}", input_content);

    if let Ok(parsed_file) = syn::parse_str::<syn::File>(&file_str) {
        eprintln!("Parser: Successfully parsed file with {} items", parsed_file.items.len());
        for item in parsed_file.items {
            if let syn::Item::Mod(item_mod) = item {
                if let Some((_, items)) = item_mod.content {
                    eprintln!("Parser: Module has {} items", items.len());
                    // First pass: collect opaque struct definitions
                    let mut opaque_types = std::collections::HashSet::new();
                    for inner_item in &items {
                        if let syn::Item::Struct(item_struct) = inner_item {
                            eprintln!("Parser: Found struct: {}", item_struct.ident);
                            if is_opaque_struct(item_struct) {
                                eprintln!("Parser:   -> Marked as OPAQUE");
                                opaque_types.insert(item_struct.ident.to_string());
                            }
                        }
                    }
                    eprintln!("Parser: Total opaque types: {}", opaque_types.len());

                    // Second pass: collect trait impls and inherent impls, mark opaque types
                    for inner_item in &items {
                        if let syn::Item::Impl(item_impl) = inner_item {
                            eprintln!("Parser: Found impl block");
                            // Try parsing as trait impl
                            if let Some(mut trait_impl) = parse_trait_impl(item_impl.clone()) {
                                eprintln!(
                                    "Parser:   -> Parsed as TRAIT impl for {}",
                                    trait_impl.target_type
                                );
                                // Mark as opaque if the type was declared as opaque
                                if opaque_types.contains(&trait_impl.target_type) {
                                    trait_impl.is_opaque = true;
                                    trait_impl.is_zst = false; // Opaque types
                                                               // are not ZST
                                }
                                trait_impl_types.insert(trait_impl.target_type.clone());
                                trait_impls.push(trait_impl);
                            } else {
                                // Try parsing as inherent impl (for constructor/destructor)
                                eprintln!("Parser:   -> Trying as INHERENT impl");
                                if let Some(inherent_impl) =
                                    parse_inherent_impl(item_impl.clone(), &opaque_types)
                                {
                                    eprintln!(
                                        "Parser:   -> SUCCESS: Parsed inherent impl for {}",
                                        inherent_impl.target_type
                                    );
                                    trait_impl_types.insert(inherent_impl.target_type.clone());
                                    trait_impls.push(inherent_impl);
                                } else {
                                    eprintln!("Parser:   -> FAILED to parse as inherent impl");
                                }
                            }
                        }
                    }
                    eprintln!("Parser: Total trait impls collected: {}", trait_impls.len());

                    // Third pass: collect everything else, skipping structs that will be generated
                    for inner_item in items {
                        // Debug: log what type of item this is
                        let item_type = match &inner_item {
                            syn::Item::Enum(_) => "Enum",
                            syn::Item::Struct(_) => "Struct",
                            syn::Item::Fn(_) => "Fn",
                            syn::Item::ForeignMod(_) => "ForeignMod",
                            syn::Item::Verbatim(_) => "Verbatim",
                            syn::Item::Impl(_) => "Impl",
                            _ => "Other",
                        };
                        eprintln!("Parser: Processing item type: {}", item_type);

                        match inner_item {
                            syn::Item::Enum(item_enum) => {
                                eprintln!("Parser:   -> Collecting Enum");
                                enums.push(RustEnumDefinition { item: item_enum });
                            },
                            syn::Item::Struct(item_struct) => {
                                eprintln!("Parser:   -> Checking Struct");
                                // Skip opaque struct declarations (they will be generated by macro)
                                // Skip structs that will be generated by trait impl
                                let struct_name = item_struct.ident.to_string();
                                if !trait_impl_types.contains(&struct_name)
                                    && !is_opaque_struct(&item_struct)
                                {
                                    structs.push(RustStructDefinition { item: item_struct });
                                }
                            },
                            syn::Item::Fn(item_fn) => {
                                signatures
                                    .push(parse_function_signature(item_fn.sig, &item_fn.attrs));
                            },
                            syn::Item::Impl(_) => {
                                // Already processed in first pass
                            },
                            syn::Item::ForeignMod(foreign_mod) => {
                                for foreign_item in foreign_mod.items {
                                    if let syn::ForeignItem::Fn(fn_item) = foreign_item {
                                        signatures.push(parse_function_signature(
                                            fn_item.sig,
                                            &fn_item.attrs,
                                        ));
                                    }
                                }
                            },
                            syn::Item::Verbatim(tokens) => {
                                // Verbatim items are unparsed token streams
                                // Try to parse as a function signature
                                let tokens_str = tokens.to_string();

                                // Normalize: replace newlines with spaces (same as line 227)
                                let tokens_normalized = tokens_str.replace(['\n', '\r'], " ");
                                let tokens_normalized = tokens_normalized
                                    .split_whitespace()
                                    .collect::<Vec<_>>()
                                    .join(" ");

                                eprintln!("Parser:   Verbatim content: '{}'", tokens_str);
                                eprintln!("Parser:   Normalized: '{}'", tokens_normalized);
                                eprintln!(
                                    "Parser:   Starts with 'fn ': {}",
                                    tokens_normalized.trim().starts_with("fn ")
                                );

                                if tokens_normalized.trim().starts_with("fn ")
                                    || tokens_normalized.trim().starts_with("async fn ")
                                    || tokens_normalized.contains("fn ")
                                {
                                    // Try adding a body and parsing as ItemFn (use normalized
                                    // version)
                                    let fn_with_body = format!(
                                        "{} {{ unimplemented!() }}",
                                        tokens_normalized.trim_end_matches(';').trim()
                                    );

                                    // Debug output
                                    eprintln!("Parser: Attempting to parse Verbatim function:");
                                    eprintln!("Parser:   Original: {}", tokens_str);
                                    eprintln!("Parser:   With body: {}", fn_with_body);

                                    if let Ok(item_fn) =
                                        syn::parse_str::<syn::ItemFn>(&fn_with_body)
                                    {
                                        eprintln!(
                                            "Parser:   ✓ SUCCESS: Parsed as ItemFn: {}",
                                            item_fn.sig.ident
                                        );
                                        signatures.push(parse_function_signature(
                                            item_fn.sig,
                                            &item_fn.attrs,
                                        ));
                                    } else {
                                        eprintln!("Parser:   ✗ FAILED: Could not parse as ItemFn");
                                    }
                                }
                            },
                            _ => {
                                // Skip other item types (use, const, etc.)
                            },
                        }
                    }
                }
            }
        }
    }

    Ok((enums, structs, signatures, trait_impls))
}

/// Parse a function signature with generics and async support (Phase 3)
fn parse_function_signature(sig: Signature, attrs: &[syn::Attribute]) -> RustFunctionSignature {
    // Extract generic parameters
    let generic_params = sig
        .generics
        .params
        .iter()
        .filter_map(|param| {
            if let syn::GenericParam::Type(type_param) = param {
                Some(GenericParam {
                    name: type_param.ident.to_string(),
                    bounds: type_param
                        .bounds
                        .iter()
                        .filter_map(|bound| {
                            if let syn::TypeParamBound::Trait(trait_bound) = bound {
                                trait_bound
                                    .path
                                    .segments
                                    .last()
                                    .map(|s| s.ident.to_string())
                            } else {
                                None
                            }
                        })
                        .collect(),
                })
            } else {
                None
            }
        })
        .collect();

    // Check if function is async
    let is_async = sig.asyncness.is_some();

    // Extract monomorphize types from attributes
    let monomorphize_types = extract_monomorphize_types(attrs);

    RustFunctionSignature {
        sig,
        generic_params,
        is_async,
        monomorphize_types,
    }
}

/// Extract types from #[monomorphize(T1, T2, ...)] attribute
fn extract_monomorphize_types(attrs: &[syn::Attribute]) -> Vec<String> {
    for attr in attrs {
        if let syn::Meta::List(meta_list) = &attr.meta {
            if meta_list.path.is_ident("monomorphize") {
                // Parse the token stream: (i32, f64, u8)
                let tokens = &meta_list.tokens;
                let tokens_str = tokens.to_string();
                // Simple comma-separated parsing
                return tokens_str
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
            }
        }
    }
    Vec::new()
}

/// Parse a trait implementation (impl Trait for Type)
fn parse_trait_impl(item_impl: ItemImpl) -> Option<RustTraitImpl> {
    // Check if this is a trait implementation (has a trait path)
    let trait_path = item_impl.trait_.as_ref()?;
    let trait_name = trait_path.1.segments.last()?.ident.to_string();

    // Get the target type name
    let target_type = if let syn::Type::Path(type_path) = &*item_impl.self_ty {
        type_path.path.segments.last()?.ident.to_string()
    } else {
        return None;
    };

    // Check if this is a zero-sized type (ZST) - no fields
    // For Phase 1, we assume all trait impl types are ZST
    let is_zst = true;

    // Phase 2: Check for opaque pointer type
    let is_opaque = false; // Will be detected from struct definition in second pass

    // Parse methods from impl block
    let mut methods = Vec::new();
    let mut constructor = None;
    let mut destructor = None;

    for impl_item in &item_impl.items {
        if let syn::ImplItem::Fn(method) = impl_item {
            // Check for #[constructor] or #[destructor] attributes
            let is_constructor_attr = has_attribute(&method.attrs, "constructor");
            let is_destructor_attr = has_attribute(&method.attrs, "destructor");

            // Extract Zig function name from method body
            if let Some(zig_function) = extract_zig_function_call(&method.block) {
                let trait_method = TraitMethod {
                    name: method.sig.ident.to_string(),
                    sig: method.sig.clone(),
                    zig_function,
                    body: Some(method.block.clone()),
                    zig_return_type: None, // Will be filled by macro with Zig code analysis
                    is_constructor: is_constructor_attr,
                    is_destructor: is_destructor_attr,
                };

                if is_constructor_attr {
                    constructor = Some(trait_method.clone());
                } else if is_destructor_attr {
                    destructor = Some(trait_method.clone());
                } else {
                    methods.push(trait_method);
                }
            }
        }
    }

    if methods.is_empty() && constructor.is_none() && destructor.is_none() {
        return None;
    }

    Some(RustTraitImpl {
        trait_name,
        target_type,
        methods,
        is_zst,
        is_opaque,
        constructor,
        destructor,
    })
}

/// Extract Zig function call from a method body
/// Looks for patterns like: zig_add(a, b)
/// Also handles bodies with conditionals like divide that checks result
fn extract_zig_function_call(block: &syn::Block) -> Option<String> {
    // Try to find any function call in the block that looks like a Zig function
    // (starts with "zig_")
    for stmt in &block.stmts {
        if let Some(zig_fn) = extract_zig_function_from_stmt(stmt) {
            return Some(zig_fn);
        }
    }
    None
}

/// Helper to extract Zig function name from a statement
fn extract_zig_function_from_stmt(stmt: &syn::Stmt) -> Option<String> {
    match stmt {
        syn::Stmt::Expr(expr, _) => extract_zig_function_from_expr(expr),
        syn::Stmt::Local(local) => {
            if let Some(init) = &local.init {
                extract_zig_function_from_expr(&init.expr)
            } else {
                None
            }
        },
        _ => None,
    }
}

/// Helper to recursively extract Zig function name from an expression
fn extract_zig_function_from_expr(expr: &syn::Expr) -> Option<String> {
    match expr {
        syn::Expr::Call(call) => {
            if let syn::Expr::Path(path) = &*call.func {
                let fn_name = path.path.segments.last()?.ident.to_string();
                // Accept any function call (not just zig_* prefix)
                // This allows for Phase 2 naming like hasher_new, hasher_free, etc.
                return Some(fn_name);
            }
            None
        },
        syn::Expr::Block(block) => {
            for stmt in &block.block.stmts {
                if let Some(zig_fn) = extract_zig_function_from_stmt(stmt) {
                    return Some(zig_fn);
                }
            }
            None
        },
        syn::Expr::If(if_expr) => {
            // Check condition
            if let Some(zig_fn) = extract_zig_function_from_expr(&if_expr.cond) {
                return Some(zig_fn);
            }
            // Check then branch
            for stmt in &if_expr.then_branch.stmts {
                if let Some(zig_fn) = extract_zig_function_from_stmt(stmt) {
                    return Some(zig_fn);
                }
            }
            // Check else branch
            if let Some((_, else_branch)) = &if_expr.else_branch {
                if let Some(zig_fn) = extract_zig_function_from_expr(else_branch) {
                    return Some(zig_fn);
                }
            }
            None
        },
        syn::Expr::Let(let_expr) => extract_zig_function_from_expr(&let_expr.expr),
        _ => None,
    }
}

/// Check if a method has a specific attribute (e.g., #[constructor])
fn has_attribute(attrs: &[syn::Attribute], name: &str) -> bool {
    attrs.iter().any(|attr| {
        if let syn::Meta::Path(path) = &attr.meta {
            path.is_ident(name)
        } else {
            false
        }
    })
}

/// Parse an inherent impl block (impl Type { ... }) for constructor/destructor
fn parse_inherent_impl(
    item_impl: ItemImpl,
    opaque_types: &std::collections::HashSet<String>,
) -> Option<RustTraitImpl> {
    // Check that this is NOT a trait implementation (no trait path)
    if item_impl.trait_.is_some() {
        return None;
    }

    // Get the target type name
    let target_type = if let syn::Type::Path(type_path) = &*item_impl.self_ty {
        type_path.path.segments.last()?.ident.to_string()
    } else {
        return None;
    };

    // Only process if this is an opaque type
    if !opaque_types.contains(&target_type) {
        return None;
    }

    // Parse methods from impl block looking for constructor/destructor
    let mut constructor = None;
    let mut destructor = None;

    eprintln!("Parser: parse_inherent_impl: Scanning {} methods", item_impl.items.len());
    for impl_item in &item_impl.items {
        if let syn::ImplItem::Fn(method) = impl_item {
            eprintln!("Parser: parse_inherent_impl:   Method: {}", method.sig.ident);
            let is_constructor_attr = has_attribute(&method.attrs, "constructor");
            let is_destructor_attr = has_attribute(&method.attrs, "destructor");
            eprintln!(
                "Parser: parse_inherent_impl:     constructor={}, destructor={}",
                is_constructor_attr, is_destructor_attr
            );

            if is_constructor_attr || is_destructor_attr {
                // Extract Zig function name from method body
                eprintln!("Parser: parse_inherent_impl:     Extracting zig function...");
                if let Some(zig_function) = extract_zig_function_call(&method.block) {
                    eprintln!(
                        "Parser: parse_inherent_impl:     Found zig function: {}",
                        zig_function
                    );
                    let trait_method = TraitMethod {
                        name: method.sig.ident.to_string(),
                        sig: method.sig.clone(),
                        zig_function,
                        body: Some(method.block.clone()),
                        zig_return_type: None,
                        is_constructor: is_constructor_attr,
                        is_destructor: is_destructor_attr,
                    };

                    if is_constructor_attr {
                        constructor = Some(trait_method);
                    } else if is_destructor_attr {
                        destructor = Some(trait_method);
                    }
                }
            }
        }
    }

    // Must have at least constructor or destructor
    if constructor.is_none() && destructor.is_none() {
        return None;
    }

    // Create a "pseudo trait impl" for the inherent impl
    // This allows us to generate the constructor/destructor without a real trait
    Some(RustTraitImpl {
        trait_name: String::new(), // No trait for inherent impl
        target_type,
        methods: Vec::new(), // No regular methods in inherent impl
        is_zst: false,
        is_opaque: true,
        constructor,
        destructor,
    })
}

/// Check if a struct is marked as opaque: struct Name(opaque);
fn is_opaque_struct(item: &ItemStruct) -> bool {
    // Check for tuple struct with single field named "opaque"
    if let syn::Fields::Unnamed(fields) = &item.fields {
        if fields.unnamed.len() == 1 {
            if let Some(field) = fields.unnamed.first() {
                if let syn::Type::Path(type_path) = &field.ty {
                    if let Some(ident) = type_path.path.get_ident() {
                        return ident == "opaque";
                    }
                }
            }
        }
    }
    false
}

/// Configuration for include_zig! macro (external file mode)
#[derive(Debug, Clone)]
pub struct IncludeZigConfig {
    /// Path to external Zig file (relative to cargo manifest dir)
    pub file_path: String,
    /// Rust function signatures for safe wrappers
    pub rust_signatures: Vec<RustFunctionSignature>,
    /// Rust struct definitions for FFI types
    pub rust_structs: Vec<RustStructDefinition>,
    /// Rust enum definitions for FFI types
    pub rust_enums: Vec<RustEnumDefinition>,
    /// Rust trait implementations
    pub rust_trait_impls: Vec<RustTraitImpl>,
}

impl Parse for IncludeZigConfig {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        // Format: include_zig!("path/to/file.zig", { Rust signatures })
        // Or: include_zig!("path/to/file.zig")

        // Parse the file path (must be a string literal)
        let file_path_lit: syn::LitStr = input.parse()?;
        let file_path = file_path_lit.value();

        // Check if there's a comma and more content
        if input.peek(syn::Token![,]) {
            let _: syn::Token![,] = input.parse()?;

            // Parse the rest as Rust definitions
            let tokens: TokenStream = input.parse()?;
            let token_str = tokens.to_string();

            let (rust_enums, rust_structs, rust_signatures, rust_trait_impls) =
                parse_rust_definitions(&token_str)?;

            Ok(IncludeZigConfig {
                file_path,
                rust_signatures,
                rust_structs,
                rust_enums,
                rust_trait_impls,
            })
        } else {
            // No signatures provided
            Ok(IncludeZigConfig {
                file_path,
                rust_signatures: Vec::new(),
                rust_structs: Vec::new(),
                rust_enums: Vec::new(),
                rust_trait_impls: Vec::new(),
            })
        }
    }
}

impl IncludeZigConfig {
    /// Get the module name for generated bindings
    pub fn get_mod_name(&self) -> &str {
        "ffi"
    }

    /// Get a unique module name based on the file path
    /// Example: "zig/math.zig" -> "ffi_zig_math"
    pub fn get_unique_mod_name(&self) -> String {
        // Remove extension and convert path separators to underscores
        let path_without_ext = self
            .file_path
            .trim_end_matches(".zig")
            .replace(['/', '\\', '.', '-'], "_");
        format!("ffi_{}", path_without_ext)
    }

    /// Check if this config has any Rust signatures
    pub fn has_rust_signatures(&self) -> bool {
        !self.rust_signatures.is_empty()
    }
}

impl AutoZigConfig {
    /// Get the module name for generated bindings
    pub fn get_mod_name(&self) -> &str {
        "ffi"
    }

    /// Check if this config has any Rust signatures
    pub fn has_rust_signatures(&self) -> bool {
        !self.rust_signatures.is_empty()
    }

    /// Check if this config uses external file
    pub fn is_external_mode(&self) -> bool {
        self.external_file.is_some()
    }
}

#[cfg(test)]
mod tests {
    use quote::quote;

    use super::*;

    #[test]
    fn test_parse_zig_only() {
        let input = quote! {
            const std = @import("std");
            export fn add(a: i32, b: i32) i32 {
                return a + b;
            }
        };

        let config: AutoZigConfig = syn::parse2(input).unwrap();
        assert!(!config.zig_code.is_empty());
        assert_eq!(config.rust_signatures.len(), 0);
    }

    #[test]
    fn test_parse_with_separator() {
        let input = quote! {
            const std = @import("std");
            export fn add(a: i32, b: i32) i32 {
                return a + b;
            }
            ---
            fn add(a: i32, b: i32) -> i32;
        };

        let config: AutoZigConfig = syn::parse2(input).unwrap();
        assert!(!config.zig_code.is_empty());
        assert_eq!(config.rust_signatures.len(), 1);
    }

    #[test]
    fn test_parse_generic_function() {
        let input = quote! {
            export fn process_i32(ptr: [*]const i32, len: usize) usize {
                return len;
            }
            ---
            #[monomorphize(i32, f64)]
            fn process<T>(data: &[T]) -> usize;
        };

        let config: AutoZigConfig = syn::parse2(input).unwrap();
        assert_eq!(config.rust_signatures.len(), 1);
        let sig = &config.rust_signatures[0];
        assert_eq!(sig.generic_params.len(), 1);
        assert_eq!(sig.generic_params[0].name, "T");
        assert_eq!(sig.monomorphize_types, vec!["i32", "f64"]);
    }

    #[test]
    fn test_parse_async_function() {
        let input = quote! {
            export fn async_compute(ptr: [*]const u8, len: usize) void {}
            ---
            async fn async_compute(data: &[u8]) -> Result<Vec<u8>, i32>;
        };

        let config: AutoZigConfig = syn::parse2(input).unwrap();
        assert_eq!(config.rust_signatures.len(), 1);
        let sig = &config.rust_signatures[0];
        assert!(sig.is_async);
    }
}
