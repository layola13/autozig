# Modular Complex Example

这个示例展示了autozig的新模块化编译功能。

## 特性

### 🎯 模块化编译
- **多个独立的.zig文件**：不再将所有代码合并到一个文件
- **目录结构组织**：支持在不同目录中组织Zig模块
- **独立维护**：每个模块可以单独修改和测试

### ✅ 解决的问题
- **全局变量冲突**：解决了allocator等全局变量重复定义的问题
- **更好的代码组织**：类似C++的模块化编译方式
- **增量编译**：只重新编译修改的模块

### 🏗️ 编译模式
默认使用`ModularBuildZig`模式（方案2）：
- 生成`build.zig`文件管理编译
- 使用Zig原生构建系统
- 支持复杂的模块依赖

## 项目结构

```
modular_complex/
├── Cargo.toml
├── build.rs          # 使用模块化编译
├── src/
│   ├── main.rs       # Rust主程序
│   ├── math/
│   │   └── vector.zig    # 向量运算模块
│   ├── utils/
│   │   └── string_ops.zig # 字符串操作模块
│   └── data/
│       └── array_ops.zig  # 数组操作模块
```

## 构建和运行

```bash
# 构建
cd autozig/examples/modular_complex
cargo build

# 运行
cargo run
```

## 预期输出

```
=== Modular Complex Example ===

This example demonstrates modular Zig compilation:
- Multiple .zig files in different directories
- Each module compiled independently
- No global variable conflicts

--- Vector Operations ---
v1 = (3, 4)
v2 = (1, 2)
v1 + v2 = (4, 6)
|v1| = 5
v1 · v2 = 11

--- String Operations ---
Length of 'Hello': 5
Length of 'World': 5
Compare 'Hello' vs 'World': -1
Compare 'Hello' vs 'Hello': 0
Hash of 'Hello': 210676686969

--- Array Operations ---
Original array: [5, 2, 8, 1, 9, 3, 7, 4, 6]
Sum: 45
Min: 1, Max: 9
Reversed array: [6, 4, 7, 3, 9, 1, 8, 2, 5]

=== All tests passed! ===
✓ Modular compilation works correctly
✓ Multiple independent Zig modules
✓ No global variable conflicts
```

## 技术细节

### 编译流程

1. **扫描阶段**：`scanner.rs`收集所有`.zig`文件路径
2. **生成阶段**：
   - 生成`build.zig`文件
   - 生成`generated_main.zig`作为主模块
   - 复制外部`.zig`文件到输出目录
3. **编译阶段**：使用`zig build`编译整个项目
4. **链接阶段**：链接生成的静态库

### 与旧模式的对比

**旧模式（Merged）**：
```
所有.zig文件 → 合并 → generated_autozig.zig → 编译
问题：全局变量重复定义
```

**新模式（ModularBuildZig）**：
```
vector.zig    ↘
string_ops.zig → build.zig + generated_main.zig → zig build → libautozig.a
array_ops.zig ↗
优势：独立模块，无冲突
```

## 向后兼容

现有代码仍然可以工作！可以通过环境变量或配置选择编译模式：

```rust
// build.rs
use autozig_build::CompilationMode;

fn main() {
    // 方式1：使用默认模式（ModularBuildZig）
    autozig_build::build("src").unwrap();
    
    // 方式2：显式指定模式
    autozig_build::build_with_mode("src", CompilationMode::ModularBuildZig).unwrap();
    
    // 方式3：使用旧的合并模式（向后兼容）
    autozig_build::build_with_mode("src", CompilationMode::Merged).unwrap();
}
```

## 相关文档

- [autozig设计文档](../../docs/DESIGN.md)
- [快速开始指南](../../docs/QUICK_START.md)