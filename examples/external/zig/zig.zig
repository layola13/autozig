const std = @import("std");
const builtin = @import("builtin");

pub fn panic(msg: []const u8, error_return_trace: ?*builtin.StackTrace, ret_addr: ?usize) noreturn {
    _ = msg;
    _ = error_return_trace;
    _ = ret_addr;
    std.process.exit(0xF);
}

fn pow(base: usize, exp: usize) usize {
    var x: usize = base;
    var i: usize = 1;

    while (i < exp) : (i += 1) {
        x *= base;
    }
    return x;
}

export fn add(a: i32, b: i32) i32 {
    return a + b;
}

export fn printing(buf: [*]const u8, len: usize) void {
    const s = buf[0..len];
    std.debug.print("Zig says: {s}\n", .{s});
}

fn itoa(comptime N: type, n: N, buff: []u8) void {
    @setRuntimeSafety(false);

    const UNROLL_MAX: usize = 4;
    const DIV_CONST: usize = pow(10, UNROLL_MAX);

    var num = n;
    var len = buff.len;

    while (len >= UNROLL_MAX) : (num = std.math.divTrunc(N, num, DIV_CONST) catch return) {
        comptime var DIV10: N = 1;
        comptime var CURRENT: usize = 0;

        // Write digits backwards into the buffer
        inline while (CURRENT != UNROLL_MAX) : ({
            CURRENT += 1;
            DIV10 *= 10;
        }) {
            const q = std.math.divTrunc(N, num, DIV10) catch break;
            const r = @as(u8, @truncate(std.math.mod(N, q, 10) catch break)) + 48;
            buff[len - CURRENT - 1] = r;
        }

        len -= UNROLL_MAX;
    }

    // On an empty buffer, this will wrapparoo to 0xfffff
    len -%= 1;

    // Stops at 0xfffff
    while (len != std.math.maxInt(usize)) : (len -%= 1) {
        const q: N = std.math.divTrunc(N, num, 10) catch break;
        const r: u8 = @as(u8, @truncate(std.math.mod(N, num, 10) catch break)) + 48;
        buff[len] = r;
        num = q;
    }
}

export fn itoa_u64(n: u64, noalias buff: [*]u8, len: usize) void {
    @setRuntimeSafety(false);
    const slice = buff[0..len];

    itoa(u64, n, slice);
}

test "empty buff" {
    const small_buff: []u8 = &[_]u8{};
    const small: u64 = 100;

    _ = itoa_u64(small, @constCast(small_buff.ptr), small_buff.len);
}

test "small buff" {
    const mem = std.mem;

    var small_buff = [_]u8{10} ** 3;
    const small: u64 = 100;

    // Should only run the 2nd while-loop, which is kinda like a fixup loop.
    itoa_u64(small, &small_buff, small_buff.len);

    try std.testing.expect(mem.eql(u8, &small_buff, "100"));
}

test "big buff" {
    const mem = std.mem;

    var big_buff = [_]u8{0} ** 10;
    const big: u64 = 1234123412;

    itoa_u64(big, &big_buff, big_buff.len);

    try std.testing.expect(mem.eql(u8, &big_buff, "1234123412"));
}

test "unroll count buf" {
    const mem = std.mem;

    var small_buff = [_]u8{10} ** 4;
    const small: u64 = 1000;

    // Should only run the 2nd while-loop, which is kinda like a fixup loop.
    itoa_u64(small, &small_buff, small_buff.len);

    try std.testing.expect(mem.eql(u8, &small_buff, "1000"));
}
