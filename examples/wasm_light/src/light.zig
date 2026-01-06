//! AutoZig WASM 多光源渲染 - Zig SIMD 实现
//!
//! 演示零拷贝内存共享 + SIMD 向量化光照计算

const std = @import("std");

// 静态内存缓冲区（避免动态分配）
var pixel_buffer: [1024 * 1024 * 4]u8 = undefined;
var background_buffer: [1024 * 1024 * 4]u8 = undefined; // 底图缓冲区
var lights_buffer: [100 * 8]f32 = undefined; // 最多 100 个光源，每个 8 个 f32

/// 分配像素缓冲区并返回指针（零拷贝设计）
export fn alloc_pixel_buffer(width: u32, height: u32) [*]u8 {
    const size = width * height * 4;
    if (size > pixel_buffer.len) {
        @panic("Pixel buffer overflow");
    }
    return &pixel_buffer;
}

/// 分配底图缓冲区并返回指针
export fn alloc_background_buffer(width: u32, height: u32) [*]u8 {
    const size = width * height * 4;
    if (size > background_buffer.len) {
        @panic("Background buffer overflow");
    }
    return &background_buffer;
}

/// 分配光源缓冲区并返回指针
export fn alloc_lights_buffer(count: u32) [*]f32 {
    const size = count * 8;
    if (size > lights_buffer.len) {
        @panic("Lights buffer overflow");
    }
    return &lights_buffer;
}

/// 🔥 SIMD 向量化多光源渲染（带底图照明）
/// 使用 @Vector(4, f32) 一次处理 4 个像素
/// 使用平方衰减模拟真实光照
export fn render_lights_simd_raw(
    pixel_ptr: [*]u8,
    width: u32,
    height: u32,
    lights_ptr: [*]const f32,
    num_lights: u32,
) void {
    const Vec4 = @Vector(4, f32);
    const ambient = 0.15; // 环境光强度 (15%)

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

            // 读取底图颜色（4个像素）
            var base_r = Vec4{ 0.0, 0.0, 0.0, 0.0 };
            var base_g = Vec4{ 0.0, 0.0, 0.0, 0.0 };
            var base_b = Vec4{ 0.0, 0.0, 0.0, 0.0 };

            inline for (0..4) |offset| {
                const bg_offset = (y * width + x + offset) * 4;
                base_r[offset] = @as(f32, @floatFromInt(background_buffer[bg_offset + 0]));
                base_g[offset] = @as(f32, @floatFromInt(background_buffer[bg_offset + 1]));
                base_b[offset] = @as(f32, @floatFromInt(background_buffer[bg_offset + 2]));
            }

            // 累积光照强度（4 个像素）
            var light_intensity = Vec4{ ambient, ambient, ambient, ambient };

            // 遍历所有光源
            var i: u32 = 0;
            while (i < num_lights) : (i += 1) {
                const light_offset = i * 8;
                const light_x = lights_ptr[light_offset + 0];
                const light_y = lights_ptr[light_offset + 1];
                const light_z = lights_ptr[light_offset + 2];
                const intensity = lights_ptr[light_offset + 3];
                const radius = lights_ptr[light_offset + 7];

                // 广播光源坐标到向量
                const vec_light_x = Vec4{ light_x, light_x, light_x, light_x };
                const vec_light_y = Vec4{ light_y, light_y, light_y, light_y };
                const vec_light_z = Vec4{ light_z, light_z, light_z, light_z };
                const vec_radius = Vec4{ radius, radius, radius, radius };
                const vec_intensity = Vec4{ intensity / 100.0, intensity / 100.0, intensity / 100.0, intensity / 100.0 };

                // SIMD 距离计算（4 个像素同时计算）
                const dx = vec_x - vec_light_x;
                const dy = vec_y - vec_light_y;
                const dz = Vec4{ 0.0, 0.0, 0.0, 0.0 } - vec_light_z;
                const dist_sq = dx * dx + dy * dy + dz * dz;
                const dist = @sqrt(dist_sq);

                // 平方衰减（物理真实）+ 平滑过渡
                const in_range = dist < vec_radius;
                const norm_dist = dist / vec_radius;
                const falloff = (Vec4{ 1.0, 1.0, 1.0, 1.0 } - norm_dist * norm_dist) * vec_intensity;

                light_intensity += @select(f32, in_range, falloff, Vec4{ 0.0, 0.0, 0.0, 0.0 });
            }

            // 限制光照强度到 [ambient, 1.5]（允许过曝效果）
            const vec_ambient = Vec4{ ambient, ambient, ambient, ambient };
            const vec_max = Vec4{ 1.5, 1.5, 1.5, 1.5 };
            const clamped_intensity = @min(vec_max, @max(vec_ambient, light_intensity));

            // 应用光照到底图（相乘）
            const final_r = @min(Vec4{ 255.0, 255.0, 255.0, 255.0 }, base_r * clamped_intensity);
            const final_g = @min(Vec4{ 255.0, 255.0, 255.0, 255.0 }, base_g * clamped_intensity);
            const final_b = @min(Vec4{ 255.0, 255.0, 255.0, 255.0 }, base_b * clamped_intensity);

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
            const pixel_offset = (y * width + x) * 4;

            // 读取底图颜色
            const base_r = @as(f32, @floatFromInt(background_buffer[pixel_offset + 0]));
            const base_g = @as(f32, @floatFromInt(background_buffer[pixel_offset + 1]));
            const base_b = @as(f32, @floatFromInt(background_buffer[pixel_offset + 2]));

            var light_intensity: f32 = ambient;

            var i: u32 = 0;
            while (i < num_lights) : (i += 1) {
                const light_offset = i * 8;
                const light_x = lights_ptr[light_offset + 0];
                const light_y = lights_ptr[light_offset + 1];
                const light_z = lights_ptr[light_offset + 2];
                const intensity = lights_ptr[light_offset + 3];
                const radius = lights_ptr[light_offset + 7];

                const dx = pixel_x - light_x;
                const dy = pixel_y - light_y;
                const dz = 0.0 - light_z;
                const dist = @sqrt(dx * dx + dy * dy + dz * dz);

                if (dist < radius) {
                    const norm_dist = dist / radius;
                    const falloff = (1.0 - norm_dist * norm_dist) * (intensity / 100.0);
                    light_intensity += falloff;
                }
            }

            const clamped_intensity = @min(1.5, @max(ambient, light_intensity));
            const final_r = @min(255.0, base_r * clamped_intensity);
            const final_g = @min(255.0, base_g * clamped_intensity);
            const final_b = @min(255.0, base_b * clamped_intensity);

            pixel_ptr[pixel_offset + 0] = @intFromFloat(final_r);
            pixel_ptr[pixel_offset + 1] = @intFromFloat(final_g);
            pixel_ptr[pixel_offset + 2] = @intFromFloat(final_b);
            pixel_ptr[pixel_offset + 3] = 255;
        }
    }
}

