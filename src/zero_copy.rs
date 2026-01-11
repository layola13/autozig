//! # Zero-Copy Buffer Passing (Phase 4.2)
//!
//! This module requires unsafe code for FFI and raw pointer manipulation.
#![allow(unsafe_code)]

//! This module enables zero-copy data transfer between Zig and Rust by sharing
//! memory ownership without serialization overhead.
//!
//! ## Key Features
//!
//! - Direct memory ownership transfer from Zig to Rust
//! - No serialization/deserialization overhead
//! - Supports large data transfers (>1MB)
//! - Uses C allocator for allocator compatibility
//!
//! ## Architecture
//!
//! ```text
//! Zig Side:                          Rust Side:
//! ┌─────────────────┐               ┌─────────────────┐
//! │ c_allocator     │               │ Vec::from_raw   │
//! │ alloc memory    │───────────────│ parts           │
//! │ RustVec layout  │  ownership    │ (zero-copy)     │
//! └─────────────────┘   transfer    └─────────────────┘
//! ```
//!
//! ## Safety Contract
//!
//! Both Rust and Zig MUST use compatible allocators:
//! - Zig: `std.heap.c_allocator`
//! - Rust: System allocator (default) or explicit `std::alloc::System`
//!
//! ## Example
//!
//! ```rust,ignore
//! use autozig::zero_copy::ZeroCopyBuffer;
//!
//! // Zig function returns zero-copy buffer
//! let buffer: Vec<i32> = generate_large_data(1_000_000);
//! // No copy occurred! Direct memory ownership transfer
//! ```

use std::{
    marker::PhantomData,
    slice,
};

/// Raw components of a Rust `Vec<T>`, compatible with C FFI
///
/// This struct has the exact same memory layout as `Vec<T>`:
/// - `ptr`: pointer to the data
/// - `len`: number of elements
/// - `cap`: capacity (number of elements that can fit)
///
/// # Safety
///
/// This struct must only be used with memory allocated by
/// `std.heap.c_allocator` in Zig, which is compatible with Rust's system
/// allocator.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct RawVec<T> {
    /// Pointer to the first element
    pub ptr: *mut T,
    /// Number of elements in the vector
    pub len: usize,
    /// Capacity (allocated size in elements)
    pub cap: usize,
    /// PhantomData to ensure proper variance and drop checking (zero-sized,
    /// doesn't affect layout)
    pub _phantom: PhantomData<T>,
}

impl<T> RawVec<T> {
    /// Create a new `RawVec` from raw components
    ///
    /// # Safety
    ///
    /// - `ptr` must be allocated by a C-compatible allocator
    /// - `ptr` must be valid for reads/writes of `len` elements
    /// - `cap` must represent the actual allocated capacity
    /// - The memory must not be accessed by Zig after this call
    #[inline]
    pub const unsafe fn new(ptr: *mut T, len: usize, cap: usize) -> Self {
        Self { ptr, len, cap, _phantom: PhantomData }
    }

    /// Convert this `RawVec` into a Rust `Vec<T>` with zero-copy
    ///
    /// # Safety
    ///
    /// - The memory must have been allocated by `std.heap.c_allocator` in Zig
    /// - The pointer must be valid and aligned
    /// - No other references to this memory must exist
    /// - The memory must not be freed by Zig
    #[inline]
    pub unsafe fn into_vec(self) -> Vec<T> {
        Vec::from_raw_parts(self.ptr, self.len, self.cap)
    }

    /// Create a `RawVec` from a Rust `Vec<T>` without taking ownership
    ///
    /// This is useful for passing Rust data to Zig temporarily.
    ///
    /// # Safety
    ///
    /// The caller must ensure the `Vec` lives long enough and is not modified
    /// while Zig holds the pointer.
    #[inline]
    pub fn from_vec_ref(vec: &Vec<T>) -> Self {
        Self {
            ptr: vec.as_ptr() as *mut T,
            len: vec.len(),
            cap: vec.capacity(),
            _phantom: PhantomData,
        }
    }

    /// Get a slice view of the data without taking ownership
    ///
    /// # Safety
    ///
    /// - The pointer must be valid
    /// - The data must not be mutated while the slice exists
    #[inline]
    pub unsafe fn as_slice(&self) -> &[T] {
        slice::from_raw_parts(self.ptr, self.len)
    }

    /// Get a mutable slice view of the data without taking ownership
    ///
    /// # Safety
    ///
    /// - The pointer must be valid
    /// - No other references to this data must exist
    #[inline]
    pub unsafe fn as_slice_mut(&mut self) -> &mut [T] {
        slice::from_raw_parts_mut(self.ptr, self.len)
    }

    /// Check if the buffer is empty
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Get the number of elements
    #[inline]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Get the capacity
    #[inline]
    pub const fn capacity(&self) -> usize {
        self.cap
    }
}

/// Zero-copy buffer wrapper providing safe API
///
/// This type encapsulates `RawVec<T>` and provides a safe interface for
/// working with zero-copy data from Zig.
pub struct ZeroCopyBuffer<T> {
    raw: RawVec<T>,
}

impl<T> ZeroCopyBuffer<T> {
    /// Create a `ZeroCopyBuffer` from raw components
    ///
    /// # Safety
    ///
    /// See `RawVec::new` safety requirements
    #[inline]
    pub unsafe fn from_raw_parts(ptr: *mut T, len: usize, cap: usize) -> Self {
        Self { raw: RawVec::new(ptr, len, cap) }
    }

