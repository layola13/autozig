
# AutoZig 架构设计文档

本文档详细说明了 AutoZig 的内部架构设计和实现细节。

## 📐 总体架构

AutoZig 采用三阶段编译流水线，参考了 autocxx 的设计理念，并针对 Zig 的特性进行了优化。

```
用户代码 (src/main.rs with autozig!)
    ↓
┌─────────────────────────────────────┐
│  阶段 1: 解析 (Parsing)              │
│  - parser: 解析宏输入                │
│  - scanner: 扫描源文件               │
└─────────────────────────────────────┘
    ↓
┌─────────────────────────────────────┐
│  阶段 2: 构建 (Build - build.rs)    │
│  - engine: 核心构建引擎              │
│  - zig_compiler: Zig 编译器包装      │
│  - bindgen: 生成 FFI 绑定            │
└─────────────────────────────────────┘
    ↓
┌─────────────────────────────────────┐
│  阶段 3: 宏展开 (Macro Expansion)   │
│  - macro: 生成安全包装器             │
│  - type_mapper: 类型转换             │
└─────────────────────────────────────┘
    ↓
编译后的 Rust 二进制 + 静态链接的 Zig 代码
```

## 🔧 核心组件

### 1. Parser (`autozig-parser`)

**职责**: 解析 `autozig!` 宏的输入

**关键文件**:
- `src/lib.rs`: 主解析逻辑

**功能**:
- 解析混合的 Zig/Rust 语法
- 识别 `---` 分隔符
- 提取 Zig 代码部分
- 解析 Rust 函数签名（用于生成安全包装器）

**示例输入**:
```rust
autozig! {
    // Zig 部分
    export fn add(a: i32, b: i32) i32 {
        return a + b;
    }
    ---
    // Rust 签名部分
    fn add(a: i32, b: i32) -> i32;
}
```

**输出**: `AutoZigConfig` 结构，包含：
- `zig_code`: String - 提取的 Zig 代码
- `rust_signatures`: Vec<RustFunctionSignature> - 解析的函数签名

### 2. Engine (`autozig-engine`)

**职责**: 核心构建引擎，orchestrate 整个编译流程

**关键文件**:
- `src/lib.rs`: 主引擎逻辑
- `src/scanner.rs`: 源代码扫描器
- `src/zig_compiler.rs`: Zig 编译器包装
- `src/type_mapper.rs`: 类型映射表

**工作流程**:

```rust
AutoZigEngine::build() {
    1. 扫描源文件 (.rs)
       └→ scanner.scan()
       
    2. 提取所有 autozig! 宏中的 Zig 代码
       └→ 合并为单个 generated_autozig.zig
       
    3. 调用 Zig 编译器
       └→ zig build-lib -static -femit-h
       └→ 生成 libautozig.a 和 generated_autozig.h
       
    4. 运行 bindgen
       └→ 从 .h 文件生成 Rust FFI 绑定
       └→ 输出到 OUT_DIR/bindings.rs
       
    5. 配置 Cargo 链接
       └→ println!("cargo:rustc-link-lib=static=autozig")
}
```

#### 2.1 Scanner (`scanner.rs`)

**功能**:
- 递归遍历源目录
- 使用正则表达式匹配 `autozig! { ... }`
- 提取 Zig 代码（分隔符之前的部分）
- 合并所有 Zig 代码

**正则表达式**:
```rust
let re = Regex::new(r"autozig!\s*\{([\s\S]*?)\}")?;
```

**注意事项**:
- 当前实现使用简单正则，不处理嵌套大括号
- 生产版本应使用 syn 进行完整的语法解析

#### 2.2 Zig Compiler (`zig_compiler.rs`)

**功能**:
- 包装 `zig` 命令行工具
- 编译 Zig 源码为静态库
- 生成 C ABI 兼容的头文件

**关键命令**:
```bash
zig build-lib source.zig \
    -static \
    -femit-h=output.h \
    -femit-bin=output.a \
    -target native
```

**环境变量**:
- `ZIG_PATH`: 自定义 Zig 编译器路径（可选）
- 默认使用系统 PATH 中的 `zig`

#### 2.3 Type Mapper (`type_mapper.rs`)

**功能**:
- 维护 Zig ↔ Rust 类型映射表
- 识别需要特殊处理的类型（如切片）
- 分析参数转换策略

