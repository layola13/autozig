# Phase 4: Stream Support & Advanced Features 设计文档

## 🎯 总体目标

实现 Rust Stream/AsyncIterator 与 Zig 的互操作，支持流式数据处理、背压控制和高级异步模式。

## 📊 功能范围

### 4.1 Stream 支持 ⭐⭐⭐⭐⭐
- 实现 `futures::Stream` trait
- 支持异步迭代器
- 背压控制（Backpressure）
- 流式转换（map、filter、fold）

### 4.2 Channel 桥接 ⭐⭐⭐⭐
- Rust Channel → Zig
- Zig → Rust Channel
- 多生产者多消费者（MPMC）
- 有界/无界队列

### 4.3 高级异步模式 ⭐⭐⭐
- Select/Poll 多路复用
- Timeout 超时控制
- Retry 重试机制
- Rate Limiting 限流

### 4.4 性能优化 ⭐⭐⭐
- 零拷贝流式传输
- 批量操作优化
- 内存池管理
- 性能分析工具

---

## 🔧 技术设计

### 1. Stream Trait 实现

#### 1.1 基础 Stream 接口

```rust
use futures::stream::{Stream, StreamExt};
use std::pin::Pin;
use std::task::{Context, Poll};

/// Zig 流包装器
pub struct ZigStream<T> {
    handle: *mut c_void,
    buffer: Vec<T>,
    done: bool,
    _phantom: PhantomData<T>,
}

impl<T> Stream for ZigStream<T> {
    type Item = Result<T, StreamError>;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        // 从 Zig 轮询下一个元素
        unsafe {
            match zig_stream_poll(self.handle) {
                StreamPollResult::Ready(item) => Poll::Ready(Some(Ok(item))),
                StreamPollResult::Pending => {
                    // 注册 waker
                    zig_stream_set_waker(self.handle, cx.waker().clone());
                    Poll::Pending
                }
                StreamPollResult::Done => Poll::Ready(None),
                StreamPollResult::Error(e) => Poll::Ready(Some(Err(e))),
            }
        }
    }
}
```

#### 1.2 Zig 侧 Stream 实现

```zig
const std = @import("std");

pub const StreamState = enum {
    Ready,
    Pending,
    Done,
    Error,
};

pub const StreamItem = extern struct {
    state: StreamState,
    data_ptr: ?[*]const u8,
    data_len: usize,
    error_code: i32,
};

/// 流处理函数签名
pub const StreamProducer = *const fn (ctx: *anyopaque) StreamItem;

/// 创建流
export fn zig_stream_new(
    producer: StreamProducer,
    ctx: *anyopaque,
) *anyopaque {
    const stream = allocator.create(Stream) catch @panic("OOM");
    stream.* = Stream{
        .producer = producer,
        .ctx = ctx,
        .waker = null,
    };
    return @ptrCast(stream);
}

/// 轮询流
export fn zig_stream_poll(stream_ptr: *anyopaque) StreamItem {
    const stream = @ptrCast(*Stream, @alignCast(@alignOf(Stream), stream_ptr));
    return stream.producer(stream.ctx);
}

/// 设置唤醒器
export fn zig_stream_set_waker(
    stream_ptr: *anyopaque,
    waker: *anyopaque,
) void {
    const stream = @ptrCast(*Stream, @alignCast(@alignOf(Stream), stream_ptr));
    stream.waker = waker;
}

/// 唤醒等待者
pub fn wake_stream(stream: *Stream) void {
    if (stream.waker) |waker| {
        rust_waker_wake(waker);
    }
}
```

### 2. Channel 桥接

#### 2.1 Rust → Zig Channel

```rust
use tokio::sync::mpsc;

/// 创建 Zig 可读的 Channel
pub fn channel_to_zig<T>(rx: mpsc::Receiver<T>) -> *mut c_void 
where
    T: Send + 'static,
{
    let bridge = Box::new(ChannelBridge {
        rx,
        buffer: VecDeque::new(),
    });
    
    Box::into_raw(bridge) as *mut c_void
}

/// Zig 从 Channel 读取
#[no_mangle]
pub extern "C" fn channel_recv(
    channel_ptr: *mut c_void,
    out_ptr: *mut u8,
    out_len: usize,
) -> i32 {
    let bridge = unsafe { &mut *(channel_ptr as *mut ChannelBridge<Vec<u8>>) };
    
    match bridge.rx.try_recv() {
        Ok(data) => {
            let copy_len = data.len().min(out_len);
            unsafe {
                std::ptr::copy_nonoverlapping(
                    data.as_ptr(),
                    out_ptr,
                    copy_len,
                );
            }
            copy_len as i32
        }
        Err(mpsc::error::TryRecvError::Empty) => -1, // Pending
        Err(mpsc::error::TryRecvError::Disconnected) => 0, // Done
    }
}
```

