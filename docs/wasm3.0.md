### WebAssembly 3.0 的 64-bit 支持情况

WebAssembly (Wasm) 3.0 标准于 2025 年 9 月正式完成，这是 Wasm 生态的一个重大里程碑。其中，64-bit 支持主要通过 **Memory64** 提案实现，该提案允许线性内存和表使用 64-bit (i64) 地址类型，而不是传统的 32-bit (i32) 限制。这将地址空间从最大 4GB 扩展到理论上的 16 exabytes（约 18.4 百万 TB），但在实际应用中会受硬件和运行时限制。例如，在 Web 浏览器环境中，64-bit 内存通常被限制在 16GB 以内，以确保安全性和兼容性。

#### 关键影响和含义
- **优势**：这解决了传统 Wasm 在处理大数据集、复杂计算或大型应用时的内存瓶颈，尤其适用于非 Web 环境（如服务器侧、边缘计算或嵌入式系统）。例如，科学计算、机器学习模型或大型数据库操作现在可以更高效地在 Wasm 中运行，而无需频繁的内存分页或工作区切换。
- **兼容性**：Memory64 是 Wasm 3.0 的核心特性，已被主流引擎（如 V8、SpiderMonkey 和 Wasmtime）逐步集成。但并非所有运行时都默认启用，需要实验标志（如 Node.js 的 `--experimental-wasm-memory64` 或 Wasmtime 的 `--wasm memory64`）。在 2026 年初，大多数现代浏览器和运行时已支持，但旧版环境可能需要 polyfill 或回退到 32-bit。
- **性能考虑**：使用 64-bit 索引可能引入轻微开销（如更大的指针大小），但在需要超过 4GB 内存的场景中，这是必需的。否则，建议优先使用 32-bit 以优化性能。
- **当前状态**：截至 2026 年 1 月，Wasm 3.0 已广泛采用，但 Memory64 的生态仍在成熟。许多语言（如 C++、Go）已更新工具链以支持它，而浏览器支持率已超过 90%（基于 CanIUse 数据）。

如果你的应用不需要超过 4GB 内存，Memory64 可能不值得立即切换，因为它不会自动提升性能，只扩展容量。

### Zig 的对应处理方案

Zig 作为一种高效的系统编程语言，对 Wasm 的支持非常出色，特别是通过其内置的 Wasm 后端。从 Zig 0.11 版本开始，它原生支持 64-bit Wasm（即 Memory64），无需额外插件。Zig 的方案强调简单性和低级控制，允许开发者直接管理内存，而不依赖复杂框架。

#### 主要方案和配置
- **目标平台**：使用 `-target wasm64-freestanding` 编译标志来启用 64-bit 模式。这会自动在生成的 Wasm 模块中添加 `memory64` 部分，使内存地址使用 64-bit。
  - 示例命令：  
    ```bash
    zig build-exe mycode.zig -target wasm64-freestanding
    ```
    - 对于 WASI（WebAssembly System Interface）支持的版本，使用 `-target wasm64-wasi` 以访问文件系统等主机功能。
- **内存管理**：Zig 提供内置函数（intrinsics）来处理 64-bit 内存：
  - `@wasmMemorySize(index: u32) usize`：返回指定内存的当前大小（以 64KB 页为单位）。
  - `@wasmMemoryGrow(index: u32, delta: usize) isize`：增长内存，返回之前的大小（失败返回 -1）。
  - 示例代码（测试内存增长）：  
    ```zig
    const std = @import("std");
    const builtin = @import("builtin");

    test "@wasmMemoryGrow" {
        if (builtin.target.cpu.arch != .wasm64) return error.SkipZigTest;  // 仅在 wasm64 下运行
        const prev = @wasmMemorySize(0);
        try std.testing.expect(prev == @wasmMemoryGrow(0, 1));
        try std.testing.expect(prev + 1 == @wasmMemorySize(0));
    }
    ```
