const std = @import("std");

// Array operations module
// This demonstrates a module that could potentially use an allocator

export fn array_sum_i32(ptr: [*]const i32, len: usize) i64 {
    var sum: i64 = 0;
    var i: usize = 0;
    while (i < len) : (i += 1) {
        sum += ptr[i];
    }
    return sum;
}

export fn array_min_i32(ptr: [*]const i32, len: usize) i32 {
    if (len == 0) return 0;
    var min_val = ptr[0];
    var i: usize = 1;
    while (i < len) : (i += 1) {
        if (ptr[i] < min_val) {
            min_val = ptr[i];
        }
    }
    return min_val;
}

export fn array_max_i32(ptr: [*]const i32, len: usize) i32 {
    if (len == 0) return 0;
    var max_val = ptr[0];
    var i: usize = 1;
    while (i < len) : (i += 1) {
        if (ptr[i] > max_val) {
            max_val = ptr[i];
        }
    }
    return max_val;
}

export fn array_reverse_i32(ptr: [*]i32, len: usize) void {
    var i: usize = 0;
    const half = len / 2;
    while (i < half) : (i += 1) {
        const temp = ptr[i];
        ptr[i] = ptr[len - 1 - i];
        ptr[len - 1 - i] = temp;
    }
}
