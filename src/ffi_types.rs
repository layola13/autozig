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
    pub unsafe fn from_raw(raw: ZigBuffer) -> Self {
        Self { inner: raw, _marker: PhantomData }
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
            let _zbox = ZigBox::<u8>::from_raw(buf);
            // _zbox dropped here
        }

        assert_eq!(CALL_COUNT.load(Ordering::SeqCst), 1);
        assert_eq!(FREED_PTR.load(Ordering::SeqCst), ptr);
        assert_eq!(FREED_LEN.load(Ordering::SeqCst), 3);
        assert_eq!(FREED_CAP.load(Ordering::SeqCst), 3);
    }
}
