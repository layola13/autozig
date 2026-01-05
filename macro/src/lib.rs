//! Procedural macro implementation for autozig!
//!
//! This macro processes mixed Zig/Rust code and generates safe bindings
//! using IDL-driven FFI generation (no bindgen required)

#![forbid(unsafe_code)]

use autozig_parser::{
    AutoZigConfig,
    IncludeZigConfig,
};
use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;
use quote::quote;
use syn::parse_macro_input;

/// Main autozig! procedural macro
///
/// # Syntax
///
/// ```rust,ignore
/// autozig! {
///     // Zig code section
///     const std = @import("std");
///     export fn my_function(a: i32) i32 {
///         return a * 2;
///     }
///     
///     ---
///     
///     // Rust signatures for safe wrappers (optional)
///     fn my_function(a: i32) -> i32;
/// }
/// ```
///
/// The macro will:
/// 1. Extract Zig code to be compiled by build.rs (via Scanner)
/// 2. Generate extern "C" FFI bindings directly from Rust signatures
///    (IDL-driven)
/// 3. Generate safe Rust wrappers
#[proc_macro_error]
#[proc_macro]
pub fn autozig(input: TokenStream) -> TokenStream {
    let config = parse_macro_input!(input as AutoZigConfig);

    // Generate code with IDL-driven FFI
    // No bindgen needed - we generate extern "C" directly from user signatures
    let mod_name = syn::Ident::new(config.get_mod_name(), proc_macro2::Span::call_site());

    let output = if config.has_rust_signatures()
        || !config.rust_structs.is_empty()
        || !config.rust_enums.is_empty()
        || !config.rust_trait_impls.is_empty()
    {
        // Generate enum definitions (must come before struct definitions)
        let enum_defs = generate_enum_definitions(&config);

        // Generate struct definitions (must come before FFI declarations that use them)
        let struct_defs = generate_struct_definitions(&config);

        // Generate trait impl target types (ZST structs for Phase 1)
        let trait_impl_types = generate_trait_impl_types(&config);

        // Phase 3: Generate FFI declarations and wrappers with monomorphization and
        // async support
        let (ffi_decls, wrappers) = generate_with_monomorphization(&config);

        // Generate trait FFI declarations
        let trait_ffi_decls = generate_trait_ffi_declarations(&config);

        // Generate trait implementations
        let trait_impls = generate_trait_implementations(&config);

        quote! {
            // Enum definitions (visible at module level)
            #enum_defs

            // Struct definitions (visible at module level)
            #struct_defs

            // Trait impl target types (ZST structs)
            #trait_impl_types

            // Raw FFI module with extern "C" declarations
            mod #mod_name {
                use super::*;  // Import enums and structs from parent scope
                #ffi_decls
                #trait_ffi_decls
            }

            // Safe wrappers
            #wrappers

            // Trait implementations
            #trait_impls
        }
    } else {
        // No signatures provided - user must write their own FFI declarations
        quote! {
            // Note: No Rust signatures provided in autozig! macro
            // You must manually declare extern "C" functions or provide signatures after ---
            compile_error!("autozig! macro requires Rust function signatures after --- separator");
        }
    };

    TokenStream::from(output)
}

/// Generate enum definitions from IDL
fn generate_enum_definitions(config: &AutoZigConfig) -> proc_macro2::TokenStream {
    let enums: Vec<_> = config.rust_enums.iter().map(|e| &e.item).collect();

    quote! {
        #(#enums)*
    }
}

/// Generate struct definitions from IDL
fn generate_struct_definitions(config: &AutoZigConfig) -> proc_macro2::TokenStream {
    let structs: Vec<_> = config.rust_structs.iter().map(|s| &s.item).collect();

    quote! {
        #(#structs)*
    }
}

/// Check if a type is a reference to a slice or str
fn is_slice_or_str_ref(ty: &syn::Type) -> Option<(bool, Option<syn::Type>)> {
    if let syn::Type::Reference(type_ref) = ty {
        let is_mut = type_ref.mutability.is_some();

        // Check for &str or &mut str
        if let syn::Type::Path(type_path) = &*type_ref.elem {
            if type_path.path.is_ident("str") {
                return Some((is_mut, None)); // str has no element type
            }
        }

        // Check for &[T] or &mut [T]
        if let syn::Type::Slice(type_slice) = &*type_ref.elem {
            return Some((is_mut, Some((*type_slice.elem).clone())));
        }
    }
    None
}


/// Generate ZST struct types for trait implementations (Phase 1)
/// Generate Opaque Pointer struct types for stateful trait implementations
/// (Phase 2)
fn generate_trait_impl_types(config: &AutoZigConfig) -> proc_macro2::TokenStream {
    let mut type_defs = Vec::new();
    let mut generated_types = std::collections::HashSet::new();

    for trait_impl in &config.rust_trait_impls {
        // Skip if we've already generated this type
        if generated_types.contains(&trait_impl.target_type) {
            continue;
        }
        generated_types.insert(trait_impl.target_type.clone());

        let type_name = syn::Ident::new(&trait_impl.target_type, proc_macro2::Span::call_site());

        if trait_impl.is_opaque {
            // Phase 2: Generate opaque pointer struct
            type_defs.push(generate_opaque_struct(&type_name));
        } else if trait_impl.is_zst {
            // Phase 1: Generate zero-sized type with Default derive
            type_defs.push(quote! {
                #[derive(Default, Debug, Clone, Copy)]
                pub struct #type_name;
            });
        }
    }

    quote! {
        #(#type_defs)*
    }
}

