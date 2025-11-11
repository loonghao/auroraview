# 🚀 5分钟快速开始

## 一键启动 (推荐)

### Windows
```powershell
.\start.ps1
```

### macOS/Linux
```bash
chmod +x start.sh
./start.sh
```

## 手动启动

### 1️⃣ 安装依赖

**Python:**
```bash
cd python
pip install -r requirements.txt
```

**UI:**
```bash
cd ui
npm install
```

### 2️⃣ 启动服务

**终端 1 - UI 开发服务器:**
```bash
cd ui
npm run dev
```
✅ 应该看到: `Local: http://localhost:5173`

**终端 2 - Python 后端:**
```bash
cd python
python photoshop_tool.py
```
✅ 应该看到:
- `🚀 Starting Photoshop Tool...`
- `✅ WebSocket bridge started`
- AuroraView 窗口打开

### 3️⃣ 加载 Photoshop 插件

1. 打开 **UXP Developer Tool**
2. **Add Plugin** → 选择 `uxp_plugin/manifest.json`
3. **Load** 插件
4. Photoshop: **插件 → AuroraView (Minimal)**
5. 点击 **Connect to Python**

✅ 应该看到: `Connected to Python` (绿色状态)

### 4️⃣ 测试功能

1. 在 Photoshop 中打开任意图像
2. 在 AuroraView UI 中点击 **Get Image from Photoshop**
3. 调整滑块,点击 **Apply Blur**
4. 查看实时预览! 🎉

## 🎯 验证清单

- [ ] Vite 运行在 http://localhost:5173
- [ ] AuroraView 窗口已打开
- [ ] WebSocket 服务器监听 9001 端口
- [ ] UXP 插件已加载
- [ ] 插件显示 "Connected to Python"
- [ ] 可以获取图像并应用滤镜

## ❓ 常见问题

**Q: AuroraView 窗口没打开?**
- 检查 Python 后端是否运行
- 确认已安装: `pip install auroraview`

**Q: UXP 插件无法连接?**
- 确保 Python 后端正在运行
- 检查端口 9001 是否被占用

**Q: 图像处理失败?**
- 安装依赖: `pip install Pillow opencv-python numpy`

## 📚 下一步

- 查看 [README.md](./README.md) 了解完整功能
- 查看 [README_zh.md](./README_zh.md) 中文文档
- 尝试添加自定义滤镜
- 集成 AI 模型

## 🎉 成功!

如果所有步骤都完成,你现在拥有:
- ✅ 现代 React UI (Vite 热更新)
- ✅ Python 图像处理能力
- ✅ Photoshop 双向通信
- ✅ 完整的开发环境

开始创建你的 AI 图像处理工具吧! 🚀

