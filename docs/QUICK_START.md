# AutoZig 快速开始指南

## 🚀 5 分钟上手 AutoZig

### 前置要求

```bash
# 1. 安装 Rust (如果未安装)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 2. 安装 Zig (0.11.0 或更高版本)
# 方法 1: 从官网下载 https://ziglang.org/download/
# 方法 2: 使用包管理器
brew install zig  # macOS
snap install zig --classic --beta  # Linux
```

### 第一个项目

#### 1. 创建新项目

```bash
cargo new hello-autozig
cd hello-autozig
```

#### 2. 添加依赖

编辑 `Cargo.toml`:

```toml
[dependencies]
autozig = { path = "../autozig" }

[build-dependencies]
autozig-gen-build = { path = "../autozig/gen-build" }
```

#### 3. 创建 `build.rs`

```rust
fn main() {
    autozig_gen_build::builder()
        .build();
}
```

#### 4. 编写代码

编辑 `src/main.rs`:

```rust
use autozig::prelude::*;

autozig! {
    const std = @import("std");
    
    export fn add(a: i32, b: i32) i32 {
        return a + b;
    }
    
    export fn greet(name_ptr: [*]const u8, name_len: usize) void {
        const name = name_ptr[0..name_len];
        std.debug.print("Hello, {s}!\n", .{name});
    }
    
    ---
    
    fn add(a: i32, b: i32) -> i32;
    fn greet(name: &str);
}

fn main() {
    // 调用 Zig 函数
    let result = add(10, 20);
    println!("10 + 20 = {}", result);
    
    greet("AutoZig");
}
```

#### 5. 运行

```bash
cargo run
```

**输出**:
```
10 + 20 = 30
Hello, AutoZig!
```

---

## 📚 学习路径

### 初级（1-2 小时）

1. **[demo](demo/)** - 基础函数调用
   - 学习 `autozig!` 宏的基本用法
   - 理解 `---` 分隔符的作用
   - 掌握基本类型映射

2. **[examples/structs](examples/structs/)** - 结构体传递
   - 学习 `#[repr(C)]` 的使用
   - 理解 Zig `extern struct`
   - 掌握结构体在 FFI 边界的传递

3. **[examples/enums](examples/enums/)** - 枚举类型
   - 学习 `#[repr(C)]` 枚举
   - 理解 Zig 枚举映射
   - 掌握枚举值的传递

### 中级（2-4 小时）

4. **[examples/smart_lowering](examples/smart_lowering/)** - 智能降级
   - 学习 `&str` 和 `&[T]` 的自动转换
   - 理解智能降级的原理
   - 掌握高级类型的使用技巧

5. **[examples/external](examples/external/)** - 外部文件支持
   - 学习 `include_zig!` 宏
   - 理解多文件项目组织
   - 掌握 Zig 测试集成

6. **[examples/complex](examples/complex/)** - 复杂类型组合
   - 学习嵌套结构体
   - 理解多类型组合
   - 掌握实际项目的架构

### 高级（4-8 小时）

7. **[examples/trait_hasher](examples/trait_hasher/)** - 无状态 Trait
   - 学习 Trait 到 Zig 的映射
   - 理解无状态 Trait 模式
   - 掌握接口抽象技巧

8. **[examples/trait_calculator](examples/trait_calculator/)** - 有状态 Trait
   - 学习 Opaque Pointer 模式
   - 理解生命周期管理
   - 掌握有状态对象的 FFI 实现

9. **[examples/security_tests](examples/security_tests/)** - 安全测试
   - 学习 FFI 安全最佳实践
   - 理解常见漏洞模式
   - 掌握安全测试方法

---

## 🎯 常用模式速查

### 1. 基本函数调用

```rust
autozig! {
    export fn compute(x: f64) f64 {
        return x * x;
    }
    ---
    fn compute(x: f64) -> f64;
}
```

### 2. 传递切片

```rust
autozig! {
    export fn sum_array(ptr: [*]const i32, len: usize) i32 {
        const slice = ptr[0..len];
        var total: i32 = 0;
        for (slice) |val| {
            total += val;
        }
        return total;
    }
    ---
    fn sum_array(data: &[i32]) -> i32;
}
```

### 3. 传递字符串

```rust
autozig! {
    const std = @import("std");
    
    export fn print_message(ptr: [*]const u8, len: usize) void {
        const msg = ptr[0..len];
        std.debug.print("{s}\n", .{msg});
    }
    ---
    fn print_message(msg: &str);
}
```

### 4. 传递结构体

```rust
#[repr(C)]
struct Point {
    x: f64,
    y: f64,
}

autozig! {
    const Point = extern struct {
        x: f64,
        y: f64,
    };
    
    export fn distance(p: Point) f64 {
        return @sqrt(p.x * p.x + p.y * p.y);
    }
    ---
    fn distance(p: Point) -> f64;
}
```

### 5. 使用外部文件

```rust
include_zig! {
    file: "src/math.zig",
    functions: [
        fn add(a: i32, b: i32) -> i32;
        fn multiply(a: i32, b: i32) -> i32;
    ]
}
```

