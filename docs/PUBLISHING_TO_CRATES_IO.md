# 发布 AutoZig 到 crates.io

## 前置要求

### 1. 创建 crates.io 账户
访问 https://crates.io/ 并使用 GitHub 账户登录

### 2. 获取 API Token
1. 登录后访问 https://crates.io/settings/tokens
2. 点击 "New Token"
3. 给 token 命名（如 "autozig-publishing"）
4. 复制生成的 token

### 3. 配置 cargo 登录
```bash
cargo login <your-api-token>
```

这会将 token 保存到 `~/.cargo/credentials.toml`

## 发布前检查清单

### 1. 验证 Cargo.toml 元数据

确保所有包的 `Cargo.toml` 包含必要的元数据：

```toml
[package]
name = "package-name"
version = "0.1.0"
authors = ["Your Name <email@example.com>"]
edition = "2021"
license = "MIT OR Apache-2.0"
description = "A concise description"
repository = "https://github.com/layola13/autozig"
homepage = "https://github.com/layola13/autozig"
documentation = "https://docs.rs/package-name"
readme = "README.md"
keywords = ["zig", "ffi", "interop", "macro", "codegen"]
categories = ["development-tools::ffi", "api-bindings"]
```

### 2. 准备 README.md

每个包都应该有一个清晰的 README，包括：
- 项目简介
- 安装说明
- 快速开始示例
- 文档链接
- 许可证信息

### 3. 运行完整测试
```bash
cd autozig

# 格式检查
cargo fmt --all -- --check

# Clippy 检查
cargo clippy --all-targets --all-features -- -D warnings

# 运行所有测试
cargo test --all

# 验证所有示例
cd examples && ./verify_all.sh
```

### 4. 更新版本号

使用语义化版本控制（Semantic Versioning）：
- **0.1.0** - 初始发布
- **0.1.x** - 补丁更新（bug 修复）
- **0.x.0** - 小版本更新（新功能，向后兼容）
- **x.0.0** - 主版本更新（破坏性变更）

### 5. 创建 Git 标签
```bash
git tag -a v0.1.0 -m "Release version 0.1.0"
git push origin v0.1.0
```

## 发布顺序

AutoZig 是一个多包工作空间，需要按照依赖顺序发布：

### 步骤 1: 发布 Parser（无依赖）
```bash
cd autozig/parser
cargo publish --dry-run  # 先试运行检查
cargo publish            # 实际发布
```

### 步骤 2: 发布 Engine（依赖 parser）
```bash
cd ../engine
cargo publish --dry-run
cargo publish
```

### 步骤 3: 发布 Macro（依赖 parser）
```bash
cd ../macro
cargo publish --dry-run
cargo publish
```

### 步骤 4: 发布 Build（依赖 engine）
```bash
cd ../gen/build
cargo publish --dry-run
cargo publish
```

### 步骤 5: 发布主包（依赖所有子包）
```bash
cd ../..  # 回到 autozig 根目录
cargo publish --dry-run
cargo publish
```

## 常见问题

### 问题 1: "crate name already exists"
- 原因：包名已被占用
- 解决：在 `Cargo.toml` 中修改包名，建议使用命名空间前缀

### 问题 2: "missing required field"
- 原因：Cargo.toml 缺少必需字段
- 解决：添加 `license`, `description`, `repository` 等字段

### 问题 3: "failed to verify package tarball"
- 原因：打包的文件有问题
- 解决：运行 `cargo package --list` 检查打包内容
- 使用 `.cargo_vcs_info.json` 排除不需要的文件

### 问题 4: "documentation failed to build"
- 原因：文档构建失败
- 解决：本地运行 `cargo doc --no-deps` 测试文档构建

### 问题 5: "dependency version mismatch"
- 原因：工作空间内包版本不一致
- 解决：确保所有依赖版本使用 `version = "0.1.0"` 或 `version = "=0.1.0"`

## 发布后验证

### 1. 检查 crates.io 页面
访问 https://crates.io/crates/autozig 确认发布成功

### 2. 测试安装
```bash
# 在新目录测试
mkdir test-install
cd test-install
cargo init
cargo add autozig
cargo build
```

### 3. 检查文档
访问 https://docs.rs/autozig 确认文档已生成

## 发布 Beta/RC 版本

对于测试版本，使用预发布标识符：

```toml
version = "0.1.0-beta.1"  # Beta 版本
version = "0.1.0-rc.1"    # Release Candidate
```

用户安装时需要指定：
```bash
cargo add autozig@0.1.0-beta.1
```

## 撤回已发布版本

如果发现严重问题，可以撤回版本（但不能删除）：

```bash
cargo yank --version 0.1.0
cargo yank --version 0.1.0 --undo  # 取消撤回
```

## 自动化发布脚本

创建 `scripts/publish.sh`:

```bash
#!/bin/bash
set -e

echo "🚀 Publishing AutoZig to crates.io..."

# 1. 运行测试
echo "📋 Running tests..."
cargo test --all

# 2. 发布 parser
echo "📦 Publishing autozig-parser..."
cd parser && cargo publish && cd ..

# 等待 crates.io 索引更新
echo "⏳ Waiting for crates.io index to update..."
sleep 30

# 3. 发布 engine
echo "📦 Publishing autozig-engine..."
cd engine && cargo publish && cd ..
sleep 30

# 4. 发布 macro
echo "📦 Publishing autozig-macro..."
cd macro && cargo publish && cd ..
sleep 30

# 5. 发布 build
echo "📦 Publishing autozig-build..."
cd gen/build && cargo publish && cd ../..
sleep 30

# 6. 发布主包
echo "📦 Publishing autozig..."
cargo publish

echo "✅ All packages published successfully!"
```

## 维护版本

### 发布补丁版本
```bash
# 修复 bug 后
cargo set-version --bump patch  # 0.1.0 -> 0.1.1
git commit -am "chore: bump version to 0.1.1"
git tag -a v0.1.1 -m "Release v0.1.1"
./scripts/publish.sh
```

### 发布小版本
```bash
# 添加新功能后
cargo set-version --bump minor  # 0.1.1 -> 0.2.0
git commit -am "chore: bump version to 0.2.0"
git tag -a v0.2.0 -m "Release v0.2.0"
./scripts/publish.sh
```

## 安全建议

1. **保护 API Token**: 不要将 token 提交到 Git
2. **使用 CI/CD**: 在 GitHub Actions 中配置自动发布
3. **代码签名**: 考虑使用 GPG 签名 Git 标签
4. **审计依赖**: 定期运行 `cargo audit` 检查安全漏洞

## 相关资源

- [Cargo Book - Publishing](https://doc.rust-lang.org/cargo/reference/publishing.html)
- [crates.io 政策](https://crates.io/policies)
- [Semantic Versioning](https://semver.org/)
- [Rust API 设计指南](https://rust-lang.github.io/api-guidelines/)

## 现在就发布！

AutoZig 已经准备好发布。运行以下命令开始：

```bash
cd autozig
cargo login  # 如果还没登录
./scripts/publish.sh  # 或手动按顺序发布
```

祝发布顺利！🎉