# AutoZig 安全测试套件

⚠️ **警告**：本项目包含故意引入的安全漏洞示例，仅用于安全测试和教育目的！请勿在生产环境使用这些代码模式！

## 目的

验证 AutoZig 框架是否会暴露以下常见 FFI/ABI 漏洞：

1. **Use-After-Free (UAF)** - 内存释放后使用
2. **Buffer Overflow** - 缓冲区溢出
3. **ABI Mismatch** - ABI 布局不匹配
4. **Data Race** - 数据竞争（多线程安全）

## 测试策略

我们采用**对照测试**方法：
- ✅ **安全版本**：遵循最佳实践的正确实现
- ⚠️ **不安全版本**：故意引入漏洞，验证检测工具能否发现

## 快速开始

### 1. 运行安全版本（默认）

```bash
cd examples/security_tests
cargo run
```

预期输出：
```
=== AutoZig 安全测试套件 ===
运行安全版本（所有测试都应该通过）

1. 测试安全的缓冲区操作...
   ✓ 缓冲区操作安全测试通过
2. 测试安全的结构体传递...
   ✓ 结构体传递安全测试通过
3. 测试安全的边界检查...
   ✓ 边界检查安全测试通过

✅ 所有安全测试通过！
```

### 2. 测试 Use-After-Free 检测

```bash
# 使用 AddressSanitizer
RUSTFLAGS="-Z sanitizer=address" cargo +nightly run --release -- uaf
```

**预期结果**：ASan 应该报告 `heap-use-after-free`

### 3. 测试缓冲区溢出检测

```bash
RUSTFLAGS="-Z sanitizer=address" cargo +nightly run --release -- overflow
```

**预期结果**：ASan 应该报告 `heap-buffer-overflow`

### 4. 测试 ABI 布局不匹配

```bash
cargo run -- abi
```

**预期结果**：读取到的值与预期不符，或直接崩溃

### 5. 测试数据竞争检测

```bash
RUSTFLAGS="-Z sanitizer=thread" cargo +nightly run -- race
```

**预期结果**：TSan 应该报告 `data race`

## 详细测试方法

### AddressSanitizer (ASan)

检测内存错误（UAF、溢出、泄漏等）

```bash
# Debug 模式
RUSTFLAGS="-Z sanitizer=address" cargo +nightly run -- uaf

# Release 模式（更接近生产环境）
RUSTFLAGS="-Z sanitizer=address" cargo +nightly run --release -- overflow
```

### ThreadSanitizer (TSan)

检测数据竞争

```bash
RUSTFLAGS="-Z sanitizer=thread" cargo +nightly run -- race
```

### Valgrind

传统内存调试工具

```bash
cargo build
valgrind --leak-check=full --show-leak-kinds=all \
  target/debug/autozig-security-tests
```

### Miri

Rust UB 检测器（最严格）

```bash
cargo +nightly miri run
```

注意：Miri 可能不支持某些操作（如多线程），但能捕获隐藏的 UB。

## 测试结果解读

### 1. Use-After-Free

**安全版本行为**：
- Rust 借用检查器阻止保存引用超过生命周期
- 如果 Zig 代码尝试保存指针，应该在设计时避免

**不安全版本预期**：
```
==12345==ERROR: AddressSanitizer: heap-use-after-free on address 0x...
READ of size 1 at 0x... thread T0
    #0 0x... in unsafe_use_saved_pointer
    #1 0x... in test_use_after_free_unsafe
```

**结论**：
- ✅ 如果 ASan 捕获：说明检测工具有效
- ❌ 如果没崩溃：可能是 timing 问题，多次运行

### 2. Buffer Overflow

**安全版本行为**：
- Zig 切片自动边界检查（Debug 模式）
- 循环使用 `slice[0..len]` 而不是原始指针

**不安全版本预期**：
```
==12345==ERROR: AddressSanitizer: heap-buffer-overflow on address 0x...
WRITE of size 1 at 0x... thread T0
    #0 0x... in unsafe_overflow_write
```

**结论**：
- ✅ 如果 ASan 捕获：说明溢出被检测到
- ⚠️ 如果没报错：可能是 Release 模式关闭了检查

### 3. ABI Mismatch

**安全版本行为**：
- 所有跨 FFI 的结构体使用 `#[repr(C)]`
- Zig 使用 `extern struct` 确保 C 布局

**不安全版本预期**：
```
Rust 侧: x=10, y=0x12345678
Zig 读取到的 y=0x0a123456  // 字段错位
⚠️  检测到 ABI 不匹配！
```

