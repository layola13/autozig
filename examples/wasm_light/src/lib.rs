//! AutoZig WASM 多光源渲染示例
//!
//! 演示零拷贝内存共享 + Zig SIMD 向量化光照计算

use autozig::include_zig;
use wasm_bindgen::prelude::*;

// 使用 include_zig! 宏引入外部 Zig 文件
// 参考 external 示例的格式：简洁的单行签名
include_zig!("src/light.zig", {
    fn alloc_pixel_buffer(width: u32, height: u32) -> *mut u8;
    fn alloc_background_buffer(width: u32, height: u32) -> *mut u8;
    fn alloc_lights_buffer(count: u32) -> *mut f32;
    fn render_lights_simd_raw(
        pixel_ptr: *mut u8,
        width: u32,
        height: u32,
        lights_ptr: *const f32,
        num_lights: u32,
    );
    fn render_lights_scalar_raw(
        pixel_ptr: *mut u8,
        width: u32,
        height: u32,
        lights_ptr: *const f32,
        num_lights: u32,
    );
});

// ============================================================================
// WASM 导出接口
// ============================================================================

/// 分配像素缓冲区（零拷贝）
#[wasm_bindgen]
pub fn wasm_alloc_pixel_buffer(width: u32, height: u32) -> *mut u8 {
    alloc_pixel_buffer(width, height)
}

/// 分配底图缓冲区（零拷贝）
#[wasm_bindgen]
pub fn wasm_alloc_background_buffer(width: u32, height: u32) -> *mut u8 {
    alloc_background_buffer(width, height)
}

/// 分配光源缓冲区（零拷贝）
#[wasm_bindgen]
pub fn wasm_alloc_lights_buffer(count: u32) -> *mut f32 {
    alloc_lights_buffer(count)
}

/// Zig SIMD 向量化渲染
#[wasm_bindgen]
pub fn wasm_render_lights_simd(
    pixel_ptr: *mut u8,
    width: u32,
    height: u32,
    lights_ptr: *const f32,
    num_lights: u32,
) {
    render_lights_simd_raw(pixel_ptr, width, height, lights_ptr, num_lights);
}

/// Zig Scalar 标量渲染（对比基准）
#[wasm_bindgen]
pub fn wasm_render_lights_scalar(
    pixel_ptr: *mut u8,
    width: u32,
    height: u32,
    lights_ptr: *const f32,
    num_lights: u32,
) {
    render_lights_scalar_raw(pixel_ptr, width, height, lights_ptr, num_lights);
}

/// 获取版本信息
#[wasm_bindgen]
pub fn get_version() -> String {
    "AutoZig WASM Light v0.1.0 - Zero-Copy SIMD Multi-Light Rendering".to_string()
}

/// 初始化函数
#[wasm_bindgen(start)]
pub fn init() {
    // 设置 panic hook 以便在浏览器控制台看到错误
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}
