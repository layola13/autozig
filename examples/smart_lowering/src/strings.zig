const std = @import("std");

// 字符串处理函数 - Zig 端接收 ptr+len 参数
export fn process_string(ptr: [*]const u8, len: usize) usize {
    const s = ptr[0..len];
    var count: usize = 0;
    for (s) |c| {
        if (c == 'a' or c == 'e' or c == 'i' or c == 'o' or c == 'u') {
            count += 1;
        }
    }
    return count;
}

// 字符串拼接 - 返回新字符串的 ptr+len
export fn concat_strings(s1_ptr: [*]const u8, s1_len: usize, s2_ptr: [*]const u8, s2_len: usize, out_ptr: [*]u8) usize {
    const s1 = s1_ptr[0..s1_len];
    const s2 = s2_ptr[0..s2_len];

    // 拷贝 s1
    @memcpy(out_ptr[0..s1_len], s1);

    // 拷贝 s2
    @memcpy(out_ptr[s1_len .. s1_len + s2_len], s2);

    return s1_len + s2_len;
}

// 字符串转大写
export fn to_uppercase(ptr: [*]const u8, len: usize, out_ptr: [*]u8) void {
    const s = ptr[0..len];
    for (s, 0..) |c, i| {
        if (c >= 'a' and c <= 'z') {
            out_ptr[i] = c - 32;
        } else {
            out_ptr[i] = c;
        }
    }
}
