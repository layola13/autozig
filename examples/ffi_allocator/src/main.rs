//! FFI Allocator Example
//!
//! 演示 Zig FFI 中正确的内存分配模式：
//! 1. 本地 GPA Allocator
//! 2. 非可选返回 (*T + catch unreachable)
//! 3. Rust 端使用 *mut T

use autozig::include_zig;

/// Opaque type for FFI
#[repr(C)]
pub struct ZigCounter {
    _opaque: [u8; 0],
}

// FFI 声明 - 使用原始指针，不使用 Option
include_zig!("src/zig/counter.zig", {
    fn counter_create(start_value: u32) -> *mut ZigCounter;
    fn counter_destroy(counter: *mut ZigCounter);
    fn counter_get(counter: *const ZigCounter) -> u32;
    fn counter_increment(counter: *mut ZigCounter);
    fn counter_decrement(counter: *mut ZigCounter);
    fn counter_reset(counter: *mut ZigCounter);
    fn counter_get_increment_count(counter: *const ZigCounter) -> u32;
});

/// Safe Rust wrapper
pub struct Counter {
    inner: *mut ZigCounter,
}

impl Counter {
    pub fn new(start_value: u32) -> Self {
        Self { inner: counter_create(start_value) }
    }

    pub fn get(&self) -> u32 {
        counter_get(self.inner)
    }

    pub fn increment(&mut self) {
        counter_increment(self.inner);
    }

    pub fn decrement(&mut self) {
        counter_decrement(self.inner);
    }

    pub fn reset(&mut self) {
        counter_reset(self.inner);
    }

    pub fn increment_count(&self) -> u32 {
        counter_get_increment_count(self.inner)
    }
}

impl Drop for Counter {
    fn drop(&mut self) {
        counter_destroy(self.inner);
    }
}

fn main() {
    println!("=== FFI Allocator Example ===\n");

    let mut counter = Counter::new(10);
    println!("Initial value: {}", counter.get());

    for _ in 0..5 {
        counter.increment();
    }
    println!("After 5 increments: {}", counter.get());
    println!("Increment count: {}", counter.increment_count());

    counter.decrement();
    println!("After decrement: {}", counter.get());

    counter.reset();
    println!("After reset: {}", counter.get());

    println!("\n=== Success! ===");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_counter_creation() {
        let counter = Counter::new(42);
        assert_eq!(counter.get(), 42);
    }

    #[test]
    fn test_counter_increment() {
        let mut counter = Counter::new(0);
        counter.increment();
        counter.increment();
        counter.increment();
        assert_eq!(counter.get(), 3);
        assert_eq!(counter.increment_count(), 3);
    }

    #[test]
    fn test_counter_decrement() {
        let mut counter = Counter::new(5);
        counter.decrement();
        counter.decrement();
        assert_eq!(counter.get(), 3);
    }

    #[test]
    fn test_counter_reset() {
        let mut counter = Counter::new(100);
        counter.increment();
        counter.reset();
        assert_eq!(counter.get(), 0);
        assert_eq!(counter.increment_count(), 0);
    }
}
