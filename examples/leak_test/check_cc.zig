const std = @import("std");

pub fn main() !void {
    const info = @typeInfo(std.builtin.CallingConvention);
    std.debug.print("{any}\n", .{info});
}