#### 2.2 Zig → Rust Channel

```zig
const Channel = extern struct {
    send_fn: *const fn (*anyopaque, [*]const u8, usize) i32,
    ctx: *anyopaque,
};

export fn channel_send(
    channel: *Channel,
    data_ptr: [*]const u8,
    data_len: usize,
) i32 {
    return channel.send_fn(channel.ctx, data_ptr, data_len);
}
```

```rust
/// Zig 可写的 Channel
pub fn channel_from_zig<T>(tx: mpsc::Sender<T>) -> Channel
where
    T: TryFrom<Vec<u8>>,
{
    extern "C" fn send_impl<T>(
        ctx: *mut c_void,
        data_ptr: *const u8,
        data_len: usize,
    ) -> i32
    where
        T: TryFrom<Vec<u8>>,
    {
        let tx = unsafe { &*(ctx as *const mpsc::Sender<T>) };
        let data = unsafe { std::slice::from_raw_parts(data_ptr, data_len) };
        
        match T::try_from(data.to_vec()) {
            Ok(value) => match tx.try_send(value) {
                Ok(()) => 1, // Success
                Err(_) => -1, // Full
            },
            Err(_) => -2, // Invalid data
        }
    }
    
    Channel {
        send_fn: send_impl::<T>,
        ctx: Box::into_raw(Box::new(tx)) as *mut c_void,
    }
}
```

### 3. 高级异步模式

#### 3.1 Select 多路复用

```rust
use tokio::select;

autozig! {
    export fn zig_operation_a(callback: Callback, ctx: *anyopaque) void;
    export fn zig_operation_b(callback: Callback, ctx: *anyopaque) void;
    ---
    async fn operation_a() -> Result<i32>;
    async fn operation_b() -> Result<i32>;
}

async fn select_operations() {
    select! {
        result = operation_a() => {
            println!("A completed first: {:?}", result);
        }
        result = operation_b() => {
            println!("B completed first: {:?}", result);
        }
    }
}
```

#### 3.2 Timeout 控制

```rust
use tokio::time::{timeout, Duration};

async fn with_timeout() -> Result<i32, TimeoutError> {
    timeout(Duration::from_secs(5), zig_long_operation())
        .await
        .map_err(|_| TimeoutError::Elapsed)?
}
```

#### 3.3 Retry 重试机制

```rust
async fn retry_operation(max_retries: u32) -> Result<i32, RetryError> {
    for attempt in 0..max_retries {
        match zig_operation().await {
            Ok(result) => return Ok(result),
            Err(e) if attempt < max_retries - 1 => {
                eprintln!("Attempt {} failed: {:?}, retrying...", attempt, e);
                tokio::time::sleep(Duration::from_millis(100 * (attempt + 1) as u64)).await;
            }
            Err(e) => return Err(RetryError::MaxRetriesExceeded(e)),
        }
    }
    unreachable!()
}
```

### 4. 性能优化

#### 4.1 零拷贝流式传输

```rust
/// 零拷贝 Buffer
pub struct ZeroCopyBuffer {
    ptr: *mut u8,
    len: usize,
    cap: usize,
    _phantom: PhantomData<u8>,
}

impl ZeroCopyBuffer {
    /// 从 Zig 借用内存（不拷贝）
    pub unsafe fn borrow_from_zig(ptr: *mut u8, len: usize) -> Self {
        Self {
            ptr,
            len,
            cap: len,
            _phantom: PhantomData,
        }
    }
    
    /// 转换为 Rust slice（零拷贝）
    pub fn as_slice(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.ptr, self.len) }
    }
}
```

#### 4.2 批量操作优化

