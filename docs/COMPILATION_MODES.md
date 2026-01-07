# AutoZig 编译模式指南

## 概述

AutoZig 支持三种Zig代码编译模式，每种模式适用于不同的场景。默认使用**ModularBuildZig模式（推荐）**。

## 三种编译模式

### 1. Merged（合并模式）- 传统模式

**特点:**
- 将所有Zig代码合并到单个`generated_autozig.zig`文件
- 向后兼容旧版本
- 简单直接，适合小型项目

**优点:**
- 编译速度快（单文件）
- 无需额外配置

**缺点:**
- ❌ 全局变量重复定义问题
- ❌ 失去模块化优势
- ❌ 难以维护大型代码库
- ❌ 无法使用`@import`组织代码

**适用场景:**
- 简单的单文件Zig集成
- 遗留项目迁移

---

### 2. ModularImport（模块导入模式）- 方案1

**特点:**
- 生成主模块文件`generated_main.zig`
- 通过`@import`语句引用其他.zig文件
- 每个文件保持独立
- 使用`zig build-lib`直接编译主模块

**优点:**
- ✓ 保持文件独立性
- ✓ 支持`@import`
- ✓ 解决全局变量重复定义
- ✓ 编译速度较快

**缺点:**
- 不支持C源文件编译
- 需要手动管理文件依赖

**适用场景:**
- 纯Zig代码项目
- 需要模块化但不需要C集成

---

### 3. ModularBuildZig（build.zig模式）- 方案2 ⭐ **推荐**

**特点:**
- 生成`build.zig`构建脚本
- 使用Zig标准构建系统
- 完整的模块化编译支持
- **自动编译C源文件（.c文件）**
- 支持复杂的构建配置

**优点:**
- ✅ **完整的模块化支持**
- ✅ **自动处理C/Zig互操作**
- ✅ **支持.c文件自动编译和链接**
- ✅ 解决全局变量问题
- ✅ 强大的构建系统
- ✅ 支持增量编译
- ✅ 易于扩展和维护
- ✅ Zig社区最佳实践

**缺点:**
- 编译时间稍长（完整的build流程）
- 需要Zig 0.11+

**适用场景:**
- 🔥 **所有新项目（默认推荐）**
- 🔥 **需要Zig+C混合编程**
- 中大型项目
- 需要灵活构建配置

---

## 如何切换编译模式

### 方法1: 环境变量（推荐）

在`build.rs`中设置环境变量：

```rust
// build.rs
fn main() -> anyhow::Result<()> {
    // 方式1: 使用ModularBuildZig模式（默认，推荐）
    std::env::set_var("AUTOZIG_MODE", "modular_buildzig");
    
    // 方式2: 使用ModularImport模式
    // std::env::set_var("AUTOZIG_MODE", "modular_import");
    
    // 方式3: 使用Merged模式（旧版）
    // std::env::set_var("AUTOZIG_MODE", "merged");
    
    autozig_build::build("src")?;
    Ok(())
}
```

### 方法2: 通过API直接指定

```rust
// build.rs
use autozig_engine::{AutoZigEngine, CompilationMode};

fn main() -> anyhow::Result<()> {
    let out_dir = std::env::var("OUT_DIR")?;
    
    // 创建engine并指定模式
    let engine = AutoZigEngine::with_mode(
        "src",
        &out_dir,
        CompilationMode::ModularBuildZig  // 或 Merged, ModularImport
    );
    
    engine.build()?;
    Ok(())
}
```

### 方法3: Cargo特性标志（未来支持）

```toml
[features]
default = ["modular-buildzig"]
modular-buildzig = []
modular-import = []
merged = []
```

---

## 模式选择指南

### 决策树

```
需要C/Zig互操作（.c文件）？
├─ 是 → 使用 ModularBuildZig ⭐
└─ 否
   ├─ 项目复杂度高（多个.zig文件）？
   │  ├─ 是 → 使用 ModularBuildZig 或 ModularImport
   │  └─ 否 → 使用 Merged（简单项目）
   └─ 需要向后兼容？
      └─ 是 → 使用 Merged
```

### 快速推荐表

| 项目类型 | 推荐模式 | 原因 |
|---------|---------|------|
| 新项目 | **ModularBuildZig** | 最佳实践，功能最全 |
| Zig+C混合 | **ModularBuildZig** | 唯一支持C文件编译 |
| 纯Zig多模块 | ModularBuildZig 或 ModularImport | 模块化优势 |
| 简单单文件 | Merged | 足够简单 |
| 遗留项目 | Merged | 兼容性 |

---

## 示例对比

### 示例1: 纯Zig项目

**Merged模式:**
```
generated_autozig.zig  (所有代码合并)
```

**ModularImport模式:**
```
generated_main.zig     (主模块，含@import)
vector.zig             (独立文件)
string_ops.zig         (独立文件)
```

