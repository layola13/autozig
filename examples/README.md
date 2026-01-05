# AutoZig Examples

本目录包含AutoZig的所有示例项目，展示了各种特性和使用场景。

## 示例列表

### 基础示例

1. **structs** - 结构体绑定
   - 展示如何在Rust和Zig之间传递结构体
   - 包含嵌套结构体和复制语义

2. **enums** - 枚举类型
   - 展示Result和Option类型的FFI绑定
   - 自定义枚举和状态机示例

3. **complex** - 复杂类型
   - 泛型结构体（Pair, Triple）
   - 嵌套复杂类型处理

### 高级示例

4. **smart_lowering** - 智能类型降级
   - 字符串（&str）自动转换为ptr+len
   - 切片（&[T]）自动转换为ptr+len
   - 展示Phase 2的Smart Lowering特性

5. **external** - 外部Zig文件
   - 使用`include_zig!`宏引用外部.zig文件
   - 包含Zig测试集成

6. **security_tests** - 安全测试
   - 内存安全验证
   - 边界检查测试
   - Zero-unsafe代码验证

### Trait支持示例

7. **trait_calculator** - Trait实现（无状态）
   - 展示ZST (Zero-Sized Type)的trait实现
   - Calculator trait示例

8. **trait_hasher** - Trait实现（有状态）
   - 展示Opaque Pointer的trait实现
   - Hasher trait示例
   - 包含构造函数和析构函数

### Phase 3 示例

9. **generics** - 泛型单态化
   - 使用`#[monomorphize(...)]`属性
   - 自动生成多个类型的单态化版本
   - 示例：`sum<T>`, `max<T>`

10. **async** - 异步FFI
    - 使用`async fn`关键字
    - 采用`tokio::spawn_blocking`模式
    - Zig侧保持同步实现
    - 展示并发执行和混合async/sync使用

## 批量验证脚本

使用 `verify_all.sh` 脚本可以一次性验证所有示例：

```bash
cd examples
./verify_all.sh
```

### 脚本功能

- ✅ 自动清理、编译和运行所有示例
- ✅ 彩色输出，易于阅读
- ✅ 详细的错误报告
- ✅ 超时保护（30秒）
- ✅ 统计总结（成功/失败/跳过）

### 输出示例

```
======================================
  AutoZig Examples 验证工具
======================================

[INFO] Examples目录: /path/to/examples
[INFO] 开始批量验证...

======================================
  验证示例: Structs Example
======================================

[INFO] 清理构建产物...
[INFO] 编译项目...
[✓] Structs Example: 编译成功
[INFO] 运行项目...
[✓] Structs Example: 运行成功

...

======================================
  验证结果总结
======================================

总计: 10 个示例
成功: 10
失败: 0
跳过: 0
[✓] 所有示例验证通过！🎉
```

## 单独运行示例

每个示例都是独立的Cargo项目，可以单独运行：

```bash
# 进入示例目录
cd structs

# 编译
cargo build

# 运行
cargo run

# 测试（如果有tests目录）
cargo test
```

## 示例结构

每个示例通常包含：

```
example_name/
├── Cargo.toml       # 项目配置
├── build.rs         # 构建脚本（调用autozig_build）
├── src/
│   ├── main.rs      # Rust代码，包含autozig!宏调用
│   └── *.zig        # Zig实现（可选，用于external示例）
└── tests/           # 集成测试（可选）
```

## 学习路径建议

### 初学者
1. 先看 **structs** - 理解基本的结构体绑定
2. 然后看 **enums** - 理解枚举和Result/Option类型
3. 再看 **smart_lowering** - 了解自动类型转换

### 中级用户
4. **complex** - 学习复杂嵌套类型
5. **external** - 学习如何使用外部Zig文件
6. **trait_calculator** / **trait_hasher** - 学习trait实现

### 高级用户
7. **generics** - 学习泛型单态化（Phase 3）
8. **async** - 学习异步FFI（Phase 3）
9. **security_tests** - 了解安全最佳实践

## 技术要点

### Phase 1 & 2 特性
- ✅ 结构体和枚举的自动绑定
- ✅ Smart Lowering（&str, &[T] 自动转换）
- ✅ Trait支持（ZST和Opaque Pointer）
- ✅ 外部Zig文件引用
- ✅ Zig测试集成

### Phase 3 特性
- ✅ 泛型单态化（`#[monomorphize(...)]`）
- ✅ 异步FFI（`async fn` + `spawn_blocking`）
- ✅ 类型替换引擎
- ✅ Name mangling策略

## 常见问题

### Q: 编译失败怎么办？
A: 检查：
1. Zig版本是否正确（0.11+）
2. Rust版本是否正确（1.77+）
3. 查看构建日志：`cargo build -vv`

### Q: 如何调试Zig代码？
A: 
1. 查看生成的Zig代码：`target/debug/build/*/out/generated_autozig.zig`
2. 使用Zig的测试功能（参见external示例）
3. 添加Zig的debug打印

### Q: 如何添加新示例？
A: 
1. 复制现有示例作为模板
2. 修改Cargo.toml和代码
3. 在Cargo.toml（workspace root）的members中添加
4. 在verify_all.sh的EXAMPLES数组中添加
5. 运行`./verify_all.sh`验证

## 贡献

欢迎贡献新的示例！请确保：
- ✅ 代码简洁易懂
- ✅ 包含详细注释
- ✅ 通过`verify_all.sh`验证
- ✅ 添加到本README的示例列表

## 许可证

所有示例代码遵循AutoZig项目的许可证（MIT OR Apache-2.0）。