**类型映射表**:

| Zig 类型 | Rust FFI 类型 | 注释 |
|---------|--------------|------|
| `i8`, `i16`, `i32`, `i64` | `i8`, `i16`, `i32`, `i64` | 直接映射 |
| `u8`, `u16`, `u32`, `u64` | `u8`, `u16`, `u32`, `u64` | 直接映射 |
| `f32`, `f64` | `f32`, `f64` | 直接映射 |
| `bool` | `u8` | Zig bool 在 C ABI 中是 u8 |
| `[*]const u8` | `*const u8` | 原始指针 |
| `void` | `()` | 单元类型 |

**参数转换策略**:

```rust
enum ParamConversion {
    Direct,           // 基本类型直接传递
    SliceToPtrLen,    // &[T] → (ptr, len)
    StrToPtrLen,      // &str → (ptr, len)
}
```

### 3. Macro (`autozig-macro`)

**职责**: 过程宏实现，生成最终的 Rust 代码

**关键文件**:
- `src/lib.rs`: 宏实现

**工作流程**:

```rust
#[proc_macro]
pub fn autozig(input: TokenStream) -> TokenStream {
    1. 解析输入
       └→ parse_macro_input!(input as AutoZigConfig)
       
    2. 生成 FFI 模块引用
       └→ mod ffi { include!(concat!(env!("OUT_DIR"), "/bindings.rs")); }
       
    3. 如果有 Rust 签名，生成安全包装器
       └→ pub fn safe_func(...) { unsafe { ffi::raw_func(...) } }
       
    4. 处理类型转换
       └→ &[u8] → (ptr, len)
       └→ &str → (ptr, len)
}
```

**生成的代码示例**:

输入:
```rust
autozig! {
    export fn compute_hash(ptr: [*]const u8, len: usize) u64 { ... }
    ---
    fn compute_hash(data: &[u8]) -> u64;
}
```

输出:
```rust
mod ffi {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

pub fn compute_hash(data: &[u8]) -> u64 {
    unsafe {
        ffi::compute_hash(data.as_ptr(), data.len())
    }
}
```

### 4. Build Support (`autozig-build`)

**职责**: build.rs 辅助库

**关键文件**:
- `src/lib.rs`: Builder API

**使用示例**:
```rust
// build.rs
fn main() -> anyhow::Result<()> {
    autozig_build::build("src")?;
    Ok(())
}
```

## 🔄 完整编译流程

### 时间线视图

```
T0: cargo build 开始
    ↓
T1: build.rs 执行
    ├→ autozig_build::build("src")
    ├→ 扫描 src/*.rs
    ├→ 提取 Zig 代码
    ├→ 写入 OUT_DIR/generated_autozig.zig
    ├→ 调用 zig build-lib
    │   └→ 生成 OUT_DIR/libautozig.a
    │   └→ 生成 OUT_DIR/generated_autozig.h
    ├→ 调用 bindgen
    │   └→ 生成 OUT_DIR/bindings.rs
    └→ 配置链接器
    ↓
T2: rustc 编译 src/main.rs
    ├→ 展开 autozig! 宏
    ├→ 生成安全包装器代码
    ├→ 包含 OUT_DIR/bindings.rs
    └→ 编译所有 Rust 代码
    ↓
T3: 链接阶段
    ├→ 链接 Rust 对象文件
    ├→ 链接 libautozig.a (Zig 代码)
    └→ 生成最终可执行文件
    ↓
T4: 完成
```

### 数据流视图

```
源代码 (main.rs)
    ├─→ [Parser] ─→ AutoZigConfig
    │                    ├─ zig_code: String
    │                    └─ rust_signatures: Vec<Sig>
    │
    ├─→ [Scanner] ─→ 合并的 Zig 代码
    │                    └─→ generated_autozig.zig
    │
    └─→ [Engine]
            ├─→ [ZigCompiler]
            │       ├─→ libautozig.a
            │       └─→ generated_autozig.h
            │
            └─→ [Bindgen]
                    └─→ bindings.rs (Raw FFI)

宏展开 (autozig!)
    └─→ [Macro]
            ├─ include FFI bindings
            └─ generate safe wrappers
                    └─→ 最终 Rust 代码
```

## 🎯 设计决策

### 为什么使用静态链接？