**ModularBuildZig模式:**
```
build.zig              (构建脚本)
generated_main.zig     (主模块)
vector.zig             (独立文件)
string_ops.zig         (独立文件)
```

### 示例2: Zig+C混合项目

**只有ModularBuildZig支持:**
```
build.zig              (构建脚本，自动编译.c文件)
generated_main.zig     (Zig主模块)
wrapper.zig            (Zig包装器)
math.c                 (C源文件，自动编译)
```

编译流程：
1. `build.zig`扫描所有.c文件
2. 使用`lib.addCSourceFile()`添加C文件
3. 链接Zig和C目标文件
4. 生成最终静态库

---

## 技术细节

### ModularBuildZig的build.zig生成

自动生成的`build.zig`包含：

```zig
const std = @import("std");

pub fn build(b: *std.Build) void {
    // 1. 目标配置（baseline CPU，兼容Rust）
    const target = b.resolveTargetQuery(.{
        .cpu_model = .baseline,
        .cpu_arch = .x86_64,
        .os_tag = .linux,
        .abi = .gnu,
    });
    
    // 2. 创建模块
    const mod = b.addModule("autozig", .{
        .root_source_file = b.path("generated_main.zig"),
        .target = target,
        .optimize = optimize,
    });
    
    // 3. 创建静态库
    const lib = b.addLibrary(.{
        .name = "autozig",
        .root_module = mod,
        .linkage = .static,
    });
    
    // 4. 启用PIC（Rust FFI兼容）
    lib.root_module.pic = true;
    
    // 5. 链接libc
    lib.linkLibC();
    
    // 6. 添加C源文件（自动检测）
    lib.addCSourceFile(.{ 
        .file = b.path("math.c"), 
        .flags = &.{"-fno-sanitize=undefined"}  // 禁用UBSan
    });
    
    // 7. 安装产物
    b.installArtifact(lib);
}
```

### C文件编译特性

ModularBuildZig模式自动处理：
- ✅ 扫描src目录下所有.c文件
- ✅ 复制到OUT_DIR
- ✅ 添加到build.zig
- ✅ 使用`-fno-sanitize=undefined`标志（避免UBSan链接错误）
- ✅ 自动链接libc

---

## 常见问题

### Q: 如何知道当前使用的模式？

A: 编译时会输出警告信息：
```
warning: Using MODULAR_BUILDZIG compilation mode (recommended)
warning: Using MODULAR_IMPORT compilation mode
warning: Using MERGED compilation mode (legacy)
```

### Q: 可以在同一个项目中混用模式吗？

A: 不推荐。每个crate应该使用单一模式。如果需要不同模式，建议分离为不同的crate。

### Q: ModularBuildZig比其他模式慢多少？

A: 通常慢5-10%，但换来的是：
- 完整的模块化支持
- C文件自动编译
- 更好的可维护性
- **这点性能损失完全值得**

### Q: 旧项目如何迁移到新模式？

A: 只需在`build.rs`中设置环境变量即可：
```rust
std::env::set_var("AUTOZIG_MODE", "modular_buildzig");
```
无需修改其他代码！

### Q: C文件必须放在src目录吗？

A: 是的，scanner会自动扫描src目录下的所有.c文件并编译。

### Q: 如何调试build.zig？

A: 查看生成的build.zig文件：
```bash
cat target/debug/build/your-crate-*/out/build.zig
```

---

## 实战示例

### 示例项目

所有模式的完整示例：

1. **modular_complex** - ModularBuildZig多目录示例
   ```
   src/
   ├── main.rs
   ├── math/vector.zig
   ├── data/array_ops.zig
   └── utils/string_ops.zig
   ```

2. **zig-c** - Zig+C互操作示例（必须用ModularBuildZig）
   ```
   src/
   ├── main.rs
   ├── wrapper.zig    (Zig包装C函数)
   └── math.c         (C实现)
   ```

3. **external** - 外部.zig文件示例（可用ModularImport或ModularBuildZig）

---

## 总结

- 🎯 **默认使用ModularBuildZig**（方案2）
- 🔥 **需要C/Zig互操作必须用ModularBuildZig**
- ⚠️ Merged模式仅用于简单项目或兼容旧代码
- ✅ 通过环境变量或API轻松切换
- 📦 无需修改Zig代码本身

---

## 相关文档

- [MODULAR_COMPILATION_SUMMARY.md](../MODULAR_COMPILATION_SUMMARY.md) - 模块化编译实现总结
- [CPU_BASELINE_FIX_REPORT.md](../CPU_BASELINE_FIX_REPORT.md) - CPU架构修复报告
- [examples/zig-c/README.md](../examples/zig-c/README.md) - Zig-C互操作示例

---

**最后更新**: 2026-01-07  
**AutoZig版本**: 0.1.x+