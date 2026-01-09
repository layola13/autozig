# AutoZig WASM 3.0 64-bit Memory Demo

这是一个展示 WebAssembly 3.0 Memory64 提案的完整示例，演示了 AutoZig 如何无缝支持 64-bit WebAssembly。

## 🌟 特性

- **Memory64 支持**：完整支持 64-bit 内存地址空间
- **零拷贝内存**：Zig 和 JavaScript 之间高效的内存共享
- **内存 Intrinsics**：演示 `@wasmMemorySize` 和 `@wasmMemoryGrow`
- **大内存操作**：支持超过 4GB 的内存分配（受运行时限制）
- **性能测试**：内置内存操作性能基准测试

## 📋 系统要求

### 编译时要求

1. **Zig 0.11+**：原生支持 `wasm64-freestanding` target（stable）
2. **Rust 1.74+**：完全支持 `wasm64-unknown-unknown` target（stable，无需nightly）
3. **wasm-bindgen**：用于生成 JavaScript 绑定

### 运行时要求

WebAssembly Memory64 需要运行时支持。请根据您的环境启用相关特性：

#### 浏览器

- **Chrome/Edge 90+**：
  ```
  启用 chrome://flags/#enable-webassembly-memory64
  ```

- **Firefox 89+**：
  ```
  在 about:config 中设置 javascript.options.wasm_memory64 = true
  ```

- **Safari**：暂不支持（截至 2026 年 1 月）

#### 命令行运行时

- **Node.js 18+**：
  ```bash
  node --experimental-wasm-memory64 server.js
  ```

- **Wasmtime**：
  ```bash
  wasmtime --wasm memory64 module.wasm
  ```

- **Wasmer**：
  ```bash
  wasmer run --enable-memory64 module.wasm
  ```

## 🚀 快速开始

### 1. 编译为 WASM64

**⚠️ 重要说明**：虽然`wasm64-unknown-unknown` target在Rust 1.74+中是stable的，但由于是tier-3 target，标准库仍需使用`-Z build-std`从源码构建。

```bash
# 安装必要工具
cargo install wasm-pack
rustup component add rust-src

# 方式1: 使用 build-std 编译 wasm64
cargo build \
    --target wasm64-unknown-unknown \
    -Z build-std=std,panic_abort \
    --release

# 生成 JS 绑定
cargo install wasm-bindgen-cli
wasm-bindgen --target web \
    --out-dir www/pkg \
    target/wasm64-unknown-unknown/release/autozig_wasm64bit.wasm

# 方式2: 使用 wasm32（更稳定，推荐用于开发）
wasm-pack build --target web --out-dir www/pkg --release
```

**注意**：
- wasm64 target虽然stable，但tier-3意味着需要`-Z build-std`
- `-Z`标志需要nightly工具链，但target本身是stable的
- 推荐先用wasm32测试功能，再迁移到wasm64

### 2. 启动开发服务器

```bash
cd www
python3 -m http.server 8080
```

### 3. 访问示例

在启用了 Memory64 的浏览器中打开：
```
http://localhost:8080
```

## 📁 项目结构

```
wasm64bit/
├── Cargo.toml          # Rust 项目配置
├── build.rs            # AutoZig 构建脚本
├── src/
│   ├── lib.rs          # Rust FFI 层（wasm-bindgen）
│   └── wasm64.zig      # Zig Memory64 实现
└── www/
    ├── index.html      # 前端界面
    └── pkg/            # 编译生成的 WASM 模块（需构建）
```

## 💡 核心实现

### Zig 侧（wasm64.zig）

```zig
const std = @import("std");
const builtin = @import("builtin");

// 大内存缓冲区（16MB）
var large_buffer: [16 * 1024 * 1024]u8 = undefined;

/// 获取内存大小（64-bit）
export fn get_memory_size() usize {
    return @wasmMemorySize(0);
}

/// 增长内存
export fn grow_memory(delta: usize) isize {
    return @wasmMemoryGrow(0, delta);
}

/// 分配缓冲区（零拷贝）
export fn alloc_large_buffer() [*]u8 {
    return &large_buffer;
}
```

