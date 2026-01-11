# AutoZig WASM64 Print Example

> 🚀 解决 WASM64 环境下 `print!` 无效的问题

## 📖 概述

本示例展示如何在 **WASM64 (Memory64)** 环境下使用 `console_log!` 和 `console_error!` 宏，完美解决 Rust 标准库的 `print!` / `println!` 在 WebAssembly 中无效的问题。

### 核心特性

- ✅ **Rust → Zig → JS** 三层调用链
- ✅ 支持 **WASM64 BigInt** 指针（64位寻址）
- ✅ **零拷贝**字符串传递（直接从 WASM 线性内存读取）
- ✅ 自动 **panic hook** 集成
- ✅ 完全**类型安全**的 FFI
- ✅ 无需 `wasm-bindgen` 等笨重依赖

## 🎯 为什么需要这个示例？

在 WASM 环境中，Rust 的标准 `print!` / `println!` 宏**完全无效**，因为：

1. WASM 没有标准输出（stdout）概念
2. 浏览器环境需要通过 JavaScript 的 `console.log` 输出
3. `wasm-bindgen` 对 WASM64 支持有限，经常把指针强转为 u32 导致崩溃

### 传统方案的问题

```rust
// ❌ 在 WASM 中完全无效
println!("Hello from WASM");  // 什么都不会输出

// ❌ wasm-bindgen 在 WASM64 下有问题
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);  // WASM64 指针转换错误
}
```

### AutoZig 方案

```rust
// ✅ 完美工作
console_log!("Hello from WASM64!");
console_log!("Value: {}", 42);
console_log!("Data: {:?}", vec![1, 2, 3]);
```

## 🏗️ 架构设计

本示例采用 **AutoZig** 的核心理念：**Rust → Zig → Host (JS)** 通路。

```
┌─────────────────────────────────────────────┐
│  Rust 层 (用户代码)                         │
│  ─────────────────────────                  │
│  console_log!("Hello {}", name);            │
│  ↓ format! 宏展开                           │
│  ↓ 调用 autozig_log_impl(&formatted_string) │
└─────────────────┬───────────────────────────┘
                  │ FFI 调用
                  ↓
┌─────────────────────────────────────────────┐
│  Zig 层 (中间桥接)                          │
│  ────────────────────                       │
│  export fn autozig_log_impl(                │
│      ptr: [*]const u8,  // 64位指针         │
│      len: usize         // 64位长度         │
│  ) void {                                   │
│      js_log(ptr, len);  // 转发给 JS        │
│  }                                          │
└─────────────────┬───────────────────────────┘
                  │ extern "env"
                  ↓
┌─────────────────────────────────────────────┐
│  JavaScript 层 (浏览器)                     │
│  ──────────────────────                     │
│  js_log: (ptrBigInt, lenBigInt) => {        │
│      const ptr = Number(ptrBigInt);  // 🔑  │
│      const len = Number(lenBigInt);  // 🔑  │
│      const bytes = new Uint8Array(          │
│          memory.buffer, ptr, len            │
│      );                                     │
│      const text = new TextDecoder()         │
│          .decode(bytes);                    │
│      console.log(`[AutoZig] ${text}`);      │
│  }                                          │
└─────────────────────────────────────────────┘
```

### 关键技术点

1. **WASM64 指针处理**：Zig 的 `usize` 在 `wasm64-unknown-unknown` 目标下自动编译为 `u64`，JS 端接收到的是 `BigInt`，完美匹配 64位寻址
2. **零拷贝传递**：字符串数据保留在 WASM 线性内存中，JS 直接通过指针读取，无需序列化/反序列化
3. **类型安全**：AutoZig 的 Smart Lowering 自动处理 `&str` → `(ptr, len)` 转换

## 📦 代码结构

```
wasm64print/
├── Cargo.toml           # 项目配置
├── build.rs             # 构建脚本
├── src/
│   ├── lib.rs           # 主库入口
│   └── console.rs       # Console 日志模块（核心实现）
└── www/
    ├── index.html       # 测试页面
    └── loader.js        # WASM64 加载器
```

