//! AutoZig 安全测试套件
//! 
//! ⚠️ 警告：本文件包含故意引入的安全漏洞示例，仅用于测试和教育目的！
//! 
//! 测试方法：
//! ```bash
//! # 安全版本（默认）
//! cargo run
//! 
//! # AddressSanitizer检测（需要nightly）
//! RUSTFLAGS="-Z sanitizer=address" cargo +nightly run --release
//! ```

use autozig::prelude::*;

fn main() {
    println!("=== AutoZig 安全测试套件 ===\n");
    println!("✅ 运行安全版本测试\n");
    
    test_safe_buffer_operations();
    test_safe_bounds_checking();
    test_safe_struct_passing();
    
    println!("\n✅ 所有安全测试通过！");
    println!("\n📖 查看 README.md 了解如何测试潜在漏洞");
    println!("   例如：使用 AddressSanitizer 检测内存错误");
}

// 所有测试代码集中在一个宏中，避免命名冲突
autozig! {
    const std = @import("std");
    
    //==========================================================================
    // 安全版本测试
    //==========================================================================
    
    // 1. 安全的缓冲区操作 - 有边界检查
    export fn safe_fill_buffer(ptr: [*]u8, len: usize, value: u8) void {
        const slice = ptr[0..len];
        for (slice) |*byte| {
            byte.* = value;
        }
    }
    
    export fn safe_sum_buffer(ptr: [*]const u8, len: usize) u64 {
        const slice = ptr[0..len];
        var sum: u64 = 0;
        for (slice) |byte| {
            sum +%= byte;
        }
        return sum;
    }
    
    // 2. 安全的边界检查
    export fn safe_get_element(ptr: [*]const u8, len: usize, index: usize) i32 {
        if (index >= len) {
            return -1;  // 返回错误码而不是越界访问
        }
        return @as(i32, ptr[index]);
    }
    
    // 3. 安全的结构体传递 - 使用 extern struct 确保 C ABI
    const SafePoint = extern struct {
        x: i32,
        y: i32,
    };
    
    export fn safe_point_distance(p: SafePoint) f32 {
        const dx = @as(f32, @floatFromInt(p.x));
        const dy = @as(f32, @floatFromInt(p.y));
        return @sqrt(dx * dx + dy * dy);
    }
    
    //==========================================================================
    // 演示：故意的不安全模式（仅用于教育，已禁用）
    //==========================================================================
    
    // 注意：以下代码展示了潜在的不安全模式，但在这个测试中不会被调用
    // 要测试这些场景，需要使用sanitizer工具
    
    // 潜在风险1：如果Zig代码保存指针 (Use-After-Free风险)
    // var saved_ptr: ?[*]u8 = null;  // 危险！永远不要这样做
    
    // 潜在风险2：如果没有边界检查 (Buffer Overflow风险)
    // export fn unsafe_write(ptr: [*]u8, len: usize) void {
    //     var i: usize = 0;
    //     while (i < len + 10) : (i += 1) {  // 越界！
    //         ptr[i] = 0xFF;
    //     }
    // }
    
    // 潜在风险3：如果结构体布局不匹配 (ABI Mismatch风险)
    // const BadStruct = struct {  // 没有使用 extern！
    //     x: u8,
    //     y: u32,  // padding可能不同
    // };
    
    ---
    
    // Rust函数签名
    fn safe_fill_buffer(data: &mut [u8], value: u8);
    fn safe_sum_buffer(data: &[u8]) -> u64;
    fn safe_get_element(data: &[u8], index: usize) -> i32;
    
    #[repr(C)]
    struct SafePoint {
        x: i32,
        y: i32,
    }
    
    fn safe_point_distance(p: SafePoint) -> f32;
}

fn test_safe_buffer_operations() {
    println!("1. 测试安全的缓冲区操作...");
    
    // 测试填充
    let mut buf = vec![0u8; 100];
    safe_fill_buffer(&mut buf, 0xFF);
    assert_eq!(buf[0], 0xFF);
    assert_eq!(buf[99], 0xFF);
    
    // 测试求和
    let sum = safe_sum_buffer(&buf);
    assert_eq!(sum, 100 * 0xFF);
    
    println!("   ✓ 缓冲区操作安全 - 所有访问都有边界检查");
}

fn test_safe_bounds_checking() {
    println!("2. 测试安全的边界检查...");
    
    let data = vec![10, 20, 30, 40, 50];
    
    // 有效访问
    assert_eq!(safe_get_element(&data, 0), 10);
    assert_eq!(safe_get_element(&data, 4), 50);
    
    // 越界访问返回错误码而不是崩溃
    assert_eq!(safe_get_element(&data, 5), -1);
    assert_eq!(safe_get_element(&data, 100), -1);
    
    println!("   ✓ 边界检查有效 - 越界访问被安全处理");
}

fn test_safe_struct_passing() {
    println!("3. 测试安全的结构体传递...");
    
    let p = SafePoint { x: 3, y: 4 };
    let dist = safe_point_distance(p);
    assert!((dist - 5.0).abs() < 0.001);
    
    println!("   ✓ 结构体ABI兼容 - #[repr(C)] 确保布局一致");
}