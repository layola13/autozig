# 🧪 AutoZig WASM Light - 性能测试指南

## 📋 测试前准备

### 1. 确认文件已更新

```bash
cd autozig/examples/wasm_light
ls -lh www/pkg/autozig_wasm_light_bg.wasm
# 应该显示：16K，时间戳为最新编译时间
```

### 2. 启动 HTTP 服务器

```bash
cd www
python -m http.server 8889
# 或者
npx http-server -p 8889
```

### 3. 打开浏览器

访问：http://localhost:8889

**重要**：首次测试或代码更新后，必须**硬刷新**清除缓存！

- **Chrome/Edge**: `Ctrl + Shift + R` (Windows) / `Cmd + Shift + R` (Mac)
- **Firefox**: `Ctrl + Shift + R` (Windows) / `Cmd + Shift + R` (Mac)  
- **Safari**: `Cmd + Option + R`

---

## ✅ 验证步骤

### 第一步：检查控制台输出

打开浏览器开发者工具（F12），查看 Console 标签页，应该看到：

```
✅ AutoZig WASM Light v0.1.0 - Zero-Copy SIMD Multi-Light Rendering
📦 像素缓冲区: 1049520, 大小: 640000 bytes
✅ 初始化完成，点击"开始渲染"按钮查看效果
```

**如果出现错误**，说明浏览器缓存了旧版本，再次硬刷新！

### 第二步：开始渲染

点击页面上的 **"▶️ 开始渲染"** 按钮。

预期结果：
- ✅ 三个画布同时显示多光源动画
- ✅ 光源在画布中心周围旋转
- ✅ 实时性能数据更新（无控制台错误）

### 第三步：观察性能数据

在三个画布下方，应该看到实时更新的性能指标：

#### ⚡ Zig SIMD（最快）
```
时间: ~8-15 ms
FPS: ~60-120
```

#### 🦀 Rust Scalar（中等）
```
时间: ~15-30 ms
FPS: ~30-60
```

#### 🟨 JavaScript（最慢）
```
时间: ~30-60 ms
FPS: ~15-30
```

### 第四步：检查性能对比表

向下滚动到"📊 性能对比统计"表格，应该看到：

| 实现方式 | 平均渲染时间 | 平均 FPS | 吞吐量 | 相对性能 |
|---------|-------------|---------|--------|----------|
| ⚡ Zig SIMD | ~10 ms | ~100 | ~60 MB/s | 基准 (1.00x) |
| 🦀 Rust Scalar | ~20 ms | ~50 | ~30 MB/s | 2.00x |
| 🟨 JavaScript | ~40 ms | ~25 | ~15 MB/s | 4.00x |

**关键指标**：
- **Zig SIMD 应该比 Rust Scalar 快 1.5x - 2.5x**
- **Zig SIMD 应该比 JavaScript 快 3x - 5x**

---

## 🎛️ 交互测试

### 调整光源数量

1. 拖动"💡 光源数量"滑块（1-20）
2. 观察性能变化：
   - 光源越多，Zig SIMD 的优势越明显
   - 推荐测试：5, 10, 15, 20 个光源

### 调整光源参数

- **📏 光源半径** (50-300)：半径越大，受影响像素越多，计算量越大
- **☀️ 光源强度** (10-255)：影响颜色亮度
- **🚀 动画速度** (0-5)：控制光源移动速度

### 测试极限性能

```
光源数量：20
光源半径：300
光源强度：255
```

在这种配置下：
- **Zig SIMD** 应该仍然保持 30+ FPS
- **Rust Scalar** 可能降到 15-20 FPS
- **JavaScript** 可能低于 10 FPS

---

## 🔍 故障排查

### 问题1：控制台报错 `Cannot set properties of null`

**原因**：浏览器缓存了旧版本 HTML

**解决**：
1. 硬刷新（Ctrl+Shift+R）
2. 开发者工具 → Network → 勾选 "Disable cache"
3. 重新加载页面