### 核心模块：`console.rs`

```rust
use autozig::autozig;

autozig! {
    // Zig 实现
    extern "env" fn js_log(ptr: [*]const u8, len: usize) void;
    extern "env" fn js_error(ptr: [*]const u8, len: usize) void;

    export fn autozig_log_impl(ptr: [*]const u8, len: usize) void {
        js_log(ptr, len);
    }

    export fn autozig_error_impl(ptr: [*]const u8, len: usize) void {
        js_error(ptr, len);
    }

    ---

    // Rust 签名
    fn autozig_log_impl(msg: &str);
    fn autozig_error_impl(msg: &str);
}

// 用户友好的宏
#[macro_export]
macro_rules! console_log {
    ($($t:tt)*) => {
        {
            let s = format!($($t)*);
            $crate::console::autozig_log_impl(&s);
        }
    }
}
```

## 🚀 使用方法

### 1. 编译为 WASM64

```bash
# WASM64 需要 nightly 和 build-std
cargo +nightly build \
    --target wasm64-unknown-unknown \
    -Z build-std=std,panic_abort \
    --release
```

### 2. 复制 WASM 文件到 www 目录

```bash
cp target/wasm64-unknown-unknown/release/autozig_wasm64print.wasm www/
```

### 3. 启动 HTTP 服务器

```bash
cd www
python3 -m http.server 8080
```

### 4. 在浏览器中打开

访问 `http://localhost:8080`，打开浏览器控制台（F12）查看输出。

## 🧪 测试功能

本示例包含多个测试函数，验证不同场景：

1. **基本数值计算** - `add(10, 20)`
2. **递归计算** - `factorial(5)` 
3. **字符串处理** - `greet("AutoZig")`
4. **数组处理** - `sum_array([1,2,3,4,5])`
5. **错误处理** - `divide(10, 0)`
6. **Panic 捕获** - `test_panic()`

每个函数都会通过 `console_log!` 输出详细的执行信息。

## 📊 性能对比

| 方案 | 实现复杂度 | WASM64 支持 | 零拷贝 | 类型安全 |
|:-----|:----------:|:-----------:|:------:|:--------:|
| **AutoZig** | ⭐⭐ | ✅ 原生支持 | ✅ | ✅ |
| wasm-bindgen | ⭐⭐⭐⭐ | ❌ 有限支持 | ❌ | ⚠️ |
| 手写 JS 绑定 | ⭐⭐⭐⭐⭐ | ⚠️ 需手动处理 | ✅ | ❌ |

## 🔧 技术细节

### WASM64 内存初始化

```javascript
const memory = new WebAssembly.Memory({
    initial: 10,
    maximum: 100, 
    index: 'i64'  // 🔑 关键：声明 64 位寻址
});
```

### BigInt 指针转换

```javascript
js_log: (ptrBigInt, lenBigInt) => {
    // WASM64 传出的是 BigInt
    const ptr = Number(ptrBigInt);  // 转为 Number
    const len = Number(lenBigInt);
    
    // 零拷贝读取
    const bytes = new Uint8Array(memory.buffer, ptr, len);
    const text = new TextDecoder("utf-8").decode(bytes);
    console.log(`[AutoZig] ${text}`);
}
```

## 🎯 适用场景

本示例适用于以下场景：

- ✅ WebAssembly 应用需要调试输出
- ✅ 需要在浏览器控制台显示日志
- ✅ WASM64 环境（大内存应用）
- ✅ 需要高性能、零拷贝的日志系统
- ✅ 避免 `wasm-bindgen` 的复杂性

## 📚 扩展阅读

- [AutoZig README](../../README.md) - 了解 AutoZig 完整功能
- [PHASE 5 WASM Design](../../docs/PHASE_5_WASM_DESIGN.md) - WASM 支持设计文档
- [rust_export 示例](../rust_export/) - 另一个 WASM 示例

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

## 📄 许可证

MIT OR Apache-2.0

---

**Made with ❤️ for the Rust and Zig communities**