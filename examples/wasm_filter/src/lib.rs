//! AutoZig WASM 图像滤镜示例
//!
//! 演示如何使用 AutoZig 在 WASM 环境中调用 Zig 代码进行高性能图像处理

use autozig::autozig;
use wasm_bindgen::prelude::*;

// 使用 autozig! 宏嵌入 Zig 代码
autozig! {
    // Zig 实现：图像反色滤镜
    // 这段代码会被编译为 WASM 并与 Rust 静态链接

    // 反色滤镜 (Invert Colors)
    // 对 RGBA 图像数据进行反色处理
    export fn invert_colors_raw(ptr: [*]u8, len: usize) void {
        var i: usize = 0;
        while (i < len) : (i += 4) {
            // 反转 RGB，保持 Alpha 不变
            ptr[i] = 255 - ptr[i];         // R
            ptr[i + 1] = 255 - ptr[i + 1]; // G
            ptr[i + 2] = 255 - ptr[i + 2]; // B
            // ptr[i + 3] = Alpha (不变)
        }
    }

    // 灰度滤镜 (Grayscale)
    // 使用标准加权平均法：Gray = 0.299*R + 0.587*G + 0.114*B
    export fn grayscale_raw(ptr: [*]u8, len: usize) void {
        var i: usize = 0;
        while (i < len) : (i += 4) {
            const r = ptr[i];
            const g = ptr[i + 1];
            const b = ptr[i + 2];

            // 加权平均（使用整数运算避免浮点）
            const gray = @as(u8, @intCast((
                @as(u32, r) * 299 +
                @as(u32, g) * 587 +
                @as(u32, b) * 114
            ) / 1000));

            ptr[i] = gray;
            ptr[i + 1] = gray;
            ptr[i + 2] = gray;
            // Alpha 不变
        }
    }

    // 亮度调整 (Brightness)
    // delta: 亮度调整值 (-255 到 +255)
    export fn adjust_brightness_raw(ptr: [*]u8, len: usize, delta: i32) void {
        var i: usize = 0;
        while (i < len) : (i += 4) {
            // 调整 RGB，确保不溢出
            ptr[i] = clamp_add(ptr[i], delta);
            ptr[i + 1] = clamp_add(ptr[i + 1], delta);
            ptr[i + 2] = clamp_add(ptr[i + 2], delta);
            // Alpha 不变
        }
    }

    // 辅助函数：带范围限制的加法
    fn clamp_add(value: u8, delta: i32) u8 {
        const result = @as(i32, value) + delta;
        if (result < 0) return 0;
        if (result > 255) return 255;
        return @as(u8, @intCast(result));
    }

    ---

    // Rust FFI 签名声明
    // AutoZig 会自动生成 Rust 包装函数
    // 注意：对于带额外参数的函数，需要直接声明原始指针形式
    // Zig: fn(ptr: [*]u8, len: usize, extra_params...)
    // Rust: fn(ptr: *mut u8, len: usize, extra_params...)
    fn invert_colors_raw(ptr: *mut u8, len: usize);
    fn grayscale_raw(ptr: *mut u8, len: usize);
    fn adjust_brightness_raw(ptr: *mut u8, len: usize, delta: i32);
}

// 暴露给 JavaScript 的 WASM 接口

/// 反色滤镜
#[wasm_bindgen]
pub fn apply_invert(mut data: Vec<u8>) -> Vec<u8> {
    invert_colors_raw(data.as_mut_ptr(), data.len());
    data
}

/// 灰度滤镜
#[wasm_bindgen]
pub fn apply_grayscale(mut data: Vec<u8>) -> Vec<u8> {
    grayscale_raw(data.as_mut_ptr(), data.len());
    data
}

/// 亮度调整
#[wasm_bindgen]
pub fn apply_brightness(mut data: Vec<u8>, delta: i32) -> Vec<u8> {
    adjust_brightness_raw(data.as_mut_ptr(), data.len(), delta);
    data
}

// ============================================================================
// Rust Native 实现（用于性能对比）
// ============================================================================

/// Rust Native: 反色滤镜
#[wasm_bindgen]
pub fn apply_invert_rust(mut data: Vec<u8>) -> Vec<u8> {
    for i in (0..data.len()).step_by(4) {
        data[i] = 255 - data[i];         // R
        data[i + 1] = 255 - data[i + 1]; // G
        data[i + 2] = 255 - data[i + 2]; // B
        // data[i + 3] = Alpha (不变)
    }
    data
}

/// Rust Native: 灰度滤镜
#[wasm_bindgen]
pub fn apply_grayscale_rust(mut data: Vec<u8>) -> Vec<u8> {
    for i in (0..data.len()).step_by(4) {
        let r = data[i] as u32;
        let g = data[i + 1] as u32;
        let b = data[i + 2] as u32;
        
        // 加权平均
        let gray = ((r * 299 + g * 587 + b * 114) / 1000) as u8;
        
        data[i] = gray;
        data[i + 1] = gray;
        data[i + 2] = gray;
    }
    data
}

/// Rust Native: 亮度调整
#[wasm_bindgen]
pub fn apply_brightness_rust(mut data: Vec<u8>, delta: i32) -> Vec<u8> {
    for i in (0..data.len()).step_by(4) {
        data[i] = clamp_add_rust(data[i], delta);
        data[i + 1] = clamp_add_rust(data[i + 1], delta);
        data[i + 2] = clamp_add_rust(data[i + 2], delta);
    }
    data
}

/// Rust辅助函数：带范围限制的加法
fn clamp_add_rust(value: u8, delta: i32) -> u8 {
    let result = value as i32 + delta;
    result.clamp(0, 255) as u8
}

/// 获取版本信息
#[wasm_bindgen]
pub fn get_version() -> String {
    "AutoZig WASM Filter v0.1.0 - Powered by Zig + Rust".to_string()
}

// 初始化函数（可选）
#[wasm_bindgen(start)]
pub fn init() {
    // 设置 panic hook 以便在浏览器控制台看到错误
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}
