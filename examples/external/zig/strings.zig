const std = @import("std");

// 字符串工具函数

export fn string_length(ptr: [*]const u8, len: usize) usize {
    _ = ptr;
    return len;
}

export fn string_count_char(ptr: [*]const u8, len: usize, ch: u8) usize {
    const s = ptr[0..len];
    var count: usize = 0;
    for (s) |c| {
        if (c == ch) {
            count += 1;
        }
    }
    return count;
}

export fn string_to_lowercase(src_ptr: [*]const u8, src_len: usize, dst_ptr: [*]u8) void {
    const src = src_ptr[0..src_len];
    for (src, 0..) |c, i| {
        if (c >= 'A' and c <= 'Z') {
            dst_ptr[i] = c + 32;
        } else {
            dst_ptr[i] = c;
        }
    }
}

// Unit tests for string functions
test "string length calculation" {
    const test_str = "Hello, World!";
    const result = string_length(test_str.ptr, test_str.len);
    try std.testing.expectEqual(@as(usize, 13), result);
}

test "count character in string" {
    const test_str = "Hello, World!";
    const count_l = string_count_char(test_str.ptr, test_str.len, 'l');
    const count_o = string_count_char(test_str.ptr, test_str.len, 'o');
    const count_x = string_count_char(test_str.ptr, test_str.len, 'x');

    try std.testing.expectEqual(@as(usize, 3), count_l);
    try std.testing.expectEqual(@as(usize, 2), count_o);
    try std.testing.expectEqual(@as(usize, 0), count_x);
}

test "string to lowercase conversion" {
    const src = "Hello WORLD 123!";
    var dst: [16]u8 = undefined;

    string_to_lowercase(src.ptr, src.len, &dst);

    const expected = "hello world 123!";
    try std.testing.expect(std.mem.eql(u8, &dst, expected));
}
