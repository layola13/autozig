# AutoZig 双重绑定功能说明

## 概述

AutoZig 现在支持通过 `#[autozig(...)]` 属性自动生成**双重绑定**：
1. **wasm-bindgen 绑定**：用于 JavaScript 调用（自动生成胶水代码）
2. **C 风格导出**：用于直接 WebAssembly API 调用（手动控制）

这消除了之前需要**手写两遍**相同函数的问题！

## 问题背景

在之前的实现中（见 `lib.rs`），每个函数都需要写两遍：

```rust
// 第一遍：wasm-bindgen 导出
#[wasm_bindgen]
pub fn wasm_get_memory_size() -> usize {
    get_memory_size()
}

// 第二遍：C 风格导出
#[no_mangle]
pub extern "C" fn wasm64_get_memory_size() -> usize {
    get_memory_size()
}
```

这导致了：
- **265 行代码**中有大量重复
- 维护成本高（修改一个函数需要改两处）
- 容易出错（忘记同步修改）

## 解决方案

使用新的 `#[autozig(...)]` 属性，只需声明一次：

```rust
include_zig!("src/wasm64.zig", {
    #[autozig(strategy = "dual")]
    fn get_memory_size() -> usize;
});
```

宏会**自动生成**两个版本：
- `wasm_get_memory_size()` - wasm-bindgen 版本
- `wasm64_get_memory_size()` - C 风格版本

## 配置选项

### 1. `strategy` - 导出策略

- `"dual"` (默认)：生成两种绑定
- `"bindgen"`：只生成 wasm-bindgen 绑定
- `"c_only"`：只生成 C 风格绑定

```rust
#[autozig(strategy = "dual")]
fn my_function() -> i32;
```

### 2. `prefix_bindgen` - wasm-bindgen 前缀

默认：`"wasm_"`

```rust
#[autozig(strategy = "dual", prefix_bindgen = "js_")]
fn my_function() -> i32;
// 生成：js_my_function() 和 wasm64_my_function()
```

### 3. `prefix_c` - C 风格前缀

默认：`"wasm64_"`

```rust
#[autozig(strategy = "dual", prefix_c = "native_")]
fn my_function() -> i32;
// 生成：wasm_my_function() 和 native_my_function()
```

### 4. `c_ret` + `map_fn` - 返回值类型转换

用于处理 **C ABI 不兼容的类型**（如指针、bool）

#### 示例 1：指针转整数

```rust
#[autozig(
    strategy = "dual",
    c_ret = "usize",
    map_fn = "|ptr| ptr as usize"
)]
fn alloc_large_buffer() -> *mut u8;
```

生成：
- `wasm_alloc_large_buffer()` → 返回 `*mut u8`（JavaScript 可以用）
- `wasm64_alloc_large_buffer()` → 返回 `usize`（C API 友好）

#### 示例 2：bool 转 u32

```rust
#[autozig(
    strategy = "dual",
    c_ret = "u32",
    map_fn = "|b: bool| if b { 1 } else { 0 }"
)]
fn write_at_high_address(value: u64) -> bool;
```

生成：
- `wasm_write_at_high_address()` → 返回 `bool`
- `wasm64_write_at_high_address()` → 返回 `u32` (0 或 1)

## 完整示例

查看 `lib_new.rs` 获取完整的使用示例。

### 之前：265 行重复代码

```rust
include_zig!("src/wasm64.zig", {
    fn get_memory_size() -> usize;
    fn grow_memory(delta: usize) -> isize;
    // ... 12 个函数
});

// 手动写 wasm-bindgen 包装器（~70 行）
#[wasm_bindgen]
pub fn wasm_get_memory_size() -> usize { get_memory_size() }

#[wasm_bindgen]
pub fn wasm_grow_memory(delta: usize) -> isize { grow_memory(delta) }
// ... 重复 12 次

// 手动写 C 风格包装器（~70 行）
#[no_mangle]
pub extern "C" fn wasm64_get_memory_size() -> usize { get_memory_size() }

#[no_mangle]
pub extern "C" fn wasm64_grow_memory(delta: usize) -> isize { grow_memory(delta) }
// ... 重复 12 次
```

