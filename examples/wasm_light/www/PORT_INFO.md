# ⚠️ 重要：正确的访问端口

## 错误端口
❌ **http://localhost:8089** - 错误！

## 正确端口  
✅ **http://localhost:8889** - 正确！

---

## 如果看到错误

如果浏览器控制台显示：
```
GET http://localhost:8089/favicon.ico 404 (File not found)
Uncaught TypeError: Cannot set properties of null
```

**原因**: 你访问了错误的端口 8089，应该访问 8889

**解决方法**:
1. 关闭当前浏览器标签页
2. 在地址栏输入正确地址: **http://localhost:8889**
3. 硬刷新页面 (`Ctrl+Shift+R` 或 `Cmd+Shift+R`)

---

## 如何确认服务器端口

运行诊断脚本:
```bash
cd autozig/examples/wasm_light/www
./diagnose.sh
```

或检查终端输出，应该看到：
```
Serving HTTP on :: port 8889 (http://[::]:8889/) ...
```

---

## 预期的正确输出

访问 **http://localhost:8889** 后，浏览器控制台应显示：
```
✅ AutoZig WASM Light v0.1.0 - Zero-Copy SIMD Multi-Light Rendering
📦 像素缓冲区: 1049520, 大小: 640000 bytes
✅ 初始化完成，点击"开始渲染"按钮查看效果
```

**无任何错误！**

点击"▶️ 开始渲染"按钮，三个画布同时显示动画效果。