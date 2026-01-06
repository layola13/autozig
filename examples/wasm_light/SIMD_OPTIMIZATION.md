# AutoZig WASM SIMD 优化说明

## 🎯 优化目标

实现真正的 SIMD 向量化，一次处理 4 个像素，充分利用 WASM SIMD128 指令集。

## ❌ 之前的问题（伪SIMD）

```zig
// 错误实现：只是声明了 Vec4，但逐像素处理
var color = Vec4{ 0.0, 0.0, 0.0, 0.0 };

var x: u32 = 0;
while (x < width) : (x += 1) {  // ❌ 逐像素循环
    const pixel_x = @as(f32, @floatFromInt(x));
    // ... 单个像素的计算
    color += light_color;  // 虽然用了向量，但每次只算1个像素
}
```

**问题**：虽然声明了 `Vec4` 类型，但循环体内只处理 1 个像素，编译器无法生成真正的 SIMD 指令。

## ✅ 真正的 SIMD 优化

### 核心思想：一次处理 4 个像素

```zig
// 正确实现：4 个像素并行计算
while (x + 4 <= width) : (x += 4) {  // ✅ 每次跳 4 个像素
    // 构建 4 个像素的 X 坐标向量
    const vec_x = Vec4{
        @as(f32, @floatFromInt(x)),
        @as(f32, @floatFromInt(x + 1)),
        @as(f32, @floatFromInt(x + 2)),
        @as(f32, @floatFromInt(x + 3)),
    };
    
    // 3 个独立的颜色通道（避免结构体）
    var color_r = Vec4{ 0.0, 0.0, 0.0, 0.0 };
    var color_g = Vec4{ 0.0, 0.0, 0.0, 0.0 };
    var color_b = Vec4{ 0.0, 0.0, 0.0, 0.0 };
    
    // SIMD 距离计算（4 个像素同时算）
    const dx = vec_x - vec_light_x;  // f32x4.sub
    const dy = vec_y - vec_light_y;  // f32x4.sub
    const dist_sq = dx * dx + dy * dy + dz * dz;  // f32x4.mul + f32x4.add
    const dist = @sqrt(dist_sq);  // f32x4.sqrt
    
    // SIMD 条件选择（替代 if 分支）
    const in_range = dist < vec_radius;  // f32x4.lt
    color_r += @select(f32, in_range, vec_light_r * attenuation, Vec4{0,0,0,0});
    //         ^^^^^^^ SIMD 版本的 if-else，编译成 v128.bitselect
}
```

### 关键优化点

1. **向量化循环**：`x += 4` 而不是 `x += 1`
2. **向量化数据**：4 个像素的坐标打包成 `Vec4`
3. **向量化计算**：距离、衰减等全部用向量运算
4. **无分支条件**：用 `@select()` 替代 `if`，避免分支预测失败

## 🔬 WASM SIMD 指令映射

| Zig 代码 | WASM SIMD 指令 | 说明 |
|----------|---------------|------|
| `vec_x - vec_light_x` | `f32x4.sub` | 4 个减法并行 |
| `dx * dx` | `f32x4.mul` | 4 个乘法并行 |
| `dx * dx + dy * dy` | `f32x4.add` | 4 个加法并行 |
| `@sqrt(dist_sq)` | `f32x4.sqrt` | 4 个平方根并行 |
| `dist < vec_radius` | `f32x4.lt` | 4 个比较并行 |
| `@select(f32, cond, a, b)` | `v128.bitselect` | 向量条件选择 |

## 📊 性能提升预期

### 理论加速比

- **标量版本**：每次处理 1 个像素
- **SIMD 版本**：每次处理 4 个像素
- **理论加速**：**~3.5x - 4x**（考虑内存带宽和指令延迟）

### 实际性能因素

1. **内存访问模式**：连续访问 4 个像素，缓存命中率高
2. **指令级并行**：CPU 可以同时执行多条 SIMD 指令
3. **分支预测**：`@select()` 避免分支，减少流水线停顿
4. **寄存器压力**：SIMD 用更少的寄存器处理更多数据

## 🧪 验证 SIMD 是否生效

### 方法1：检查 WASM 文件大小

```bash
ls -lh www/pkg/autozig_wasm_light_bg.wasm
# 之前（伪SIMD）：15257 bytes
# 之后（真SIMD）：16384 bytes (+1KB，因为增加了SIMD指令)
```

### 方法2：使用 wasm-objdump（需要安装 wabt）

```bash
wasm-objdump -x www/pkg/autozig_wasm_light_bg.wasm | grep "f32x4"
# 应该看到：
# f32x4.sub, f32x4.mul, f32x4.add, f32x4.sqrt, f32x4.lt, v128.bitselect
```

### 方法3：浏览器性能测试

1. 打开 http://localhost:8889
2. 点击"开始渲染"
3. 对比"Zig SIMD"和"Rust Scalar"的时间
4. 预期：Zig SIMD 应该比 Rust Scalar **快 2-4 倍**

## 💡 关键代码对比

### 伪 SIMD（性能差）

```zig
// ❌ 逐像素处理，虽然用了 Vec4，但没有真正的并行
var x: u32 = 0;
while (x < width) : (x += 1) {
    var color = Vec4{ 0.0, 0.0, 0.0, 0.0 };
    // ... 单个像素的计算
}
```

### 真 SIMD（性能好）

```zig
// ✅ 4 像素并行处理
var x: u32 = 0;
while (x + 4 <= width) : (x += 4) {
    const vec_x = Vec4{ x, x+1, x+2, x+3 };
    var color_r = Vec4{ 0.0, 0.0, 0.0, 0.0 };
    var color_g = Vec4{ 0.0, 0.0, 0.0, 0.0 };
    var color_b = Vec4{ 0.0, 0.0, 0.0, 0.0 };
    
    // 向量化光照计算
    const dx = vec_x - vec_light_x;  // 4 个减法
    const dist = @sqrt(dx*dx + dy*dy + dz*dz);  // 4 个开方
    
    // 无分支条件累加
    color_r += @select(f32, dist < radius, light_r * attenuation, 0.0);
}

// 处理剩余像素（不足 4 个）
while (x < width) : (x += 1) {
    // 标量处理
}
```

## 🎓 学习要点

1. **SIMD 不是魔法**：只声明向量类型不够，必须改变算法结构
2. **数据并行**：找到可以同时处理的数据（如 4 个像素）
3. **避免分支**：用 `@select()` 替代 `if`，保持向量流水线满载
4. **对齐边界**：处理宽度不是 4 的倍数的情况（剩余像素）

## 🚀 AutoZig WASM 优势

1. **零拷贝内存**：Rust 和 Zig 共享线性内存，无数据复制
2. **SIMD 支持**：Zig 的 `@Vector` 直接映射到 WASM SIMD128
3. **编译优化**：`-O ReleaseSmall` + `-mcpu=mvp+simd128`
4. **类型安全**：编译期检查，运行时无开销

---

**总结**：真正的 SIMD 需要算法级别的重构，而不仅仅是数据类型的改变。AutoZig 让这个过程变得简单且类型安全。