### 之后：~60 行干净代码

```rust
include_zig!("src/wasm64.zig", {
    #[autozig(strategy = "dual")]
    fn get_memory_size() -> usize;
    
    #[autozig(strategy = "dual")]
    fn grow_memory(delta: usize) -> isize;
    
    #[autozig(strategy = "dual", c_ret = "usize", map_fn = "|ptr| ptr as usize")]
    fn alloc_large_buffer() -> *mut u8;
    
    #[autozig(strategy = "dual", c_ret = "u32", map_fn = "|b: bool| if b { 1 } else { 0 }")]
    fn write_at_high_address(value: u64) -> bool;
    
    // ... 其他函数
});

// 就这样！所有绑定都自动生成了！
```

## 代码量对比

| 版本 | 总行数 | include_zig! 部分 | 手动包装器 |
|------|--------|-------------------|------------|
| **旧版** (lib.rs) | 265 行 | 24 行 | 141 行（重复！） |
| **新版** (lib_new.rs) | 173 行 | 60 行 | 0 行（自动生成） |
| **减少** | **-92 行 (-35%)** | +36 行 | **-141 行 (-100%)** |

## 技术原理

### 三步生成过程

1. **内部导入**（隐藏实现细节）
   ```rust
   extern "C" {
       fn __autozig_internal_get_memory_size() -> usize;
   }
   ```

2. **wasm-bindgen 包装器**
   ```rust
   #[wasm_bindgen]
   pub fn wasm_get_memory_size() -> usize {
       unsafe { __autozig_internal_get_memory_size() }
   }
   ```

3. **C 风格包装器**
   ```rust
   #[no_mangle]
   pub extern "C" fn wasm64_get_memory_size() -> usize {
       unsafe { __autozig_internal_get_memory_size() }
   }
   ```

### 类型转换支持

对于不兼容 C ABI 的类型，宏会插入转换逻辑：

```rust
// C 风格包装器（带转换）
#[no_mangle]
pub extern "C" fn wasm64_alloc_large_buffer() -> usize {
    let res = unsafe { __autozig_internal_alloc_large_buffer() };
    let mapper = |ptr| ptr as usize;
    mapper(res)
}
```

## 优势

1. **消除重复**：函数只需声明一次
2. **类型安全**：编译时检查两种绑定的类型一致性
3. **灵活转换**：支持指针、bool 等类型的 ABI 转换
4. **可维护性**：修改一处，两种绑定自动同步
5. **向后兼容**：不使用 `#[autozig]` 的函数保持原有行为

## 迁移指南

### 步骤 1：添加属性

在 `include_zig!` 中的函数声明上添加 `#[autozig(strategy = "dual")]`

### 步骤 2：删除手动包装器

删除所有 `#[wasm_bindgen]` 和 `#[no_mangle]` 手动包装器

### 步骤 3：处理特殊类型

对于指针和 bool 类型，添加 `c_ret` 和 `map_fn` 参数

### 步骤 4：测试

运行测试确保生成的绑定工作正常

## 实现文件

- **Parser**: `parser/src/lib.rs` - 添加 `AutoZigBindingConfig` 结构体和属性解析
- **Macro**: `macro/src/lib.rs` - 添加 `generate_dual_binding_wrappers()` 函数
- **示例**: `examples/wasm64bit/src/lib_new.rs` - 完整使用示例

## 未来改进

1. 支持更多类型转换模板
2. 自动检测常见类型转换模式
3. 生成类型转换文档
4. IDE 支持（自动完成、类型提示）

## 总结

这不仅是一个优化，而是将 `autozig` 从一个简单的"链接器助手"升级为一个真正的**"FFI 绑定生成器"**！

通过使用 `#[autozig(...)]` 属性，您可以：
- ✅ 减少 35% 的代码量
- ✅ 消除 100% 的重复包装器
- ✅ 提高代码可维护性
- ✅ 保持类型安全
- ✅ 支持灵活的类型转换

立即试用新功能，享受更简洁的 Rust-Zig FFI 开发体验！