**结论**：
- 字段错位是因为 padding 不同
- 这不会被 sanitizer 捕获，需要人工检查值

### 4. Data Race

**安全版本行为**：
- 不在 Zig 内部 spawn 线程修改共享数据
- 或使用 Rust 的 `Mutex`/`Arc` 包装

**不安全版本预期**：
```
==================
WARNING: ThreadSanitizer: data race (pid=12345)
  Write of size 1 at 0x... by thread T1:
    #0 unsafe_race_write
  Previous write of size 1 at 0x... by thread T2:
    #0 unsafe_race_write
```

**结论**：
- ✅ 如果 TSan 报告 race：说明检测有效
- ⚠️ 需要 nightly Rust 和 `-Z sanitizer=thread`

## 最佳实践总结

基于这些测试，AutoZig 用户应遵循以下规则：

### ✅ DO（推荐做法）

1. **生命周期管理**
   ```rust
   // ✅ 立即使用并返回
   fn process_data(data: &[u8]) -> u64 {
       zig_process(data)  // Zig 函数在调用期间完成所有操作
   }
   ```

2. **边界检查**
   ```zig
   // ✅ 使用切片而不是原始指针
   export fn safe_process(ptr: [*]const u8, len: usize) void {
       const slice = ptr[0..len];  // 自动边界检查（Debug模式）
       for (slice) |byte| {
           // 安全访问
       }
   }
   ```

3. **结构体布局**
   ```rust
   // ✅ 始终使用 #[repr(C)]
   #[repr(C)]
   struct MyData {
       x: i32,
       y: i32,
   }
   ```

4. **线程安全**
   ```rust
   // ✅ 用 Rust 管理并发
   let data = Arc::new(Mutex::new(vec![0u8; 100]));
   let data_clone = data.clone();
   thread::spawn(move || {
       let mut d = data_clone.lock().unwrap();
       zig_process(&mut d);
   });
   ```

### ❌ DON'T（避免的做法）

1. **保存指针**
   ```zig
   // ❌ 永远不要这样做！
   var global_ptr: ?[*]u8 = null;
   export fn bad_save(ptr: [*]u8) void {
       global_ptr = ptr;  // 危险！
   }
   ```

2. **越界访问**
   ```zig
   // ❌ 不检查边界
   export fn bad_access(ptr: [*]u8, len: usize) void {
       ptr[len + 10] = 0;  // 溢出！
   }
   ```

3. **ABI 不匹配**
   ```rust
   // ❌ 忘记 #[repr(C)]
   struct BadData {  // 布局不确定！
       x: u8,
       y: u32,
   }
   ```

4. **跨线程共享**
   ```zig
   // ❌ Zig 内部 spawn 线程
   export fn bad_thread(ptr: [*]u8) void {
       const t = std.Thread.spawn(...);  // 危险！
   }
   ```

## 工具链要求

- **Rust**: nightly (sanitizers 需要)
- **Zig**: 0.11+ 或 0.12+
- **工具**:
  - AddressSanitizer: `-Z sanitizer=address`
  - ThreadSanitizer: `-Z sanitizer=thread`
  - Valgrind: `apt install valgrind`
  - Miri: `rustup +nightly component add miri`

## CI/CD 集成

可以将这些测试集成到 CI 流程中：

```yaml
# .github/workflows/security-tests.yml
name: Security Tests

on: [push, pull_request]

jobs:
  asan:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: nightly
          override: true
      - name: Run ASan tests
        run: |
          cd examples/security_tests
          RUSTFLAGS="-Z sanitizer=address" cargo run --release -- overflow

  tsan:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: nightly
      - name: Run TSan tests
        run: |
          cd examples/security_tests
          RUSTFLAGS="-Z sanitizer=thread" cargo run -- race
```

## 参考资料

- [AddressSanitizer Documentation](https://clang.llvm.org/docs/AddressSanitizer.html)
- [ThreadSanitizer Documentation](https://clang.llvm.org/docs/ThreadSanitizer.html)
- [Rust Sanitizers](https://doc.rust-lang.org/unstable-book/compiler-flags/sanitizer.html)
- [Zig Safety](https://ziglang.org/documentation/master/#Undefined-Behavior)
- [AutoZig Security Best Practices](../../SECURITY_BEST_PRACTICES.md)

## 贡献

如果你发现新的潜在漏洞模式，欢迎提交 PR 添加测试用例！

## 许可证

与 AutoZig 主项目相同（MIT OR Apache-2.0）