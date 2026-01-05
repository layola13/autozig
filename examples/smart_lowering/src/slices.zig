const std = @import("std");

// 数组求和 - 接收 &[i32]
export fn sum_array(ptr: [*]const i32, len: usize) i32 {
    const arr = ptr[0..len];
    var sum: i32 = 0;
    for (arr) |val| {
        sum += val;
    }
    return sum;
}

// 数组最大值 - 接收 &[i32]
export fn max_array(ptr: [*]const i32, len: usize) i32 {
    const arr = ptr[0..len];
    if (arr.len == 0) return 0;

    var max_val = arr[0];
    for (arr[1..]) |val| {
        if (val > max_val) {
            max_val = val;
        }
    }
    return max_val;
}

// 数组修改 - 接收 &mut [i32]，所有元素乘以2
export fn double_array(ptr: [*]i32, len: usize) void {
    const arr = ptr[0..len];
    for (arr) |*val| {
        val.* *= 2;
    }
}

// 数组过滤 - 接收 &[i32]，返回偶数个数
export fn count_even(ptr: [*]const i32, len: usize) usize {
    const arr = ptr[0..len];
    var count: usize = 0;
    for (arr) |val| {
        if (@mod(val, 2) == 0) {
            count += 1;
        }
    }
    return count;
}

// 字节数组处理 - 接收 &[u8]
export fn checksum(ptr: [*]const u8, len: usize) u8 {
    const data = ptr[0..len];
    var sum: u8 = 0;
    for (data) |byte| {
        sum = sum +% byte; // wrapping add
    }
    return sum;
}
