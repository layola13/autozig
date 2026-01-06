#!/bin/bash

echo "=== AutoZig WASM Light 诊断脚本 ==="
echo ""

echo "1. 检查 index.html 导入语句（第335-341行）:"
echo "-------------------------------------------"
sed -n '335,341p' index.html
echo ""

echo "2. 检查 pkg/autozig_wasm_light.js 实际导出:"
echo "-------------------------------------------"
grep -A 1 "^export function" pkg/autozig_wasm_light.js
echo ""

echo "3. 检查文件时间戳:"
echo "-------------------------------------------"
ls -lh index.html pkg/autozig_wasm_light.js pkg/autozig_wasm_light_bg.wasm
echo ""

echo "=== 诊断完成 ==="
echo ""
echo "如果 index.html 中仍然显示 'alloc_lights_buffer'（无 wasm_ 前缀）,"
echo "说明文件未更新。请执行："
echo "  git status  # 查看文件是否真的被修改"
echo "  git diff index.html  # 查看具体修改"
echo ""
echo "如果 index.html 已正确更新为 'wasm_alloc_lights_buffer',"
echo "但浏览器仍然报错，请硬刷新浏览器："
echo "  Chrome/Edge: Ctrl+Shift+R 或 Cmd+Shift+R"
echo "  Firefox: Ctrl+Shift+R"
echo "  Safari: Cmd+Option+R"