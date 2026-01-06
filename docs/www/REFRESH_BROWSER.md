# ⚠️ 浏览器缓存问题 - 需要硬刷新

## 问题原因

HTML 文件已经更新，但浏览器缓存了旧版本的 `index.html`，导致仍然尝试导入不存在的函数名。

## 解决方法

### 方法 1: 硬刷新浏览器（推荐）

**Chrome/Edge:**
- Windows: `Ctrl + Shift + R` 或 `Ctrl + F5`
- Mac: `Cmd + Shift + R`

**Firefox:**
- Windows: `Ctrl + Shift + R` 或 `Ctrl + F5`
- Mac: `Cmd + Shift + R`

**Safari:**
- Mac: `Cmd + Option + R`

### 方法 2: 清除浏览器缓存

1. 打开开发者工具 (F12)
2. 右键点击刷新按钮
3. 选择"清空缓存并硬性重新加载"

### 方法 3: 强制重启HTTP服务器

```bash
# 1. 停止当前服务器 (Ctrl+C)
# 2. 清理缓存（可选）
rm -rf autozig/examples/wasm_light/www/.cache

# 3. 重新启动
cd autozig/examples/wasm_light/www
python3 -m http.server 8889
```

## 验证修复成功

打开浏览器控制台 (F12)，应该看到：

✅ **成功标志:**
```
✅ AutoZig WASM Light v0.1.0 - Zero-Copy SIMD Multi-Light Rendering
✅ 初始化完成，点击"开始渲染"按钮查看效果
```

❌ **失败标志（仍然缓存旧文件）:**
```
Uncaught SyntaxError: The requested module './pkg/autozig_wasm_light.js' does not provide an export named 'alloc_lights_buffer'
```

## 当前正确的函数名

HTML 文件已更新为正确的函数名（带 `wasm_` 前缀）：

```javascript
import init, {
    wasm_alloc_pixel_buffer,      // ✅ 正确
    wasm_alloc_lights_buffer,     // ✅ 正确
    wasm_render_lights_scalar,    // ✅ 正确
    wasm_render_lights_simd,      // ✅ 正确
    get_version                    // ✅ 正确（无前缀）
} from './pkg/autozig_wasm_light.js';
```

这些函数名与生成的 `autozig_wasm_light.js` 中的导出完全匹配。