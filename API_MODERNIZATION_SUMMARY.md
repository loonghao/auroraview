# AuroraView API 现代化总结

## 📚 参考示例

基于官方示例：`c:\Users\hallo\Documents\augment-projects\dcc_webview\examples\maya_qt_echo_demo.py`

## ✅ 完成的工作

### 第一阶段：基础 API 更新（提交 4b52880）

#### WebView 创建现代化
- ✅ 使用 `WebView.create()` 工厂方法
- ✅ 更新参数名称：`dev_tools` → `debug`, `parent_hwnd` → `parent`
- ✅ 启用 `auto_timer=True` 自动管理 EventTimer
- ✅ 使用 `mode="auto"` 自动检测嵌入模式

#### EventTimer 自动管理
- ✅ 移除手动 EventTimer 创建代码
- ✅ 删除 `_start_event_processing()` 和 `_stop_event_processing()`
- ✅ WebView.close() 自动停止 EventTimer

### 第二阶段：API 模式现代化（提交 29d2354）

#### 后端改进

**1. 创建 MayaOutlinerAPI 类**
```python
class MayaOutlinerAPI:
    """API object exposed to JavaScript via auroraview.api.*"""
    
    def get_scene_hierarchy(self) -> List[Dict[str, Any]]:
        """Get Maya scene hierarchy"""
        
    def select_node(self, node_name: str) -> Dict[str, Any]:
        """Select a node in Maya"""
        
    def set_visibility(self, node_name: str, visible: bool) -> Dict[str, Any]:
        """Set node visibility"""
```

**2. 使用 bind_api() 注册 API**
```python
# 创建 API 对象
self.api = MayaOutlinerAPI(self)

# 绑定到 JavaScript
self.webview.bind_api(self.api, namespace="api")
```

**3. 改进返回值**
```python
# 之前：无返回值
def select_node(self, node_name: str):
    cmds.select(node_name)

# 现在：返回结果字典
def select_node(self, node_name: str) -> Dict[str, Any]:
    try:
        cmds.select(node_name)
        return {"ok": True, "message": f"Selected: {node_name}"}
    except Exception as e:
        return {"ok": False, "message": str(e)}
```

#### 前端改进

**1. 添加 TypeScript 类型定义**
```typescript
declare global {
  interface Window {
    auroraview?: {
      api?: {
        get_scene_hierarchy?: () => Promise<any[]>
        select_node?: (node_name: string) => Promise<{ ok: boolean; message: string }>
        set_visibility?: (node_name: string, visible: boolean) => Promise<{ ok: boolean; message: string }>
      }
    }
  }
}
```

**2. 创建 callAPI() 辅助函数**
```typescript
const callAPI = async <T = any>(method: string, ...args: any[]): Promise<T> => {
  if (window.auroraview?.api) {
    const apiMethod = (window.auroraview.api as any)[method]
    if (typeof apiMethod === 'function') {
      return await apiMethod(...args)
    }
  }
  // Fallback to legacy IPC
  throw new Error('API not available')
}
```

**3. 添加便捷方法**
```typescript
const getSceneHierarchy = async () => {
  return callAPI<any[]>('get_scene_hierarchy')
}

const selectNode = async (nodeName: string) => {
  return callAPI<{ ok: boolean; message: string }>('select_node', nodeName)
}
```

**4. 更新 Vue 组件使用 async/await**
```typescript
// 之前：事件驱动
sendToMaya('select_node', { node_name: nodeName })

// 现在：API 调用
const result = await selectNode(nodeName)
console.log('Result:', result)
```

## 📊 API 对比

### 调用方式对比

| 方面 | 旧方式（事件） | 新方式（API） |
|------|---------------|--------------|
| Python 端 | `@webview.on("event")` | `bind_api(api_object)` |
| JavaScript 端 | `dispatchEvent(CustomEvent)` | `await auroraview.api.method()` |
| 返回值 | 通过事件回调 | 直接返回 Promise |
| 类型安全 | 无 | TypeScript 类型定义 |
| 错误处理 | 事件监听 | try-catch |
| 代码可读性 | 低（事件名称字符串） | 高（方法调用） |

### 代码示例对比

#### 获取场景层级

**旧方式（事件驱动）**
```typescript
// Frontend
sendToMaya('get_scene_hierarchy', {})

onMayaEvent('scene_updated', (data) => {
  sceneData.value = data
})

// Backend
@webview.on("get_scene_hierarchy")
def handle_get_hierarchy(data):
    hierarchy = self.get_scene_hierarchy()
    self.webview.emit("scene_updated", hierarchy)
```

**新方式（API 调用）**
```typescript
// Frontend
const hierarchy = await getSceneHierarchy()
sceneData.value = hierarchy

// Backend
class MayaOutlinerAPI:
    def get_scene_hierarchy(self) -> List[Dict]:
        return self._outliner.get_scene_hierarchy()

# 自动绑定
webview.bind_api(api, namespace="api")
```

## 🎯 关键改进

### 1. 更清晰的 API 设计
- **之前**: 事件名称字符串，容易拼写错误
- **现在**: 方法调用，IDE 自动补全和类型检查

### 2. 更好的错误处理
- **之前**: 事件丢失无提示
- **现在**: Promise rejection，可以 try-catch

### 3. 更好的类型安全
- **之前**: 无类型定义
- **现在**: TypeScript 类型定义，编译时检查

### 4. 更好的可维护性
- **之前**: 事件处理器分散在代码中
- **现在**: API 类集中管理，清晰的接口

### 5. 更符合最佳实践
- 参考官方示例 `maya_qt_echo_demo.py`
- 使用 `bind_api()` 模式
- 返回结果字典而不是 void

## 📝 Git 提交记录

```
29d2354 feat: modernize API to use bind_api pattern
4b52880 feat: update to latest AuroraView API (2025)
```

## 🧪 测试验证

### 1. 启动 Maya
```bash
just maya-2024-local
```

### 2. 打开浏览器开发者工具（F12）

### 3. 测试 API 调用
```javascript
// 测试获取场景层级
const hierarchy = await window.auroraview.api.get_scene_hierarchy()
console.log('Hierarchy:', hierarchy)

// 测试选择节点
const result = await window.auroraview.api.select_node('pCube1')
console.log('Select result:', result)

// 测试设置可见性
const visResult = await window.auroraview.api.set_visibility('pCube1', false)
console.log('Visibility result:', visResult)
```

## 🎉 总结

本次更新成功将项目从旧的事件驱动模式迁移到现代的 API 调用模式：

1. ✅ 使用 `bind_api()` 暴露 Python 方法
2. ✅ 前端使用 `await auroraview.api.method()` 调用
3. ✅ 添加 TypeScript 类型定义
4. ✅ 改进错误处理和日志
5. ✅ 遵循 AuroraView 最佳实践

代码更清晰、更安全、更易维护！🚀

