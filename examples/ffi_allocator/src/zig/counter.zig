const std = @import("std");

// ============================================================================
// FFI Allocator 正确模式示例
// ============================================================================
// 1. 本地 GPA Allocator - 每个文件直接定义，不跨文件导入
// 2. 非可选返回 (*T + catch unreachable)
// 3. Rust 端使用 *mut T 而不是 Option<*mut T>

var gpa_instance = std.heap.GeneralPurposeAllocator(.{}){};
pub const g_allocator = gpa_instance.allocator();

// 简单的计数器结构 - 演示 FFI 内存分配
pub const Counter = struct {
    value: u32,
    increment_count: u32,

    pub fn init(start_value: u32) Counter {
        return Counter{
            .value = start_value,
            .increment_count = 0,
        };
    }

    pub fn increment(self: *Counter) void {
        self.value += 1;
        self.increment_count += 1;
    }

    pub fn decrement(self: *Counter) void {
        if (self.value > 0) {
            self.value -= 1;
        }
    }

    pub fn reset(self: *Counter) void {
        self.value = 0;
        self.increment_count = 0;
    }
};

// ============================================================================
// FFI 导出 - 正确模式
// ============================================================================

// ✅ 正确：返回 *Counter 而不是 ?*Counter
export fn counter_create(start_value: u32) *Counter {
    const counter = g_allocator.create(Counter) catch unreachable;
    counter.* = Counter.init(start_value);
    return counter;
}

export fn counter_destroy(counter: *Counter) void {
    g_allocator.destroy(counter);
}

export fn counter_get(counter: *const Counter) u32 {
    return counter.value;
}

export fn counter_increment(counter: *Counter) void {
    counter.increment();
}

export fn counter_decrement(counter: *Counter) void {
    counter.decrement();
}

export fn counter_reset(counter: *Counter) void {
    counter.reset();
}

export fn counter_get_increment_count(counter: *const Counter) u32 {
    return counter.increment_count;
}
