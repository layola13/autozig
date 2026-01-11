#!/bin/bash

set -e  # Exit on error

echo "🚀 Building AutoZig WASM64 Print Example..."
echo ""

# 检查是否在 www/ 目录中运行
if [[ ! -f "build.sh" ]]; then
    echo "❌ Error: Please run this script from the www/ directory"
    echo "   cd autozig/examples/wasm64print/www && ./build.sh"
    exit 1
fi

# 返回到项目根目录
cd ..

echo "📦 Checking Rust toolchain..."
if ! command -v cargo &> /dev/null; then
    echo "❌ Error: cargo not found. Please install Rust."
    exit 1
fi

echo "🔨 Compiling to WASM32 (standard target)..."
echo "   Target: wasm32-unknown-unknown"
echo "   Profile: release"
echo ""

# 使用 wasm32 编译（WASM64 需要特殊的 nightly 支持）
cargo build --target wasm32-unknown-unknown --release

# 检查生成的文件
WASM_FILE="../target/wasm32-unknown-unknown/release/autozig_wasm64print.wasm"
BINDINGS_FILE="../target/wasm32-unknown-unknown/release/build/autozig-wasm64print-*/out/bindings.js"

if [ ! -f "$WASM_FILE" ]; then
    echo "❌ Build failed: WASM file not found at $WASM_FILE"
    exit 1
fi

echo "✅ Build successful!"
echo ""

# 创建 pkg 目录
mkdir -p www/pkg

# 复制 WASM 文件
cp "$WASM_FILE" www/pkg/
echo "📦 Copied WASM file to www/pkg/"

# 复制 bindings.js（如果存在）
BINDINGS_SRC=$(ls ../target/wasm32-unknown-unknown/release/build/autozig-wasm64print-*/out/bindings.js 2>/dev/null | head -1)
if [ -f "$BINDINGS_SRC" ]; then
    cp "$BINDINGS_SRC" www/pkg/
    echo "📦 Copied bindings.js to www/pkg/"
else
    echo "⚠️  Warning: bindings.js not found (will use manual loader)"
fi

# 显示文件大小
WASM_SIZE=$(du -h "www/pkg/autozig_wasm64print.wasm" | cut -f1)
echo ""
echo "📊 WASM file size: $WASM_SIZE"
echo ""
echo "✅ Build complete!"
echo ""
echo "🌐 Next steps:"
echo "   1. cd www"
echo "   2. python3 -m http.server 8080"
echo "   3. Open http://localhost:8080 in your browser"
echo ""