### 问题2：Zig SIMD 反而比 Rust Scalar 慢

**原因**：SIMD 指令未生效，或者浏览器不支持 WASM SIMD

**检查**：
```javascript
// 在浏览器控制台执行
console.log(WebAssembly.validate(
  new Uint8Array([0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00,
                  0x01, 0x05, 0x01, 0x60, 0x00, 0x01, 0x7b, 0x03,
                  0x02, 0x01, 0x00, 0x0a, 0x0a, 0x01, 0x08, 0x00,
                  0xfd, 0x0c, 0xfd, 0x0c, 0xfd, 0x12, 0x0b])
));
// 应该返回 true（支持 SIMD）
```

**解决**：
- 使用最新版 Chrome (>91) / Firefox (>89) / Edge (>91)
- Safari 需要 16.4+ 版本才支持 WASM SIMD

### 问题3：三个画布不同步

**原因**：这是正常的！因为它们使用不同的渲染实现

**预期行为**：
- 光源位置相同（由 JS 控制）
- 渲染效果相同（光照算法一致）
- 性能不同（实现方式不同）

### 问题4：FPS 都很低（< 20）

**可能原因**：
1. CPU 性能不足（老旧设备）
2. 浏览器 CPU 节流（笔记本省电模式）
3. 其他标签页占用资源

**解决**：
1. 关闭其他浏览器标签页
2. 笔记本接通电源，设置高性能模式
3. 降低光源数量和半径

---

## 📊 性能基准参考

### 现代 CPU（Intel i7/AMD Ryzen 7）

| 配置 | Zig SIMD | Rust Scalar | JavaScript |
|-----|----------|-------------|------------|
| 5 光源 | 5-8 ms (120+ FPS) | 12-18 ms (55-80 FPS) | 25-35 ms (28-40 FPS) |
| 10 光源 | 10-15 ms (66-100 FPS) | 25-35 ms (28-40 FPS) | 50-70 ms (14-20 FPS) |
| 20 光源 | 20-30 ms (33-50 FPS) | 50-70 ms (14-20 FPS) | 100-150 ms (6-10 FPS) |

### 移动设备（M1/M2 Mac）

| 配置 | Zig SIMD | Rust Scalar | JavaScript |
|-----|----------|-------------|------------|
| 5 光源 | 8-12 ms (83-125 FPS) | 18-25 ms (40-55 FPS) | 35-50 ms (20-28 FPS) |
| 10 光源 | 15-22 ms (45-66 FPS) | 35-50 ms (20-28 FPS) | 70-100 ms (10-14 FPS) |

---

## 🎓 学习要点

### 为什么 Zig SIMD 快？

1. **真正的向量化**：一次处理 4 个像素
2. **零拷贝内存**：直接操作 WASM 线性内存
3. **SIMD 指令**：f32x4.mul, f32x4.add, f32x4.sqrt
4. **无分支**：用 v128.bitselect 替代 if

### 为什么 Rust Scalar 慢？

1. **逐像素处理**：每次只算 1 个像素
2. **标量指令**：f32.mul, f32.add, f32.sqrt
3. **有分支**：每个光源都有 if 判断

### 为什么 JavaScript 最慢？

1. **解释执行**：即使有 JIT，仍比 WASM 慢
2. **类型检查**：动态类型运行时开销
3. **内存分配**：创建 ImageData 有拷贝开销

---

## 🚀 总结

AutoZig WASM 的核心优势：

1. ✅ **零拷贝**：Rust 和 Zig 共享内存，无数据传输
2. ✅ **SIMD**：Zig 的 `@Vector` 直接映射 WASM SIMD128
3. ✅ **类型安全**：编译期检查，运行时无开销
4. ✅ **简洁**：用 `include_zig!` 宏轻松集成

现在，**硬刷新浏览器**，开始测试吧！🎉