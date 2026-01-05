# AutoZig Trait 支持实现路线图

## 🎯 总体目标

让 Zig 代码能够实现 Rust Trait，使 AutoZig 从"FFI 工具"进化为"Rust 生态公民"。

## 📊 实现阶段

### Phase 1: 无状态 Trait (Stateless) ✅ **已完成**

**时间**: 2-3 天
**风险**: 低
**价值**: 中等
**完成日期**: 2026-01-05

#### 特点
- 零大小类型 (Zero-Sized Type)
- 无内存管理
- 纯算法实现

#### 语法示例
```rust
autozig! {
    export fn zig_add(a: i32, b: i32) i32 { return a + b; }
    ---
    struct ZigMath;
    impl Calculator for ZigMath {
        fn add(&self, a: i32, b: i32) -> i32 { zig_add(a, b) }
    }
}
```

#### 实现任务
- [x] Parser: 识别 `impl Trait for Struct` 语法
- [x] Parser: 提取方法签名和 Zig 函数映射
- [x] Macro: 生成 Trait impl 块
- [x] Macro: 自动忽略 `&self` 参数（ZST 优化）
- [x] Macro: 从 Zig 代码提取返回类型（修复 FFI 声明）
- [x] 创建 `examples/trait_calculator` 示例
- [x] 测试验证（7/7 测试通过）

---

### Phase 2: 有状态 Trait (Stateful with Opaque Pointer) ⭐⭐⭐⭐⭐

**时间**: 1-2 周  
**风险**: 高（内存管理、生命周期）  
**价值**: 极高

#### 特点
- 持有 Opaque Pointer (`*mut c_void`)
- 自动生成 `Drop` 实现
- 完整的状态管理

#### 语法示例
```rust
autozig! {
    export fn hasher_new() *State { /* ... */ }
    export fn hasher_free(ptr: *State) void { /* ... */ }
    export fn hasher_write(ptr: *mut State, data: [*]const u8, len: usize) void { /* ... */ }
    ---
    #[opaque_pointer(constructor = hasher_new, destructor = hasher_free)]
    struct ZigHasher;
    impl std::hash::Hasher for ZigHasher {
        #[map_method(hasher_write)]
        fn write(&mut self, bytes: &[u8]);
    }
}
```

#### 实现任务

**阶段 2.1: Parser 扩展**
- [ ] 解析 `#[opaque_pointer]` 属性
- [ ] 解析 `constructor` 和 `destructor` 参数
- [ ] 解析 `#[map_method]` 属性
- [ ] 验证方法签名兼容性

**阶段 2.2: 代码生成**
- [ ] 生成持有指针的 Struct
- [ ] 生成 `new()` 构造函数
- [ ] 生成 `Drop` 实现
- [ ] 生成 Trait impl 块
- [ ] 处理 `&self` vs `&mut self` 的指针转换

**阶段 2.3: 类型转换**
- [ ] 自动转换 `&[u8]` → `*const u8, usize`
- [ ] 自动转换 `&str` → `*const u8, usize`
- [ ] 自动转换 `&mut [u8]` → `*mut u8, usize`
- [ ] 处理返回值转换

**阶段 2.4: 安全性**
- [ ] 内存分配器对齐（Zig c_allocator）
- [ ] Double Free 防护
- [ ] Use-After-Free 检测
- [ ] 线程安全性分析（Send/Sync）

**阶段 2.5: 测试**
- [ ] 创建 `examples/trait_hasher` 示例
- [ ] 实现 `std::hash::Hasher`
- [ ] 集成到 `HashMap`
- [ ] 性能测试
- [ ] 内存泄漏测试（valgrind/miri）

---

## 🧪 示例项目规划

### Example 1: `trait_calculator` (Phase 1)

无状态数学计算器

```rust
trait Calculator {
    fn add(&self, a: i32, b: i32) -> i32;
    fn multiply(&self, a: i32, b: i32) -> i32;
}

struct ZigCalculator;  // ZST

impl Calculator for ZigCalculator {
    // 直接调用 Zig 函数，无状态
}
```

### Example 2: `trait_hasher` (Phase 2)

有状态哈希器

```rust
impl std::hash::Hasher for ZigHasher {
    fn write(&mut self, bytes: &[u8]);
    fn finish(&self) -> u64;
}

// 可用于 HashMap
let map: HashMap<String, i32, BuildHasherDefault<ZigHasher>> = HashMap::default();
```

### Example 3: `trait_reader` (Phase 2 Advanced)

实现 `std::io::Read`

```rust
impl std::io::Read for ZigReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize>;
}
```

---

## 🔧 技术挑战与解决方案

### 挑战 1: 方法签名映射

**问题**: Rust 的 `&self` 需要映射到 Zig 的指针

**解决方案**:
```rust
// Rust:    fn write(&mut self, bytes: &[u8])
// Zig:     fn write(ptr: *State, buf: [*]const u8, len: usize)
// 
// 宏生成:
// unsafe { zig_write(self.ptr as *mut _, bytes.as_ptr(), bytes.len()) }
```

### 挑战 2: 内存管理

**问题**: Zig 和 Rust 的分配器不同