- **链接器选项**：控制初始和最大内存大小：
  - 示例：  
    ```bash
    zig build-exe mycode.zig -target wasm64-freestanding \
        -Wl,-z,stack-size=65536 \  # 栈大小
        -Wl,--max-memory=1048576   # 最大内存（2^20 页，约 68GB）
    ```
- **导出函数**：允许主机（浏览器或运行时）调用 Zig 函数来动态增长内存或访问高地址。
  - 示例（导出增长函数）：  
    ```zig
    export fn growMemory(pages: u64) u64 {
        return @wasmMemoryGrow(0, pages);
    }

    export fn writeAtHighAddress() void {
        const ptr = @intToPtr(*u64, 0x1_0000_0000_0000_0000);  // 高于 4GB 的地址
        ptr.* = 42;
    }
    ```
- **标准库集成**：使用 `std.mem.WasmPageAllocator` 作为高级分配器，它包装了低级 intrinsics。检测目标：`if (builtin.target.isWasm64) { ... }`。
- **局限性**：Memory64 需要兼容运行时（不所有浏览器默认支持）。Zig 的标准库对 64-bit 的高层 API 仍有限制，适合低级开发。测试时，非 Wasm 环境会自动跳过相关测试。

Zig 的方案适合性能敏感的应用，编译产物小巧，且易于与 C 互操作。

### Rust 的对应处理方案

Rust 通过其工具链（rustc 和 Cargo）提供对 Wasm 的强大支持，包括专用的 64-bit target。从 Rust 1.70+ 开始，`wasm64-unknown-unknown` target 正式可用，专为 Memory64 设计。它将 `usize` 和指针大小设置为 8 字节，支持完整的 64-bit 地址空间。

#### 主要方案和配置
- **目标平台**：`wasm64-unknown-unknown` 是专为 64-bit Wasm 设计的 target，支持 `std` 库（但 I/O 如文件/网络会返回错误，无实际效果）。它依赖 Memory64 提案，并启用 bulk-memory、sign-ext 等扩展。
  - 安装和构建：Rust 不预编译此 target 的标准库，需要手动构建：
    - 编辑 `config.toml`（或 `bootstrap.toml`）：  
      ```toml
      [build]
      target = ["wasm64-unknown-unknown"]

      [rust]
      lld = true  # 启用内置链接器
      ```
    - 构建 Rust：`./x.py build`。
    - 编译项目：  
      ```bash
      cargo build --target wasm64-unknown-unknown
      ```
      - 或使用 `build-std`：`cargo build --target wasm64-unknown-unknown -Z build-std`。
- **内存管理**：Rust 的 `std::alloc` 使用 dlmalloc 作为默认分配器，支持 64-bit 增长。开发者无需手动处理 intrinsics；标准库会自动使用 64-bit 指令（如 `memory.grow` 的 64-bit 变体）。
  - 配置检测：使用 `cfg!` 宏：  
    ```rust
    #[cfg(target_arch = "wasm64")]
    fn main() {
        println!("Running in 64-bit Wasm mode");
    }
    ```
- **示例代码**：简单 64-bit 指针操作：  
  ```rust
  fn main() {
      let large_ptr: *mut u64 = 0x1_0000_0000_0000_0000 as *mut u64;  // 高地址
      unsafe { *large_ptr = 42; }
  }
  ```
- **运行时要求**：需要支持 Memory64 的引擎（如 Wasmtime with `--wasm memory64` 或 Node.js with `--experimental-wasm-memory64`）。浏览器支持需实验标志。
- **局限性**：
  - 不稳定：Memory64 未完全标准化，LLVM 可能有 bug。
  - 无 `panic=unwind` 支持（仅 `panic=abort`）。
  - 测试支持差：Rust 项目中无 wasm64 测试。
  - 无法与 C 混合（无 libc）。
  - 最大内存：理论全 64-bit 空间，但实际受主机限制。

Rust 的方案更适合大型项目，利用其安全性和生态（如 wasm-bindgen for Web 集成），但构建过程较 Zig 复杂。

总体而言，Wasm 3.0 的 64-bit 支持已成熟，但实际采用取决于你的运行环境。如果你有特定代码或用例，建议测试兼容性。

