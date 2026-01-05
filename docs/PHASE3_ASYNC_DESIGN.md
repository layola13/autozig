# Phase 3: 异步支持设计文档

## 1. 概述

为 AutoZig 添加异步函数的 FFI 绑定支持，实现 Rust async/await 与 Zig 异步代码的互操作。

## 2. 设计目标

- 支持异步函数：`async fn process(data: &[u8]) -> Result<Vec<u8>, Error>`
- 实现 Future 包装器跨 FFI 边界
- 集成 Tokio 或 async-std 运行时
- 提供错误处理和取消支持

## 3. 架构设计

### 3.1 异步模型选择

**方案 A：回调模式（推荐）**
- Zig 端接收回调函数指针
- 完成时调用回调传递结果
- 简单、跨平台、无运行时依赖

**方案 B：Future 轮询**
- 实现自定义 Future 类型
- 通过 FFI 轮询 Zig 状态
- 复杂但更符合 Rust 惯例

初期实现方案 A。

### 3.2 FFI 边界设计

```rust
// Rust 侧
pub struct AsyncHandle {
    handle: *mut c_void,
    runtime: Arc<Runtime>,
}

// C FFI 回调类型
type AsyncCallback = extern "C" fn(
    user_data: *mut c_void,
    result_ptr: *const u8,
    result_len: usize,
    error_code: i32,
);

// Zig 侧
export fn async_process(
    data_ptr: [*]const u8,
    data_len: usize,
    callback: AsyncCallback,
    user_data: *mut c_void,
) void {
    // 异步处理，完成后调用 callback
}
```

### 3.3 Parser 层扩展

```rust
/// 异步函数签名
#[derive(Debug, Clone)]
pub struct AsyncFunctionSignature {
    pub sig: Signature,
    pub is_async: bool,
    pub error_type: Option<syn::Type>,
}
```

### 3.4 Macro 层扩展

```rust
/// 生成异步包装器
fn generate_async_wrapper(sig: &AsyncFunctionSignature) -> TokenStream {
    // 创建 Future 实现
    // 设置回调
    // 处理结果和错误
}
```

## 4. 实现策略

### 4.1 运行时选择

支持两种运行时（通过 feature flags）：
- `tokio` (default)
- `async-std`

```toml
[features]
default = ["async", "tokio-runtime"]
async = []
tokio-runtime = ["tokio"]
async-std-runtime = ["async-std"]
```

### 4.2 错误处理

```rust
#[derive(Debug)]
pub enum AsyncError {
    ZigError(i32),
    Cancelled,
    Timeout,
}

pub type AsyncResult<T> = Result<T, AsyncError>;
```

### 4.3 取消支持

```rust
pub struct CancellableAsyncHandle {
    handle: AsyncHandle,
    cancel_token: CancellationToken,
}

impl Drop for CancellableAsyncHandle {
    fn drop(&mut self) {
        // 通知 Zig 侧取消操作
        unsafe { zig_cancel(self.handle.handle); }
    }
}
```

## 5. 使用示例

```rust
use autozig::prelude::*;

autozig! {
    // Zig 异步实现
    const std = @import("std");
    
    const Callback = *const fn (*anyopaque, [*]const u8, usize, i32) callconv(.C) void;
    
    export fn async_hash(
        data_ptr: [*]const u8,
        data_len: usize,
        callback: Callback,
        user_data: *anyopaque,
    ) void {
        const data = data_ptr[0..data_len];
        
        // 模拟异步计算
        var hash: u64 = 0;
        for (data) |byte| {
            hash +%= byte;
        }
        
        // 调用回调
        const result = @ptrCast([*]const u8, &hash);
        callback(user_data, result, 8, 0);
    }
    
    ---
    
    // Rust 异步接口
    async fn async_hash(data: &[u8]) -> Result<u64, AsyncError>;
}

#[tokio::main]
async fn main() {
    let data = b"Hello Async AutoZig";
    
    match async_hash(data).await {
        Ok(hash) => println!("Hash: {}", hash),
        Err(e) => eprintln!("Error: {:?}", e),
    }
}
```

## 6. 实现步骤

### Phase 3.4: Parser 异步识别
1. 检测 `async fn` 关键字
2. 提取 Result 类型的错误类型
3. 标记异步函数签名

### Phase 3.5: Macro 异步包装生成
1. 实现回调桥接逻辑
2. 创建 Future 包装器
3. 添加运行时集成
4. 实现错误转换

### Phase 3.6: 示例项目
1. 创建 `examples/async/`
2. 演示异步数据处理
3. 展示错误处理和取消

## 7. 性能考虑

- 回调开销：约 100ns per call
- Future 创建：约 1μs
- 适合 I/O 密集型操作（> 1ms）
- 不适合极高频调用（> 100k/s）

## 8. 限制和未来工作

### 当前限制
- 仅支持单个 Result 返回值
- 回调模式（非 Stream）
- 需要运行时支持

### 未来扩展
- 支持 Stream/AsyncIterator
- 零开销异步（无回调）
- Zig 原生 async/await 集成