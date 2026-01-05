# Phase 3: 泛型支持设计文档

## 1. 概述

为 AutoZig 添加泛型函数的 FFI 绑定支持，使用单态化（Monomorphization）策略生成特定类型的 FFI 绑定。

## 2. 设计目标

- 支持泛型函数的 IDL 声明：`fn process<T>(data: &[T]) -> usize`
- Parser 识别泛型参数和约束
- Macro 自动为常见类型生成单态化版本
- 初期支持基本类型：i32, i64, u32, u64, f32, f64, u8

## 3. 架构设计

### 3.1 Parser 层扩展

在 `parser/src/lib.rs` 中添加：

```rust
/// 泛型参数定义
#[derive(Debug, Clone)]
pub struct GenericParam {
    pub name: String,              // 参数名称，如 "T"
    pub bounds: Vec<TypeBound>,    // 约束条件
}

/// 类型约束
#[derive(Debug, Clone)]
pub enum TypeBound {
    Trait(String),                 // trait 约束，如 "Copy"
    Lifetime(String),              // 生命周期约束
}

/// 扩展 RustFunctionSignature
pub struct RustFunctionSignature {
    pub sig: Signature,
    pub generic_params: Vec<GenericParam>,  // 新增
}
```

### 3.2 Macro 层扩展

在 `macro/src/lib.rs` 中添加单态化逻辑：

```rust
/// 泛型单态化配置
const MONOMORPHIZE_TYPES: &[&str] = &["i32", "i64", "u32", "u64", "f32", "f64", "u8"];

/// 为泛型函数生成单态化版本
fn generate_monomorphized_functions(
    config: &AutoZigConfig
) -> Vec<MonomorphizedFunction> {
    // 对每个泛型函数，生成所有目标类型的版本
}
```

### 3.3 命名约定

单态化函数命名：
- 原函数：`fn process<T>(data: &[T]) -> usize`
- 单态化：
  - `process_i32(data: &[i32]) -> usize`
  - `process_f64(data: &[f64]) -> usize`
  - ...

Zig 侧对应：
- `export fn process_i32(data_ptr: [*]const i32, data_len: usize) usize`
- `export fn process_f64(data_ptr: [*]const f64, data_len: usize) usize`

## 4. 实现步骤

### Phase 3.1: Parser 泛型识别
1. 扩展 `syn::Signature` 解析，提取泛型参数
2. 添加 `GenericParam` 和 `TypeBound` 结构
3. 测试泛型函数签名解析

### Phase 3.2: Macro 单态化生成
1. 实现类型替换逻辑
2. 为每个目标类型生成 FFI 声明
3. 生成对应的安全包装器
4. 添加文档注释说明单态化

### Phase 3.3: 示例项目
1. 创建 `examples/generics/`
2. 实现泛型数组处理示例
3. 演示多种类型的单态化

## 5. 使用示例

```rust
use autozig::prelude::*;

autozig! {
    // Zig 泛型实现（通过单态化）
    export fn sum_i32(ptr: [*]const i32, len: usize) i32 {
        var sum: i32 = 0;
        for (ptr[0..len]) |val| sum += val;
        return sum;
    }
    
    export fn sum_f64(ptr: [*]const f64, len: usize) f64 {
        var sum: f64 = 0.0;
        for (ptr[0..len]) |val| sum += val;
        return sum;
    }
    
    ---
    
    // Rust 泛型声明（自动单态化）
    #[monomorphize(i32, f64)]
    fn sum<T>(data: &[T]) -> T;
}

fn main() {
    let ints = vec![1, 2, 3, 4, 5];
    let sum_int = sum_i32(&ints);  // 单态化版本
    
    let floats = vec![1.0, 2.0, 3.0];
    let sum_float = sum_f64(&floats);
}
```

## 6. 限制和未来工作

### 当前限制
- 仅支持基本类型泛型
- 不支持泛型结构体
- 不支持复杂约束（where 子句）

### 未来扩展
- 支持用户自定义单态化类型列表
- 支持泛型结构体
- 智能类型推导减少显式单态化