### WebAssembly 3.0 的 64-bit 支持情况

WebAssembly (Wasm) 3.0 标准于 2025 年 9 月正式完成，这是 Wasm 生态的一个重大里程碑。其中，64-bit 支持主要通过 **Memory64** 提案实现，该提案允许线性内存和表使用 64-bit (i64) 地址类型，而不是传统的 32-bit (i32) 限制。这将地址空间从最大 4GB 扩展到理论上的 16 exabytes（约 18.4 百万 TB），但在实际应用中会受硬件和运行时限制。例如，在 Web 浏览器环境中，64-bit 内存通常被限制在 16GB 以内，以确保安全性和兼容性。

#### 关键影响和含义
- **优势**：这解决了传统 Wasm 在处理大数据集、复杂计算或大型应用时的内存瓶颈，尤其适用于非 Web 环境（如服务器侧、边缘计算或嵌入式系统）。例如，科学计算、机器学习模型或大型数据库操作现在可以更高效地在 Wasm 中运行，而无需频繁的内存分页或工作区切换。
- **兼容性**：Memory64 是 Wasm 3.0 的核心特性，已被主流引擎（如 V8、SpiderMonkey 和 Wasmtime）逐步集成。但并非所有运行时都默认启用，需要实验标志（如 Node.js 的 `--experimental-wasm-memory64` 或 Wasmtime 的 `--wasm memory64`）。在 2026 年初，大多数现代浏览器和运行时已支持，但旧版环境可能需要 polyfill 或回退到 32-bit。
- **性能考虑**：使用 64-bit 索引可能引入轻微开销（如更大的指针大小），但在需要超过 4GB 内存的场景中，这是必需的。否则，建议优先使用 32-bit 以优化性能。
- **当前状态**：截至 2026 年 1 月，Wasm 3.0 已广泛采用，但 Memory64 的生态仍在成熟。许多语言（如 C++、Go）已更新工具链以支持它，而浏览器支持率已超过 90%（基于 CanIUse 数据）。

如果你的应用不需要超过 4GB 内存，Memory64 可能不值得立即切换，因为它不会自动提升性能，只扩展容量。

### Zig 的对应处理方案

Zig 作为一种高效的系统编程语言，对 Wasm 的支持非常出色，特别是通过其内置的 Wasm 后端。从 Zig 0.11 版本开始，它原生支持 64-bit Wasm（即 Memory64），无需额外插件。Zig 的方案强调简单性和低级控制，允许开发者直接管理内存，而不依赖复杂框架。

#### 主要方案和配置
- **目标平台**：使用 `-target wasm64-freestanding` 编译标志来启用 64-bit 模式。这会自动在生成的 Wasm 模块中添加 `memory64` 部分，使内存地址使用 64-bit。
  - 示例命令：  
    ```bash
    zig build-exe mycode.zig -target wasm64-freestanding
    ```
    - 对于 WASI（WebAssembly System Interface）支持的版本，使用 `-target wasm64-wasi` 以访问文件系统等主机功能。
- **内存管理**：Zig 提供内置函数（intrinsics）来处理 64-bit 内存：
  - `@wasmMemorySize(index: u32) usize`：返回指定内存的当前大小（以 64KB 页为单位）。
  - `@wasmMemoryGrow(index: u32, delta: usize) isize`：增长内存，返回之前的大小（失败返回 -1）。
  - 示例代码（测试内存增长）：  
    ```zig
    const std = @import("std");
    const builtin = @import("builtin");

    test "@wasmMemoryGrow" {
        if (builtin.target.cpu.arch != .wasm64) return error.SkipZigTest;  // 仅在 wasm64 下运行
        const prev = @wasmMemorySize(0);
        try std.testing.expect(prev == @wasmMemoryGrow(0, 1));
        try std.testing.expect(prev + 1 == @wasmMemorySize(0));
    }
    ```