**解决方案**:
```zig
// 统一使用 C 分配器
const allocator = std.heap.c_allocator;

export fn create() *State {
    return allocator.create(State) catch @panic("OOM");
}

export fn destroy(ptr: *State) void {
    allocator.destroy(ptr);
}
```

### 挑战 3: 生命周期安全

**问题**: 防止 Use-After-Free

**解决方案**:
```rust
// Rust 的 Drop 保证在对象销毁时调用
impl Drop for ZigHasher {
    fn drop(&mut self) {
        unsafe { hasher_free(self.ptr as *mut _); }
    }
}
```

### 挑战 4: 线程安全

**问题**: 判断是否实现 `Send`/`Sync`

**解决方案**:
```rust
// 默认不实现（保守）
// 用户显式标记
#[opaque_pointer(constructor = new, destructor = free, thread_safe)]
struct ZigHasher;

// 宏生成:
// unsafe impl Send for ZigHasher {}
// unsafe impl Sync for ZigHasher {}
```

---

## 📈 性能考虑

### 零成本抽象验证

```rust
// 1. 直接调用 Zig 函数
let result = zig_add(1, 2);

// 2. 通过 Trait 调用（静态分发）
let calc = ZigCalculator;
let result = calc.add(1, 2);

// 编译后应该生成相同的汇编代码（LLVM 内联优化）
```

### Trait 对象开销

```rust
// 动态分发（vtable）
let calc: Box<dyn Calculator> = Box::new(ZigCalculator);
let result = calc.add(1, 2);  // 一次间接调用
```

---

## 🎓 学习价值

此功能展示了：
1. **高级宏编程** - 解析复杂 Rust 语法
2. **FFI 设计模式** - Opaque Pointer 模式
3. **内存安全** - 跨语言生命周期管理
4. **零成本抽象** - Trait 编译时优化
5. **生态集成** - 标准库 Trait 实现

---

## 📝 决策：现在做还是之后做？

### 立即实现 Phase 1 的理由 ✅

1. **低风险高回报** - 实现简单，立即可用
2. **验证架构** - 测试 Parser 和 Macro 扩展能力
3. **用户反馈** - 尽早获取社区意见
4. **渐进式交付** - 分阶段发布功能

### 推迟 Phase 2 的理由 🔄

1. **复杂度高** - 需要精心设计内存管理
2. **测试充分** - 需要 miri/valgrind 验证
3. **文档完善** - 需要详细的安全性文档
4. **社区需求** - 先看 Phase 1 的使用情况

---

## 🚀 建议行动

### 立即行动（本周）

1. ✅ 创建设计文档（已完成）
2. 实现 Phase 1: 无状态 Trait
   - 扩展 Parser 识别 `impl Trait`
   - 扩展 Macro 生成 Trait impl
   - 创建 `examples/trait_calculator`
   - 验证测试

### 中期计划（1-2 周后）

1. 收集 Phase 1 的用户反馈
2. 设计 Phase 2 的详细 API
3. 实现 Opaque Pointer 支持
4. 创建 `examples/trait_hasher`

### 长期规划（1 个月后）

1. 完善文档和教程
2. 性能基准测试
3. 与 Rust 标准库 Trait 集成测试
4. 社区推广

---

## 📊 优先级评估

| 功能 | 优先级 | 难度 | 价值 | 建议 |
|------|--------|------|------|------|
| Phase 1 无状态 Trait | P0 | 低 | 中 | **立即实现** |
| Phase 2 有状态 Trait | P1 | 高 | 极高 | 下一阶段 |
| 线程安全分析 | P2 | 中 | 高 | Phase 2 后 |
| 性能优化 | P2 | 中 | 中 | 按需优化 |
| 错误处理集成 | P3 | 中 | 中 | 社区需求驱动 |

---

## 🎯 成功指标

### Phase 1 成功标准 ✅
- [x] 能够实现任意无状态 Trait
- [x] 编译无警告（仅 dead_code 警告）
- [x] 测试全部通过（7/7）
- [x] 文档完整
- [x] 示例可运行（trait_calculator）

### Phase 2 成功标准
- [ ] 能够实现 `std::hash::Hasher`
- [ ] 可用于 `HashMap`
- [ ] 零内存泄漏（valgrind 验证）
- [ ] 线程安全（如果标记）
- [ ] 性能接近纯 Rust 实现（<5% 开销）

---

## 💬 需要讨论的问题

1. **语法设计**
   - `#[opaque_pointer]` vs `#[zig_trait]`？
   - `#[map_method]` vs 自动推导？

2. **内存分配器**
   - 强制使用 `c_allocator`？
   - 支持自定义分配器？

3. **线程安全**
   - 默认不安全（保守）？
   - 提供 `#[thread_safe]` 标记？
   - 自动分析（复杂）？

4. **错误处理**
   - Zig 错误 → Rust Result 自动转换？
   - Panic 跨 FFI 边界处理？

---

**下一步行动**: 创建 `examples/trait_calculator` 作为 Phase 1 的概念验证

**预计完成时间**: Phase 1 - 本周内，Phase 2 - 2周后

**文档状态**: 设计完成 ✅，实现待启动 🔄