//! AutoZig WASM 多光源渲染 - Zig SIMD 实现
//!
//! 演示零拷贝内存共享 + SIMD 向量化光照计算

const std = @import("std");

// 静态内存缓冲区（避免动态分配）
var pixel_buffer: [1024 * 1024 * 4]u8 = undefined;
var lights_buffer: [100 * 8]f32 = undefined; // 最多 100 个光源，每个 8 个 f32

/// 分配像素缓冲区并返回指针（零拷贝设计）
export fn alloc_pixel_buffer(width: u32, height: u32) [*]u8 {
    const size = width * height * 4;
    if (size > pixel_buffer.len) {
        @panic("Pixel buffer overflow");
    }
    return &pixel_buffer;
}

/// 分配光源缓冲区并返回指针
export fn alloc_lights_buffer(count: u32) [*]f32 {
    const size = count * 8;
    if (size > lights_buffer.len) {
        @panic("Lights buffer overflow");
    }
    return &lights_buffer;
}

/// 🔥 SIMD 向量化多光源渲染
/// 使用 @Vector(4, f32) 一次处理 4 个像素
export fn render_lights_simd_raw(
    pixel_ptr: [*]u8,
    width: u32,
    height: u32,
    lights_ptr: [*]const f32,
    num_lights: u32,
) void {
    const Vec4 = @Vector(4, f32);

    var y: u32 = 0;
    while (y < height) : (y += 1) {
        const pixel_y = @as(f32, @floatFromInt(y));
        const vec_y = Vec4{ pixel_y, pixel_y, pixel_y, pixel_y };

        var x: u32 = 0;
        // SIMD 核心：每次处理 4 个像素
        while (x + 4 <= width) : (x += 4) {
            // 构建 4 个像素的 X 坐标向量
            const vec_x = Vec4{
                @as(f32, @floatFromInt(x)),
                @as(f32, @floatFromInt(x + 1)),
                @as(f32, @floatFromInt(x + 2)),
                @as(f32, @floatFromInt(x + 3)),
            };

            // 累积颜色（4 个像素的 RGB）
            var color_r = Vec4{ 0.0, 0.0, 0.0, 0.0 };
            var color_g = Vec4{ 0.0, 0.0, 0.0, 0.0 };
            var color_b = Vec4{ 0.0, 0.0, 0.0, 0.0 };

            // 遍历所有光源
            var i: u32 = 0;
            while (i < num_lights) : (i += 1) {
                const light_offset = i * 8;
                const light_x = lights_ptr[light_offset + 0];
                const light_y = lights_ptr[light_offset + 1];
                const light_z = lights_ptr[light_offset + 2];
                const intensity = lights_ptr[light_offset + 3];
                const light_r = lights_ptr[light_offset + 4];
                const light_g = lights_ptr[light_offset + 5];
                const light_b = lights_ptr[light_offset + 6];
                const radius = lights_ptr[light_offset + 7];

                // 广播光源坐标到向量
                const vec_light_x = Vec4{ light_x, light_x, light_x, light_x };
                const vec_light_y = Vec4{ light_y, light_y, light_y, light_y };
                const vec_light_z = Vec4{ light_z, light_z, light_z, light_z };
                const vec_radius = Vec4{ radius, radius, radius, radius };
                const vec_intensity = Vec4{ intensity, intensity, intensity, intensity };

                // SIMD 距离计算（4 个像素同时计算）
                const dx = vec_x - vec_light_x;
                const dy = vec_y - vec_light_y;
                const dz = Vec4{ 0.0, 0.0, 0.0, 0.0 } - vec_light_z;
                const dist_sq = dx * dx + dy * dy + dz * dz;
                const dist = @sqrt(dist_sq);

                // SIMD 衰减计算
                const in_range = dist < vec_radius;
                const attenuation = (Vec4{ 1.0, 1.0, 1.0, 1.0 } - dist / vec_radius) * vec_intensity;

                // 使用 select 实现条件累加（SIMD 版本的 if）
                const vec_light_r = Vec4{ light_r, light_r, light_r, light_r };
                const vec_light_g = Vec4{ light_g, light_g, light_g, light_g };
                const vec_light_b = Vec4{ light_b, light_b, light_b, light_b };

                color_r += @select(f32, in_range, vec_light_r * attenuation, Vec4{ 0.0, 0.0, 0.0, 0.0 });
                color_g += @select(f32, in_range, vec_light_g * attenuation, Vec4{ 0.0, 0.0, 0.0, 0.0 });
                color_b += @select(f32, in_range, vec_light_b * attenuation, Vec4{ 0.0, 0.0, 0.0, 0.0 });
            }

            // 限制到 [0, 255]
            const vec_255 = Vec4{ 255.0, 255.0, 255.0, 255.0 };
            const vec_0 = Vec4{ 0.0, 0.0, 0.0, 0.0 };
            const final_r = @min(vec_255, @max(vec_0, color_r));
            const final_g = @min(vec_255, @max(vec_0, color_g));
            const final_b = @min(vec_255, @max(vec_0, color_b));

            // 写入 4 个像素（展开循环）
            inline for (0..4) |offset| {
                const pixel_offset = (y * width + x + offset) * 4;
                pixel_ptr[pixel_offset + 0] = @intFromFloat(final_r[offset]);
                pixel_ptr[pixel_offset + 1] = @intFromFloat(final_g[offset]);
                pixel_ptr[pixel_offset + 2] = @intFromFloat(final_b[offset]);
                pixel_ptr[pixel_offset + 3] = 255;
            }
        }

        // 处理剩余像素（不足 4 个）
        while (x < width) : (x += 1) {
            const pixel_x = @as(f32, @floatFromInt(x));
            var color_r: f32 = 0.0;
            var color_g: f32 = 0.0;
            var color_b: f32 = 0.0;

            var i: u32 = 0;
            while (i < num_lights) : (i += 1) {
                const light_offset = i * 8;
                const light_x = lights_ptr[light_offset + 0];
                const light_y = lights_ptr[light_offset + 1];
                const light_z = lights_ptr[light_offset + 2];
                const intensity = lights_ptr[light_offset + 3];
                const light_r = lights_ptr[light_offset + 4];
                const light_g = lights_ptr[light_offset + 5];
                const light_b = lights_ptr[light_offset + 6];
                const radius = lights_ptr[light_offset + 7];

                const dx = pixel_x - light_x;
                const dy = pixel_y - light_y;
                const dz = 0.0 - light_z;
                const dist = @sqrt(dx * dx + dy * dy + dz * dz);

                if (dist < radius) {
                    const attenuation = (1.0 - dist / radius) * intensity;
                    color_r += light_r * attenuation;
                    color_g += light_g * attenuation;
                    color_b += light_b * attenuation;
                }
            }

            const r = @min(255.0, @max(0.0, color_r));
            const g = @min(255.0, @max(0.0, color_g));
            const b = @min(255.0, @max(0.0, color_b));

            const pixel_offset = (y * width + x) * 4;
            pixel_ptr[pixel_offset + 0] = @intFromFloat(r);
            pixel_ptr[pixel_offset + 1] = @intFromFloat(g);
            pixel_ptr[pixel_offset + 2] = @intFromFloat(b);
            pixel_ptr[pixel_offset + 3] = 255;
        }
    }
}