    /// Create a `ZeroCopyBuffer` from a `RawVec` (safe wrapper)
    ///
    /// This is the safe API that should be used in example code.
    /// It assumes the `RawVec<T>` was created correctly by Zig via
    /// `include_zig!`.
    #[inline]
    pub fn from_zig_vec(raw: RawVec<T>) -> Self {
        // SAFETY: When RawVec comes from Zig via include_zig!, it's guaranteed to be:
        // - Allocated by c_allocator (compatible with Rust's allocator)
        // - Valid pointer with correct len and cap
        // - Not accessed by Zig anymore (ownership transferred)
        // This is enforced by AutoZig's code generation.
        unsafe { Self::from_raw_vec(raw) }
    }

    /// Create a `ZeroCopyBuffer` from a `RawVec`
    ///
    /// # Safety
    ///
    /// See `RawVec::new` safety requirements
    #[inline]
    pub const unsafe fn from_raw_vec(raw: RawVec<T>) -> Self {
        Self { raw }
    }

    /// Convert this buffer into a `Vec<T>` with zero-copy
    ///
    /// # Safety
    ///
    /// This requires the Zig allocator to be compatible with the Rust allocator
    /// (e.g. `std.heap.c_allocator` and system allocator). For a safer,
    /// allocator-independent transfer, use `ZigBuffer` and `ZigBox`.
    #[deprecated(
        note = "Use ZigBox for safe ownership transfer. This method requires shared allocator \
                assumptions."
    )]
    #[inline]
    pub fn into_vec(self) -> Vec<T> {
        unsafe { self.raw.into_vec() }
    }

    /// Get the raw components
    #[inline]
    pub const fn raw(&self) -> &RawVec<T> {
        &self.raw
    }

    /// Get a slice view of the data
    #[inline]
    pub fn as_slice(&self) -> &[T] {
        unsafe { self.raw.as_slice() }
    }

    /// Get the number of elements
    #[inline]
    pub const fn len(&self) -> usize {
        self.raw.len()
    }

    /// Check if the buffer is empty
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.raw.is_empty()
    }

    /// Get the capacity
    #[inline]
    pub const fn capacity(&self) -> usize {
        self.raw.capacity()
    }
}

impl<T> From<ZeroCopyBuffer<T>> for Vec<T> {
    #[inline]
    #[allow(deprecated)]
    fn from(buffer: ZeroCopyBuffer<T>) -> Self {
        buffer.into_vec()
    }
}

impl<T> AsRef<[T]> for ZeroCopyBuffer<T> {
    #[inline]
    fn as_ref(&self) -> &[T] {
        self.as_slice()
    }
}

// Ensure RawVec has the same layout as Vec
#[cfg(test)]
mod layout_tests {
    use std::mem;

    use super::*;

    #[test]
    fn test_raw_vec_layout() {
        // Verify that RawVec has the same size and alignment as Vec
        assert_eq!(
            mem::size_of::<RawVec<u8>>(),
            mem::size_of::<Vec<u8>>(),
            "RawVec size must match Vec size"
        );
        assert_eq!(
            mem::align_of::<RawVec<u8>>(),
            mem::align_of::<Vec<u8>>(),
            "RawVec alignment must match Vec alignment"
        );
    }
}

#[cfg(test)]
#[allow(deprecated)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_copy_buffer_basic() {
        // Create a Vec in Rust
        let vec = vec![1, 2, 3, 4, 5];
        let ptr = vec.as_ptr() as *mut i32;
        let len = vec.len();
        let cap = vec.capacity();

        // Forget the Vec to avoid double-free
        std::mem::forget(vec);

        // Create zero-copy buffer
        let buffer = unsafe { ZeroCopyBuffer::from_raw_parts(ptr, len, cap) };

        assert_eq!(buffer.len(), 5);
        assert_eq!(buffer.as_slice(), &[1, 2, 3, 4, 5]);

        // Convert back to Vec
        let recovered = buffer.into_vec();
        assert_eq!(recovered, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_raw_vec_from_vec_ref() {
        let vec = vec![10, 20, 30];
        let raw = RawVec::from_vec_ref(&vec);

        assert_eq!(raw.len(), 3);
        assert_eq!(raw.capacity(), vec.capacity());

        unsafe {
            assert_eq!(raw.as_slice(), &[10, 20, 30]);
        }
    }

    #[test]
    fn test_zero_copy_buffer_empty() {
        let vec: Vec<i32> = Vec::new();
        let ptr = vec.as_ptr() as *mut i32;
        let len = vec.len();
        let cap = vec.capacity();

        std::mem::forget(vec);

        let buffer = unsafe { ZeroCopyBuffer::from_raw_parts(ptr, len, cap) };

        assert!(buffer.is_empty());
        assert_eq!(buffer.len(), 0);

        let recovered = buffer.into_vec();
        assert!(recovered.is_empty());
    }

    #[test]
    fn test_zero_copy_buffer_large() {
        // Test with larger buffer (> 1MB as per spec)
        let size = 1_000_000;
        let vec: Vec<u32> = (0..size).collect();
        let ptr = vec.as_ptr() as *mut u32;
        let len = vec.len();
        let cap = vec.capacity();

        std::mem::forget(vec);

        let buffer = unsafe { ZeroCopyBuffer::from_raw_parts(ptr, len, cap) };

        assert_eq!(buffer.len(), size as usize);
        assert!(buffer
            .as_slice()
            .iter()
            .enumerate()
            .all(|(i, &v)| v == i as u32));

        let recovered = buffer.into_vec();
        assert_eq!(recovered.len(), size as usize);
        assert_eq!(recovered[0], 0);
        assert_eq!(recovered[size as usize - 1], (size - 1));
    }
}
