//! AutoZig WASM 3.0 64-bit 手动绑定实现
//!
//! 本实现同时支持两种绑定方式：
//! 1. wasm-bindgen 绑定（用于生成完整的 wasm 模块）
//! 2. 手动 C 风格导出（用于直接 WebAssembly.instantiate 调用）

use autozig::include_zig;
use wasm_bindgen::prelude::*;

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
});

// ============================================================================
// wasm-bindgen 导出接口
// ============================================================================

/// 获取当前 WebAssembly 内存大小
#[wasm_bindgen]
pub fn wasm_get_memory_size() -> usize {
    get_memory_size()
}

/// 增长 WebAssembly 内存
#[wasm_bindgen]
pub fn wasm_grow_memory(delta: usize) -> isize {
    grow_memory(delta)
}

/// 分配大缓冲区
#[wasm_bindgen]
pub fn wasm_alloc_large_buffer() -> *mut u8 {
    alloc_large_buffer()
}

/// 获取缓冲区大小
#[wasm_bindgen]
pub fn wasm_get_buffer_size() -> usize {
    get_buffer_size()
}

/// 写入缓冲区
#[wasm_bindgen]
pub fn wasm_write_buffer(offset: usize, value: u8) {
    write_buffer(offset, value);
}

/// 读取缓冲区
#[wasm_bindgen]
pub fn wasm_read_buffer(offset: usize) -> u8 {
    read_buffer(offset)
}

/// 填充缓冲区
#[wasm_bindgen]
pub fn wasm_fill_buffer(start: usize, length: usize, value: u8) {
    fill_buffer(start, length, value);
}

/// 计算校验和
#[wasm_bindgen]
pub fn wasm_checksum_buffer(start: usize, length: usize) -> u64 {
    checksum_buffer(start, length)
}

/// 高地址写入
#[wasm_bindgen]
pub fn wasm_write_at_high_address(value: u64) -> bool {
    write_at_high_address(value)
}

/// 高地址读取
#[wasm_bindgen]
pub fn wasm_read_at_high_address() -> u64 {
    read_at_high_address()
}

/// 获取架构信息
#[wasm_bindgen]
pub fn wasm_get_arch_info() -> u32 {
    get_arch_info()
}

/// 获取指针大小
#[wasm_bindgen]
pub fn wasm_get_pointer_size() -> usize {
    get_pointer_size()
}

/// 运行完整测试
#[wasm_bindgen]
pub fn run_memory_test() -> String {
    let start_size = get_memory_size();
    let buffer_size = get_buffer_size();
    let arch = get_arch_info();

    let test_size = 1024 * 1024;
    fill_buffer(0, test_size, 0xAA);

    let checksum = checksum_buffer(0, test_size);
    let expected = 0xAA * test_size as u64;

    let high_addr_test = write_at_high_address(0xDEADBEEFCAFEBABE);
    let high_addr_value = if high_addr_test {
        read_at_high_address()
    } else {
        0
    };

    format!(
        "WASM{} Memory Test:\nMemory: {} pages\nBuffer: {} bytes\nChecksum: {} (expected: {}) - \
         {}\nHigh addr: {} (value: 0x{:X})",
        arch,
        start_size,
        buffer_size,
        checksum,
        expected,
        if checksum == expected { "PASS" } else { "FAIL" },
        if high_addr_test { "PASS" } else { "SKIP" },
        high_addr_value
    )
}

/// 初始化
#[wasm_bindgen(start)]
pub fn init() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

// ============================================================================
// 手动 C 风格导出（用于直接 WebAssembly API 调用）
// ============================================================================

/// C 风格导出：获取内存大小
#[no_mangle]
pub extern "C" fn wasm64_get_memory_size() -> usize {
    get_memory_size()
}

/// C 风格导出：增长内存
#[no_mangle]
pub extern "C" fn wasm64_grow_memory(delta: usize) -> isize {
    grow_memory(delta)
}

/// C 风格导出：分配缓冲区
#[no_mangle]
pub extern "C" fn wasm64_alloc_large_buffer() -> usize {
    alloc_large_buffer() as usize
}

/// C 风格导出：获取缓冲区大小
#[no_mangle]
pub extern "C" fn wasm64_get_buffer_size() -> usize {
    get_buffer_size()
}

/// C 风格导出：写入缓冲区
#[no_mangle]
pub extern "C" fn wasm64_write_buffer(offset: usize, value: u8) {
    write_buffer(offset, value);
}

/// C 风格导出：读取缓冲区
#[no_mangle]
pub extern "C" fn wasm64_read_buffer(offset: usize) -> u8 {
    read_buffer(offset)
}

/// C 风格导出：填充缓冲区
#[no_mangle]
pub extern "C" fn wasm64_fill_buffer(start: usize, length: usize, value: u8) {
    fill_buffer(start, length, value);
}

/// C 风格导出：计算校验和
#[no_mangle]
pub extern "C" fn wasm64_checksum_buffer(start: usize, length: usize) -> u64 {
    checksum_buffer(start, length)
}

/// C 风格导出：高地址写入
#[no_mangle]
pub extern "C" fn wasm64_write_at_high_address(value: u64) -> u32 {
    if write_at_high_address(value) {
        1
    } else {
        0
    }
}

/// C 风格导出：高地址读取
#[no_mangle]
pub extern "C" fn wasm64_read_at_high_address() -> u64 {
    read_at_high_address()
}

/// C 风格导出：获取架构信息
#[no_mangle]
pub extern "C" fn wasm64_get_arch_info() -> u32 {
    get_arch_info()
}

/// C 风格导出：获取指针大小
#[no_mangle]
pub extern "C" fn wasm64_get_pointer_size() -> usize {
    get_pointer_size()
}

/// C 风格导出：运行内存测试
#[no_mangle]
pub extern "C" fn wasm64_run_memory_test() -> u32 {
    let mut result: u32 = 0;

    let test_size = 1024 * 1024;
    fill_buffer(0, test_size, 0xAA);

    let checksum = checksum_buffer(0, test_size);
    let expected = 0xAA * test_size as u64;
    if checksum == expected {
        result |= 0x1;
    }

    if write_at_high_address(0xDEADBEEFCAFEBABE) {
        let value = read_at_high_address();
        if value == 0xDEADBEEFCAFEBABE {
            result |= 0x2;
        }
    }

    result
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
}
