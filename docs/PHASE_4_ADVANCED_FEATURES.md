
# AutoZig Phase 4.2+ Advanced Features

本文档介绍 AutoZig Phase 4.2+ 实现的高级优化功能。

## 目录

1. [零拷贝 Buffer 传递 (Phase 4.2)](#零拷贝-buffer-传递)
2. [编译时 SIMD 优化检测 (Phase 4.2)](#编译时-simd-优化检测)
3. [Async Zig 函数支持 (Phase 4.3)](#async-zig-函数支持)
4. [Zig 泛型映射 (Phase 4.4)](#zig-泛型映射)

---

## 零拷贝 Buffer 传递

### 概述

零拷贝技术允许 Zig 和 Rust 之间直接共享内存，避免数据序列化/反序列化开销。适用于大数据量传输（>1MB）场景。

### 核心API

```rust
use autozig::zero_copy::{ZeroCopyBuffer, RawVec};

// 从 Zig 生成的数据创建零拷贝 buffer（安全API）
let buffer: ZeroCopyBuffer<i32> = ZeroCopyBuffer::from_zig_vec(raw_vec);

// 转换为 Rust Vec（获取所有权）
let vec: Vec<i32> = buffer.into_vec();

// 或者获取不可变引用
let slice: &[i32] = buffer.as_slice();
```

### Zig 侧实现

```zig
const std = @import("std");

// RawVec 结构与 Rust 兼容
fn RawVec(comptime T: type) type {
    return extern struct {
        ptr: [*]T,
        len: usize,
        cap: usize,
        _phantom: u8 = 0,
    };
}

// 生成数据并返回 RawVec
export fn generate_data(size: usize) RawVec(i32) {
    const allocator = std.heap.c_allocator;
    const data = allocator.alloc(i32, size) catch return .{
        .ptr = undefined,
        .len = 0,
        .cap = 0,
    };
    
    // 填充数据...
    
    return RawVec(i32){
        .ptr = data.ptr,
        .len = data.len,
        .cap = data.len,
    };
}
```

### 性能优势

- **速度提升**: 1.5x - 2.5x（根据数据大小）
- **内存效率**: 零额外分配
- **适用场景**: 
  - 大型数组传输（>1MB）
  - 图像/视频数据
  - 数值计算结果
  - 流式数据处理

### 安全保证

- ✅ **主代码无 unsafe**: 用户代码完全安全
- ✅ **自动内存管理**: Drop 时自动释放
- ✅ **类型安全**: 泛型支持所有 POD 类型
- ✅ **所有权语义**: 遵循 Rust 所有权规则

### 示例代码

查看完整示例: [`examples/zero_copy`](../examples/zero_copy)

```bash
cd examples/zero_copy && cargo run
```

---

## 编译时 SIMD 优化检测

### 概述

在构建时自动检测 CPU 特性（AVX2/SSE4/NEON），并为 Zig 代码生成相应的优化标志。

### 支持的 SIMD 特性

#### x86_64 架构
- **SSE2**: x86_64 基准特性
- **SSE4.2**: 字符串处理优化
- **AVX**: 256-bit 向量运算
- **AVX2**: 整数向量运算
- **AVX-512**: 512-bit 向量运算

#### ARM 架构
- **NEON**: ARM SIMD 扩展

### 使用方法

#### 1. 在 build.rs 中检测

```rust
fn main() {
    // 检测并报告 SIMD 配置
    let simd_config = autozig_build::detect_and_report();
    
    println!("cargo:warning=Detected SIMD: {}", simd_config.description);
    println!("cargo:warning=Zig will use: {}", simd_config.as_zig_flag());
    
    // 构建 Zig 代码
    autozig_build::build("src").expect("Failed to build Zig code");
}
```

#### 2. Zig 代码自动优化

```zig
const std = @import("std");

// Vector 运算自动使用 SIMD 指令
export fn vector_add_f32(a: [*]const f32, b: [*]const f32, result: [*]f32, len: usize) void {
    const vec_size = 4;
    var i: usize = 0;
    
    // 向量化循环
    while (i + vec_size <= len) : (i += vec_size) {
        const vec_a: @Vector(vec_size, f32) = a[i..][0..vec_size].*;
        const vec_b: @Vector(vec_size, f32) = b[i..][0..vec_size].*;
        const vec_result = vec_a + vec_b; // 自动使用 SIMD
        
        const result_array: [vec_size]f32 = vec_result;
        @memcpy(result[i..][0..vec_size], &result_array);
    }
    
    // 标量余数
    while (i < len) : (i += 1) {
        result[i] = a[i] + b[i];
    }
}
```

### 性能提升

| 操作 | 标量 | SSE2 | AVX2 | AVX-512 |
|------|------|------|------|---------|
| 向量加法 | 1x | 4x | 8x | 16x |
| 点积 | 1x | 4x | 8x | 16x |
| 矩阵乘法 | 1x | 3x | 6x | 12x |

### CPU 特性检测

#### 编译时检测
```rust
// build.rs 中自动检测
let config = autozig_build::detect_and_report();
println!("SIMD Level: {}", config.description);
```

#### 运行时检测
```zig
export fn get_simd_features() u32 {
    var features: u32 = 0;
    const cpu = builtin.cpu;
    
    if (cpu.arch == .x86_64) {
        if (std.Target.x86.featureSetHas(cpu.features, .sse2)) {
            features |= 0x01; // SSE2
        }
        if (std.Target.x86.featureSetHas(cpu.features, .avx2)) {
            features |= 0x08; // AVX2
        }
    }
    
    return features;
}
```

### 示例代码

查看完整示例: [`examples/simd_detect`](../examples/simd_detect)

```bash
cd examples/simd_detect && cargo run
```

### 优化建议

1. **数据对齐**: 使用 16/32 字节对齐以获得最佳性能
2. **批量处理**: 一次处理多个元素（4/8/16 个）
3. **避免分支**: SIMD 代码中尽量避免条件分支
4. **向量大小**: 根据目标 CPU 选择合适的向量大小

---

## Async Zig 函数支持

### 概述

AutoZig 支持将 Zig 同步函数自动包装为 Rust async 函数，使用 `tokio::spawn_blocking` 模式在线程池中执行。

### 设计模式

- **Rust 侧**: 提供 async/await 接口
- **Zig 侧**: 实现同步函数（无需 Zig async）
- **执行模式**: 线程池卸载（thread pool offload）

### 使用方法

#### 1. 定义 async 函数

```rust
use autozig::include_zig;

include_zig!("src/async_impl.zig", {
    // 声明为 async 函数
    async fn heavy_computation(data: i32) -> i32;
    async fn process_data(input: &[u8]) -> usize;
    async fn query_database(id: i32) -> i32;
});
```

#### 2. Zig 侧同步实现

```zig
const std = @import("std");

// 同步实现（无需 Zig async/await）
export fn heavy_computation(data: i32) i32 {
    // CPU 密集型计算
    var result: i32 = data;
    var i: i32 = 0;
    while (i < 1000000) : (i += 1) {
        result = @addWithOverflow(result, 1)[0];
        result = @subWithOverflow(result, 1)[0];
    }
    return result * 2;
}

export fn process_data(input_ptr: [*]const u8, input_len: usize) usize {
    var sum: usize = 0;
    var i: usize = 0;
    while (i < input_len) : (i += 1) {
        sum +%= input_ptr[i];
    }
    return sum;
}
```

#### 3. Rust 侧使用

```rust
#[tokio::main]
async fn main() {
    // 单个异步调用
    let result = heavy_computation(42).await;
    println!("Result: {}", result);
    
    // 并发执行
    let tasks = vec![
        tokio::spawn(async { heavy_computation(10).await }),
        tokio::spawn(async { heavy_computation(20).await }),
        tokio::spawn(async { heavy_computation(30).await }),
    ];
    
    let results: Vec<i32> = futures::future::join_all(tasks)
        .await
        .into_iter()
        .map(|r| r.unwrap())
        .collect();
}
```

### 适用场景

- ✅ CPU 密集型计算
- ✅ I/O 阻塞操作
- ✅ 数据库查询
- ✅ 文件处理
- ❌ 需要真正异步 I/O（使用 tokio 直接实现）

### 性能特点

- **线程池**: 使用 tokio 的 blocking 线程池
- **开销**: ~50-100µs 线程切换开销
- **并发**: 支持高并发异步任务
- **隔离**: 阻塞操作不影响异步运行时

### 示例代码

查看完整示例: [`examples/async`](../examples/async)

```bash
cd examples/async && cargo run
```

---

## Zig 泛型映射

### 概述

AutoZig 支持 Rust 泛型函数的单态化（monomorphization），为每个具体类型生成独立的 Zig 函数绑定。

### 使用方法

#### 1. 定义泛型函数

```rust
use autozig::autozig;

autozig! {
    const std = @import("std");
    
    // Zig 实现 - 为每个类型生成独立函数
    export fn sum_i32(data_ptr: [*]const i32, data_len: usize) i32 {
        var total: i32 = 0;
        var i: usize = 0;
        while (i < data_len) : (i += 1) {
            total += data_ptr[i];
        }
        return total;
    }
    
    export fn sum_f64(data_ptr: [*]const f64, data_len: usize) f64 {
        var total: f64 = 0.0;
        var i: usize = 0;
        while (i < data_len) : (i += 1) {
            total += data_ptr[i];
        }
        return total;
    }
    
    ---
    
    // Rust 泛型签名 - 指定单态化类型
    #[monomorphize(i32, f64, u64)]
    fn sum<T>(data: &[T]) -> T;
}
```

#### 2. 使用泛型函数

```rust
fn main() {
    // 自动选择 sum_i32
    let integers = vec![1, 2, 3, 4, 5];
    let int_sum = sum_i32(&integers);
    
    // 自动选择 sum_f64
    let floats = vec![1.5, 2.5, 3.5];
    let float_sum = sum_f64(&floats);
    
    // 自动选择 sum_u64
    let unsigneds = vec![100u64, 200, 300];
    let unsigned_sum = sum_u64(&unsigneds);
}
```

### 单态化模式

#### 模式 1: 手动单态化（当前实现）

```rust
// 为每个类型手动实现
export fn sum_i32(...) i32 { ... }
export fn sum_f64(...) f64 { ... }

// Rust 侧分别调用
#[monomorphize(i32, f64)]
fn sum<T>(data: &[T]) -> T;
```

#### 模式 2: Zig comptime 泛型（未来）

```zig
// Zig 侧真正的泛型函数
fn sum(comptime T: type, data: []const T) T {
    var total: T = 0;
    for (data) |item| {
        total += item;
    }
    return total;
}

// AutoZig 自动生成单态化版本
export fn sum_i32(data_ptr: [*]const i32, data_len: usize) i32 {
    return sum(i32, data_ptr[0..data_len]);
}
```

### 