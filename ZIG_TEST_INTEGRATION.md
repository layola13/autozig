# Zig Test Integration Guide

AutoZig 现在支持将 Zig 的单元测试集成到 Rust 的测试框架中！

## 功能概述

这个功能允许你：
- 在 `.zig` 文件中编写 Zig 测试（使用 `test` 块）
- 在 `build.rs` 中自动编译这些测试为可执行文件
- 从 Rust 的 `#[test]` 函数中调用和验证 Zig 测试
- 集成到标准的 `cargo test` 工作流中

## 工作原理

1. **编译阶段**（build.rs）：
   - `autozig_build::build_tests()` 扫描指定目录中的所有 `.zig` 文件
   - 使用 `zig test` 命令编译每个文件的测试
   - 生成独立的测试可执行文件（命名为 `test_{filename}`）

2. **测试阶段**（Rust tests）：
   - Rust 测试函数调用编译好的 Zig 测试可执行文件
   - 捕获并验证测试输出和退出状态
   - 集成到 Cargo 的测试报告中

## 使用方法

### 1. 在 Zig 文件中添加测试

```zig
// math.zig
const std = @import("std");

export fn factorial(n: u32) u64 {
    if (n <= 1) return 1;
    var result: u64 = 1;
    var i: u32 = 2;
    while (i <= n) : (i += 1) {
        result *= i;
    }
    return result;
}

// 添加单元测试
test "factorial basic cases" {
    try std.testing.expectEqual(@as(u64, 1), factorial(0));
    try std.testing.expectEqual(@as(u64, 1), factorial(1));
    try std.testing.expectEqual(@as(u64, 2), factorial(2));
    try std.testing.expectEqual(@as(u64, 6), factorial(3));
}
```

### 2. 在 build.rs 中编译测试

```rust
// build.rs
fn main() {
    // 编译主库
    autozig_build::build("src").expect("Failed to build Zig code");
    
    // 编译 zig/ 目录中的所有测试
    autozig_build::build_tests("zig").expect("Failed to build Zig tests");
}
```

### 3. 创建 Rust 测试文件

```rust
// tests/zig_tests.rs
use std::process::Command;
use std::path::PathBuf;

fn get_test_exe_path(name: &str) -> PathBuf {
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR not set");
    PathBuf::from(out_dir).join(format!("test_{}", name))
}

#[test]
fn test_math_zig_tests() {
    let test_exe = get_test_exe_path("math");
    
    let output = Command::new(&test_exe)
        .output()
        .expect("Failed to execute Zig math tests");
    
    assert!(
        output.status.success(),
        "Zig math tests failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
```

### 4. 运行测试

```bash
cd examples/external
cargo test --test zig_tests
```

## 输出示例

```
running 4 tests
test test_all_zig_tests_exist ... ok
test test_math_zig_tests ... ok
test test_strings_zig_tests ... ok
test test_zig_zig_tests ... ok

Zig math tests output:
STDERR:
1/4 math.test.factorial basic cases...OK
2/4 math.test.fibonacci sequence...OK
3/4 math.test.gcd calculations...OK
4/4 math.test.prime number check...OK
All 4 tests passed.

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured
```

## API 参考

### `autozig_build::build_tests()`

```rust
pub fn build_tests(zig_dir: impl Into<PathBuf>) -> Result<Vec<PathBuf>>
```

**功能**：编译指定目录中所有 `.zig` 文件的测试

**参数**：
- `zig_dir` - 包含 `.zig` 文件的目录路径

**返回**：
- `Ok(Vec<PathBuf>)` - 编译成功，返回测试可执行文件路径列表
- `Err(_)` - 编译失败

**行为**：
- 扫描目录中的所有 `.zig` 文件
- 为每个文件编译测试可执行文件
- 测试可执行文件命名格式：`test_{filename}`（不含 `.zig` 扩展名）
- 输出到 `$OUT_DIR` 目录

### ZigCompiler 方法

```rust
// 编译测试到可执行文件
pub fn compile_tests(
    &self,
    source: &Path,
    output_exe: &Path,
    target: &str,
) -> Result<()>

// 运行测试可执行文件
pub fn run_test_executable(&self, test_exe: &Path) -> Result<String>
```

## 实现细节

### 编译命令

```bash
zig test source.zig -femit-bin=output_exe -target native -O ReleaseFast
```

- `-femit-bin=<path>` - 输出测试可执行文件路径
- `-target native` - 编译为本地架构
- `-O ReleaseFast` - 优化级别

### 测试可执行文件位置

测试可执行文件存储在 Cargo 的 `OUT_DIR` 中：
```
target/debug/build/<package-hash>/out/test_<filename>
```

### 测试输出格式

Zig 测试输出到 `stderr`，格式如下：
```
1/4 test_name...OK
2/4 another_test...OK
...
All 4 tests passed.
```

## 最佳实践

1. **测试文件组织**
   - 将 `.zig` 文件放在专门的测试目录（如 `zig/`）
   - 每个文件包含相关功能的测试
   - 使用描述性的测试名称

2. **测试编写**
   - 使用 `std.testing` 模块的断言函数
   - 为每个导出函数编写测试
   - 测试边界情况和错误条件

3. **集成到 CI**
   - 在 CI 管道中运行 `cargo test`
   - 确保 Zig 编译器已安装
   - 使用 `--test zig_tests` 单独运行 Zig 测试

## 示例项目

查看 `examples/external` 获取完整的工作示例：

```
examples/external/
├── build.rs              # 编译测试
├── zig/                  # Zig 源文件
│   ├── math.zig         # 带测试的数学函数
│   ├── strings.zig      # 带测试的字符串函数
│   └── zig.zig          # 带测试的工具函数
└── tests/
    └── zig_tests.rs     # Rust 测试调用 Zig 测试
```

## 故障排除

### 测试可执行文件未找到

确保：
1. `build.rs` 中调用了 `build_tests()`
2. `.zig` 文件在正确的目录中
3. 检查 `OUT_DIR` 环境变量

### 测试编译失败

检查：
1. Zig 编译器版本兼容性
2. `.zig` 文件语法是否正确
3. 测试代码是否使用了正确的 Zig 标准库 API

### 测试运行失败

验证：
1. 测试断言是否正确
2. 测试逻辑是否有误
3. 查看详细输出（使用 `--nocapture`）

## 技术优势

✅ **零手动配置** - 自动发现和编译测试  
✅ **原生 Zig 测试** - 使用标准 Zig 测试语法  
✅ **集成到 Cargo** - 统一的测试命令和报告  
✅ **快速反馈** - 独立的测试可执行文件  
✅ **CI 友好** - 标准 Cargo 测试工作流  

## 未来改进

- [ ] 支持测试过滤（只运行特定测试）
- [ ] 并行运行多个测试文件
- [ ] 测试覆盖率报告
- [ ] 测试失败时的详细诊断信息

## 相关文档

- [Zig 测试文档](https://ziglang.org/documentation/master/#Testing)
- [Cargo 测试指南](https://doc.rust-lang.org/cargo/guide/tests.html)
- [autozig README](README.md)