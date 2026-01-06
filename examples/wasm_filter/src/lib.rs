//! AutoZig WASM 图像滤镜示例
//!
//! 演示如何使用 AutoZig 在 WASM 环境中调用 Zig 代码进行高性能图像处理

use autozig::autozig;
use wasm_bindgen::prelude::*;

// 使用 autozig! 宏嵌入 Zig 代码
autozig! {
    // 🚀 Zig SIMD 优化实现 - 使用 @Vector 进行真正的向量化
    // 配合 -mcpu=mvp+simd128 编译标志，将生成 v128.* 指令
    
    // 🔥 反色滤镜 - SIMD 向量化版本
    // 一条 SIMD 指令处理 16 字节（比循环展开快 5-10 倍）
    export fn invert_colors_raw(ptr: [*]u8, len: usize) void {
        const vec_len = 16; // WASM SIMD128 标准宽度
        var i: usize = 0;
        
        // 🎯 向量主循环：编译为 v128.load + v128.sub + v128.store
        while (i + vec_len <= len) : (i += vec_len) {
            const vec_ptr: *@Vector(vec_len, u8) = @ptrCast(@alignCast(ptr + i));
            const splat_255 = @as(@Vector(vec_len, u8), @splat(255));
            vec_ptr.* = splat_255 - vec_ptr.*;
        }
        
        // 标量 fallback：处理尾部不足 16 字节的数据
        while (i < len) : (i += 1) {
            ptr[i] = 255 - ptr[i];
        }
    }

    // 灰度滤镜 - 标量版本（SIMD 优化需要复杂的像素重排）
    export fn grayscale_raw(ptr: [*]u8, len: usize) void {
        var i: usize = 0;
        while (i < len) : (i += 4) {
            const r = @as(u32, ptr[i]);
            const g = @as(u32, ptr[i + 1]);
            const b = @as(u32, ptr[i + 2]);
            const gray = @as(u8, @intCast((r * 299 + g * 587 + b * 114) / 1000));
            ptr[i] = gray;
            ptr[i + 1] = gray;
            ptr[i + 2] = gray;
        }
    }

    // 🔥 亮度调整 - SIMD 饱和运算版本
    export fn adjust_brightness_raw(ptr: [*]u8, len: usize, delta: i32) void {
        const vec_len = 16;
        var i: usize = 0;
        
        if (delta >= 0) {
            // 增加亮度：SIMD 饱和加法
            const delta_u8 = @as(u8, @intCast(@min(delta, 255)));
            const delta_vec = @as(@Vector(vec_len, u8), @splat(delta_u8));
            
            while (i + vec_len <= len) : (i += vec_len) {
                const vec_ptr: *@Vector(vec_len, u8) = @ptrCast(@alignCast(ptr + i));
                // 编译为 v128.add_sat_u (饱和加法，防止溢出)
                vec_ptr.* = vec_ptr.* +| delta_vec;
            }
        } else {
            // 减少亮度：SIMD 饱和减法
            const delta_u8 = @as(u8, @intCast(@min(-delta, 255)));
            const delta_vec = @as(@Vector(vec_len, u8), @splat(delta_u8));
            
            while (i + vec_len <= len) : (i += vec_len) {
                const vec_ptr: *@Vector(vec_len, u8) = @ptrCast(@alignCast(ptr + i));
                // 编译为 v128.sub_sat_u (饱和减法)
                vec_ptr.* = vec_ptr.* -| delta_vec;
            }
        }
        
        // 标量 fallback：处理尾部
        while (i < len) : (i += 1) {
            const result = @as(i32, ptr[i]) + delta;
            ptr[i] = @intCast(@max(0, @min(255, result)));
        }
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
        data[i] = 255 - data[i]; // R
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