---

## 🔧 构建配置

### 基础配置

```rust
// build.rs
fn main() {
    autozig_gen_build::builder()
        .build();
}
```

### 高级配置

```rust
// build.rs
fn main() {
    autozig_gen_build::builder()
        .zig_version("0.11.0")           // 指定 Zig 版本
        .optimization(autozig_gen_build::OptimizationLevel::ReleaseFast)
        .add_include_path("src/zig")     // 添加 Zig 搜索路径
        .add_lib_path("/usr/local/lib")  // 添加库搜索路径
        .build();
}
```

---

## 🐛 常见问题

### Q1: 编译错误 "zig: command not found"

**解决方案**: 安装 Zig 编译器

```bash
# macOS
brew install zig

# Linux
snap install zig --classic --beta

# 或从官网下载: https://ziglang.org/download/
```

### Q2: 类型不匹配错误

**问题**:
```
error: expected type 'i32', found 'u32'
```

**解决方案**: 检查 Rust 和 Zig 类型是否匹配

```rust
// ❌ 错误
fn bad(x: u32) -> i32;  // Rust 侧

export fn bad(x: i32) i32 { ... }  // Zig 侧

// ✅ 正确
fn good(x: u32) -> i32;  // Rust 侧

export fn good(x: u32) i32 { ... }  // Zig 侧
```

### Q3: 结构体字段错位

**问题**: 结构体传递后字段值不正确

**解决方案**: 确保使用 `#[repr(C)]` 和 `extern struct`

```rust
// Rust 侧
#[repr(C)]  // ✅ 必须！
struct Point {
    x: f64,
    y: f64,
}

// Zig 侧
const Point = extern struct {  // ✅ 必须 extern！
    x: f64,
    y: f64,
};
```

### Q4: 增量编译不生效

**问题**: 每次都重新编译 Zig 代码

**解决方案**: 确保没有修改 Zig 代码，增量编译会自动使用缓存

```bash
# 首次编译
cargo build  # ~3 秒

# 增量编译（Zig 代码未变）
cargo build  # ~0.2 秒
```

### Q5: 如何调试 Zig 代码？

**方法 1**: 使用 `std.debug.print`

```zig
export fn debug_example(x: i32) i32 {
    std.debug.print("Input: {}\n", .{x});
    const result = x * 2;
    std.debug.print("Result: {}\n", .{result});
    return result;
}
```

**方法 2**: 使用 GDB/LLDB

```bash
cargo build
gdb target/debug/your-app
```

---

## 📖 进阶资源

### 官方文档

- [README.md](README.md) - 项目介绍
- [DESIGN.md](DESIGN.md) - 架构设计
- [ARCHITECTURE.md](ARCHITECTURE.md) - 系统架构
- [TRAIT_SUPPORT.md](TRAIT_SUPPORT.md) - Trait 支持
- [SECURITY_BEST_PRACTICES.md](SECURITY_BEST_PRACTICES.md) - 安全指南

### 示例项目

完整示例列表: [EXAMPLES_GUIDE.md](EXAMPLES_GUIDE.md)

### 外部资源

- [Rust FFI 文档](https://doc.rust-lang.org/nomicon/ffi.html)
- [Zig 官方文档](https://ziglang.org/documentation/master/)
- [Zig C 互操作](https://ziglang.org/documentation/master/#C)

---

## 💡 最佳实践

### 1. 始终使用 `#[repr(C)]`

```rust
// ✅ 正确
#[repr(C)]
struct Data {
    field1: i32,
    field2: f64,
}

// ❌ 错误
struct Data {  // 布局不确定！
    field1: i32,
    field2: f64,
}
```

### 2. 使用智能降级

```rust
// ✅ 推荐：让 AutoZig 自动处理
fn process_text(text: &str) -> usize;

// ❌ 不推荐：手动传递指针和长度
fn process_text_manual(ptr: *const u8, len: usize) -> usize;
```

### 3. 边界检查

```zig
// ✅ 推荐：使用切片自动检查
export fn safe_access(ptr: [*]const u8, len: usize, index: usize) i32 {
    if (index >= len) return -1;
    const slice = ptr[0..len];
    return slice[index];
}

// ❌ 危险：直接使用指针
export fn unsafe_access(ptr: [*]const u8, index: usize) i32 {
    return ptr[index];  // 可能越界！
}
```

### 4. 错误处理

```zig
// ✅ 推荐：返回错误码
export fn fallible_op(x: i32) i32 {
    if (x < 0) return -1;  // 错误码
    return x * 2;
}

// ❌ 危险：直接 panic
export fn panicking_op(x: i32) i32 {
    if (x < 0) @panic("Invalid input");  // 会 abort 进程！
    return x * 2;
}
```

---

## 🎉 开始你的 AutoZig 之旅！

1. 从 **demo** 示例开始
2. 按学习路径逐步深入
3. 参考最佳实践编写代码
4. 遇到问题查看常见问题解答

**祝你使用愉快！** 🚀