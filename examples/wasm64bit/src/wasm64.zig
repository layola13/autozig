//! AutoZig WASM 3.0 64-bit 内存支持示例
//!
//! 演示 Memory64 提案的核心特性：
//! - 64-bit 内存地址空间
//! - @wasmMemorySize 和 @wasmMemoryGrow intrinsics
//! - 大内存分配和访问
//! - 零拷贝内存共享

const std = @import("std");
const builtin = @import("builtin");

// 大内存缓冲区（演示 64-bit 地址空间）
// 在 wasm64 模式下，可以分配超过 4GB 的内存
var large_buffer: [16 * 1024 * 1024]u8 = undefined; // 16MB 缓冲区

/// 获取当前 WASM 内存大小（以 64KB 页为单位）
/// 在 wasm64 模式下返回 64-bit 地址
export fn get_memory_size() usize {
    // 注意：@wasmMemorySize 在 wasm64 下返回 usize (64-bit)
    // 在 wasm32 下返回 u32
    if (builtin.cpu.arch == .wasm64) {
        return @wasmMemorySize(0);
    } else {
        // 回退到 32-bit
        return @wasmMemorySize(0);
    }
}

/// 增长 WASM 内存
/// @param delta 要增长的页数（每页 64KB）
/// @return 增长前的内存大小，失败返回 -1
export fn grow_memory(delta: usize) isize {
    if (builtin.cpu.arch == .wasm64) {
        return @wasmMemoryGrow(0, delta);
    } else {
        // 回退到 32-bit
        return @wasmMemoryGrow(0, @intCast(delta));
    }
}

/// 分配大缓冲区并返回指针（零拷贝）
/// 演示 64-bit 地址空间
export fn alloc_large_buffer() [*]u8 {
    return &large_buffer;
}

/// 获取大缓冲区大小
export fn get_buffer_size() usize {
    return large_buffer.len;
}

/// 写入数据到缓冲区
/// @param offset 偏移量（支持 64-bit）
/// @param value 要写入的值
export fn write_buffer(offset: usize, value: u8) void {
    if (offset < large_buffer.len) {
        large_buffer[offset] = value;
    }
}

/// 从缓冲区读取数据
/// @param offset 偏移量（支持 64-bit）
/// @return 读取的值
export fn read_buffer(offset: usize) u8 {
    if (offset < large_buffer.len) {
        return large_buffer[offset];
    }
    return 0;
}

/// 填充缓冲区（演示批量内存操作）
/// @param start 起始偏移
/// @param length 长度
/// @param value 填充值
export fn fill_buffer(start: usize, length: usize, value: u8) void {
    const end = @min(start + length, large_buffer.len);
    if (start >= large_buffer.len) return;

    var i: usize = start;
    while (i < end) : (i += 1) {
        large_buffer[i] = value;
    }
}

/// 计算缓冲区校验和（演示计算密集型操作）
/// @param start 起始偏移
/// @param length 长度
/// @return 校验和
export fn checksum_buffer(start: usize, length: usize) u64 {
    const end = @min(start + length, large_buffer.len);
    if (start >= large_buffer.len) return 0;

    var sum: u64 = 0;
    var i: usize = start;
    while (i < end) : (i += 1) {
        sum +%= large_buffer[i];
    }
    return sum;
}

/// 在高地址写入数据（演示 >4GB 地址访问）
/// 注意：这需要运行时支持真正的 64-bit 内存
/// 在实际浏览器环境中通常限制在 16GB 以内
export fn write_at_high_address(value: u64) bool {
    // 检测是否为 wasm64 架构
    if (builtin.cpu.arch != .wasm64) {
        return false;
    }

    // 在实际应用中，你需要先增长内存到足够大
    // 这里只是演示概念
    // const high_addr: usize = 0x1_0000_0000; // 4GB
    // 由于浏览器限制，我们使用较小的地址

    // 演示：写入到缓冲区末尾
    if (large_buffer.len >= 8) {
        const ptr = @as(*u64, @ptrCast(@alignCast(&large_buffer[large_buffer.len - 8])));
        ptr.* = value;
        return true;
    }

    return false;
}

/// 从高地址读取数据
export fn read_at_high_address() u64 {
    if (builtin.cpu.arch != .wasm64) {
        return 0;
    }

    // 演示：从缓冲区末尾读取
    if (large_buffer.len >= 8) {
        const ptr = @as(*const u64, @ptrCast(@alignCast(&large_buffer[large_buffer.len - 8])));
        return ptr.*;
    }

    return 0;
}

/// 获取架构信息
export fn get_arch_info() u32 {
    // 返回 64 表示 wasm64，32 表示 wasm32
    if (builtin.cpu.arch == .wasm64) {
        return 64;
    } else {
        return 32;
    }
}

/// 获取指针大小（字节）
export fn get_pointer_size() usize {
    return @sizeOf(usize);
}

// 内存测试：验证 @wasmMemoryGrow 功能
// 这个测试在 wasm64 环境下运行
test "wasm64_memory_grow" {
    if (builtin.cpu.arch != .wasm64) {
        return error.SkipZigTest; // 仅在 wasm64 下运行
    }

    const prev = @wasmMemorySize(0);
    try std.testing.expect(prev == @wasmMemoryGrow(0, 1));
    try std.testing.expect(prev + 1 == @wasmMemorySize(0));
}

// 性能测试：大内存填充
test "large_buffer_fill" {
    const test_size: usize = 1024 * 1024; // 1MB
    fill_buffer(0, test_size, 0xFF);

    // 验证填充结果
    try std.testing.expect(large_buffer[0] == 0xFF);
    try std.testing.expect(large_buffer[test_size - 1] == 0xFF);
}

// 测试校验和计算
test "checksum_calculation" {
    // 填充测试数据
    fill_buffer(0, 1000, 42);

    // 计算校验和
    const sum = checksum_buffer(0, 1000);

    // 验证结果（42 * 1000 = 42000）
    try std.testing.expect(sum == 42000);
}
