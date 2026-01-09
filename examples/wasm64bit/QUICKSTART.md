# 🚀 快速入门指南

## 5 分钟上手 AutoZig WASM64

### 前置条件

确保已安装：
- Rust 1.74+ (stable)
- Cargo
- wasm-pack

### 步骤 1：构建项目

#### 选项 A：使用 wasm32（推荐用于测试）

```bash
# 使用标准 32-bit WASM（最兼容）
wasm-pack build --target web --out-dir www/pkg --release
```

#### 选项 B：使用 wasm64（需要 build-std）

```bash
# 安装 rust-src 组件
rustup component add rust-src

# 使用 build-std 编译（tier-3 target需要）
cargo build \
    --target wasm64-unknown-unknown \
    -Z build-std=std,panic_abort \
    --release

# 生成 JS 绑定
cargo install wasm-bindgen-cli
wasm-bindgen --target web \
    --out-dir www/pkg \
    target/wasm64-unknown-unknown/release/autozig_wasm64bit.wasm
```

#### 选项 C：使用构建脚本（推荐）

```bash
./build.sh
# 然后选择构建目标
```

### 步骤 2：启动服务器

```bash
cd www
python3 -m http.server 8080
```

### 步骤 3：访问示例

在浏览器中打开：
```
http://localhost:8080
```

## 启用 Memory64 支持（仅 wasm64）

### Chrome/Edge

1. 打开 `chrome://flags`
2. 搜索 "WebAssembly Memory64"
3. 启用该选项
4. 重启浏览器

### Firefox

1. 打开 `about:config`
2. 搜索 `javascript.options.wasm_memory64`
3. 设置为 `true`
4. 重启浏览器

### Node.js

```bash
node --experimental-wasm-memory64 server.js
```

## 常见问题

### Q: 编译失败，提示找不到 wasm64 target或core库

**A**: wasm64是tier-3 target，需要：
1. 安装rust-src: `rustup component add rust-src`
2. 使用 `-Z build-std`: `cargo build --target wasm64-unknown-unknown -Z build-std=std,panic_abort`
3. 或先用wasm32测试: `wasm-pack build --target web`

### Q: 浏览器加载失败

**A**: 检查：
1. 是否正确构建了 pkg 目录
2. 是否启动了 HTTP 服务器（不能直接打开 HTML 文件）
3. 如果使用 wasm64，是否启用了 Memory64 支持

### Q: 如何验证是 64-bit 还是 32-bit？

**A**: 查看页面上的"架构"信息：
- WASM64 = 64-bit 模式
- WASM32 = 32-bit 模式

## 下一步

- 阅读完整文档：[README.md](README.md)
- 探索 Zig 源码：[src/wasm64.zig](src/wasm64.zig)
- 查看 Rust FFI：[src/lib.rs](src/lib.rs)
- 参考文档：[/autozig/docs/wasm3.0.md](/autozig/docs/wasm3.0.md)

## 需要帮助？

提交 Issue 到 AutoZig 项目：
https://github.com/your-org/autozig/issues