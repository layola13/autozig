# AutoZig 使用教程：如何将 Zig 代码集成到 Rust 项目

> 本教程将手把手教你如何使用 AutoZig 框架，在 Rust 项目中集成 Zig 代码，实现安全、高效的跨语言互操作。

## 📋 目录

1. [快速开始](#快速开始)
2. [核心概念](#核心概念)
3. [基础用法](#基础用法)
4. [高级特性](#高级特性)
5. [最佳实践](#最佳实践)
6. [常见问题](#常见问题)
7. [完整示例](#完整示例)

---

## 🚀 快速开始

### 前置要求

| 工具 | 版本要求 | 说明 |
|-----|---------|------|
| **Rust** | 1.77+ | 需要支持 workspace 特性 |
| **Zig** | 0.15+ | 必须在系统 PATH 中 |
| **Cargo** | 最新版 | Rust 包管理器 |

验证安装：
```bash
rustc --version  # 应显示 1.77 或更高
zig version      # 应显示 0.15 或更高
```

### 五步快速入门

#### 第一步：创建项目

```bash
cargo new my-autozig-app
cd my-autozig-app
```

#### 第二步：添加依赖

编辑 `Cargo.toml`：

```toml
[dependencies]
autozig = "0.1"

[build-dependencies]
autozig-build = "0.1"
anyhow = "1.0"
```

#### 第三步：创建 build.rs

```rust
fn main() -> anyhow::Result<()> {
    autozig_build::build("src")?;
    println!("cargo:rerun-if-changed=src/");
    Ok(())
}
```

#### 第四步：编写代码

编辑 `src/main.rs`：

```rust
use autozig::autozig;

autozig! {
    // Zig 代码
    export fn add(a: i32, b: i32) i32 {
        return a + b;
    }
    
    ---
    
    // Rust 签名
    fn add(a: i32, b: i32) -> i32;
}

fn main() {
    println!("10 + 32 = {}", add(10, 32));
}
```

#### 第五步：运行

```bash
cargo run
```

---

## 📚 核心概念

### autozig! 宏结构

```rust
autozig! {
    // Zig 代码部分
    export fn function_name(...) return_type { ... }
    
    ---  // 分隔符
    
    // Rust 签名部分（可选）
    fn function_name(...) -> return_type;
}
```

### 类型映射

| Rust 类型 | Zig 类型 | 说明 |
|-----------|----------|------|
| `i32, u32, i64, u64` | `i32, u32, i64, u64` | 整数 |
| `f32, f64` | `f32, f64` | 浮点数 |
| `bool` | `bool` | 布尔值 |
| `&str` | `[*]const u8, usize` | 字符串（自动转换）|
| `&[T]` | `[*]const T, usize` | 切片（自动转换）|
| `&mut [T]` | `[*]T, usize` | 可变切片（自动转换）|

### 编译流程

```
解析阶段 → 构建阶段 → 宏展开阶段
   ↓          ↓           ↓
提取代码   编译 Zig    生成包装器
```

---

## 🎯 基础用法

### 1. 数学运算

```rust
use autozig::autozig;

autozig! {
    export fn multiply(a: f64, b: f64) f64 {
        return a * b;
    }
    
    ---
    
    fn multiply(a: f64, b: f64) -> f64;
}

fn main() {
    println!("3.14 * 2.0 = {}", multiply(3.14, 2.0));
}
```

### 2. 字符串处理

```rust
use autozig::autozig;

autozig! {
    const std = @import("std");
    
    export fn print_string(ptr: [*]const u8, len: usize) void {
        const s = ptr[0..len];
        std.debug.print("内容: {s}\n", .{s});
    }
    
    ---
    
    fn print_string(s: &str);
}

fn main() {
    print_string("Hello, AutoZig!");
}
```

### 3. 数组操作（定长和可变）

AutoZig 支持多种数组类型：

**可变长度切片**：
```rust
use autozig::autozig;

autozig! {
    export fn sum_array(ptr: [*]const i32, len: usize) i32 {
        const arr = ptr[0..len];
        var sum: i32 = 0;
        for (arr) |val| {
            sum += val;
        }
        return sum;
    }
    
    ---
    
    fn sum_array(arr: &[i32]) -> i32;  // 可变长度
}

fn main() {
    let nums = vec![1, 2, 3, 4, 5];
    println!("总和: {}", sum_array(&nums));
}
```

**定长数组**：
```rust
autozig! {
    export fn process_fixed(arr: [4]f64) f64 {
        var sum: f64 = 0;
        for (arr) |val| {
            sum += val;
        }
        return sum;
    }
    
    ---
    
    fn process_fixed(arr: [f64; 4]) -> f64;  // 定长数组
}

fn main() {
    let data = [1.0, 2.0, 3.0, 4.0];
    println!("总和: {}", process_fixed(data));
}
```

**返回数组**：
```rust
autozig! {
    // 返回定长数组
    export fn create_array() [3]i32 {
        return [3]i32{ 1, 2, 3 };
    }
    
    // 返回可变数组（通过输出参数）
    export fn fill_array(ptr: [*]i32, len: usize) void {
        var i: usize = 0;
        while (i < len) : (i += 1) {
            ptr[i] = @intCast(i32, i * 2);
        }
    }
    
    ---
    
    fn create_array() -> [i32; 3];
    fn fill_array(arr: &mut [i32]);
}

fn main() {
    let fixed = create_array();
    println!("固定数组: {:?}", fixed);
    
    let mut dynamic = vec![0; 5];
    fill_array(&mut dynamic);
    println!("动态数组: {:?}", dynamic);
}
```

### 4. 结构体

```rust
use autozig::autozig;

autozig! {
    pub const Point = extern struct {
        x: f64,
        y: f64,
    };
    
    export fn distance(p1: Point, p2: Point) f64 {
        const dx = p1.x - p2.x;
        const dy = p1.y - p2.y;
        return @sqrt(dx * dx + dy * dy);
    }
    
    ---
    
    #[repr(C)]
    #[derive(Debug, Clone, Copy)]
    struct Point {
        x: f64,
        y: f64,
    }
    
    fn distance(p1: Point, p2: Point) -> f64;
}

fn main() {
    let p1 = Point { x: 0.0, y: 0.0 };
    let p2 = Point { x: 3.0, y: 4.0 };
    println!("距离: {}", distance(p1, p2));
}
```

---

## 🚀 高级特性

### 1. 外部文件

**项目结构**：
```
my-project/
├── src/
│   ├── main.rs
│   └── math.zig
```

**math.zig**:
```zig
export fn factorial(n: u32) u64 {
    if (n <= 1) return 1;
    return n * factorial(n - 1);
}
```

**main.rs**:
```rust
use autozig::include_zig;

include_zig!("src/math.zig", {
    fn factorial(n: u32) -> u64;
});

fn main() {
    println!("5! = {}", factorial(5));
}
```

### 2. 异步支持

```rust
use autozig::include_zig;

include_zig!("src/compute.zig", {
    async fn heavy_computation(x: i32) -> i32;
});

#[tokio::main]
async fn main() {
    let result = heavy_computation(42).await;
    println!("结果: {}", result);
}
```

### 3. 编译模式选择

AutoZig 支持三种编译模式：

| 模式 | 说明 | 适用场景 |
|-----|------|---------|
| **ModularBuildZig** | 使用 build.zig（默认） | 推荐，支持增量编译 |
| **ModularImport** | 模块化导入 | Zig < 0.15.2 |
| **Merged** | 合并所有文件 | 简单项目，快速编译 |

**使用默认模式**：
```rust
// build.rs
fn main() {
    autozig_build::build("src").expect("Build failed");
}
```

**切换编译模式**：
```rust
use autozig_build::CompilationMode;

fn main() {
    // 使用 Merged 模式
    autozig_build::build_with_mode("src", CompilationMode::Merged)
        .expect("Build failed");
}
```

### 4. WebAssembly 支持

AutoZig 完全支持 WebAssembly，实现 Zig + Rust 静态链接。

**Cargo.toml 配置**：
```toml
[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
autozig = "0.1"
wasm-bindgen = "0.2"

[profile.release]
opt-level = "s"  # 体积优化
lto = true
```

**Rust + Zig 代码**：
```rust
use wasm_bindgen::prelude::*;
use autozig::autozig;

autozig! {
    // Zig WASM 代码（零拷贝处理）
    export fn invert_colors(ptr: [*]u8, len: usize) void {
        var i: usize = 0;
        while (i < len) : (i += 4) {
            ptr[i] = 255 - ptr[i];       // R
            ptr[i+1] = 255 - ptr[i+1];   // G
            ptr[i+2] = 255 - ptr[i+2];   // B
        }
    }
    
    ---
    
    fn invert_colors(data: &mut [u8]);
}

#[wasm_bindgen]
pub fn apply_filter(mut data: Vec<u8>) -> Vec<u8> {
    invert_colors(&mut data);  // 零拷贝调用
    data
}
```

**构建 WASM**：
```bash
# 添加 WASM 目标
rustup target add wasm32-unknown-unknown

# 安装 wasm-pack
cargo install wasm-pack

# 构建
wasm-pack build --target web
```

**性能优势**：
- ✅ **零拷贝**: 共享线性内存，无数据复制
- ✅ **SIMD 优化**: Zig 自动向量化
- ✅ **高性能**: 比纯 JS 快 3-5倍

### 5. 异步支持

AutoZig 支持异步函数，自动使用 `tokio::spawn_blocking`。

**Zig 代码保持同步**：
```zig
// src/compute.zig
export fn heavy_computation(x: i32) i32 {
    // 正常的同步代码，无需 async/await
    var result: i32 = 0;
    var i: i32 = 0;
    while (i < 1000000) : (i += 1) {
        result += x * i;
    }
    return result;
}
```

**Rust 端使用 async**：
```rust
use autozig::include_zig;

include_zig!("src/compute.zig", {
    async fn heavy_computation(x: i32) -> i32;
});

#[tokio::main]
async fn main() {
    // 自动在线程池执行
    let result = heavy_computation(42).await;
    println!("结果: {}", result);
    
    // 并发执行
    let tasks = vec![
        tokio::spawn(async { heavy_computation(10).await }),
        tokio::spawn(async { heavy_computation(20).await }),
    ];
    
    let results = futures::future::join_all(tasks).await;
    println!("并发结果: {:?}", results);
}
```

**优势**：
- ✅ Zig 代码简单（无需异步语法）
- ✅ Rust 获得异步 API
- ✅ 不阻塞 async runtime

---

## ✅ 最佳实践

### DO ✅

1. **使用 `#[repr(C)]`**
```rust
#[repr(C)]
struct Point { x: f64, y: f64 }
```

2. **使用智能降级**
```rust
fn process(data: &[u8]) -> usize;  // ✅ 自动转换
```

3. **使用 `export` 关键字**
```zig
export fn my_func() void { }  // ✅
```

4. **添加分隔符**
```rust
autozig! {
    export fn add(a: i32, b: i32) i32 { return a + b; }
    ---  // ✅
    fn add(a: i32, b: i32) -> i32;
}
```

### DON'T ❌

1. **忘记 `#[repr(C)]`**
```rust
struct Point { x: f64, y: f64 }  // ❌
```

2. **手动处理指针**
```rust
fn process(ptr: *const u8, len: usize) -> usize;  // ❌
```

3. **忘记 `export`**
```zig
fn my_func() void { }  // ❌
```

---

## ❓ 常见问题

### Q1: 找不到 zig 命令

**解决**：
```bash
# 安装 Zig
brew install zig  # macOS
# 或从 https://ziglang.org 下载

# 验证
zig version
```

### Q2: 链接错误

**原因**：缺少 `export` 关键字

**解决**：
```zig
export fn my_function() void { }  // ✅
```

### Q3: 类型不匹配

**解决**：检查类型映射表，确保 Rust 和 Zig 类型对应

### Q4: 如何调试

**方法 1 - 打印调试**：
```zig
const std = @import("std");
std.debug.print("x = {}\n", .{x});
```

**方法 2 - 查看生成代码**：
```bash
cat target/debug/build/*/out/generated_autozig.zig
```

### Q5: 性能优化

```bash
cargo build --release  # Zig 自动使用 -O ReleaseFast
```

---

## 📖 完整示例

### 示例 1：图像灰度化

```rust
use autozig::autozig;

autozig! {
    export fn grayscale(ptr: [*]u8, len: usize) void {
        var i: usize = 0;
        while (i < len) : (i += 4) {
            const r = @as(f32, @floatFromInt(ptr[i]));
            const g = @as(f32, @floatFromInt(ptr[i + 1]));
            const b = @as(f32, @floatFromInt(ptr[i + 2]));
            const gray = @as(u8, @intFromFloat(
                0.299 * r + 0.587 * g + 0.114 * b
            ));
            ptr[i] = gray;
            ptr[i + 1] = gray;
            ptr[i + 2] = gray;
        }
    }
    
    ---
    
    fn grayscale(pixels: &mut [u8]);
}

fn main() {
    let mut image = vec![255, 0, 0, 255, 0, 255, 0, 255];
    grayscale(&mut image);
    println!("灰度化后: {:?}", image);
}
```

### 示例 2：哈希计算

```rust
use autozig::autozig;

autozig! {
    export fn compute_hash(ptr: [*]const u8, len: usize) u64 {
        const data = ptr[0..len];
        var hash: u64 = 0;
        for (data) |byte| {
            hash = hash *% 31 +% byte;
        }
        return hash;
    }
    
    ---
    
    fn compute_hash(data: &[u8]) -> u64;
}

fn main() {
    let text = b"Hello, World!";
    println!("哈希: {}", compute_hash(text));
}
```

### 示例 3：JSON 解析（与 C 库集成）

**项目结构**：
```
project/
├── src/
│   ├── main.rs
│   ├── json.c      # C 实现
│   └── wrapper.zig  # Zig 包装
```

**build.rs**:
```rust
use autozig_build::Builder;

fn main() {
    Builder::new()
        .with_c_sources(&["src/json.c"])
        .build()
        .expect("构建失败");
}
```

**json.c**:
```c
int parse_json(const char* json) {
    // C 实现
    return 1;
}
```

**wrapper.zig**:
```zig
extern "c" fn parse_json(json: [*:0]const u8) i32;

export fn parse(json_ptr: [*]const u8, len: usize) i32 {
    _ = len;
    return parse_json(json_ptr);
}
```

**main.rs**:
```rust
use autozig::include_zig;

include_zig!("src/wrapper.zig", {
    fn parse(json: &str) -> i32;
});

fn main() {
    let result = parse("{\"key\": \"value\"}");
    println!("解析结果: {}", result);
}
```

---

## 📚 更多资源

### 官方文档

- [README.md](../README.md) - 项目概览
- [QUICKSTART.md](QUICKSTART.md) - 快速开始
- [DESIGN.md](DESIGN.md) - 架构设计
- [示例代码](../examples/) - 15+ 完整示例

### 外部资源

- [Zig 官方文档](https://ziglang.org/documentation/master/)
- [Rust FFI 指南](https://doc.rust-lang.org/nomicon/ffi.html)
- [autocxx 项目](https://github.com/google/autocxx) - 灵感来源

### 示例项目

```bash
# 查看所有示例
cd autozig/examples
ls -la

# 运行单个示例
cd structs
cargo run

# 批量验证
./verify_all.sh
```

---

## 🎓 下一步

现在你已经掌握了 AutoZig 的基础和高级用法，可以：

1. ✅ 在现有项目中集成 Zig 代码
2. ✅ 利用 Zig 的性能优势优化关键路径
3. ✅ 探索 WASM、异步等高级特性
4. ✅ 参与 AutoZig 社区贡献

**祝你编码愉快！** 🚀

---

<div align="center">

**Made with ❤️ by the AutoZig Community**

[⭐ Star on GitHub](https://github.com/layola13/autozig) • [🐛 报告问题](https://github.com/layola13/autozig/issues) • [📖 查看文档](.)

</div>