#![allow(unsafe_code)]
use std::marker::PhantomData;

/// standard exchange format for moving memory from Zig to Rust
#[repr(C)]
pub struct ZigBuffer {
    pub ptr: *mut u8,
    pub len: usize,
    pub cap: usize,
    /// Callback to free the memory.
    /// Signature: fn(ptr: *mut u8, len: usize, cap: usize)
    /// If None, it implies the memory is not managed by this specific mechanism
    /// or is static/borrowed (though this struct implies ownership transfer
    /// usually).
    pub free_fn: Option<unsafe extern "C" fn(*mut u8, usize, usize)>,
}

/// Helper function to free Rust vectors passed to Zig.
/// Use this when manually constructing ZigBuffer or as the backend for
/// `From<Vec<T>>`.
///
/// # Safety
///
/// This function must only be called with a pointer, length, and capacity that
/// form a valid `Vec<T>` previously allocated by Rust's global allocator. The
/// caller must ensure that the memory is not accessed after this call.
pub unsafe extern "C" fn rust_free_vec<T>(ptr: *mut u8, len: usize, cap: usize) {
    let _ = Vec::from_raw_parts(ptr as *mut T, len, cap);
}

impl<T> From<Vec<T>> for ZigBuffer {
    fn from(data: Vec<T>) -> Self {
        let mut manual = std::mem::ManuallyDrop::new(data);
        ZigBuffer {
            ptr: manual.as_mut_ptr() as *mut u8,
            len: manual.len(),
            cap: manual.capacity(),
            free_fn: Some(rust_free_vec::<T>),
        }
    }
}

/// A smart pointer that owns memory allocated in Zig.
/// It ensures the memory is freed using the provided callback when dropped.
pub struct ZigBox<T> {
    inner: ZigBuffer,
    _marker: PhantomData<T>,
}

impl<T> ZigBox<T> {
    /// Create a ZigBox from the raw exchange format.
    ///
    /// # Safety
    ///
    /// The `raw` ZigBuffer must contain a valid pointer, length, and capacity
    /// for a slice of `T`. The `free_fn` must be valid for deallocating this
    /// memory. The memory must be compatible with the layout of `[T]`.
    /// Safely wrap a ZigBuffer into a ZigBox with validation.
    ///
    /// # Validation
    ///
    /// This function checks basic invariants:
    /// - `ptr` must not be null (unless `len` is 0).
    /// - `cap` must be greater than or equal to `len`.
    ///
    /// If validation fails, it panics. For a non-panicking version, use
    /// `try_new`.
    pub fn new(raw: ZigBuffer) -> Self {
        match Self::try_new(raw) {
            Ok(b) => b,
            Err(e) => panic!("ZigBox::new failed: {}", e),
        }
    }

    /// Try to create a ZigBox with validation.
    pub fn try_new(raw: ZigBuffer) -> Result<Self, &'static str> {
        if !raw.ptr.is_null() && raw.len > 0 {
            // Valid non-empty
        } else if raw.ptr.is_null() && raw.len == 0 {
            // Valid empty
        } else {
            return Err("Null pointer with non-zero length");
        }

        if raw.cap < raw.len {
            return Err("Capacity less than length");
        }

        // We trust the free_fn (or lack thereof) is correct for the data.
        // If free_fn is None, it acts as a non-dropping view (static data).

        Ok(unsafe { Self::new_unchecked(raw) })
    }

    /// Unsafely construct a ZigBox without validation.
    ///
    /// # Safety
    /// Caller guarantees the `ZigBuffer` is valid.
    pub unsafe fn new_unchecked(raw: ZigBuffer) -> Self {
        Self { inner: raw, _marker: PhantomData }
    }

    /// Deprecated: Use `new_unchecked` instead for clarity.
    ///
    /// # Safety
    ///
    /// See `new_unchecked` safety requirements.
    #[deprecated(
        note = "Use new_unchecked for explicit unsafety, or new for validated construction"
    )]
    pub unsafe fn from_raw(raw: ZigBuffer) -> Self {
        Self::new_unchecked(raw)
    }

    /// Access the data as a Rust slice.
    pub fn as_slice(&self) -> &[T] {
        unsafe {
            std::slice::from_raw_parts(
                self.inner.ptr as *const T,
                self.inner.len / std::mem::size_of::<T>(),
            )
        }
    }

    /// Access the data as a mutable Rust slice.
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe {
            std::slice::from_raw_parts_mut(
                self.inner.ptr as *mut T,
                self.inner.len / std::mem::size_of::<T>(),
            )
        }
    }
}

impl<T> Drop for ZigBox<T> {
    fn drop(&mut self) {
        if let Some(free_fn) = self.inner.free_fn {
            unsafe {
                free_fn(self.inner.ptr, self.inner.len, self.inner.cap);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zig_box_drop_calls_free() {
        use std::sync::atomic::{
            AtomicPtr,
            AtomicUsize,
            Ordering,
        };

        // Static atomics to track calls safely
        static FREED_PTR: AtomicPtr<u8> = AtomicPtr::new(std::ptr::null_mut());
        static FREED_LEN: AtomicUsize = AtomicUsize::new(0);
        static FREED_CAP: AtomicUsize = AtomicUsize::new(0);
        static CALL_COUNT: AtomicUsize = AtomicUsize::new(0);

        unsafe extern "C" fn mock_free(ptr: *mut u8, len: usize, cap: usize) {
            FREED_PTR.store(ptr, Ordering::SeqCst);
            FREED_LEN.store(len, Ordering::SeqCst);
            FREED_CAP.store(cap, Ordering::SeqCst);
            CALL_COUNT.fetch_add(1, Ordering::SeqCst);
        }

        let mut data = [10u8, 20, 30];
        let ptr = data.as_mut_ptr();

        let buf = ZigBuffer {
            ptr,
            len: 3,
            cap: 3,
            free_fn: Some(mock_free),
        };

        unsafe {
            // Use new_unchecked for tests as we built it manually
            let _zbox = ZigBox::<u8>::new_unchecked(buf);
            // _zbox dropped here
        }

        assert_eq!(CALL_COUNT.load(Ordering::SeqCst), 1);
        assert_eq!(FREED_PTR.load(Ordering::SeqCst), ptr);
        assert_eq!(FREED_LEN.load(Ordering::SeqCst), 3);
        assert_eq!(FREED_CAP.load(Ordering::SeqCst), 3);
    }
}
