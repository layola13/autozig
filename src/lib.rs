//! # AutoZig - Safe Rust to Zig FFI
//!
//! AutoZig enables safe, ergonomic interop between Rust and Zig code.
//!
//! ## Architecture
//!
//! AutoZig follows a three-stage pipeline inspired by autocxx:
//!
//! 1. **Parsing Stage**: Extract Zig code from `autozig!` macro invocations
//! 2. **Build Stage**: Compile Zig to static library (.a) and generate C
//!    headers
//! 3. **Binding Stage**: Generate safe Rust wrappers around raw FFI
//!
//! ## Example
//!
//! ```rust,no_run
//! use autozig::prelude::*;
//!
//! autozig! {
//!     // Zig implementation
//!     const std = @import("std");
//!
//!     export fn compute_hash(ptr: [*]const u8, len: usize) u64 {
//!         const data = ptr[0..len];
//!         var hash: u64 = 0;
//!         for (data) |byte| {
//!             hash +%= byte;
//!         }
//!         return hash;
//!     }
//!
//!     ---
//!
//!     // Rust interface (Safe wrapper)
//!     fn compute_hash(data: &[u8]) -> u64;
//! }
//!
//! let data = b"Hello AutoZig";
//! let hash = compute_hash(data); // Safe call, no unsafe needed!
//! println!("Hash: {}", hash);
//! ```

// Note: We cannot use #![forbid(unsafe_code)] because the zero_copy module
// requires unsafe for FFI and raw pointer manipulation.
#![warn(unsafe_code)]

/// Re-export the procedural macros
pub use autozig_macro::autozig;
pub use autozig_macro::{
    autozig_export,
    include_zig,
};

/// Stream support for async Zig FFI
#[cfg(feature = "stream")]
pub mod stream;

/// Zero-copy buffer passing between Zig and Rust (Phase 4.2)
pub mod zero_copy;

/// Common imports for using AutoZig
pub mod prelude {
    pub use crate::{
        autozig,
        include_zig,
    };
}

/// Placeholder for Zig type wrappers
pub mod types {
    /// Wrapper for Zig slice types
    pub struct ZigSlice<T> {
        ptr: *const T,
        len: usize,
    }

    impl<T> ZigSlice<T> {
        /// Create a ZigSlice from Rust slice
        ///
        /// # Safety
        ///
        /// The caller must ensure the slice lives long enough
        pub fn from_slice(slice: &[T]) -> Self {
            Self { ptr: slice.as_ptr(), len: slice.len() }
        }

        pub fn ptr(&self) -> *const T {
            self.ptr
        }

        pub fn len(&self) -> usize {
            self.len
        }

        pub fn is_empty(&self) -> bool {
            self.len == 0
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
