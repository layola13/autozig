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

            // 🔥 累积彩色光照（RGB独立累加）
            var light_r = Vec4{ 0.0, 0.0, 0.0, 0.0 };
            var light_g = Vec4{ 0.0, 0.0, 0.0, 0.0 };
            var light_b = Vec4{ 0.0, 0.0, 0.0, 0.0 };

            // 遍历所有光源
            var i: u32 = 0;
            while (i < num_lights) : (i += 1) {
                const light_offset = i * 8;
                const light_x = lights_ptr[light_offset + 0];
                const light_y = lights_ptr[light_offset + 1];
                const light_z = lights_ptr[light_offset + 2];
                const intensity = lights_ptr[light_offset + 3];
                const light_color_r = lights_ptr[light_offset + 4]; // 🔥 光源RGB颜色
                const light_color_g = lights_ptr[light_offset + 5];
                const light_color_b = lights_ptr[light_offset + 6];
                const radius = lights_ptr[light_offset + 7];

                // 广播光源坐标和颜色到向量
                const vec_light_x = Vec4{ light_x, light_x, light_x, light_x };
                const vec_light_y = Vec4{ light_y, light_y, light_y, light_y };
                const vec_light_z = Vec4{ light_z, light_z, light_z, light_z };
                const vec_radius = Vec4{ radius, radius, radius, radius };
                const vec_intensity = Vec4{ intensity / 100.0, intensity / 100.0, intensity / 100.0, intensity / 100.0 };
                const vec_color_r = Vec4{ light_color_r, light_color_r, light_color_r, light_color_r };
                const vec_color_g = Vec4{ light_color_g, light_color_g, light_color_g, light_color_g };
                const vec_color_b = Vec4{ light_color_b, light_color_b, light_color_b, light_color_b };

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

                // 🔥 彩色光照贡献（每个RGB通道独立累加）
                light_r += @select(f32, in_range, vec_color_r * falloff, Vec4{ 0.0, 0.0, 0.0, 0.0 });
                light_g += @select(f32, in_range, vec_color_g * falloff, Vec4{ 0.0, 0.0, 0.0, 0.0 });
                light_b += @select(f32, in_range, vec_color_b * falloff, Vec4{ 0.0, 0.0, 0.0, 0.0 });
            }

            // 环境光照亮底图 + 彩色光照混合
            const vec_ambient = Vec4{ ambient, ambient, ambient, ambient };
            const ambient_r = base_r * vec_ambient;
            const ambient_g = base_g * vec_ambient;
            const ambient_b = base_b * vec_ambient;

            // 应用彩色光照到底图（加法混合，允许过曝）
            const final_r = @min(Vec4{ 255.0, 255.0, 255.0, 255.0 }, ambient_r + light_r);
            const final_g = @min(Vec4{ 255.0, 255.0, 255.0, 255.0 }, ambient_g + light_g);
            const final_b = @min(Vec4{ 255.0, 255.0, 255.0, 255.0 }, ambient_b + light_b);

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

            var light_r: f32 = 0.0;
            var light_g: f32 = 0.0;
            var light_b: f32 = 0.0;

            var i: u32 = 0;
            while (i < num_lights) : (i += 1) {
                const light_offset = i * 8;
                const light_x = lights_ptr[light_offset + 0];
                const light_y = lights_ptr[light_offset + 1];
                const light_z = lights_ptr[light_offset + 2];
                const intensity = lights_ptr[light_offset + 3];
                const light_color_r = lights_ptr[light_offset + 4];
                const light_color_g = lights_ptr[light_offset + 5];
                const light_color_b = lights_ptr[light_offset + 6];
                const radius = lights_ptr[light_offset + 7];

                const dx = pixel_x - light_x;
                const dy = pixel_y - light_y;
                const dz = 0.0 - light_z;
                const dist = @sqrt(dx * dx + dy * dy + dz * dz);

                if (dist < radius) {
                    const norm_dist = dist / radius;
                    const falloff = (1.0 - norm_dist * norm_dist) * (intensity / 100.0);
                    light_r += light_color_r * falloff;
                    light_g += light_color_g * falloff;
                    light_b += light_color_b * falloff;
                }
            }

            const final_r = @min(255.0, base_r * ambient + light_r);
            const final_g = @min(255.0, base_g * ambient + light_g);
            const final_b = @min(255.0, base_b * ambient + light_b);

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

            var light_r: f32 = 0.0;
            var light_g: f32 = 0.0;
            var light_b: f32 = 0.0;

            var i: u32 = 0;
            while (i < num_lights) : (i += 1) {
                const light_offset = i * 8;
                const light_x = lights_ptr[light_offset + 0];
                const light_y = lights_ptr[light_offset + 1];
                const light_z = lights_ptr[light_offset + 2];
                const intensity = lights_ptr[light_offset + 3];
                const light_color_r = lights_ptr[light_offset + 4];
                const light_color_g = lights_ptr[light_offset + 5];
                const light_color_b = lights_ptr[light_offset + 6];
                const radius = lights_ptr[light_offset + 7];

                const dx = pixel_x - light_x;
                const dy = pixel_y - light_y;
                const dz = 0.0 - light_z;
                const dist = @sqrt(dx * dx + dy * dy + dz * dz);

                if (dist < radius) {
                    const norm_dist = dist / radius;
                    const falloff = (1.0 - norm_dist * norm_dist) * (intensity / 100.0);
                    light_r += light_color_r * falloff;
                    light_g += light_color_g * falloff;
                    light_b += light_color_b * falloff;
                }
            }

            const final_r = @min(255.0, base_r * ambient + light_r);
            const final_g = @min(255.0, base_g * ambient + light_g);
            const final_b = @min(255.0, base_b * ambient + light_b);

            pixel_ptr[pixel_offset + 0] = @intFromFloat(final_r);
            pixel_ptr[pixel_offset + 1] = @intFromFloat(final_g);
            pixel_ptr[pixel_offset + 2] = @intFromFloat(final_b);
            pixel_ptr[pixel_offset + 3] = 255;
        }
    }
}