/// Generate an opaque pointer struct (Phase 2)
fn generate_opaque_struct(type_name: &syn::Ident) -> proc_macro2::TokenStream {
    quote! {
        pub struct #type_name {
            inner: std::ptr::NonNull<std::ffi::c_void>,
            _marker: std::marker::PhantomData<*mut ()>,
        }

        // Opaque types are !Send and !Sync by default (via PhantomData<*mut ()>)
        // Users can manually implement Send/Sync if their Zig code is thread-safe

        // Implement Default by calling the constructor (if available)
        impl Default for #type_name {
            fn default() -> Self {
                Self::new()
            }
        }
    }
}

/// Generate trait implementations (Phase 1 & 2)
fn generate_trait_implementations(config: &AutoZigConfig) -> proc_macro2::TokenStream {
    let mut impls = Vec::new();
    let mod_name = syn::Ident::new(config.get_mod_name(), proc_macro2::Span::call_site());

    for trait_impl in &config.rust_trait_impls {
        let type_name = syn::Ident::new(&trait_impl.target_type, proc_macro2::Span::call_site());

        // Phase 2: Generate constructor if present
        if let Some(constructor) = &trait_impl.constructor {
            impls.push(generate_constructor(&type_name, constructor, &mod_name));
        }

        // Phase 2: Generate Drop implementation if destructor present
        if let Some(destructor) = &trait_impl.destructor {
            impls.push(generate_drop_impl(&type_name, destructor, &mod_name));
        }

        // Skip trait impl generation if this is an inherent impl (empty trait name)
        if trait_impl.trait_name.is_empty() {
            continue;
        }

        let trait_name = syn::Ident::new(&trait_impl.trait_name, proc_macro2::Span::call_site());

        // Generate methods for the trait implementation
        let mut methods = Vec::new();
        for method in &trait_impl.methods {
            let method_sig = &method.sig;
            let method_name = &method_sig.ident;
            let inputs = &method_sig.inputs;
            let return_type = &method_sig.output;

            // Phase 2: For opaque types, always generate FFI call (ignore user's simplified
            // body) Phase 1: Use original method body if available (preserves
            // user logic like Option wrapping)
            let should_generate_ffi_call = trait_impl.is_opaque || method.body.is_none();

            if !should_generate_ffi_call {
                // Phase 1: Use the original body with unsafe wrapper (for ZST with complex
                // logic)
                if let Some(original_body) = &method.body {
                    methods.push(quote! {
                        fn #method_name(#inputs) #return_type {
                            unsafe #original_body
                        }
                    });
                }
            } else {
                // Fallback: generate simple FFI call
                let zig_fn = syn::Ident::new(&method.zig_function, proc_macro2::Span::call_site());

                let mut ffi_args = Vec::new();

                // Phase 2: Inject self pointer for opaque types
                if trait_impl.is_opaque {
                    ffi_args.push(inject_self_pointer(&method_sig));
                }

                for input in &method_sig.inputs {
                    if let syn::FnArg::Receiver(_) = input {
                        // Skip self/&self/&mut self - already handled above
                        continue;
                    }

                    if let syn::FnArg::Typed(pat_type) = input {
                        if let syn::Pat::Ident(ident) = &*pat_type.pat {
                            let param_name = &ident.ident;

                            if let Some((is_mut, _elem_type)) = is_slice_or_str_ref(&pat_type.ty) {
                                if is_mut {
                                    ffi_args.push(quote! { #param_name.as_mut_ptr() });
                                } else {
                                    ffi_args.push(quote! { #param_name.as_ptr() });
                                }
                                ffi_args.push(quote! { #param_name.len() });
                            } else {
                                ffi_args.push(quote! { #param_name });
                            }
                        }
                    }
                }

                methods.push(quote! {
                    fn #method_name(#inputs) #return_type {
                        unsafe {
                            #mod_name::#zig_fn(#(#ffi_args),*)
                        }
                    }
                });
            }
        }

        // Generate the complete impl block
        impls.push(quote! {
            impl #trait_name for #type_name {
                #(#methods)*
            }
        });
    }

    quote! {
        #(#impls)*
    }
}

/// Generate constructor for opaque types (Phase 2)
fn generate_constructor(
    type_name: &syn::Ident,
    constructor: &autozig_parser::TraitMethod,
    mod_name: &syn::Ident,
) -> proc_macro2::TokenStream {
    let zig_fn = syn::Ident::new(&constructor.zig_function, proc_macro2::Span::call_site());
    let method_name = syn::Ident::new(&constructor.name, proc_macro2::Span::call_site());

    // Get parameters (excluding self)
    let params: Vec<_> = constructor
        .sig
        .inputs
        .iter()
        .filter_map(|input| {
            if let syn::FnArg::Typed(pat_type) = input {
                Some(pat_type)
            } else {
                None
            }
        })
        .collect();

    let param_names: Vec<_> = params
        .iter()
        .filter_map(|pat_type| {
            if let syn::Pat::Ident(ident) = &*pat_type.pat {
                Some(&ident.ident)
            } else {
                None
            }
        })
        .collect();

    let inputs = &constructor.sig.inputs;

    quote! {
        impl #type_name {
            pub fn #method_name(#inputs) -> Self {
                unsafe {
                    let ptr = #mod_name::#zig_fn(#(#param_names),*);
                    std::ptr::NonNull::new(ptr as *mut std::ffi::c_void)
                        .map(|inner| Self {
                            inner,
                            _marker: std::marker::PhantomData,
                        })
                        .expect("Zig allocation failed (OOM)")
                }
            }
        }
    }
}

/// Generate Drop implementation for opaque types (Phase 2)
fn generate_drop_impl(
    type_name: &syn::Ident,
    destructor: &autozig_parser::TraitMethod,
    mod_name: &syn::Ident,
) -> proc_macro2::TokenStream {
    let zig_fn = syn::Ident::new(&destructor.zig_function, proc_macro2::Span::call_site());

    quote! {
        impl Drop for #type_name {
            fn drop(&mut self) {
                unsafe {
                    #mod_name::#zig_fn(self.inner.as_ptr());
                }
            }
        }
    }
}

/// Inject self pointer as first argument for opaque types (Phase 2)
fn inject_self_pointer(sig: &syn::Signature) -> proc_macro2::TokenStream {
    // Check receiver type: &self or &mut self
    for input in &sig.inputs {
        if let syn::FnArg::Receiver(receiver) = input {
            if receiver.mutability.is_some() {
                // &mut self -> *mut c_void
                return quote! { self.inner.as_ptr() };
            } else {
                // &self -> *const c_void
                return quote! { self.inner.as_ptr() as *const std::ffi::c_void };
            }
        }
    }

    // No receiver, shouldn't happen for trait methods
    quote! {}
}

/// Generate FFI declarations for Zig functions used in trait implementations
/// (Phase 1 & 2)
fn generate_trait_ffi_declarations(config: &AutoZigConfig) -> proc_macro2::TokenStream {
    let mut decls = Vec::new();

    for trait_impl in &config.rust_trait_impls {
        // Phase 2: Generate constructor FFI declaration
        if let Some(constructor) = &trait_impl.constructor {
            let zig_fn = syn::Ident::new(&constructor.zig_function, proc_macro2::Span::call_site());
            let params: Vec<_> = constructor
                .sig
                .inputs
                .iter()
                .filter_map(|input| {
                    if let syn::FnArg::Typed(pat_type) = input {
                        let param_name = &pat_type.pat;
                        let param_type = &pat_type.ty;
                        Some(quote! { #param_name: #param_type })
                    } else {
                        None
                    }
                })
                .collect();

            decls.push(quote! {
                extern "C" {
                    pub fn #zig_fn(#(#params),*) -> *mut std::ffi::c_void;
                }
            });
        }

        // Phase 2: Generate destructor FFI declaration
        if let Some(destructor) = &trait_impl.destructor {
            let zig_fn = syn::Ident::new(&destructor.zig_function, proc_macro2::Span::call_site());

            decls.push(quote! {
                extern "C" {
                    pub fn #zig_fn(ptr: *mut std::ffi::c_void);
                }
            });
        }

        for method in &trait_impl.methods {
            let zig_fn = syn::Ident::new(&method.zig_function, proc_macro2::Span::call_site());
            let method_sig = &method.sig;

            // Build FFI parameter list
            let mut ffi_params = Vec::new();

            // Phase 2: Add self pointer parameter for opaque types
            if trait_impl.is_opaque {
                let self_param = handle_receiver_type(&method_sig);
                if !self_param.is_empty() {
                    ffi_params.push(self_param);
                }
            }

            for input in &method_sig.inputs {
                if let syn::FnArg::Receiver(_) = input {
                    // Skip &self / &mut self for ZST (Phase 1)
                    // For opaque types, already handled above
                    continue;
                }

                if let syn::FnArg::Typed(pat_type) = input {
                    let param_name = &pat_type.pat;
                    let param_type = &pat_type.ty;

                    // Check if this is a slice or str reference
                    if let Some((is_mut, elem_type)) = is_slice_or_str_ref(param_type) {
                        // Extract parameter name as string
                        let param_name_str = if let syn::Pat::Ident(ident) = &*pat_type.pat {
                            ident.ident.to_string()
                        } else {
                            continue;
                        };

                        // Lower to ptr + len
                        let ptr_type = if let Some(elem) = elem_type {
                            if is_mut {
                                quote! { *mut #elem }
                            } else {
                                quote! { *const #elem }
                            }
                        } else {
                            if is_mut {
                                quote! { *mut u8 }
                            } else {
                                quote! { *const u8 }
                            }
                        };

                        let ptr_name = quote::format_ident!("{}_ptr", param_name_str);
                        let len_name = quote::format_ident!("{}_len", param_name_str);

                        ffi_params.push(quote! { #ptr_name: #ptr_type });
                        ffi_params.push(quote! { #len_name: usize });
                    } else {
                        ffi_params.push(quote! { #param_name: #param_type });
                    }
                }
            }

            // Extract Zig function return type from Zig code
            let zig_return_type = extract_zig_return_type(&config.zig_code, &method.zig_function);
            let return_type = if let Some(zig_ret) = zig_return_type {
                zig_ret
            } else {
                // Fallback: use method signature for most cases, but we need special handling
                // If the method returns Option but doesn't have the Option in its body,
                // it likely means the Zig function returns the unwrapped type
                method_sig.output.clone()
            };

            decls.push(quote! {
                extern "C" {
                    pub fn #zig_fn(#(#ffi_params),*) #return_type;
                }
            });
        }
    }

    quote! {
        #(#decls)*
    }
}

/// Extract return type from Zig function definition
/// Looks for patterns like: `export fn function_name(...) TYPE {`
fn extract_zig_return_type(zig_code: &str, fn_name: &str) -> Option<syn::ReturnType> {
    // Simple string-based extraction
    // Find "export fn function_name" - handle possible newline before function name
    let search_pattern1 = format!("export fn {}", fn_name);
    let search_pattern2 = format!("export fn\n{}", fn_name);

    let start_pos = if let Some(pos) = zig_code.find(&search_pattern1) {
        pos
    } else if let Some(pos) = zig_code.find(&search_pattern2) {
        pos
    } else {
        return None;
    };

    // Find the closing parenthesis of parameters
    let after_fn = &zig_code[start_pos..];
    let paren_start = after_fn.find('(')?;
    let mut paren_count = 1;
    let mut paren_end = paren_start + 1;

    for (i, ch) in after_fn[paren_start + 1..].chars().enumerate() {
        match ch {
            '(' => paren_count += 1,
            ')' => {
                paren_count -= 1;
                if paren_count == 0 {
                    paren_end = paren_start + 1 + i;
                    break;
                }
            },
            _ => {},
        }
    }

    // Extract return type between ')' and '{'
    let after_paren = &after_fn[paren_end + 1..];
    let brace_pos = after_paren.find('{')?;
    let return_type_str = after_paren[..brace_pos].trim();

    // Map Zig types to Rust types
    let rust_type = match return_type_str {
        "i32" => quote! { -> i32 },
        "u32" => quote! { -> u32 },
        "i64" => quote! { -> i64 },
        "u64" => quote! { -> u64 },
        "f32" => quote! { -> f32 },
        "f64" => quote! { -> f64 },
        "bool" => quote! { -> bool },
        "void" => quote! {},
        _ => return None, // Unknown type, fall back to method signature
    };

    syn::parse2(rust_type).ok()
}

/// Handle receiver type for FFI parameter list (Phase 2)
/// Returns the self pointer parameter for opaque types
fn handle_receiver_type(sig: &syn::Signature) -> proc_macro2::TokenStream {
    for input in &sig.inputs {
        if let syn::FnArg::Receiver(receiver) = input {
            if receiver.mutability.is_some() {
                // &mut self -> *mut c_void
                return quote! { self_ptr: *mut std::ffi::c_void };
            } else {
                // &self -> *const c_void
                return quote! { self_ptr: *const std::ffi::c_void };
            }
        }
    }
    quote! {}
}

/// include_zig! macro for referencing external Zig files
///
/// # Syntax
///
/// ```rust,ignore
/// include_zig!("path/to/file.zig", {
///     // Rust function signatures
///     fn my_function(a: i32) -> i32;
/// });
/// ```
///
/// The path is relative to the Cargo manifest directory.
#[proc_macro_error]
#[proc_macro]
pub fn include_zig(input: TokenStream) -> TokenStream {
    let config = parse_macro_input!(input as IncludeZigConfig);

    // Generate unique module name based on file path
    // Convert "zig/math.zig" to "ffi_zig_math"
    let mod_name = config.get_unique_mod_name();
    let mod_name_ident = syn::Ident::new(&mod_name, proc_macro2::Span::call_site());
    let file_path = &config.file_path;

    // Generate a marker that build.rs can detect
    // We use a const string that scanner will find
    let marker_code = format!("// @autozig:include:{}", file_path);

    let output = if config.has_rust_signatures()
        || !config.rust_structs.is_empty()
        || !config.rust_enums.is_empty()
    {
        // Generate enum definitions
        let enum_defs = generate_enum_definitions_for_include(&config);

        // Generate struct definitions
        let struct_defs = generate_struct_definitions_for_include(&config);

        // Phase 3: Use monomorphization-aware generation for include_zig! too
        let (ffi_decls, wrappers) = generate_with_monomorphization_for_include(&config);

        quote! {
            // Marker for scanner (will be removed in final output)
            #[doc = #marker_code]

            // Enum definitions (visible at module level)
            #enum_defs

            // Struct definitions (visible at module level)
            #struct_defs

            // Raw FFI module with extern "C" declarations (unique name per file)
            mod #mod_name_ident {
                use super::*;
                #ffi_decls
            }

            // Safe wrappers
            #wrappers
        }
    } else {
        quote! {
            #[doc = #marker_code]
            compile_error!("include_zig! macro requires Rust function signatures");
        }
    };

    TokenStream::from(output)
}

/// Helper functions for include_zig! - reuse the same logic as autozig!
fn generate_enum_definitions_for_include(config: &IncludeZigConfig) -> proc_macro2::TokenStream {
    let enums: Vec<_> = config.rust_enums.iter().map(|e| &e.item).collect();
    quote! {
        #(#enums)*
    }
}

fn generate_struct_definitions_for_include(config: &IncludeZigConfig) -> proc_macro2::TokenStream {
    let structs: Vec<_> = config.rust_structs.iter().map(|s| &s.item).collect();
    quote! {
        #(#structs)*
    }
}

fn generate_ffi_declarations_for_include(config: &IncludeZigConfig) -> proc_macro2::TokenStream {
    let mut decls = Vec::new();

    for rust_sig in &config.rust_signatures {
        let sig = &rust_sig.sig;
        let fn_name = &sig.ident;
        let output = &sig.output;

        let mut ffi_params = Vec::new();

        for input in &sig.inputs {
            if let syn::FnArg::Typed(pat_type) = input {
                let param_type = &pat_type.ty;
                let param_name_str = if let syn::Pat::Ident(ident) = &*pat_type.pat {
                    ident.ident.to_string()
                } else {
                    continue;
                };

                if let Some((is_mut, elem_type)) = is_slice_or_str_ref(param_type) {
                    let ptr_type = if let Some(elem) = elem_type {
                        if is_mut {
                            quote! { *mut #elem }
                        } else {
                            quote! { *const #elem }
                        }
                    } else {
                        if is_mut {
                            quote! { *mut u8 }
                        } else {
                            quote! { *const u8 }
                        }
                    };

                    let ptr_name = quote::format_ident!("{}_ptr", param_name_str);
                    let len_name = quote::format_ident!("{}_len", param_name_str);

                    ffi_params.push(quote! { #ptr_name: #ptr_type });
                    ffi_params.push(quote! { #len_name: usize });
                } else {
                    let param_name = &pat_type.pat;
                    ffi_params.push(quote! { #param_name: #param_type });
                }
            }
        }

        decls.push(quote! {
            extern "C" {
                pub fn #fn_name(#(#ffi_params),*) #output;
            }
        });
    }

    quote! {
        #(#decls)*
    }
}

fn generate_safe_wrappers_for_include(config: &IncludeZigConfig) -> proc_macro2::TokenStream {
    let mut wrappers = Vec::new();
    let mod_name_str = config.get_unique_mod_name();
    let mod_name = syn::Ident::new(&mod_name_str, proc_macro2::Span::call_site());

    for rust_sig in &config.rust_signatures {
        let sig = &rust_sig.sig;
        let fn_name = &sig.ident;
        let inputs = &sig.inputs;
        let output = &sig.output;

        let mut ffi_args = Vec::new();

        for input in &sig.inputs {
            if let syn::FnArg::Typed(pat_type) = input {
                if let syn::Pat::Ident(ident) = &*pat_type.pat {
                    let param_name = &ident.ident;
                    let param_type = &pat_type.ty;

                    if let Some((is_mut, _elem_type)) = is_slice_or_str_ref(param_type) {
                        if is_mut {
                            ffi_args.push(quote! { #param_name.as_mut_ptr() });
                        } else {
                            ffi_args.push(quote! { #param_name.as_ptr() });
                        }
                        ffi_args.push(quote! { #param_name.len() });
                    } else {
                        ffi_args.push(quote! { #param_name });
                    }
                }
            }
        }

        let wrapper = quote! {
            pub fn #fn_name(#inputs) #output {
                unsafe {
                    #mod_name::#fn_name(#(#ffi_args),*)
                }
            }
        };

        wrappers.push(wrapper);
    }

    quote! {
        #(#wrappers)*
    }
}

fn generate_trait_impl_types_for_include(config: &IncludeZigConfig) -> proc_macro2::TokenStream {
    let mut type_defs = Vec::new();

    for trait_impl in &config.rust_trait_impls {
        if trait_impl.is_zst {
            let type_name =
                syn::Ident::new(&trait_impl.target_type, proc_macro2::Span::call_site());

            type_defs.push(quote! {
                #[derive(Default, Debug, Clone, Copy)]
                pub struct #type_name;
            });
        }
    }

    quote! {
        #(#type_defs)*
    }
}

fn generate_trait_implementations_for_include(
    config: &IncludeZigConfig,
) -> proc_macro2::TokenStream {
    let mut impls = Vec::new();
    let mod_name_str = config.get_unique_mod_name();
    let mod_name = syn::Ident::new(&mod_name_str, proc_macro2::Span::call_site());

    for trait_impl in &config.rust_trait_impls {
        let trait_name = syn::Ident::new(&trait_impl.trait_name, proc_macro2::Span::call_site());
        let type_name = syn::Ident::new(&trait_impl.target_type, proc_macro2::Span::call_site());

        let mut methods = Vec::new();
        for method in &trait_impl.methods {
            let method_sig = &method.sig;
            let method_name = &method_sig.ident;
            let zig_fn = syn::Ident::new(&method.zig_function, proc_macro2::Span::call_site());

            let mut ffi_args = Vec::new();
            for input in &method_sig.inputs {
                if let syn::FnArg::Typed(pat_type) = input {
                    if let syn::Pat::Ident(ident) = &*pat_type.pat {
                        let param_name = &ident.ident;

                        if let Some((is_mut, _elem_type)) = is_slice_or_str_ref(&pat_type.ty) {
                            if is_mut {
                                ffi_args.push(quote! { #param_name.as_mut_ptr() });
                            } else {
                                ffi_args.push(quote! { #param_name.as_ptr() });
                            }
                            ffi_args.push(quote! { #param_name.len() });
                        } else {
                            ffi_args.push(quote! { #param_name });
                        }
                    }
                }
            }

            let return_type = &method_sig.output;

            methods.push(quote! {
                fn #method_name(#method_sig) #return_type {
                    unsafe {
                        #mod_name::#zig_fn(#(#ffi_args),*)
                    }
                }
            });
        }

        impls.push(quote! {
            impl #trait_name for #type_name {
                #(#methods)*
            }
        });
    }

    quote! {
        #(#impls)*
    }
}

// ============================================================================
// Phase 3: Generics and Async Support
// ============================================================================

/// Phase 3: Generate FFI declarations and wrappers with monomorphization
/// support
fn generate_with_monomorphization(
    config: &AutoZigConfig,
) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
    let mut all_ffi_decls = Vec::new();
    let mut all_wrappers = Vec::new();

    for rust_sig in &config.rust_signatures {
        if !rust_sig.generic_params.is_empty() && !rust_sig.monomorphize_types.is_empty() {
            // Generic function with monomorphization attribute
            let (mono_ffi, mono_wrappers) =
                generate_monomorphized_versions(rust_sig, config.get_mod_name());
            all_ffi_decls.push(mono_ffi);
            all_wrappers.push(mono_wrappers);
        } else if rust_sig.is_async {
            // Async function
            let (async_ffi, async_wrapper) =
                generate_async_ffi_and_wrapper(rust_sig, config.get_mod_name());
            all_ffi_decls.push(async_ffi);
            all_wrappers.push(async_wrapper);
        } else {
            // Regular function (non-generic, non-async)
            let ffi_decl = generate_single_ffi_declaration(rust_sig);
            let wrapper = generate_single_safe_wrapper(rust_sig, config.get_mod_name());
            all_ffi_decls.push(ffi_decl);
            all_wrappers.push(wrapper);
        }
    }

    let ffi_decls = quote! { #(#all_ffi_decls)* };
    let wrappers = quote! { #(#all_wrappers)* };

    (ffi_decls, wrappers)
}

/// Generate single FFI declaration for regular (non-generic) function
fn generate_single_ffi_declaration(
    rust_sig: &autozig_parser::RustFunctionSignature,
) -> proc_macro2::TokenStream {
    let sig = &rust_sig.sig;
    let fn_name = &sig.ident;
    let output = &sig.output;

    let mut ffi_params = Vec::new();

    for input in &sig.inputs {
        if let syn::FnArg::Typed(pat_type) = input {
            let param_type = &pat_type.ty;
            let param_name_str = if let syn::Pat::Ident(ident) = &*pat_type.pat {
                ident.ident.to_string()
            } else {
                continue;
            };

            if let Some((is_mut, elem_type)) = is_slice_or_str_ref(param_type) {
                let ptr_type = if let Some(elem) = elem_type {
                    if is_mut {
                        quote! { *mut #elem }
                    } else {
                        quote! { *const #elem }
                    }
                } else {
                    if is_mut {
                        quote! { *mut u8 }
                    } else {
                        quote! { *const u8 }
                    }
                };

                let ptr_name = quote::format_ident!("{}_ptr", param_name_str);
                let len_name = quote::format_ident!("{}_len", param_name_str);

                ffi_params.push(quote! { #ptr_name: #ptr_type });
                ffi_params.push(quote! { #len_name: usize });
            } else {
                let param_name = &pat_type.pat;
                ffi_params.push(quote! { #param_name: #param_type });
            }
        }
    }

    quote! {
        extern "C" {
            pub fn #fn_name(#(#ffi_params),*) #output;
        }
    }
}

/// Generate single safe wrapper for regular (non-generic) function
fn generate_single_safe_wrapper(
    rust_sig: &autozig_parser::RustFunctionSignature,
    mod_name: &str,
) -> proc_macro2::TokenStream {
    let sig = &rust_sig.sig;
    let fn_name = &sig.ident;
    let inputs = &sig.inputs;
    let output = &sig.output;
    let mod_ident = syn::Ident::new(mod_name, proc_macro2::Span::call_site());

    let mut ffi_args = Vec::new();

    for input in &sig.inputs {
        if let syn::FnArg::Typed(pat_type) = input {
            if let syn::Pat::Ident(ident) = &*pat_type.pat {
                let param_name = &ident.ident;
                let param_type = &pat_type.ty;

                if let Some((is_mut, _elem_type)) = is_slice_or_str_ref(param_type) {
                    if is_mut {
                        ffi_args.push(quote! { #param_name.as_mut_ptr() });
                    } else {
                        ffi_args.push(quote! { #param_name.as_ptr() });
                    }
                    ffi_args.push(quote! { #param_name.len() });
                } else {
                    ffi_args.push(quote! { #param_name });
                }
            }
        }
    }

    quote! {
        pub fn #fn_name(#inputs) #output {
            unsafe {
                #mod_ident::#fn_name(#(#ffi_args),*)
            }
        }
    }
}

/// Phase 3: Generate monomorphized versions for a generic function
fn generate_monomorphized_versions(
    rust_sig: &autozig_parser::RustFunctionSignature,
    mod_name: &str,
) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
    let mut ffi_decls = Vec::new();
    let mut wrappers = Vec::new();

    let base_name = &rust_sig.sig.ident;

    for mono_type in &rust_sig.monomorphize_types {
        // Generate mangled name: process<T> + i32 -> process_i32
        let mono_name = syn::Ident::new(
            &format!("{}_{}", base_name, mono_type.replace("::", "_")),
            proc_macro2::Span::call_site(),
        );

        // Substitute generic type T with concrete type
        let mono_sig = substitute_generic_type(&rust_sig.sig, mono_type);

        // Generate FFI declaration for this monomorphized version
        let ffi_decl = generate_ffi_declaration_from_sig(&mono_name, &mono_sig);
        ffi_decls.push(ffi_decl);

        // Generate safe wrapper for this monomorphized version
        let wrapper = generate_wrapper_from_sig(&mono_name, &mono_sig, mod_name);
        wrappers.push(wrapper);
    }

    let ffi_output = quote! { #(#ffi_decls)* };
    let wrapper_output = quote! { #(#wrappers)* };

    (ffi_output, wrapper_output)
}

/// Substitute generic type parameter with concrete type
fn substitute_generic_type(sig: &syn::Signature, concrete_type: &str) -> syn::Signature {
    let mut new_sig = sig.clone();

    // Parse concrete type
    let concrete_ty: syn::Type =
        syn::parse_str(concrete_type).unwrap_or_else(|_| panic!("Invalid type: {}", concrete_type));

    // Get generic parameter name (e.g., "T")
    let generic_name =
        if let Some(syn::GenericParam::Type(type_param)) = sig.generics.params.first() {
            type_param.ident.to_string()
        } else {
            return new_sig; // No generics
        };

    // Remove generics from signature
    new_sig.generics = syn::Generics::default();

    // Substitute type in parameters
    for input in &mut new_sig.inputs {
        if let syn::FnArg::Typed(pat_type) = input {
            pat_type.ty =
                Box::new(substitute_type_recursive(&pat_type.ty, &generic_name, &concrete_ty));
        }
    }

    // Substitute type in return type
    if let syn::ReturnType::Type(_, ret_ty) = &mut new_sig.output {
        *ret_ty = Box::new(substitute_type_recursive(ret_ty, &generic_name, &concrete_ty));
    }

    new_sig
}

/// Recursively substitute generic type in a type expression
fn substitute_type_recursive(
    ty: &syn::Type,
    generic_name: &str,
    concrete_ty: &syn::Type,
) -> syn::Type {
    match ty {
        syn::Type::Path(type_path) => {
            // Check if this is the generic parameter
            if type_path.path.is_ident(generic_name) {
                concrete_ty.clone()
            } else {
                ty.clone()
            }
        },
        syn::Type::Reference(type_ref) => {
            let mut new_ref = type_ref.clone();
            new_ref.elem =
                Box::new(substitute_type_recursive(&type_ref.elem, generic_name, concrete_ty));
            syn::Type::Reference(new_ref)
        },
        syn::Type::Slice(type_slice) => {
            let mut new_slice = type_slice.clone();
            new_slice.elem =
                Box::new(substitute_type_recursive(&type_slice.elem, generic_name, concrete_ty));
            syn::Type::Slice(new_slice)
        },
        _ => ty.clone(),
    }
}

/// Generate FFI declaration from signature with specific name
fn generate_ffi_declaration_from_sig(
    fn_name: &syn::Ident,
    sig: &syn::Signature,
) -> proc_macro2::TokenStream {
    let output = &sig.output;

    let mut ffi_params = Vec::new();

    for input in &sig.inputs {
        if let syn::FnArg::Typed(pat_type) = input {
            let param_type = &pat_type.ty;
            let param_name_str = if let syn::Pat::Ident(ident) = &*pat_type.pat {
                ident.ident.to_string()
            } else {
                continue;
            };

            if let Some((is_mut, elem_type)) = is_slice_or_str_ref(param_type) {
                let ptr_type = if let Some(elem) = elem_type {
                    if is_mut {
                        quote! { *mut #elem }
                    } else {
                        quote! { *const #elem }
                    }
                } else {
                    if is_mut {
                        quote! { *mut u8 }
                    } else {
                        quote! { *const u8 }
                    }
                };

                let ptr_name = quote::format_ident!("{}_ptr", param_name_str);
                let len_name = quote::format_ident!("{}_len", param_name_str);

                ffi_params.push(quote! { #ptr_name: #ptr_type });
                ffi_params.push(quote! { #len_name: usize });
            } else {
                let param_name = &pat_type.pat;
                ffi_params.push(quote! { #param_name: #param_type });
            }
        }
    }

    quote! {
        extern "C" {
            pub fn #fn_name(#(#ffi_params),*) #output;
        }
    }
}

/// Generate safe wrapper from signature with specific name
fn generate_wrapper_from_sig(
    fn_name: &syn::Ident,
    sig: &syn::Signature,
    mod_name: &str,
) -> proc_macro2::TokenStream {
    let mod_ident = syn::Ident::new(mod_name, proc_macro2::Span::call_site());
    let inputs = &sig.inputs;
    let output = &sig.output;

    let mut ffi_args = Vec::new();

    for input in &sig.inputs {
        if let syn::FnArg::Typed(pat_type) = input {
            if let syn::Pat::Ident(ident) = &*pat_type.pat {
                let param_name = &ident.ident;
                let param_type = &pat_type.ty;

                if let Some((is_mut, _elem_type)) = is_slice_or_str_ref(param_type) {
                    if is_mut {
                        ffi_args.push(quote! { #param_name.as_mut_ptr() });
                    } else {
                        ffi_args.push(quote! { #param_name.as_ptr() });
                    }
                    ffi_args.push(quote! { #param_name.len() });
                } else {
                    ffi_args.push(quote! { #param_name });
                }
            }
        }
    }

    quote! {
        /// Monomorphized wrapper (generated by autozig)
        pub fn #fn_name(#inputs) #output {
            unsafe {
                #mod_ident::#fn_name(#(#ffi_args),*)
            }
        }
    }
}

/// Phase 3.2: Generate async FFI and wrapper using spawn_blocking pattern
/// Architecture: "Rust Async Wrapper, Zig Sync Execution"
/// - Zig writes normal synchronous code (no async/await needed in Zig)
/// - Rust async fn automatically uses tokio::task::spawn_blocking
/// - This prevents blocking the async runtime while maintaining async interface
fn generate_async_ffi_and_wrapper(
    rust_sig: &autozig_parser::RustFunctionSignature,
    mod_name: &str,
) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
    let fn_name = &rust_sig.sig.ident;
    let sig = &rust_sig.sig;

    // Generate standard synchronous FFI declaration
    // Zig side is always synchronous - no async/await needed!
    let ffi_decl = generate_ffi_declaration_from_sig(fn_name, sig);

    // Build wrapper parameters and FFI call arguments
    let inputs = &sig.inputs;
    let output = &sig.output;
    let mod_ident = syn::Ident::new(mod_name, proc_macro2::Span::call_site());

    let mut ffi_args = Vec::new();
    let mut param_captures = Vec::new();

    for input in &sig.inputs {
        if let syn::FnArg::Typed(pat_type) = input {
            if let syn::Pat::Ident(ident) = &*pat_type.pat {
                let param_name = &ident.ident;
                let param_type = &pat_type.ty;

                // For async, we need to move parameters into the closure
                // For slices/strings, we need to convert to owned data
                if let Some((_is_mut, _elem_type)) = is_slice_or_str_ref(param_type) {
                    // Convert slice to Vec to own the data
                    param_captures.push(quote! {
                        let #param_name = #param_name.to_vec();
                    });

                    ffi_args.push(quote! { #param_name.as_ptr() });
                    ffi_args.push(quote! { #param_name.len() });
                } else {
                    // For Copy types, just capture them
                    ffi_args.push(quote! { #param_name });
                }
            }
        }
    }

    // Generate async wrapper using spawn_blocking
    let wrapper = quote! {
        /// Async wrapper (auto-generated by AutoZig Phase 3.2)
        ///
        /// This function uses tokio::task::spawn_blocking to offload the
        /// synchronous Zig FFI call to a dedicated thread pool, preventing
        /// blocking of the async runtime.
        ///
        /// Zig side: Write normal synchronous code, no async/await needed!
        pub async fn #fn_name(#inputs) #output {
            // Capture parameters (convert slices to owned Vec)
            #(#param_captures)*

            // Offload to blocking thread pool
            tokio::task::spawn_blocking(move || {
                unsafe {
                    #mod_ident::#fn_name(#(#ffi_args),*)
                }
            })
            .await
            .expect("Zig task panicked or was cancelled")
        }
    };

    (ffi_decl, wrapper)
}

/// Phase 3: Generate FFI declarations and wrappers with monomorphization
/// support for include_zig!
fn generate_with_monomorphization_for_include(
    config: &IncludeZigConfig,
) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
    let mut all_ffi_decls = Vec::new();
    let mut all_wrappers = Vec::new();
    let mod_name = config.get_unique_mod_name();

    for rust_sig in &config.rust_signatures {
        if !rust_sig.generic_params.is_empty() && !rust_sig.monomorphize_types.is_empty() {
            // Generic function with monomorphization attribute
            let (mono_ffi, mono_wrappers) = generate_monomorphized_versions(rust_sig, &mod_name);
            all_ffi_decls.push(mono_ffi);
            all_wrappers.push(mono_wrappers);
        } else if rust_sig.is_async {
            // Async function
            let (async_ffi, async_wrapper) = generate_async_ffi_and_wrapper(rust_sig, &mod_name);
            all_ffi_decls.push(async_ffi);
            all_wrappers.push(async_wrapper);
        } else {
            // Regular function (non-generic, non-async)
            let ffi_decl = generate_single_ffi_declaration(rust_sig);
            let wrapper = generate_single_safe_wrapper(rust_sig, &mod_name);
            all_ffi_decls.push(ffi_decl);
            all_wrappers.push(wrapper);
        }
    }

    let ffi_decls = quote! { #(#all_ffi_decls)* };
    let wrappers = quote! { #(#all_wrappers)* };

    (ffi_decls, wrappers)
}

#[cfg(test)]
mod tests {
    // Proc macro tests would go here
    // Testing proc macros is tricky, usually done with integration tests
}
