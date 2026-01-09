//! AutoZig WASM 3.0 64-bit 自动双重绑定实现
//!
//! 使用新的 #[autozig(...)] 属性自动生成两种绑定：
//! 1. wasm-bindgen 绑定（用于生成完整的 wasm 模块）
//! 2. 手动 C 风格导出（用于直接 WebAssembly.instantiate 调用）
//!
//! 这消除了代码重复，只需要声明一次函数！

use autozig::include_zig;
use wasm_bindgen::prelude::*;

// 使用 include_zig! 宏引入 Zig wasm64 实现
// 新增 #[autozig(...)] 属性实现自动双重绑定
include_zig!("src/wasm64.zig", {
    // 1. 普通透传 (Dual) - 自动生成 wasm_get_memory_size 和 wasm64_get_memory_size
    #[autozig(strategy = "dual")]
    fn get_memory_size() -> usize;

    // 2. 普通透传 (Dual)
    #[autozig(strategy = "dual")]
    fn grow_memory(delta: usize) -> isize;

    // 3. 指针转换 (Dual with Mapping)
    // wasm-bindgen: 返回 *mut u8
    // C ABI: 返回 usize (指针转换为整数)
    #[autozig(strategy = "dual", c_ret = "usize", map_fn = "|ptr| ptr as usize")]
    fn alloc_large_buffer() -> *mut u8;

    // 4. 普通透传
    #[autozig(strategy = "dual")]
    fn get_buffer_size() -> usize;

    // 5. 普通透传
    #[autozig(strategy = "dual")]
    fn write_buffer(offset: usize, value: u8);

    // 6. 普通透传
    #[autozig(strategy = "dual")]
    fn read_buffer(offset: usize) -> u8;

    // 7. 普通透传
    #[autozig(strategy = "dual")]
    fn fill_buffer(start: usize, length: usize, value: u8);

    // 8. 普通透传
    #[autozig(strategy = "dual")]
    fn checksum_buffer(start: usize, length: usize) -> u64;

    // 9. Bool 转换 (Dual with Mapping)
    // wasm-bindgen: 返回 bool
    // C ABI: 返回 u32 (0 或 1)
    #[autozig(
        strategy = "dual",
        c_ret = "u32",
        map_fn = "|b: bool| if b { 1 } else { 0 }"
    )]
    fn write_at_high_address(value: u64) -> bool;

    // 10. 普通透传
    #[autozig(strategy = "dual")]
    fn read_at_high_address() -> u64;

    // 11. 普通透传
    #[autozig(strategy = "dual")]
    fn get_arch_info() -> u32;

    // 12. 普通透传
    #[autozig(strategy = "dual")]
    fn get_pointer_size() -> usize;

    // 13. 内存测试函数 - 自动生成双重绑定！
    // 生成：wasm_run_memory_test() 和 wasm64_run_memory_test()
    #[autozig(strategy = "dual")]
    fn run_memory_test() -> u32;
});

/// 格式化测试结果（可选的包装器）
#[wasm_bindgen]
pub fn run_memory_test_formatted() -> String {
    let result = wasm_run_memory_test();
    let arch = wasm_get_arch_info();
    let start_size = wasm_get_memory_size();
    let buffer_size = wasm_get_buffer_size();

    format!(
        "WASM{} Memory Test:\nMemory: {} pages\nBuffer: {} bytes\nTest Result: 0x{:X}\n  \
         Checksum: {}\n  High Address: {}",
        arch,
        start_size,
        buffer_size,
        result,
        if result & 0x1 != 0 { "PASS" } else { "FAIL" },
        if result & 0x2 != 0 { "PASS" } else { "SKIP" }
    )
}

/// 初始化
#[wasm_bindgen(start)]
pub fn init() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arch_info() {
        let arch = wasm_get_arch_info();
        assert!(arch == 32 || arch == 64);
    }

    #[test]
    fn test_buffer_operations() {
        let size = wasm_get_buffer_size();
        assert!(size > 0);

        wasm_write_buffer(0, 42);
        assert_eq!(wasm_read_buffer(0), 42);
    }
}
