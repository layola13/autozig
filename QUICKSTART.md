# AutoZig 快速开始指南

## 5 分钟上手 AutoZig

### 1. 创建新项目

```bash
cargo new my-autozig-project
cd my-autozig-project
```

### 2. 添加依赖

编辑 `Cargo.toml`：

```toml
[dependencies]
autozig = { path = "../autozig" }

[build-dependencies]
autozig-build = { path = "../autozig/gen/build" }
```

### 3. 创建 build.rs

```rust
fn main() {
    autozig_build::build("src").expect("Failed to build Zig code");
}
```

### 4. 编写代码

编辑 `src/main.rs`：

```rust
use autozig::autozig;

autozig! {
    // Zig 代码
    const std = @import("std");
    
    export fn fibonacci(n: u32) u32 {
        if (n <= 1) return n;
        return fibonacci(n - 1) + fibonacci(n - 2);
    }
    
    export fn greet(name_ptr: [*]const u8, name_len: usize) void {
        const name = name_ptr[0..name_len];
        std.debug.print("Hello, {s}!\n", .{name});
    }
    
    ---
    
    // Rust 签名
    fn fibonacci(n: u32) -> u32;
    fn greet(name: &str);
}

fn main() {
    // 调用 Zig 函数（完全安全，无 unsafe）
    let result = fibonacci(10);
    println!("fibonacci(10) = {}", result);
    
    greet("AutoZig");
}
```

### 5. 运行

```bash
cargo run
```

输出：
```
fibonacci(10) = 55
Hello, AutoZig!
```

## 核心概念

### autozig! 宏结构

```rust
autozig! {
    // 第一部分：Zig 代码
    // 使用 export 导出函数
    export fn my_function(...) ... { ... }
    
    ---  // 分隔符
    
    // 第二部分：Rust 函数签名
    // 定义安全的 Rust 接口
    fn my_function(...) -> ...;
}
```

### 类型映射

| Rust 类型 | Zig 类型 | 说明 |
|-----------|----------|------|
| `i32`, `u32`, `i64`, `u64` | `i32`, `u32`, `i64`, `u64` | 整数类型 |
| `f32`, `f64` | `f32`, `f64` | 浮点类型 |
| `bool` | `bool` | 布尔类型 |
| `&str` | `[*]const u8, usize` | 字符串切片（智能降级） |
| `&[T]` | `[*]const T, usize` | 不可变切片（智能降级） |
| `&mut [T]` | `[*]T, usize` | 可变切片（智能降级） |
| `#[repr(C)] struct` | `extern struct` | C 兼容结构体 |
| `#[repr(C)] enum` | `enum(type)` | C 兼容枚举 |

### 智能降级示例

**自动处理复杂类型，无需手动 unsafe 代码：**

```rust
autozig! {
    export fn process_string(ptr: [*]const u8, len: usize) usize {
        const s = ptr[0..len];
        return s.len;
    }
    
    ---
    
    // Rust 侧使用高级类型
    fn process_string(s: &str) -> usize;
}

fn main() {
    // 直接传递 &str，自动转换为 ptr+len
    let result = process_string("Hello");
    println!("Length: {}", result);
}
```

## 示例项目

### 1. 基础算术（demo/）

```bash
cd autozig/demo
cargo run --release
```

演示：基本函数调用、类型转换

### 2. 结构体操作（examples/structs/）

```bash
cd autozig/examples/structs
cargo run --release
```

演示：自定义结构体、嵌套结构体、指针参数

### 3. 枚举类型（examples/enums/）

```bash
cd autozig/examples/enums
cargo run --release
```

演示：Result 模式、Option 模式、自定义枚举

### 4. 复杂类型（examples/complex/）

```bash
cd autozig/examples/complex
cargo run --release
```

演示：字符串、数组、元组、复杂数据结构

### 5. 智能降级（examples/smart_lowering/）

```bash
cd autozig/examples/smart_lowering
cargo run --release
```

演示：零 unsafe 的高级类型使用

## 常见模式

### 模式 1：结构体传递

```rust
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
    #[derive(Clone, Copy)]
    struct Point {
        x: f64,
        y: f64,
    }
    
    fn distance(p1: Point, p2: Point) -> f64;
}
```

### 模式 2：错误处理

```rust
autozig! {
    pub const Result = enum(i32) {
        Ok = 0,
        Error = -1,
    };
    
    export fn safe_divide(a: i32, b: i32) Result {
        if (b == 0) return Result.Error;
        return Result.Ok;
    }
    
    ---
    
    #[repr(C)]
    enum Result {
        Ok = 0,
        Error = -1,
    }
    
    fn safe_divide(a: i32, b: i32) -> Result;
}

fn main() {
    match safe_divide(10, 0) {
        Result::Ok => println!("Success"),
        Result::Error => println!("Division by zero"),
    }
}
```

### 模式 3：数组处理

```rust
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
    
    fn sum_array(arr: &[i32]) -> i32;
}

fn main() {
    let numbers = vec![1, 2, 3, 4, 5];
    let total = sum_array(&numbers);  // 自动转换
    println!("Sum: {}", total);
}
```

## 最佳实践

### ✅ 推荐做法

1. **使用智能降级**：让 AutoZig 自动处理 `&str`、`&[T]` 等类型
2. **结构体用 `#[repr(C)]`**：保证内存布局兼容
3. **枚举用 `#[repr(C)]`**：明确指定判别式类型
4. **导出函数用 `export`**：确保 Zig 函数可见
5. **使用增量编译**：利用缓存加速构建

### ❌ 避免的错误

1. ❌ 不要在 Rust 签名中使用 `pub`（宏会自动生成）
2. ❌ 不要忘记 `---` 分隔符
3. ❌ 不要混用 Zig 和 Rust 的命名约定
4. ❌ 不要在没有 `#[repr(C)]` 的结构体上使用 FFI
5. ❌ 不要手写 unsafe 代码（除非必要）

## 调试技巧

### 查看生成的代码

```bash
# 查看生成的 Zig 代码
cat target/release/build/*/out/generated_autozig.zig

# 查看编译日志
RUST_LOG=debug cargo build --release
```

### 常见错误

**错误 1：找不到函数**
```
error: cannot find function `my_func` in this scope
```
**解决**：检查是否在 Zig 代码中使用了 `export` 关键字

**错误 2：类型不匹配**
```
error: mismatched types
```
**解决**：确保 Rust 和 Zig 的类型定义一致

**错误 3：编译 Zig 失败**
```
error: Zig compilation failed
```
**解决**：检查 Zig 代码语法，确保 Zig 编译器已安装

## 性能优化

### 启用增量编译

AutoZig 默认启用增量编译，使用 SHA-256 哈希检测代码变化。

### 使用 release 模式

```bash
cargo build --release
```

Zig 代码将使用 `-O ReleaseFast` 优化。

### 内联优化

小函数会被自动内联，实现零成本抽象。

## 下一步

- 📖 阅读 [完整文档](./README.md)
- 🎯 查看 [设计文档](./DESIGN.md)
- 📊 了解 [实现总结](./IMPLEMENTATION_SUMMARY.md)
- 💡 探索 [示例项目](./examples/)

## 获取帮助

- 查看示例代码
- 阅读 Zig 文档：https://ziglang.org/documentation/master/
- 检查错误日志
- 提交 Issue（如果是开源项目）

---

**开始构建你的 Rust-Zig 混合项目吧！** 🚀