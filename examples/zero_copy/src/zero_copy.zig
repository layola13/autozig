const std = @import("std");

// RawVec structure compatible with Rust's RawVec<T>
fn RawVec(comptime T: type) type {
    return extern struct {
        ptr: [*]T,
        len: usize,
        cap: usize,
        _phantom: u8 = 0, // PhantomData is zero-sized in Rust
    };
}

// Generate i32 data using c_allocator (compatible with Rust)
export fn generate_i32_data(size: usize) RawVec(i32) {
    const allocator = std.heap.c_allocator;
    const data = allocator.alloc(i32, size) catch |err| {
        std.debug.print("Allocation failed: {}\n", .{err});
        return RawVec(i32){ .ptr = undefined, .len = 0, .cap = 0 };
    };

    // Fill with sequential data
    for (data, 0..) |*item, i| {
        item.* = @intCast(i);
    }

    return RawVec(i32){
        .ptr = data.ptr,
        .len = data.len,
        .cap = data.len,
    };
}

// Generate f32 data
export fn generate_f32_data(size: usize) RawVec(f32) {
    const allocator = std.heap.c_allocator;
    const data = allocator.alloc(f32, size) catch |err| {
        std.debug.print("Allocation failed: {}\n", .{err});
        return RawVec(f32){ .ptr = undefined, .len = 0, .cap = 0 };
    };

    // Fill with float data
    for (data, 0..) |*item, i| {
        item.* = @as(f32, @floatFromInt(i)) * 1.5;
    }

    return RawVec(f32){
        .ptr = data.ptr,
        .len = data.len,
        .cap = data.len,
    };
}

// Generate image data (RGBA)
export fn generate_image_data(width: usize, height: usize) RawVec(u8) {
    const allocator = std.heap.c_allocator;
    const size = width * height * 4; // RGBA
    const data = allocator.alloc(u8, size) catch |err| {
        std.debug.print("Allocation failed: {}\n", .{err});
        return RawVec(u8){ .ptr = undefined, .len = 0, .cap = 0 };
    };

    // Fill with gradient pattern
    var y: usize = 0;
    while (y < height) : (y += 1) {
        var x: usize = 0;
        while (x < width) : (x += 1) {
            const idx = (y * width + x) * 4;
            data[idx + 0] = @intCast(x % 256); // R
            data[idx + 1] = @intCast(y % 256); // G
            data[idx + 2] = @intCast((x + y) % 256); // B
            data[idx + 3] = 255; // A
        }
    }

    return RawVec(u8){
        .ptr = data.ptr,
        .len = data.len,
        .cap = data.len,
    };
}

// DataStats structure
pub const DataStats = extern struct {
    min: i32,
    max: i32,
    sum: i64,
    count: usize,
};

// Compute statistics on i32 array
export fn compute_stats(data: [*]const i32, len: usize) DataStats {
    if (len == 0) {
        return DataStats{
            .min = 0,
            .max = 0,
            .sum = 0,
            .count = 0,
        };
    }

    const slice = data[0..len];
    var min_val = slice[0];
    var max_val = slice[0];
    var sum: i64 = 0;

    for (slice) |val| {
        if (val < min_val) min_val = val;
        if (val > max_val) max_val = val;
        sum += val;
    }

    return DataStats{
        .min = min_val,
        .max = max_val,
        .sum = sum,
        .count = len,
    };
}

// Double all values in-place
export fn double_values(data: [*]i32, len: usize) void {
    const slice = data[0..len];
    for (slice) |*val| {
        val.* *= 2;
    }
}
