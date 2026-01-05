
# AutoZig Trait 支持设计文档

## 🎯 目标

让 Zig 代码能够实现 Rust Trait，使其成为 Rust 生态的"原生公民"。

## 💡 核心价值

### 1. **生态融入**
```rust
// Zig 实现的哈希算法可以直接用于 HashMap
let mut map: HashMap<String, i32, BuildHasherZig> = HashMap::default();
```

### 2. **零感替换**
```rust
// 用户不知道底层是 Zig，只看到标准 Trait
fn process<T: Serialize>(data: T) { /* ... */ }
process(MyZigStruct::new());  // Just works!
```

### 3. **依赖注入**
```rust
trait Compressor { fn compress(&mut self, data: &[u8]) -> Vec<u8>; }
struct ZigCompressor;  // 高性能 Zig 实现
struct RustCompressor; // 纯 Rust 实现
// 运行时切换，零开销抽象
```

## 📐 分阶段实现方案

### Phase 1: 无状态 Trait（Stateless / ZST）

#### 适用场景
- 纯算法库（数学计算、编码解码）
- 无需维护内部状态的工具函数
- 标记 Trait（Marker Traits）

#### 语法设计

```rust
use autozig::prelude::*;

autozig! {
    // Zig 侧：纯函数实现
    export fn zig_add(a: i32, b: i32) i32 {
        return a + b;
    }
    
    export fn zig_multiply(a: i32, b: i32) i32 {
        return a * b;
    }
    
    ---
    
    // Rust 侧：Trait 映射
    #[derive(Default)]
    struct ZigCalculator;
    
    impl Calculator for ZigCalculator {
        fn add(&self, a: i32, b: i32) -> i32 {
            zig_add(a, b)
        }
        
        fn multiply(&self, a: i32, b: i32) -> i32 {
            zig_multiply(a, b)
        }
    }
}
```

#### 实现复杂度
- **难度**: ⭐⭐☆☆☆
- **风险**: 低（无内存管理）
- **价值**: 中（适用场景有限）

#### 实现要点
1. 解析 `impl Trait for Struct` 块
2. 提取方法签名和对应的 Zig 函数名
3. 生成转发代码（forwarding code）
4. 确保 `self` 参数被正确忽略（ZST 优化）

---

### Phase 2: 有状态 Trait（Stateful / Opaque Pointer）

#### 适用场景
- 需要维护内部状态的算法（哈希、压缩、加密）
- 流式处理（Reader, Writer, Iterator）
- 复杂状态机

#### 语法设计

```rust
use autozig::prelude::*;
use std::hash::Hasher;

autozig! {
    const std = @import("std");

    // === Zig 侧：状态管理 ===
    
    const ZigHasherState = struct {
        sum: u64,
        count: u64,
    };

    export fn hasher_new() *ZigHasherState {
        const ptr = std.heap.c_allocator.create(ZigHasherState) catch @panic("OOM");
        ptr.* = .{ .sum = 0, .count = 0 };
        return ptr;
    }

    export fn hasher_free(ptr: *ZigHasherState) void {
        std.heap.c_allocator.destroy(ptr);
    }

    export fn hasher_write(ptr: *ZigHasherState, buf_ptr: [*]const u8, len: usize) void {
        const slice = buf_ptr[0..len];
        for (slice) |b| {
            ptr.sum +%= b;
            ptr.count += 1;
        }
    }

    export fn hasher_finish(ptr: *const ZigHasherState) u64 {
        return ptr.sum +% ptr.count;
    }

    ---

    // === Rust 侧：Trait 映射（带状态） ===
    
    #[opaque_pointer(
        constructor = hasher_new,
        destructor = hasher_free
    )]
    struct ZigHasher;

    impl std::hash::Hasher for ZigHasher {
        #[map_method(hasher_write)]
        fn write(&mut self, bytes: &[u8]);
        
        #[map_method(hasher_finish)]
        fn finish(&self) -> u64;
    }
}
```

#### 宏展开后的代码

```rust
pub struct ZigHasher {
    ptr: *mut std::ffi::c_void,
}

impl ZigHasher {
    pub fn new() -> Self {
        unsafe {
            Self { ptr: hasher_new() as *mut std::ffi::c_void }
        }
    }
}

impl Drop for ZigHasher {
    fn drop(&mut self) {
        unsafe {
            hasher_free(self.ptr as *mut _);
        }
    }
}

impl std::hash::Hasher for ZigHasher {
    fn write(&mut self, bytes: &[u8]) {
        unsafe {
            hasher_write(
                self.ptr as *mut _,
                bytes.as_ptr(),
                bytes.len()
            );
        }
    }
    
    fn finish(&self) -> u64 {
        unsafe {
            hasher_finish(self.ptr as *const _)
        }
    }
}

// 自动实现线程安全性标记（如果需要）
// unsafe impl Send for ZigHasher {}
// unsafe impl Sync for ZigHasher {}
```

#### 实现复杂度
- **难度**: ⭐⭐⭐⭐⭐
- **风险**: 高（内存管理、生命周期）
- **价值**: 极高（完全融入 Rust 生态）

#### 实现要点

1. **Opaque Pointer 管理**
   - 自动生成 `ptr: *mut c_void` 字段
   - 自动生成 `Drop` 实现
   - 处理 `&self` vs `&mut self` 的指针类型转换

