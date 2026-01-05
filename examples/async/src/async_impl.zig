const std = @import("std");

// 重计算任务 - 同步实现
export fn heavy_computation(data: i32) i32 {
    // 模拟CPU密集型计算
    var result: i32 = data;
    var i: i32 = 0;
    while (i < 1000000) : (i += 1) {
        result = @addWithOverflow(result, 1)[0];
        result = @subWithOverflow(result, 1)[0];
    }
    return result * 2;
}

// 数据处理任务 - 同步实现
export fn process_data(input_ptr: [*]const u8, input_len: usize) usize {
    // 计算字节数组的和
    var sum: usize = 0;
    var i: usize = 0;
    while (i < input_len) : (i += 1) {
        sum +%= input_ptr[i];
    }
    return sum;
}

// 数据库查询模拟 - 同步实现
export fn query_database(id: i32) i32 {
    // 模拟数据库查询延迟
    var result: i32 = id;
    var i: i32 = 0;
    while (i < 500000) : (i += 1) {
        result = @addWithOverflow(result, 1)[0];
        result = @subWithOverflow(result, 1)[0];
    }
    return result + 100;
}

// Zig单元测试
test "heavy_computation" {
    const result = heavy_computation(42);
    try std.testing.expectEqual(@as(i32, 84), result);
}

test "process_data" {
    const data = [_]u8{ 1, 2, 3, 4, 5 };
    const result = process_data(&data, data.len);
    try std.testing.expectEqual(@as(usize, 15), result);
}

test "query_database" {
    const result = query_database(123);
    try std.testing.expectEqual(@as(i32, 223), result);
}
