
<div align="center">

# AutoZig

![AutoZig Logo](logos/logofull.png)

### Safe Rust to Zig FFI with Generics & Async Support

[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)
[![Rust](https://img.shields.io/badge/rust-1.77%2B-orange.svg)](https://www.rust-lang.org/)
[![Zig](https://img.shields.io/badge/zig-0.11%2B-f7a41d.svg)](https://ziglang.org/)
[![CI](https://img.shields.io/badge/CI-passing-brightgreen.svg)](.github/workflows/ci.yml)
[![Tests](https://img.shields.io/badge/tests-35%20passing-success.svg)](.)

**AutoZig** enables **safe**, **ergonomic** interop between Rust and Zig code, inspired by [autocxx](https://github.com/google/autocxx) for C++.

[Quick Start](#-quick-start) • [Features](#-features) • [Phase 3: Generics & Async](#-phase-3-generics--async-new) • [Documentation](#-further-reading) • [Examples](examples/) • [Contributing](CONTRIBUTING.md)

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
use autozig::autozig;

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

### 🎉 Phase 3: Generics & Async (NEW!)

> **Latest Release** - AutoZig now supports generic monomorphization and async FFI!

#### 🔷 Generic Monomorphization

Write generic Rust functions and let AutoZig generate type-specific Zig implementations:

```rust
use autozig::autozig;

autozig! {
    // Zig implementations for each type
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
    
    // Declare once, use with multiple types!
    #[monomorphize(i32, f64, u64)]
    fn sum<T>(data: &[T]) -> T;
}

fn main() {
    let ints = vec![1i32, 2, 3, 4, 5];
    let floats = vec![1.5f64, 2.5, 3.5];
    
    println!("Sum of ints: {}", sum_i32(&ints));      // 15
    println!("Sum of floats: {}", sum_f64(&floats));  // 7.5
}
```

**Features:**
- ✅ C++-style template instantiation for Rust generics
- ✅ Automatic name mangling (`process<T>` → `process_i32`, `process_f64`)
- ✅ Type substitution engine (handles `&[T]`, `&mut [T]`, nested types)
- ✅ Zero runtime overhead

#### ⚡ Async FFI with spawn_blocking

Write async Rust APIs backed by synchronous Zig implementations:

```rust
use autozig::include_zig;

include_zig!("src/compute.zig", {
    // Declare async functions
    async fn heavy_computation(data: i32) -> i32;
    async fn process_data(input: &[u8]) -> usize;
});

#[tokio::main]
async fn main() {
    // Async API - automatically uses tokio::spawn_blocking
    let result = heavy_computation(42).await;
    println!("Result: {}", result);
    
    // Concurrent execution
    let tasks = vec![
        tokio::spawn(async { heavy_computation(10).await }),
        tokio::spawn(async { heavy_computation(20).await }),
        tokio::spawn(async { heavy_computation(30).await }),
    ];
    
    let results = futures::future::join_all(tasks).await;
    println!("Concurrent results: {:?}", results);
}
```

**Zig side (stays synchronous!):**
```zig
// src/compute.zig
export fn heavy_computation(data: i32) i32 {
    // Write normal synchronous Zig code
    // No async/await needed!
    return data * 2;
}
```

**Features:**
- ✅ Rust: Async wrappers using `tokio::spawn_blocking`
- ✅ Zig: Synchronous implementations (no async/await complexity)
- ✅ Thread pool offload prevents blocking async runtime
- ✅ Automatic parameter capture and conversion

> 📖 **Learn More**: [examples/generics](examples/generics) | [examples/async](examples/async)

---

### 🧪 Zig Test Integration

> 🎉 Run Zig unit tests as part of your Rust test suite!

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

> 📖 **详细文档**：[docs/ZIG_TEST_INTEGRATION.md](docs/ZIG_TEST_INTEGRATION.md)

---

### 📦 External File Support

> 📁 Import external `.zig` files into your Rust project

使用 `include_zig!` 宏引用外部 `.zig` 文件：

```rust
use autozig::include_zig;

include_zig!("zig/math.zig", {
    fn factorial(n: u32) -> u64;
    fn fibonacci(n: u32) -> u64;
});

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

### 🧩 Trait Support

> Implement Rust traits with Zig backends

#### Zero-Sized Types (ZST)
```rust
autozig! {
    export fn calculator_add(a: i32, b: i32) i32 { return a + b; }
    
    ---
    
    trait Calculator {
        fn add(&self, a: i32, b: i32) -> i32 => calculator_add;
    }
}

let calc = Calculator::default();
assert_eq!(calc.add(2, 3), 5);
```

#### Opaque Pointers (Stateful)
```rust
autozig! {
    export fn hasher_new() *anyopaque { /* ... */ }
    export fn hasher_update(ptr: *anyopaque, data: [*]const u8, len: usize) void { /* ... */ }
    export fn hasher_finalize(ptr: *anyopaque) u64 { /* ... */ }
    export fn hasher_destroy(ptr: *anyopaque) void { /* ... */ }
    
    ---
    
    trait Hasher opaque {
        fn new() -> Self => hasher_new;
        fn update(&mut self, data: &[u8]) => hasher_update;
        fn finalize(&self) -> u64 => hasher_finalize;
        fn destroy(self) => hasher_destroy;
    }
}
```

> 📖 **Learn More**: [docs/TRAIT_SUPPORT_DESIGN.md](docs/TRAIT_SUPPORT_DESIGN.md)

---

## 📦 Examples & Verification

### 📚 10 Working Examples

All examples are fully tested and ready to run:

1. **structs** - Structure bindings
2. **enums** - Enum types and Result/Option
3. **complex** - Complex nested types
4. **smart_lowering** - Automatic type conversion
5. **external** - External Zig files with `include_zig!`
6. **trait_calculator** - Trait implementation (ZST)
7. **trait_hasher** - Trait implementation (Opaque Pointer)
8. **security_tests** - Memory safety tests
9. **generics** - Generic monomorphization (Phase 3)
10. **async** - Async FFI with spawn_blocking (Phase 3)

### 🔍 Batch Verification

Run all examples at once:

```bash
cd examples
./verify_all.sh
```

Output:
```
======================================
  验证结果总结
======================================

总计: 10 个示例
成功: 10
失败: 0
跳过: 0
[✓] 所有示例验证通过！🎉
```

> 📖 **Learn More**: [examples/README.md](examples/README.md)

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
│  • Detect generics & async              │
└──────┬──────────────────────────────────┘
       │
       ▼
┌─────────────────────────────────────────┐
│  Stage 2: Build (build.rs)              │
│  ──────────────────────────────         │
│  • Compile Zig → static library (.a)    │
│  • Generate monomorphized versions      │
│  • Link with Rust binary                │
└──────┬──────────────────────────────────┘
       │
       ▼
┌─────────────────────────────────────────┐
│  Stage 3: Macro Expansion               │
│  ────────────────────────────           │
│  • Generate safe Rust wrappers          │
│  • Handle &str → (ptr, len) conversion  │
│  • Generate async spawn_blocking        │
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
│   └── src/lib.rs       # Parse generics & async
├── macro/               # Procedural macro
│   └── src/lib.rs       # Code generation (Phase 3)
├── engine/              # Core build engine
│   ├── scanner.rs       # Source code scanner
│   ├── zig_compiler.rs  # Zig compiler wrapper
│   └── type_mapper.rs   # Type conversion logic
├── gen/build/           # Build script helpers
├── examples/            # 10 working examples
│   ├── verify_all.sh    # Batch verification script
│   └── README.md        # Examples documentation
└── docs/                # Technical documentation
```

---

## 🔧 Requirements

| 
Component | Version | Notes |
|-----------|---------|-------|
| **Rust** | 1.77+ | Workspace features required |
| **Zig** | 0.11+ or 0.12+ | Must be in PATH |
| **Tokio** | 1.0+ | Required for async examples |

---

## 🎓 Comparison with autocxx

<div align="center">

| Feature | autocxx (C++) | **autozig (Zig)** |
|:--------|:-------------:|:-----------------:|
| Target Language | C++ | **Zig** |
| Binding Generator | bindgen + cxx | bindgen |
| Safe Wrappers | ✅ | ✅ |
| Inline Code | ❌ | **✅** |
| Generics Support | ✅ | **✅** |
| Async Support | ❌ | **✅** |
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
- 🎯 Add new examples

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
- 🚀 Async architecture inspired by Tokio best practices

---

## ⚠️ Status

> **✅ Phase 3 Complete!** - AutoZig now supports generics and async FFI with 100% feature completion.
> 
> **Current Status:**
> - ✅ Phase 1: Basic FFI bindings
> - ✅ Phase 2: Smart Lowering & Traits
> - ✅ Phase 3: Generics & Async
> - 🔜 Phase 4: Stream support & advanced features (planned)

---

## 📖 Further Reading

### Core Documentation
- 📝 [Design Notes](DESIGN.md) - Architecture overview
- 🎯 [Quick Start](QUICK_START.md) - Get started in 5 minutes
- 📚 [Implementation Summary](IMPLEMENTATION_SUMMARY.md) - Technical deep dive

### Phase-Specific Documentation
- 🔷 [Phase 3: Generics Design](PHASE3_GENERICS_DESIGN.md)
- ⚡ [Phase 3: Async Design](PHASE3_ASYNC_DESIGN.md)
- ✅ [Phase 3: Complete Status](PHASE3_COMPLETE_FINAL_STATUS.md)

### Feature Documentation
- 🧪 [Zig Test Integration](ZIG_TEST_INTEGRATION.md)
- 🗺️ [Trait Support Roadmap](TRAIT_SUPPORT_ROADMAP.md)
- 🛡️ [Security Best Practices](SECURITY_BEST_PRACTICES.md)
- 🔒 [Zero Unsafe Achievement](ZERO_UNSAFE_ACHIEVEMENT.md)

### Examples
- 📂 [Examples Directory](examples/) - 10 working examples
- 📖 [Examples README](examples/README.md) - Detailed guide
- 🔍 [Batch Verification](examples/verify_all.sh) - Test all examples

---

<div align="center">

**Made with ❤️ for the Rust and Zig communities**

[⭐ Star on GitHub](https://github.com/layola13/autozig) • [🐛 Report Issues](https://github.com/layola13/autozig/issues) • [📖 Read Docs](.)

</div>