**优点**:
1. **简单性**: 不需要管理动态库路径
2. **可移植性**: 单个二进制文件，无外部依赖
3. **性能**: 可能有更好的内联优化
4. **部署**: 更容易分发

**缺点**:
1. 二进制文件更大
2. 无法在运行时替换 Zig 代码

### 为什么在 build.rs 而不是宏中编译？

**原因**:
1. **编译时机**: build.rs 在宏展开前运行
2. **环境隔离**: build.rs 有独立的依赖
3. **文件操作**: 更容易处理文件 I/O
4. **错误处理**: 更清晰的错误报告

### 为什么使用 bindgen？

**原因**:
1. **成熟工具**: bindgen 是 Rust FFI 的标准工具
2. **Zig C ABI**: Zig 的 `export` 函数使用 C ABI
3. **减少工作**: 不需要手动写 FFI 绑定

### 为什么需要两种语法（Zig + Rust 签名）？

**原因**:
1. **灵活性**: 纯 Zig 适合简单用例
2. **安全性**: Rust 签名允许生成安全包装器
3. **类型转换**: 明确指定 Rust 侧的类型
4. **渐进式**: 可以先写 Zig，后续添加安全包装

## 🔒 安全性考虑

### 内存安全

1. **指针传递**:
   - Zig 接收 `[*]const u8` 和 `len`
   - Rust 保证切片的生命周期有效
   - 不允许 Zig 持有指针超过函数调用范围

2. **生命周期**:
   - 所有传递给 Zig 的引用都是临时的
   - Zig 函数必须在调用期间完成所有操作
   - 不支持异步或回调（需要额外设计）

3. **类型安全**:
   - 通过 bindgen 确保 C ABI 兼容性
   - Rust 侧的安全包装器处理类型转换
   - 编译时检查类型匹配

### 未定义行为预防

1. **空指针检查**:
   - 当前实现未检查空指针
   - 生产版本应添加检查
   - 或使用 `Option<&[T]>` 语义

2. **整数溢出**:
   - Zig 使用 wrapping 语义 (`+%`)
   - Rust 在 debug 模式检查溢出
   - 需要明确文档说明行为差异

3. **并发安全**:
   - Zig 代码不应使用全局可变状态
   - 如需共享状态，使用 Rust 的并发原语
   - 通过接口传递状态指针

## 🚀 未来改进方向

### 短期目标

1. **完善安全包装器生成**:
   - 自动处理 `&str` → `(ptr, len)` 转换
   - 支持 `&[T]` 的泛型转换
   - 处理返回值的生命周期

2. **改进错误处理**:
   - 更好的编译错误消息
   - Zig 编译错误的友好显示
   - 源码位置追踪

3. **测试基础设施**:
   - 集成测试套件
   - 自动化测试 Zig 编译
   - 跨平台测试

### 中期目标

1. **高级类型支持**:
   - Zig 结构体 ↔ Rust 结构体
   - 枚举类型映射
   - 可选类型 (`?T` ↔ `Option<T>`)

2. **性能优化**:
   - 缓存 Zig 编译结果
   - 增量编译支持
   - 并行构建多个 autozig! 宏

3. **更好的IDE支持**:
   - rust-analyzer 集成
   - Zig 代码的语法高亮
   - 跳转到定义

### 长期目标

1. **异步支持**:
   - Zig 异步函数 ↔ Rust async/await
   - 回调函数支持
   - 事件循环集成

2. **动态链接选项**:
   - 支持生成 .so/.dylib
   - 插件系统
   - 热重载

3. **标准库集成**:
   - 预定义的常用 Zig 标准库绑定
   - 数据结构互操作
   - 字符串处理工具

## 🐛 已知限制

### 当前版本限制

1. **语法解析**:
   - 使用简单正则表达式，不处理嵌套大括号
   - 可能错误匹配注释中的 `autozig!`
   - **解决方案**: 使用 syn 完整解析

2. **类型系统**:
   - 仅支持基本类型
   - 不支持复杂结构体
   - 不支持泛型
   - **解决方案**: 逐步添加类型支持

3. **错误处理**:
   - Zig 错误无法传递到 Rust
   - 必须在 Zig 侧处理所有错误
   - **解决方案**: 设计错误传递机制

4. **并发**:
   - 未测试多线程环境
   - Zig 全局状态可能不安全
   - **解决方案**: 添加线程安全文档

