#!/bin/bash

# AutoZig WASM Light 构建和服务脚本

set -e

echo "🚀 开始构建 AutoZig WASM Light Demo..."

# 检查 wasm-pack
if ! command -v wasm-pack &> /dev/null; then
    echo "❌ 错误: 未找到 wasm-pack"
    echo "请运行: cargo install wasm-pack"
    exit 1
fi

# 构建 WASM
echo "📦 构建 WASM 模块..."
wasm-pack build --target web --out-dir www/pkg

if [ $? -eq 0 ]; then
    echo "✅ WASM 构建成功"
else
    echo "❌ WASM 构建失败"
    exit 1
fi

# 启动服务器
echo ""
echo "🌐 启动本地服务器..."
echo "📍 访问: http://localhost:8089"
echo "⏹️  按 Ctrl+C 停止服务器"
echo ""

cd www
python3 -m http.server 8089