// wrapper.zig - Zig wrapper that calls C code and exposes to Rust
const std = @import("std");

// 直接声明C函数接口（不使用@cImport）
// C函数在 math.c 中实现，由 build.rs 编译并链接
extern "c" fn c_add(a: i32, b: i32) i32;
extern "c" fn c_multiply(a: i32, b: i32) i32;
extern "c" fn c_sum_array(arr: [*c]const i32, len: u32) i32;
extern "c" fn c_string_length(str: [*c]const u8) u32;

// === 直接包装C函数 ===

export fn add(a: i32, b: i32) i32 {
    return c_add(a, b);
}

export fn multiply(a: i32, b: i32) i32 {
    return c_multiply(a, b);
}

// === Zig增强功能：使用C函数实现幂运算 ===

export fn power(base: i32, exp: u32) i32 {
    var result: i32 = 1;
    var i: u32 = 0;
    while (i < exp) : (i += 1) {
        result = c_multiply(result, base);
    }
    return result;
}

// === 智能降级：&[i32] → ptr + len ===

export fn sum_array(arr_ptr: [*c]const i32, arr_len: usize) i32 {
    if (arr_len == 0) return 0;
    return c_sum_array(arr_ptr, @intCast(arr_len));
}

// === 智能降级：&str → ptr + len ===

export fn string_length(str_ptr: [*c]const u8, str_len: usize) u32 {
    // 直接使用Rust传入的长度（Rust字符串不是null终止的）
    _ = str_ptr;
    return @intCast(str_len);
}

// === Zig独有功能：计算平均值（组合C求和 + Zig浮点运算）===

export fn average(arr_ptr: [*c]const i32, arr_len: usize) f64 {
    if (arr_len == 0) return 0.0;

    const sum = c_sum_array(arr_ptr, @intCast(arr_len));
    return @as(f64, @floatFromInt(sum)) / @as(f64, @floatFromInt(arr_len));
}
