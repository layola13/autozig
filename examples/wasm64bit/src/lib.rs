//! AutoZig WASM64 Example
//!
//! 演示如何使用 AutoZig 在 WASM64 环境中调用 Zig 代码
//! 使用 #[autozig_export] 宏导出函数，无需 wasm-bindgen

use autozig::{
    autozig_export,
    include_zig,
};

// 使用 include_zig! 宏引入 Zig wasm64 实现
include_zig!("src/wasm64.zig", {
    fn get_memory_size() -> usize;
    fn grow_memory(delta: usize) -> isize;
    fn alloc_large_buffer() -> *mut u8;
    fn get_buffer_size() -> usize;
    fn write_buffer(offset: usize, value: u8);
    fn read_buffer(offset: usize) -> u8;
    fn fill_buffer(start: usize, length: usize, value: u8);
    fn checksum_buffer(start: usize, length: usize) -> u64;
    fn write_at_high_address(value: u64) -> bool;
    fn read_at_high_address() -> u64;
    fn get_arch_info() -> u32;
    fn get_pointer_size() -> usize;
    fn run_memory_test() -> u32;
});

// 导出函数给 WASM 使用
#[autozig_export]
pub fn wasm_get_memory_size() -> usize {
    get_memory_size()
}

#[autozig_export]
pub fn wasm_grow_memory(delta: usize) -> isize {
    grow_memory(delta)
}

#[autozig_export]
pub fn wasm_alloc_large_buffer() -> usize {
    alloc_large_buffer() as usize
}

#[autozig_export]
pub fn wasm_get_buffer_size() -> usize {
    get_buffer_size()
}

#[autozig_export]
pub fn wasm_write_buffer(offset: usize, value: u8) {
    write_buffer(offset, value);
}

#[autozig_export]
pub fn wasm_read_buffer(offset: usize) -> u8 {
    read_buffer(offset)
}

#[autozig_export]
pub fn wasm_fill_buffer(start: usize, length: usize, value: u8) {
    fill_buffer(start, length, value);
}

#[autozig_export]
pub fn wasm_checksum_buffer(start: usize, length: usize) -> u64 {
    checksum_buffer(start, length)
}

#[autozig_export]
pub fn wasm_write_at_high_address(value: u64) -> u32 {
    if write_at_high_address(value) {
        1
    } else {
        0
    }
}

#[autozig_export]
pub fn wasm_read_at_high_address() -> u64 {
    read_at_high_address()
}

#[autozig_export]
pub fn wasm_get_arch_info() -> u32 {
    get_arch_info()
}

#[autozig_export]
pub fn wasm_get_pointer_size() -> usize {
    get_pointer_size()
}

#[autozig_export]
pub fn wasm_run_memory_test() -> u32 {
    run_memory_test()
}

/// 获取版本信息
#[autozig_export]
pub fn get_version() -> u32 {
    100 // v1.0.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arch_info() {
        let arch = get_arch_info();
        assert!(arch == 32 || arch == 64);
    }

    #[test]
    fn test_buffer_operations() {
        let size = get_buffer_size();
        assert!(size > 0);

        write_buffer(0, 42);
        assert_eq!(read_buffer(0), 42);
    }

    #[test]
    fn test_memory_size() {
        let size = get_memory_size();
        assert!(size > 0);
    }
}
