const std = @import("std");
const builtin = @import("builtin");

// Vector addition using SIMD (auto-vectorized by Zig)
export fn vector_add_f32(a: [*]const f32, b: [*]const f32, result: [*]f32, len: usize) void {
    const vec_size = 4; // Process 4 elements at a time
    var i: usize = 0;

    // Vectorized loop (Zig's @Vector automatically uses SIMD instructions)
    while (i + vec_size <= len) : (i += vec_size) {
        const vec_a: @Vector(vec_size, f32) = a[i..][0..vec_size].*;
        const vec_b: @Vector(vec_size, f32) = b[i..][0..vec_size].*;
        const vec_result = vec_a + vec_b;

        // Store result
        const result_array: [vec_size]f32 = vec_result;
        @memcpy(result[i..][0..vec_size], &result_array);
    }

    // Scalar remainder
    while (i < len) : (i += 1) {
        result[i] = a[i] + b[i];
    }
}

// Dot product using SIMD
export fn dot_product_f32(a: [*]const f32, b: [*]const f32, len: usize) f32 {
    const vec_size = 4;
    var sum: f32 = 0.0;
    var i: usize = 0;

    // Vectorized accumulation
    var vec_sum: @Vector(vec_size, f32) = @splat(0.0);

    while (i + vec_size <= len) : (i += vec_size) {
        const vec_a: @Vector(vec_size, f32) = a[i..][0..vec_size].*;
        const vec_b: @Vector(vec_size, f32) = b[i..][0..vec_size].*;
        vec_sum += vec_a * vec_b;
    }

    // Horizontal sum of vector
    const arr: [vec_size]f32 = vec_sum;
    for (arr) |val| {
        sum += val;
    }

    // Scalar remainder
    while (i < len) : (i += 1) {
        sum += a[i] * b[i];
    }

    return sum;
}

// 4x4 Matrix multiplication using SIMD
export fn matrix_mul_4x4(a: [*]const f32, b: [*]const f32, result: [*]f32) void {
    // Matrix multiplication: C = A * B
    // For 4x4 matrices, we can use @Vector(4, f32) for each row/column

    var row: usize = 0;
    while (row < 4) : (row += 1) {
        var col: usize = 0;
        while (col < 4) : (col += 1) {
            const row_vec: @Vector(4, f32) = a[row * 4 ..][0..4].*;
            const col_vec: @Vector(4, f32) = .{
                b[col],
                b[4 + col],
                b[8 + col],
                b[12 + col],
            };

            const product = row_vec * col_vec;

            // Sum components
            const arr: [4]f32 = product;
            var sum: f32 = 0;
            for (arr) |val| {
                sum += val;
            }

            result[row * 4 + col] = sum;
        }
    }
}

// Runtime SIMD feature detection
export fn get_simd_features() u32 {
    var features: u32 = 0;

    // Check for x86_64 SIMD features at compile time
    const cpu = builtin.cpu;

    // SSE2 is baseline for x86_64
    if (cpu.arch == .x86_64) {
        features |= 0x01; // SSE2

        // Check for SSE4.2
        if (std.Target.x86.featureSetHas(cpu.features, .sse4_2)) {
            features |= 0x02;
        }

        // Check for AVX
        if (std.Target.x86.featureSetHas(cpu.features, .avx)) {
            features |= 0x04;
        }

        // Check for AVX2
        if (std.Target.x86.featureSetHas(cpu.features, .avx2)) {
            features |= 0x08;
        }

        // Check for AVX-512
        if (std.Target.x86.featureSetHas(cpu.features, .avx512f)) {
            features |= 0x10;
        }
    } else if (cpu.arch == .aarch64 or cpu.arch == .arm) {
        // Check for NEON on ARM
        if (std.Target.arm.featureSetHas(cpu.features, .neon)) {
            features |= 0x100;
        }
    }

    return features;
}

// Benchmark vector operations
export fn benchmark_vector_ops(size: usize, iterations: u32) u64 {
    const allocator = std.heap.c_allocator;

    // Allocate test data
    const a = allocator.alloc(f32, size) catch return 0;
    defer allocator.free(a);
    const b = allocator.alloc(f32, size) catch return 0;
    defer allocator.free(b);
    const result = allocator.alloc(f32, size) catch return 0;
    defer allocator.free(result);

    // Initialize data
    for (a, 0..) |*val, i| {
        val.* = @as(f32, @floatFromInt(i));
    }
    for (b, 0..) |*val, i| {
        val.* = @as(f32, @floatFromInt(i)) * 2.0;
    }

    // Benchmark
    const start_time = std.time.nanoTimestamp();

    var iter: u32 = 0;
    while (iter < iterations) : (iter += 1) {
        vector_add_f32(a.ptr, b.ptr, result.ptr, size);
    }

    const end_time = std.time.nanoTimestamp();

    return @intCast(end_time - start_time);
}
