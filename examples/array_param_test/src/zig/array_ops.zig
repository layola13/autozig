// Array operations for testing fixed-size array parameter conversion
// Zig uses *const [N]T pointer types for FFI compatibility

const std = @import("std");

/// Vec3 (3D vector) operations
pub const Vec3 = extern struct {
    x: f32,
    y: f32,
    z: f32,
};

/// Add two Vec3 and return result
export fn vec3_add(a: *const [3]f32, b: *const [3]f32) Vec3 {
    return Vec3{
        .x = a[0] + b[0],
        .y = a[1] + b[1],
        .z = a[2] + b[2],
    };
}

/// Compute dot product of two Vec3
export fn vec3_dot(a: *const [3]f32, b: *const [3]f32) f32 {
    return a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
}

/// Create Vec3 from array
export fn vec3_from_array(arr: *const [3]f32) Vec3 {
    return Vec3{ .x = arr[0], .y = arr[1], .z = arr[2] };
}

/// Quaternion (4D) operations
pub const Quat = extern struct {
    x: f32,
    y: f32,
    z: f32,
    w: f32,
};

/// Create quaternion from array [x, y, z, w]
export fn quat_from_array(arr: *const [4]f32) Quat {
    return Quat{ .x = arr[0], .y = arr[1], .z = arr[2], .w = arr[3] };
}

/// Conjugate of quaternion
export fn quat_conjugate(q: *const [4]f32) Quat {
    return Quat{ .x = -q[0], .y = -q[1], .z = -q[2], .w = q[3] };
}

/// Mat4 (4x4 matrix) operations - column-major order
pub const Mat4 = extern struct {
    data: [16]f32,
};

/// Create identity matrix
export fn mat4_identity() Mat4 {
    return Mat4{
        .data = [16]f32{
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0,
        },
    };
}

/// Create Mat4 from array
export fn mat4_from_array(arr: *const [16]f32) Mat4 {
    return Mat4{ .data = arr.* };
}

/// Get translation from matrix (column 3)
export fn mat4_get_translation(m: *const [16]f32) Vec3 {
    return Vec3{
        .x = m[12],
        .y = m[13],
        .z = m[14],
    };
}

/// Multiply matrix by scalar
export fn mat4_scale(m: *const [16]f32, s: f32, out: *[16]f32) void {
    var i: usize = 0;
    while (i < 16) : (i += 1) {
        out[i] = m[i] * s;
    }
}