/// Scalar 标量实现（对比基准，带底图照明）
export fn render_lights_scalar_raw(
    pixel_ptr: [*]u8,
    width: u32,
    height: u32,
    lights_ptr: [*]const f32,
    num_lights: u32,
) void {
    const ambient = 0.15; // 环境光强度

    var y: u32 = 0;
    while (y < height) : (y += 1) {
        var x: u32 = 0;
        while (x < width) : (x += 1) {
            const pixel_x = @as(f32, @floatFromInt(x));
            const pixel_y = @as(f32, @floatFromInt(y));
            const pixel_offset = (y * width + x) * 4;

            // 读取底图颜色
            const base_r = @as(f32, @floatFromInt(background_buffer[pixel_offset + 0]));
            const base_g = @as(f32, @floatFromInt(background_buffer[pixel_offset + 1]));
            const base_b = @as(f32, @floatFromInt(background_buffer[pixel_offset + 2]));

            var light_intensity: f32 = ambient;

            var i: u32 = 0;
            while (i < num_lights) : (i += 1) {
                const light_offset = i * 8;
                const light_x = lights_ptr[light_offset + 0];
                const light_y = lights_ptr[light_offset + 1];
                const light_z = lights_ptr[light_offset + 2];
                const intensity = lights_ptr[light_offset + 3];
                const radius = lights_ptr[light_offset + 7];

                const dx = pixel_x - light_x;
                const dy = pixel_y - light_y;
                const dz = 0.0 - light_z;
                const dist = @sqrt(dx * dx + dy * dy + dz * dz);

                if (dist < radius) {
                    const norm_dist = dist / radius;
                    const falloff = (1.0 - norm_dist * norm_dist) * (intensity / 100.0);
                    light_intensity += falloff;
                }
            }

            const clamped_intensity = @min(1.5, @max(ambient, light_intensity));
            const final_r = @min(255.0, base_r * clamped_intensity);
            const final_g = @min(255.0, base_g * clamped_intensity);
            const final_b = @min(255.0, base_b * clamped_intensity);

            pixel_ptr[pixel_offset + 0] = @intFromFloat(final_r);
            pixel_ptr[pixel_offset + 1] = @intFromFloat(final_g);
            pixel_ptr[pixel_offset + 2] = @intFromFloat(final_b);
            pixel_ptr[pixel_offset + 3] = 255;
        }
    }
}
