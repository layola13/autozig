#!/bin/bash

# AutoZig WASM Filter - 构建和运行脚本
# 用法: ./build_and_serve.sh

set -e  # 遇到错误立即退出

echo "🚀 AutoZig WASM Filter - 构建和运行"
echo "=================================="
echo ""

# 1. 检查依赖
echo "📋 检查依赖..."

if ! command -v rustup &> /dev/null; then
    echo "❌ 错误: 未安装 rustup"
    echo "请访问 https://rustup.rs/ 安装 Rust"
    exit 1
fi

if ! command -v zig &> /dev/null; then
    echo "❌ 错误: 未找到 zig 命令"
    echo "请确保 Zig 已安装并在 PATH 中"
    echo "下载: https://ziglang.org/download/"
    exit 1
fi

echo "✅ Rust: $(rustc --version)"
echo "✅ Zig: $(zig version)"
echo ""

# 2. 安装 WASM 工具链
echo "🔧 检查 WASM 工具链..."

if ! rustup target list --installed | grep -q "wasm32-unknown-unknown"; then
    echo "📦 安装 wasm32-unknown-unknown 目标..."
    rustup target add wasm32-unknown-unknown
else
    echo "✅ wasm32-unknown-unknown 已安装"
fi

if ! command -v wasm-pack &> /dev/null; then
    echo "📦 安装 wasm-pack..."
    cargo install wasm-pack
else
    echo "✅ wasm-pack 已安装: $(wasm-pack --version)"
fi
echo ""

# 3. 构建 WASM
echo "🔨 构建 WASM 模块..."
echo "这将编译 Zig 代码到 WASM 并与 Rust 静态链接..."
echo ""

wasm-pack build --target web --out-dir www/pkg

if [ $? -eq 0 ]; then
    echo ""
    echo "✅ 构建成功！"
    echo ""
    
    # 显示生成的文件
    echo "📦 生成的文件:"
    ls -lh www/pkg/*.wasm www/pkg/*.js 2>/dev/null || echo "未找到生成的文件"
    echo ""
    
    # 显示 WASM 文件大小
    if [ -f www/pkg/autozig_wasm_filter_bg.wasm ]; then
        WASM_SIZE=$(stat -f%z www/pkg/autozig_wasm_filter_bg.wasm 2>/dev/null || stat -c%s www/pkg/autozig_wasm_filter_bg.wasm 2>/dev/null)
        WASM_SIZE_KB=$((WASM_SIZE / 1024))
        echo "📊 WASM 文件大小: ${WASM_SIZE_KB} KB"
        echo ""
    fi
else
    echo ""
    echo "❌ 构建失败"
    exit 1
fi

# 4. 启动 HTTP 服务器
echo "🌐 启动 HTTP 服务器..."
echo ""

PORT=8080

# 检测可用的 HTTP 服务器
if command -v python3 &> /dev/null; then
    echo "使用 Python HTTP 服务器"
    echo "访问: http://localhost:${PORT}"
    echo ""
    echo "按 Ctrl+C 停止服务器"
    echo ""
    cd www
    python3 -m http.server $PORT
elif command -v python &> /dev/null; then
    echo "使用 Python 2 HTTP 服务器"
    echo "访问: http://localhost:${PORT}"
    echo ""
    echo "按 Ctrl+C 停止服务器"
    echo ""
    cd www
    python -m SimpleHTTPServer $PORT
elif command -v npx &> /dev/null; then
    echo "使用 http-server (Node.js)"
    echo "访问: http://localhost:${PORT}"
    echo ""
    echo "按 Ctrl+C 停止服务器"
    echo ""
    npx http-server www -p $PORT
else
    echo "⚠️  未找到 HTTP 服务器"
    echo ""
    echo "请手动启动 HTTP 服务器:"
    echo "  cd www"
    echo "  python3 -m http.server ${PORT}"
    echo ""
    echo "或安装 http-server:"
    echo "  npm install -g http-server"
    echo "  http-server www -p ${PORT}"
    echo ""
    echo "然后访问: http://localhost:${PORT}"
fi