### Rust 侧（lib.rs）

```rust
use autozig::include_zig;
use wasm_bindgen::prelude::*;

include_zig!("src/wasm64.zig", {
    fn get_memory_size() -> usize;
    fn grow_memory(delta: usize) -> isize;
    fn alloc_large_buffer() -> *mut u8;
});

#[wasm_bindgen]
pub fn wasm_get_memory_size() -> usize {
    get_memory_size()
}
```

## 🧪 功能测试

示例包含多个交互式测试：

1. **基础内存测试**：验证写入/读取操作
2. **填充缓冲区**：批量内存填充性能测试
3. **校验和计算**：计算密集型操作测试
4. **增长内存**：动态内存分配测试
5. **高地址访问**：演示 >4GB 地址访问（需运行时支持）
6. **完整测试**：综合测试所有功能

## 📊 性能特点

### Memory64 vs Memory32

| 特性 | Memory32 | Memory64 |
|------|----------|----------|
| 最大地址空间 | 4 GB | 16 EB（实际受限） |
| 指针大小 | 4 字节 | 8 字节 |
| 索引类型 | i32 | i64 |
| 浏览器限制 | ~2 GB | ~16 GB |

### 性能考虑

- **优势**：突破 4GB 内存限制，适合大数据处理
- **开销**：64-bit 指针略大（8 vs 4 字节）
- **兼容性**：需要较新的运行时支持

## 🔧 故障排除

### 编译错误

**问题**：`error: unknown target triple 'wasm64-unknown-unknown'`

**解决方案**：
```bash
# 使用 build-std 从源码构建标准库
cargo build --target wasm64-unknown-unknown -Z build-std
```

### 运行时错误

**问题**：`WebAssembly.instantiate(): invalid memory64 import`

**解决方案**：确保运行时已启用 Memory64 支持（见上文"系统要求"）

### 浏览器不支持

**问题**：浏览器不识别 Memory64 模块

**解决方案**：
1. 检查浏览器版本（需要较新版本）
2. 启用实验性标志
3. 或回退到 wasm32 版本

## 📚 参考资料

- [WebAssembly Memory64 提案](https://github.com/WebAssembly/memory64)
- [Zig WASM 文档](https://ziglang.org/documentation/master/#WebAssembly)
- [Rust WASM Book](https://rustwasm.github.io/docs/book/)
- [AutoZig 文档](../../README.md)
- [wasm-bindgen 文档](https://rustwasm.github.io/wasm-bindgen/)

## 📝 技术细节

### Zig 编译选项

使用 `-target wasm64-freestanding` 启用 64-bit 模式：

```bash
zig build-exe mycode.zig -target wasm64-freestanding
```

### Rust 配置

在 `.cargo/config.toml` 中配置 wasm64：

```toml
[build]
target = "wasm64-unknown-unknown"

[target.wasm64-unknown-unknown]
rustflags = ["-C", "link-arg=--no-entry"]
```

### 内存布局

```
┌─────────────────────────────────────┐
│  WebAssembly Linear Memory (64-bit)  │
├─────────────────────────────────────┤
│  0x0000_0000_0000_0000 - Stack      │
│  0x0000_0000_0010_0000 - Heap       │
│  0x0000_0000_1000_0000 - Large Buf  │
│  ...                                │
│  0x0000_0004_0000_0000 - > 4GB      │ ← Memory64 独有
│  ...                                │
│  0xFFFF_FFFF_FFFF_FFFF - End        │
└─────────────────────────────────────┘
```

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

## 📄 许可证

本示例遵循 AutoZig 项目的许可证。

## 🔗 相关示例

- [wasm_light](../wasm_light/) - WASM32 + SIMD 光照渲染
- [wasm_filter](../wasm_filter/) - WASM 图像滤镜
- [external](../external/) - 基础 AutoZig FFI 示例