### 平台限制

1. **依赖 Zig 编译器**:
   - 需要用户安装 Zig
   - 版本兼容性未充分测试
   - **建议**: 锁定 Zig 0.11 或 0.12

2. **交叉编译**:
   - 未测试交叉编译场景
   - 目标三元组映射可能不完整
   - **解决方案**: 添加目标映射表

## 📊 性能考虑

### 编译时间

**影响因素**:
1. Zig 编译器调用（每次构建 ~1-5 秒）
2. bindgen 运行（取决于头文件复杂度）
3. 源文件扫描（对大项目可能显著）

**优化策略**:
1. 缓存编译结果
2. 仅在 Zig 代码变化时重新编译
3. 并行处理多个 autozig! 宏

### 运行时性能

**FFI 开销**:
- 函数调用：极小（内联可能）
- 类型转换：几乎为零（仅指针操作）
- 无动态分配（静态链接）

**与直接 C FFI 比较**:
- **相同**: 都通过 C ABI 调用
- **优势**: Zig 的安全特性（边界检查等）
- **劣势**: 无（性能相当）

## 🧪 测试策略

### 单元测试

**parser 测试**:
```rust
#[test]
fn test_parse_zig_only() {
    let config: AutoZigConfig = parse_quote! {
        export fn add(a: i32, b: i32) i32 { return a + b; }
    };
    assert!(!config.zig_code.is_empty());
}
```

**type_mapper 测试**:
```rust
#[test]
fn test_type_mapping() {
    let mapper = TypeMapper::new();
    assert_eq!(mapper.map_type("i32"), Some("i32"));
}
```

### 集成测试

**完整流程测试**:
1. 创建临时项目
2. 写入测试代码
3. 运行 `cargo build`
4. 验证编译成功
5. 运行可执行文件
6. 验证输出正确

### 性能测试

**基准测试**:
- FFI 调用延迟
- 类型转换开销
- 编译时间测量

## 📖 参考资料

### 相关项目

1. **autocxx**: https://github.com/google/autocxx
   - 本项目的主要灵感来源
   - C++ FFI 的类似方法

2. **bindgen**: https://github.com/rust-lang/rust-bindgen
   - Rust FFI 绑定生成器
   - AutoZig 的核心依赖

3. **cxx**: https://github.com/dtolnay/cxx
   - 安全的 C++ FFI
   - 不同的设计理念

4. **Zig**: https://ziglang.org/
   - Zig 语言官网
   - C 互操作文档

### 技术文档

1. **Zig C ABI**: https://ziglang.org/documentation/master/#C
2. **Rust FFI**: https://doc.rust-lang.org/nomicon/ffi.html
3. **Procedural Macros**: https://doc.rust-lang.org/reference/procedural-macros.html

## 🤝 贡献指南

### 代码风格

- 遵循 Rust 标准风格（`rustfmt`）
- 所有公共 API 必须有文档注释
- 使用 `#![forbid(unsafe_code)]`（除非绝对必要）

### 提交流程

1. Fork 项目
2. 创建功能分支
3. 编写测试
4. 确保所有测试通过
5. 提交 Pull Request

### 测试要求

- 所有新功能必须有测试
- 保持测试覆盖率 > 80%
- 集成测试验证端到端流程

## 📝 总结

AutoZig 通过三阶段编译流水线实现了 Rust 和 Zig 之间的安全互操作：

1. **解析阶段**: 提取和解析 Zig 代码及 Rust 签名
2. **构建阶段**: 编译 Zig 为静态库并生成 FFI 绑定
3. **宏展开阶段**: 生成安全的 Rust 包装器

关键设计决策：
- ✅ 静态链接（简单、可移植）
- ✅ build.rs 驱动（时序正确）
- ✅ bindgen 生成绑定（成熟可靠）
- ✅ 可选安全包装器（灵活实用）

项目目前处于实验阶段，适合：
- 探索 Rust-Zig 互操作
- 性能关键的小型模块
- 学习 FFI 和构建系统

不适合：
- 生产环境（尚未充分测试）
- 复杂类型互操作（功能有限）
- 需要稳定 API（仍在演进）

---

**版本**: 0.1.0
**最后更新**: 2024-01-04
**维护者**: AutoZig Contributors