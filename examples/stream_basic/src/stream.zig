const std = @import("std");

// 简单的数据生成函数 - 生成递增序列
export fn generate_sequence(start: u32, count: u32, output: [*]u32) void {
    var i: u32 = 0;
    while (i < count) : (i += 1) {
        output[i] = start + i;
    }
}

// 计算数组的和
export fn sum_array(data: [*]const u32, len: usize) u64 {
    var sum: u64 = 0;
    var i: usize = 0;
    while (i < len) : (i += 1) {
        sum += data[i];
    }
    return sum;
}

// 生成斐波那契数列
export fn generate_fibonacci(count: u32, output: [*]u64) void {
    if (count == 0) return;

    var a: u64 = 0;
    var b: u64 = 1;
    var i: u32 = 0;

    while (i < count) : (i += 1) {
        output[i] = a;
        const temp = a + b;
        a = b;
        b = temp;
    }
}

// 过滤偶数（原地操作，返回新长度）
export fn filter_even(data: [*]u32, len: usize) usize {
    var write_pos: usize = 0;
    var read_pos: usize = 0;

    while (read_pos < len) : (read_pos += 1) {
        if (data[read_pos] % 2 == 0) {
            data[write_pos] = data[read_pos];
            write_pos += 1;
        }
    }

    return write_pos;
}

// 倍增所有元素
export fn double_values(data: [*]u32, len: usize) void {
    var i: usize = 0;
    while (i < len) : (i += 1) {
        data[i] *= 2;
    }
}
