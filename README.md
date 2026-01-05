<div align="center">

# AutoZig

![AutoZig Logo](logo.png)

### Safe Rust to Zig FFI

[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)
[![Rust](https://img.shields.io/badge/rust-1.77%2B-orange.svg)](https://www.rust-lang.org/)
[![Zig](https://img.shields.io/badge/zig-0.11%2B-f7a41d.svg)](https://ziglang.org/)

**AutoZig** enables **safe**, **ergonomic** interop between Rust and Zig code, inspired by [autocxx](https://github.com/google/autocxx) for C++.

[Quick Start](#-quick-start) • [Features](#-features) • [Documentation](#-further-reading) • [Examples](examples/)

</div>

---

## 🎯 Core Goals

<table>
<tr>
<td width="50%">

### 🛡️ Safety First
**Zero `unsafe` in user code** - All FFI complexity is handled by the framework

### ⚡ Performance
**Compile-time code generation** - Zig code is compiled during `cargo build`

</td>
<td width="50%">

### 🔒 Type Safety
**Automatic type conversion** - Safe bindings between Rust and Zig types

### 🚀 Developer Experience
**Write Zig inline** - Embed Zig code directly in your Rust files

</td>
</tr>
</table>

## 🚀 Quick Start

### 1. Add dependencies

```toml
# Cargo.toml
[dependencies]
autozig = "0.1"

[build-dependencies]
autozig-build = "0.1"
```

### 2. Create build.rs

```rust
// build.rs
fn main() -> anyhow::Result<()> {
    autozig_build::build("src")?;
    Ok(())
}
```

### 3. Write your code

```rust
// src/main.rs
use autozig::prelude::*;

autozig! {
    // Zig implementation
    const std = @import("std");
    
    export fn compute_hash(ptr: [*]const u8, len: usize) u64 {
        const data = ptr[0..len];
        var hash: u64 = 0;
        for (data) |byte| {
            hash +%= byte;
        }
        return hash;
    }
    
    ---
    
    // Rust signatures (optional - enables safe wrappers)
    fn compute_hash(data: &[u8]) -> u64;
}

fn main() {
    let data = b"Hello AutoZig";
    let hash = compute_hash(data); // Safe call, no unsafe!
    println!("Hash: {}", hash);
}
```

## ✨ Key Features

### 🧪 Zig Test Integration

> 🎉 **NEW!** Run Zig unit tests as part of your Rust test suite!

AutoZig 支持将 Zig 的单元测试集成到 Rust 的测试框架中！

```rust
// build.rs
fn main() -> anyhow::Result<()> {
    autozig_build::build("src")?;
    autozig_build::build_tests("zig")?;  // 编译 Zig 测试
    Ok(())
}
```

```zig
// zig/math.zig
export fn factorial(n: u32) u64 {
    // ... implementation
}

test "factorial basic cases" {
    try std.testing.expectEqual(@as(u64, 120), factorial(5));
}
```

```rust
// tests/zig_tests.rs
#[test]
fn test_math_zig_tests() {
    let test_exe = get_test_exe_path("math");
    let output = Command::new(&test_exe).output().unwrap();
    assert!(output.status.success());
}
```

运行测试：
```bash
cargo test  # 自动运行 Rust 和 Zig 测试
```

> 📖 **详细文档**：[ZIG_TEST_INTEGRATION.md](ZIG_TEST_INTEGRATION.md)

---

### 📦 External File Support

> 📁 Import external `.zig` files into your Rust project

使用 `include_zig!` 宏引用外部 `.zig` 文件：

```rust
use autozig::prelude::*;

include_zig! {
    path: "zig/math.zig",
    functions: [
        fn factorial(n: u32) -> u64;
        fn fibonacci(n: u32) -> u64;
    ]
}

fn main() {
    println!("5! = {}", factorial(5));
    println!("fib(10) = {}", fibonacci(10));
}
```

---

### 🎯 Smart Lowering

> 🔄 Automatic conversion between Rust high-level types and Zig FFI-compatible types

<div align="center">

| Rust Type | Zig Signature | Auto Conversion |
|:---------:|:-------------:|:---------------:|
| `&str` | `[*]const u8, usize` | ✅ |
| `&[T]` | `[*]const T, usize` | ✅ |
| `&mut [T]` | `[*]T, usize` | ✅ |
| `String` | `[*]const u8, usize` | ✅ |

</div>

---

## 📐 Architecture

> AutoZig follows a **three-stage pipeline** for seamless Rust-Zig interop:

```
┌─────────────┐
│  Rust Code  │
│  with       │
│  autozig!   │
└──────┬──────┘
       │
       ▼
┌─────────────────────────────────────────┐
│  Stage 1: Parsing (Compile Time)        │
│  ─────────────────────────────────      │
│  • Scan .rs files for autozig! macros   │
│  • Extract Zig code                     │
│  • Parse Rust signatures                │
└──────┬──────────────────────────────────┘
       │
       ▼
┌─────────────────────────────────────────┐
│  Stage 2: Build (build.rs)              │
│  ──────────────────────────────         │
│  • Compile Zig → static library (.a)    │
│  • Generate C header (.h)               │
│  • Run bindgen → raw FFI bindings       │
└──────┬──────────────────────────────────┘
       │
       ▼
┌─────────────────────────────────────────┐
│  Stage 3: Macro Expansion               │
│  ────────────────────────────           │
│  • Generate safe Rust wrappers          │
│  • Handle &str → (ptr, len) conversion  │
│  • Include FFI bindings                 │
└──────┬──────────────────────────────────┘
       │
       ▼
┌─────────────┐
│  Safe Rust  │
│  API        │
└─────────────┘
```

---

## 📦 Project Structure

```
autozig/
├── src/lib.rs           # Main library
├── parser/              # Macro input parser
├── macro/               # Procedural macro
├── engine/              # Core build engine
│   ├── scanner.rs       # Source code scanner
│   ├── zig_compiler.rs  # Zig compiler wrapper
│   └── type_mapper.rs   # Type conversion logic
├── gen/build/           # Build script helpers
└── demo/                # Example usage
```

---

## 🔧 Requirements

| Component | Version | Notes |
|-----------|---------|-------|
| **Rust** | 1.77+ | Workspace features required |
| **Zig** | 0.11+ or 0.12+ | Must be in PATH |
| **C Compiler** | Any | Required for bindgen |

---

## 🎓 Comparison with autocxx

<div align="center">

| Feature | autocxx (C++) | **autozig (Zig)** |
|:--------|:-------------:|:-----------------:|
| Target Language | C++ | **Zig** |
| Binding Generator | bindgen + cxx | bindgen |
| Safe Wrappers | ✅ | ✅ |
| Inline Code | ❌ | **✅** |
| Build Complexity | High | **Medium** |
| Type Safety | Strong | **Strong** |

</div>

---

## 📚 Type Mapping

<div align="center">

| Zig Type | Rust Type | Notes |
|:---------|:----------|:------|
| `i8`, `i16`, `i32`, `i64` | `i8`, `i16`, `i32`, `i64` | ✅ Direct mapping |
| `u8`, `u16`, `u32`, `u64` | `u8`, `u16`, `u32`, `u64` | ✅ Direct mapping |
| `f32`, `f64` | `f32`, `f64` | ✅ Direct mapping |
| `bool` | `u8` | ⚠️ Zig bool is u8 in C ABI |
| `[*]const u8` | `*const u8` | 🔧 Raw pointer |
| `[*]const u8` + `len` | `&[u8]` | 🛡️ With safe wrapper |

</div>

---

## 🤝 Contributing

Contributions are welcome! This is an experimental project exploring Rust-Zig interop.

**Ways to contribute:**
- 🐛 Report bugs and issues
- 💡 Suggest new features
- 📖 Improve documentation
- 🔧 Submit pull requests

---

## 📄 License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

---

## 🙏 Acknowledgments

- 💡 Inspired by [autocxx](https://github.com/google/autocxx)
- 🔨 Built on [bindgen](https://github.com/rust-lang/rust-bindgen)
- ⚡ Leverages the excellent [Zig](https://ziglang.org/) language

---

## ⚠️ Status

> **⚠️ Experimental** - This is a proof-of-concept implementation. Not recommended for production use yet.

---

## 📖 Further Reading

- 📝 [Design Notes](todo/autozig.md) - Detailed design documentation
- 🎯 [Examples](demo/) - Working code examples
- 📚 [Implementation Summary](IMPLEMENTATION_SUMMARY.md) - Technical deep dive
- 🗺️ [Trait Support Roadmap](TRAIT_SUPPORT_ROADMAP.md) - Future plans
- 🧪 [Zig Test Integration](ZIG_TEST_INTEGRATION.md) - Testing guide

---

<div align="center">

**Made with ❤️ for the Rust and Zig communities**

</div>