2. **方法映射**
   ```rust
   // Rust:    fn write(&mut self, bytes: &[u8])
   // Zig:     fn write(ptr: *State, buf: [*]const u8, len: usize)
   // 转换:    zig_write(self.ptr, bytes.as_ptr(), bytes.len())
   ```

3. **生命周期安全**
   - 确保 Zig 分配器和 Rust Drop 对齐
   - 防止 Double Free
   - 防止 Use-After-Free

4. **线程安全**
   - 根据 Zig 实现决定是否实现 `Send`/`Sync`
   - 需要用户显式标记（默认不安全）

---

## 🧠 核心技术挑战

### 1. 内存管理对齐

**问题**: Zig 使用 `c_allocator`，Rust 使用 `Global` allocator

**解决方案**:
```zig
// 统一使用 C 分配器
const allocator = std.heap.c_allocator;

export fn create_state() *State {
    return allocator.create(State) catch @panic("OOM");
}

export fn destroy_state(ptr: *State) void {
    allocator.destroy(ptr);
}
```

### 2. Self 参数映射

| Rust Signature | Zig Signature | 转换规则 |
|----------------|---------------|----------|
| `&self` | `ptr: *const State` | `self.ptr as *const _` |
| `&mut self` | `ptr: *mut State` | `self.ptr as *mut _` |
| 无 self (ZST) | 无参数 | 直接调用 |

### 3. 类型转换自动化

```rust
// Rust:  &[u8]
// Zig:   [*]const u8, usize
// 宏自动展开为: slice.as_ptr(), slice.len()

// Rust:  &str
// Zig:   [*]const u8, usize
// 宏自动展开为: s.as_ptr(), s.len()

// Rust:  &mut [u8]
// Zig:   [*]u8, usize
// 宏自动展开为: slice.as_mut_ptr(), slice.len()
```

### 4. 错误处理

```rust
// Zig 返回错误联合类型
export fn fallible_operation(ptr: *State) !void {
    // ...
}

// Rust Trait 方法返回 Result
impl MyTrait for ZigWrapper {
    fn operation(&mut self) -> Result<(), Error> {
        // 宏需要处理 Zig 的错误码 -> Rust Result 转换
    }
}
```

---

## 🎓 示例：完整的 std::hash::Hasher 实现

### 文件: `examples/trait_hasher/src/main.rs`

```rust
use autozig::prelude::*;
use std::hash::{Hash, Hasher};
use std::collections::HashMap;

autozig! {
    const std = @import("std");

    const ZigHasherState = struct {
        sum: u64,
        count: u64,
    };

    export fn hasher_new() *ZigHasherState {
        const ptr = std.heap.c_allocator.create(ZigHasherState) catch @panic("OOM");
        ptr.* = .{ .sum = 5381, .count = 0 };  // DJB2 magic number
        return ptr;
    }

    export fn hasher_free(ptr: *ZigHasherState) void {
        std.heap.c_allocator.destroy(ptr);
    }

    export fn hasher_write(ptr: *ZigHasherState, buf_ptr: [*]const u8, len: usize) void {
        const slice = buf_ptr[0..len];
        for (slice) |b| {
            ptr.sum = ((ptr.sum << 5) +% ptr.sum) +% b;  // hash * 33 + byte
            ptr.count += 1;
        }
    }

    export fn hasher_finish(ptr: *const ZigHasherState) u64 {
        return ptr.sum;
    }

    ---

    #[opaque_pointer(
        constructor = hasher_new,
        destructor = hasher_free
    )]
    struct ZigHasher;

    impl std::hash::Hasher for ZigHasher {
        #[map_method(hasher_write)]
        fn write(&mut self, bytes: &[u8]);
        
        #[map_method(hasher_finish)]
        fn finish(&self) -> u64;
    }
}

fn main() {
    println!("=== AutoZig Trait Support: std::hash::Hasher ===\n");

    // 1. 直接使用
    let mut hasher = ZigHasher::new();
    hasher.write(b"Hello");
    hasher.write(b"World");
    println!("Hash (direct): {}", hasher.finish());

    // 2. 通过 Trait 对象使用（动态分发）
    let mut hasher: Box<dyn Hasher> = Box::new(ZigHasher::new());
    "Hello".hash(&mut *hasher);
    println!("Hash (trait object): {}", hasher.finish());

    // 3. 泛型约束（静态分发）
    generic_hash(&mut ZigHasher::new(), b"Generic");

    // 4. 与标准库集成：BuildHasher
    let mut map = HashMap::with_hasher(ZigHasherBuilder::default());
    map.insert("key1", 42);
    map.insert("key2", 100);
    println!("\nHashMap with ZigHasher:");
    for (k, v) in &map {
        println!("  {} => {}", k, v);
    }
}

fn generic_hash<H: Hasher>(hasher: &mut H, data: &[u8]) {
    hasher.write(data);
    println!("Hash (generic): {}", hasher.finish());
}

// BuildHasher 实现（用于 HashMap）
#[derive(Default)]
struct ZigHasherBuilder;

impl std::hash::BuildHasher for ZigHasherBuilder {
    type Hasher = ZigHasher;
    
    fn build_hasher(&self) -> Self::Hasher {
        ZigHasher::new()
    }
}
```

---

## 🚀 实现路线图

### Milestone 1: Parser 扩展 (1-2 days)
- [ ] 解析 `#[opaque_pointer]` 属性
- [ ] 解析 `impl Trait for Struct` 块
- [ ] 解析 `#[map_method]` 属性
- [ ] 提取方法签名和映射关系

### Milestone 