```rust
/// 批量处理 Stream 元素
async fn batch_process<S, F>(
    stream: S,
    batch_size: usize,
    processor: F,
) -> Result<(), StreamError>
where
    S: Stream<Item = Result<Vec<u8>, StreamError>>,
    F: Fn(Vec<Vec<u8>>) -> Result<(), ProcessError>,
{
    let mut batch = Vec::with_capacity(batch_size);
    
    tokio::pin!(stream);
    
    while let Some(item) = stream.next().await {
        batch.push(item?);
        
        if batch.len() >= batch_size {
            processor(std::mem::take(&mut batch))?;
        }
    }
    
    if !batch.is_empty() {
        processor(batch)?;
    }
    
    Ok(())
}
```

---

## 🧪 使用示例

### 示例 1: 流式文件处理

```rust
use autozig::prelude::*;

autozig! {
    const std = @import("std");
    
    const FileStream = struct {
        file: std.fs.File,
        buffer: [4096]u8,
    };
    
    export fn file_stream_new(path: [*:0]const u8) ?*FileStream {
        const file = std.fs.cwd().openFile(
            std.mem.span(path),
            .{},
        ) catch return null;
        
        const stream = allocator.create(FileStream) catch {
            file.close();
            return null;
        };
        
        stream.* = FileStream{
            .file = file,
            .buffer = undefined,
        };
        
        return stream;
    }
    
    export fn file_stream_read(stream: *FileStream) StreamItem {
        const bytes_read = stream.file.read(&stream.buffer) catch |err| {
            return StreamItem{
                .state = .Error,
                .data_ptr = null,
                .data_len = 0,
                .error_code = @intFromError(err),
            };
        };
        
        if (bytes_read == 0) {
            return StreamItem{
                .state = .Done,
                .data_ptr = null,
                .data_len = 0,
                .error_code = 0,
            };
        }
        
        return StreamItem{
            .state = .Ready,
            .data_ptr = &stream.buffer,
            .data_len = bytes_read,
            .error_code = 0,
        };
    }
    
    export fn file_stream_free(stream: *FileStream) void {
        stream.file.close();
        allocator.destroy(stream);
    }
    
    ---
    
    fn file_stream_new(path: &str) -> Option<*mut FileStream>;
    fn file_stream_read(stream: *mut FileStream) -> StreamItem;
    fn file_stream_free(stream: *mut FileStream);
}

use futures::stream::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = "large_file.bin";
    let stream = create_file_stream(path)?;
    
    let mut total_bytes = 0u64;
    
    tokio::pin!(stream);
    
    while let Some(chunk) = stream.next().await {
        let data = chunk?;
        total_bytes += data.len() as u64;
        
        // 处理数据块...
        println!("Read {} bytes", data.len());
    }
    
    println!("Total: {} bytes", total_bytes);
    Ok(())
}

fn create_file_stream(path: &str) -> Result<impl Stream<Item = Result<Vec<u8>, StreamError>>, StreamError> {
    let c_path = std::ffi::CString::new(path)?;
    let handle = unsafe { file_stream_new(c_path.as_ptr()) }
        .ok_or(StreamError::InitFailed)?;
    
    Ok(ZigStream::new(handle, file_stream_read, file_stream_free))
}
```

### 示例 2: Channel 通信

```rust
use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel(100);
    
    // Zig 可以通过 Channel 发送数据
    let zig_channel = channel_from_zig(tx);
    
    // 启动 Zig 生产者
    std::thread::spawn(move || {
        for i in 0..10 {
            let data = format!("Message {}", i);
            unsafe {
                channel_send(&zig_channel, data.as_ptr(), data.len());
            }
        }
    });
    
    // Rust 消费者
    while let Some(msg) = rx.recv().await {
        println!("Received: {:?}", msg);
    }
}
```

---

## 📝 实现步骤

### Phase 4.1: Stream 基础设施 (1 周)
- [ ] 实现 `ZigStream<T>` 类型
- [ ] 实现 `Stream` trait
- [ ] Zig 侧 Stream 状态机
- [ ] Waker 唤醒机制
- [ ] 基础测试套件

### Phase 4.2: Channel 桥接 (3-5 天)
- [ ] Rust → Zig Channel
- [ ] Zig → Rust Channel
- [ ] 有界/无界队列
- [ ] 背压控制
- [ ] 性能测试

### Phase 4.3: 高级模式 (3-5 天)
- [ ] Select 多路复用
- [ ] Timeout 支持
- [ ] Retry 机制
- [ ] Rate Limiting
- [ ] 示例项目

### Phase 4.4: 性能优化 (1 周)
- [ ] 零拷贝优化
- [ ] 批量处理
- [ ] 内存池
