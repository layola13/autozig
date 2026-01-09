# WASM64 编译指南

本文档记录了 AutoZig WASM64 编译的配置方法和常见问题解决方案。

## 快速开始

```bash
cd autozig/examples/wasm64bit
cargo +nightly build    # 使用 nightly 编译
# 或
bash build.sh           # 使用构建脚本
```

---

## 核心配置

### 1. `.cargo/config.toml` 配置

```toml
[build]
# 必须：设置 wasm64 为目标平台
target = "wasm64-unknown-unknown"
rustflags = ["--cfg", "autozig_modular"]

[unstable]
# 必须：wasm64 是 Tier 3 目标，需要从源码构建标准库
build-std = ["std", "panic_abort"]

[env]
AUTOZIG_MODE = "modular"
```

### 2. 为什么必须使用 nightly？

`wasm64-unknown-unknown` 是 Rust 的 **Tier 3 目标**，这意味着：
- ❌ 没有预编译的标准库
- ❌ 不在 stable/beta 通道支持
- ✅ 必须使用 `cargo +nightly` 和 `-Zbuild-std` 从源码构建

### 3. 安装 rust-src 组件

```bash
rustup component add rust-src --toolchain nightly
```

---

## 常见问题排查

### 问题 1：Zig 编译为 x86_64 而不是 wasm64

**症状**：
```
error: builtin @wasmMemorySize is available when targeting WebAssembly; 
targeted CPU architecture is x86_64
```

**原因**：`autozig-engine` 中的 `generate_build_zig_with_c` 函数没有正确检测 wasm64 目标。

**解决方案**：确保 `engine/src/lib.rs` 中包含 wasm64 检测逻辑：

```rust
let is_wasm32 = zig_target.contains("wasm32");
let is_wasm64 = zig_target.contains("wasm64");
let is_wasm = is_wasm32 || is_wasm64;

// 生成 build.zig 时
if is_wasm64 {
    build.push_str("        .cpu_arch = .wasm64,\n");
    build.push_str("        .os_tag = .freestanding,\n");
} else if is_wasm32 {
    // ...
}
```

### 问题 2：TARGET 环境变量不正确

**症状**：build.rs 看到的 `TARGET` 是 `x86_64-unknown-linux-gnu` 而不是 `wasm64-unknown-unknown`。

**原因**：`.cargo/config.toml` 中没有设置 `target = "wasm64-unknown-unknown"`，导致 cargo 编译的是 host 目标。

**验证方法**：
```bash
cargo build -vv 2>&1 | grep "TARGET="
```

**解决方案**：确保 `.cargo/config.toml` 中设置了 `target`。

### 问题 3：缓存导致的旧配置

**症状**：修改了 engine 代码但生成的 build.zig 仍然是旧的。

**解决方案**：
```bash
cargo clean
cargo +nightly build
```

---

## 目标映射参考

| Rust Target | Zig Target |
|-------------|------------|
| `wasm32-unknown-unknown` | `wasm32-freestanding` |
| `wasm32-wasi` | `wasm32-wasi` |
| `wasm64-unknown-unknown` | `wasm64-freestanding` |
| `wasm64-wasi` | `wasm64-wasi` |

---

## #[autozig(strategy = "dual")] 双重绑定

新的属性语法可以自动生成两种绑定：

```rust
include_zig!("src/wasm64.zig", {
    // 自动生成 wasm_get_memory_size 和 wasm64_get_memory_size
    #[autozig(strategy = "dual")]
    fn get_memory_size() -> usize;

    // 带类型转换的双重绑定
    #[autozig(
        strategy = "dual",
        c_ret = "u32",          // C 导出返回 u32
        map_fn = "|b: bool| if b { 1 } else { 0 }"
    )]
    fn some_bool_fn() -> bool;
});
```

生成结果：
- `wasm_get_memory_size()` - wasm-bindgen 绑定
- `wasm64_get_memory_size()` - C 风格 `#[no_mangle] pub extern "C"` 导出

---

## 浏览器 Memory64 支持

| 浏览器 | Flag |
|--------|------|
| Chrome | `chrome://flags/#enable-webassembly-memory64` |
| Firefox | `about:config` → `javascript.options.wasm_memory64` |
| Safari | 实验性支持 |

---

## 文件结构

```
wasm64bit/
├── .cargo/
│   └── config.toml      # 编译配置（关键！）
├── src/
│   ├── lib.rs           # Rust 绑定
│   └── wasm64.zig       # Zig WASM64 实现
├── www/
│   ├── index.html       # 测试页面
│   └── pkg/             # 编译输出
├── build.rs             # 构建脚本
├── build.sh             # 一键构建
└── Cargo.toml
```

---

## 调试技巧

### 查看生成的 build.zig
```bash
cat target/wasm64-unknown-unknown/*/build/autozig-wasm64bit-*/out/build.zig
```

### 验证 WASM 导出函数
```bash
strings target/wasm64-unknown-unknown/release/*.wasm | grep -E "^wasm"
```

### 查看宏展开结果
```bash
cargo expand 2>&1 | grep -E "pub fn wasm_|pub extern"
```

---

*更新日期：2026-01-09*
