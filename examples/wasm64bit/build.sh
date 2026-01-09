#!/bin/bash
# AutoZig WASM 3.0 64-bit 手动绑定构建脚本

set -e

echo "🚀 AutoZig WASM 3.0 64-bit 手动绑定构建"
echo "===================================="
echo ""

# 检查必要工具
check_tool() {
    if ! command -v $1 &> /dev/null; then
        echo "❌ 错误: $1 未安装"
        echo "   请先安装 $1"
        exit 1
    fi
    echo "✓ 检测到 $1"
}

echo "📦 检查依赖工具..."
check_tool cargo
check_tool rustc

echo ""
echo "🔧 配置 Rust 工具链..."

# 检查是否有 nightly 工具链
if ! rustup toolchain list | grep -q "nightly"; then
    echo "   安装 nightly 工具链..."
    rustup toolchain install nightly
fi

# 检查是否有rust-src组件
if ! rustup component list --installed | grep -q "rust-src"; then
    echo "   安装 rust-src 组件..."
    rustup component add rust-src
fi

echo ""
echo "⚙️  构建选项:"
echo "   1) wasm32-unknown-unknown (标准 32-bit WASM)"
echo "   2) wasm64-unknown-unknown (64-bit WASM Memory64，推荐)"
echo ""
read -p "请选择构建目标 [2]: " choice
choice=${choice:-2}

if [ "$choice" = "2" ]; then
    echo ""
    echo "🔨 使用 wasm64-unknown-unknown 构建..."
    echo "   ⚠️  注意: wasm64 target需要从源码构建标准库"
    echo ""
    
    # 检查Rust版本
    rust_version=$(rustc --version | grep -oP '\d+\.\d+' | head -1)
    echo "   检测到 Rust 版本: $rust_version"
    
    # 使用 build-std 构建
    echo "   正在构建（这可能需要几分钟）..."
    cargo +nightly build \
        --target wasm64-unknown-unknown \
        -Z build-std=std,panic_abort \
        --release \
        --lib
    
    echo ""
    echo "✅ 构建完成！"
    echo "   输出: target/wasm64-unknown-unknown/release/autozig_wasm64bit.wasm"
    
    # 复制wasm文件到www目录（手动绑定不需要wasm-bindgen）
    echo ""
    echo "📦 准备Web部署文件..."
    mkdir -p www/pkg
    cp ../../target/wasm64-unknown-unknown/release/autozig_wasm64bit.wasm www/pkg/
    
    echo "   ✅ WASM 文件已复制到 www/pkg/"
    echo "   ℹ️  使用手动绑定：无需 wasm-bindgen 处理"
    
else
    echo ""
    echo "🔨 使用 wasm32-unknown-unknown 构建（回退模式）..."
    echo "   这将生成标准 32-bit WASM 模块"
    echo ""
    
    cargo +nightly build \
        --target wasm32-unknown-unknown \
        -Z build-std=std,panic_abort \
        --release \
        --lib
    
    echo ""
    echo "✅ 构建完成！"
    echo "   输出: target/wasm32-unknown-unknown/release/autozig_wasm64bit.wasm"
    
    # 复制文件
    mkdir -p www/pkg
    cp ../../target/wasm32-unknown-unknown/release/autozig_wasm64bit.wasm www/pkg/
    
    echo "   ✅ WASM 文件已复制到 www/pkg/"
fi

echo ""
echo "📝 后续步骤:"
echo "   1. 启动开发服务器:"
echo "      cd www && python3 -m http.server 8080"
echo ""
echo "   2. 在浏览器中打开:"
echo "      http://localhost:8080"
echo ""

if [ "$choice" = "2" ]; then
    echo "   3. 确保启用 Memory64 支持:"
    echo "      Chrome: chrome://flags/#enable-webassembly-memory64"
    echo "      Firefox: about:config -> javascript.options.wasm_memory64"
    echo ""
fi

echo "🎉 构建脚本完成！"
echo ""
echo "💡 提示: 本项目使用手动绑定方案"
echo "   - 不依赖 wasm-bindgen"
echo "   - 直接通过 WebAssembly.instantiate 加载"
echo "   - 支持完整的 wasm64 特性"