/// Rust Scalar 实现（对比基准）
export fn render_lights_scalar_raw(
    pixel_ptr: [*]u8,
    width: u32,
    height: u32,
    lights_ptr: [*]const f32,
    num_lights: u32,
) void {
    var y: u32 = 0;
    while (y < height) : (y += 1) {
        var x: u32 = 0;
        while (x < width) : (x += 1) {
            const pixel_x = @as(f32, @floatFromInt(x));
            const pixel_y = @as(f32, @floatFromInt(y));

            var color_r: f32 = 0.0;
            var color_g: f32 = 0.0;
            var color_b: f32 = 0.0;

            var i: u32 = 0;
            while (i < num_lights) : (i += 1) {
                const light_offset = i * 8;
                const light_x = lights_ptr[light_offset + 0];
                const light_y = lights_ptr[light_offset + 1];
                const light_z = lights_ptr[light_offset + 2];
                const intensity = lights_ptr[light_offset + 3];
                const light_r = lights_ptr[light_offset + 4];
                const light_g = lights_ptr[light_offset + 5];
                const light_b = lights_ptr[light_offset + 6];
                const radius = lights_ptr[light_offset + 7];

                const dx = pixel_x - light_x;
                const dy = pixel_y - light_y;
                const dz = 0.0 - light_z;
                const dist_sq = dx * dx + dy * dy + dz * dz;
                const dist = @sqrt(dist_sq);

                if (dist < radius) {
                    const attenuation = (1.0 - dist / radius) * intensity;
                    color_r += light_r * attenuation;
                    color_g += light_g * attenuation;
                    color_b += light_b * attenuation;
                }
            }

            const r = @min(255.0, @max(0.0, color_r));
            const g = @min(255.0, @max(0.0, color_g));
            const b = @min(255.0, @max(0.0, color_b));

            const pixel_offset = (y * width + x) * 4;
            pixel_ptr[pixel_offset + 0] = @intFromFloat(r);
            pixel_ptr[pixel_offset + 1] = @intFromFloat(g);
            pixel_ptr[pixel_offset + 2] = @intFromFloat(b);
            pixel_ptr[pixel_offset + 3] = 255;
        }
    }
}