- **链接器选项**：控制初始和最大内存大小：
  - 示例：  
    ```bash
    zig build-exe mycode.zig -target wasm64-freestanding \
        -Wl,-z,stack-size=65536 \  # 栈大小
        -Wl,--max-memory=1048576   # 最大内存（2^20 页，约 68GB）
    ```
- **导出函数**：允许主机（浏览器或运行时）调用 Zig 函数来动态增长内存或访问高地址。
  - 示例（导出增长函数）：  
    ```zig
    export fn growMemory(pages: u64) u64 {
        return @wasmMemoryGrow(0, pages);
    }

    export fn writeAtHighAddress() void {
        const ptr = @intToPtr(*u64, 0x1_0000_0000_0000_0000);  // 高于 4GB 的地址
        ptr.* = 42;
    }
    ```
- **标准库集成**：使用 `std.mem.WasmPageAllocator` 作为高级分配器，它包装了低级 intrinsics。检测目标：`if (builtin.target.isWasm64) { ... }`。
- **局限性**：Memory64 需要兼容运行时（不所有浏览器默认支持）。Zig 的标准库对 64-bit 的高层 API 仍有限制，适合低级开发。测试时，非 Wasm 环境会自动跳过相关测试。

Zig 的方案适合性能敏感的应用，编译产物小巧，且易于与 C 互操作。

### Rust 的对应处理方案

Rust 通过其工具链（rustc 和 Cargo）提供对 Wasm 的强大支持，包括专用的 64-bit target。从 Rust 1.70+ 开始，`wasm64-unknown-unknown` target 正式可用，专为 Memory64 设计。它将 `usize` 和指针大小设置为 8 字节，支持完整的 64-bit 地址空间。

#### 主要方案和配置
- **目标平台**：`wasm64-unknown-unknown` 是专为 64-bit Wasm 设计的 target，支持 `std` 库（但 I/O 如文件/网络会返回错误，无实际效果）。它依赖 Memory64 提案，并启用 bulk-memory、sign-ext 等扩展。
  - 安装和构建：Rust 不预编译此 target 的标准库，需要手动构建：
    - 编辑 `config.toml`（或 `bootstrap.toml`）：  
      ```toml
      [build]
      target = ["wasm64-unknown-unknown"]

      [rust]
      lld = true  # 启用内置链接器
      ```
    - 构建 Rust：`./x.py build`。
    - 编译项目：  
      ```bash
      cargo build --target wasm64-unknown-unknown
      ```
      - 或使用 `build-std`：`cargo build --target wasm64-unknown-unknown -Z build-std`。
- **内存管理**：Rust 的 `std::alloc` 使用 dlmalloc 作为默认分配器，支持 64-bit 增长。开发者无需手动处理 intrinsics；标准库会自动使用 64-bit 指令（如 `memory.grow` 的 64-bit 变体）。
  - 配置检测：使用 `cfg!` 宏：  
    ```rust
    #[cfg(target_arch = "wasm64")]
    fn main() {
        println!("Running in 64-bit Wasm mode");
    }
    ```
- **示例代码**：简单 64-bit 指针操作：  
  ```rust
  fn main() {
      let large_ptr: *mut u64 = 0x1_0000_0000_0000_0000 as *mut u64;  // 高地址
      unsafe { *large_ptr = 42; }
  }
  ```
- **运行时要求**：需要支持 Memory64 的引擎（如 Wasmtime with `--wasm memory64` 或 Node.js with `--experimental-wasm-memory64`）。浏览器支持需实验标志。
- **局限性**：
  - 不稳定：Memory64 未完全标准化，LLVM 可能有 bug。
  - 无 `panic=unwind` 支持（仅 `panic=abort`）。
  - 测试支持差：Rust 项目中无 wasm64 测试。
  - 无法与 C 混合（无 libc）。
  - 最大内存：理论全 64-bit 空间，但实际受主机限制。

Rust 的方案更适合大型项目，利用其安全性和生态（如 wasm-bindgen for Web 集成），但构建过程较 Zig 复杂。

总体而言，Wasm 3.0 的 64-bit 支持已成熟，但实际采用取决于你的运行环境。如果你有特定代码或用例，建议测试兼容性。