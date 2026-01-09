# AutoZig WASM64 开发指南

本指南详细说明了如何使用 AutoZig 构建高性能的 WASM64 应用，以及我们如何解决了 Rust 与 Zig 在 WASM 环境下的混合编译问题。

## 🎯 核心架构：直接导出模式 (Direct Export Pattern)

为了解决 WASM 环境下 Rust FFI (`extern "C"`) 会生成错误的 "env" 模块导入的问题，我们采用了全新的架构：

1.  **Native 目标**：继续使用标准的 Rust FFI 调用 Zig 静态库。
2.  **WASM 目标**：采用 **"直接导出"** 模式。
    *   Rust 代码不生成 FFI 包装函数。
    *   Zig 函数直接从 WASM 模块导出。
    *   JavaScript 前端通过自动生成的 `bindings.js` 直接调用 Zig 导出函数。

## 🛠️ 修复与实现细节

我们通过以下步骤实现了这一架构：

### 1. 宏层面的条件编译
`include_zig!` 宏被修改为智能感知目标架构：
```rust
// 宏生成的代码逻辑
#[cfg(not(target_family = "wasm"))]
mod ffi { ... extern "C" ... } // 仅在非 WASM 环境生成 FFI

#[cfg(not(target_family = "wasm"))]
pub fn wrapper() { ... } // 仅在非 WASM 环境生成 Rust Wrapper
```
这防止了 WASM 编译时生成错误的导入声明。

### 2. 强制符号链接 (Whole Archive)
由于 Rust 代码不再引用 Zig 函数，链接器默认会由 "Tree Shaking" 机制移除这些未使用的符号。我们在 `build.rs` 中强制开启了全文档链接：
```rust
println!("cargo:rustc-link-lib=static:+whole-archive=autozig");
```

### 3. 强制导出注入 (Forced Exports)
为了确保 Zig 函数在最终的 WASM 二进制文件中可见，AutoZig Engine 会自动扫描源码，并为每个 `#[autozig]` 函数注入导出指令：
```rust
println!("cargo:rustc-link-arg=--export={}", func_name);
```

### 4. 自动绑定生成
Engine 会解析 Zig 函数签名，自动生成 TypeScript 定义 (`.d.ts`) 和 JavaScript 加载器 (`.js`)，自动处理 `BigInt` (wasm64) 与 JS `number` 之间的转换。

## 🚀 开发流程

### 1. 编写 Zig 代码 (`src/wasm64.zig`)
只需要标准的 `export fn`：
```zig
export fn get_memory_size() usize {
    return @wasmMemorySize(0);
}
```

### 2. 声明接口 (`src/lib.rs`)
使用 `include_zig!` 宏，它会自动处理所有复杂的跨语言绑定：
```rust
include_zig!("src/wasm64.zig", {
    #[autozig(strategy = "dual")]
    fn get_memory_size() -> usize;
});
```

### 3. 编译
```bash
bash build.sh
# 或
cargo build --target wasm64-unknown-unknown --release
```

### 4. 前端调用
在 HTML 中直接使用生成的绑定：
```html
<script type="module">
    import { loadWasm } from './pkg/bindings.js';
    
    const wasm = await loadWasm('./pkg/autozig_wasm64bit.wasm');
    // 直接调用，类型已被自动处理
    console.log(wasm.exports.get_memory_size());
</script>
```

## ✅ 验证状态
- **编译**：成功生成 WASM 文件和绑定文件。
- **链接**：所有 Zig 导出函数均正确保留在 WASM 中。
- **功能**：内存测试 (`run_memory_test`) 通过，验证了 5GB+ 内存寻址能力。
