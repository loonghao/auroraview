# AuroraView - DCC集成指南

## 概述

AuroraView 专为DCC软件（Maya、Houdini、Blender等）设计，提供了原生的集成支持。本指南展示如何在各种DCC软件中使用 AuroraView。

---

## Maya集成

### 基础示例

```python
# maya_tool.py
from auroraview import WebView
import maya.cmds as cmds

class MayaTool:
    def __init__(self):
        self.webview = WebView(
            title="Maya Tool",
            width=800,
            height=600
        )
        self.setup_events()
    
    def setup_events(self):
        """设置事件处理"""
        @self.webview.on("export_scene")
        def handle_export(data):
            path = data.get('path')
            cmds.file(path, save=True)
        
        @self.webview.on("get_scene_info")
        def handle_get_info(data):
            nodes = cmds.ls()
            self.webview.emit("scene_info", {
                "node_count": len(nodes),
                "nodes": nodes[:10]  # 前10个
            })
    
    def show(self):
        html = """
        <html>
        <body>
            <h1>Maya Tool</h1>
            <button onclick="exportScene()">Export Scene</button>
            <button onclick="getSceneInfo()">Get Info</button>
            <div id="info"></div>
            
            <script>
                function exportScene() {
                    window.dispatchEvent(new CustomEvent('export_scene', {
                        detail: { path: '/tmp/scene.mb' }
                    }));
                }
                
                function getSceneInfo() {
                    window.dispatchEvent(new CustomEvent('get_scene_info', {}));
                }
                
                window.addEventListener('scene_info', (e) => {
                    document.getElementById('info').innerHTML = 
                        'Nodes: ' + e.detail.node_count;
                });
            </script>
        </body>
        </html>
        """
        self.webview.load_html(html)
        self.webview.show()

# 在Maya中使用
tool = MayaTool()
tool.show()
```

### Maya插件集成

```python
# maya_plugin.py
import maya.api.OpenMaya as om
from auroraview import WebView

class MayaWebViewPlugin:
    PLUGIN_NAME = "mayaWebViewTool"
    
    def __init__(self):
        self.webview = None
    
    def create_ui(self):
        """创建UI"""
        self.webview = WebView(title="Maya WebView Tool")
        # 加载UI...
        self.webview.show()
    
    def on_scene_changed(self):
        """场景变化回调"""
        if self.webview:
            self.webview.emit("scene_changed", {
                "timestamp": om.MTime.currentTime().value()
            })

def initializePlugin(plugin):
    """插件初始化"""
    plugin_obj = om.MFnPlugin(plugin)
    # 注册命令等...

def uninitializePlugin(plugin):
    """插件卸载"""
    plugin_obj = om.MFnPlugin(plugin)
    # 清理...
```

---

## Houdini集成

### 基础示例

```python
# houdini_tool.py
from auroraview import WebView
import hou

class HoudiniTool:
    def __init__(self):
        self.webview = WebView(
            title="Houdini Tool",
            width=1024,
            height=768
        )
        self.setup_events()
    
    def setup_events(self):
        @self.webview.on("get_nodes")
        def handle_get_nodes(data):
            nodes = hou.node("/obj").children()
            self.webview.emit("nodes_list", {
                "nodes": [n.name() for n in nodes]
            })
        
        @self.webview.on("set_parameter")
        def handle_set_param(data):
            node_path = data.get('node')
            param = data.get('param')
            value = data.get('value')
            
            node = hou.node(node_path)
            if node:
                node.parm(param).set(value)
    
    def show(self):
        html = """
        <html>
        <body>
            <h1>Houdini Tool</h1>
            <button onclick="getNodes()">Get Nodes</button>
            <div id="nodes"></div>
            
            <script>
                function getNodes() {
                    window.dispatchEvent(new CustomEvent('get_nodes', {}));
                }
                
                window.addEventListener('nodes_list', (e) => {
                    const list = e.detail.nodes.join('<br>');
                    document.getElementById('nodes').innerHTML = list;
                });
            </script>
        </body>
        </html>
        """
        self.webview.load_html(html)
        self.webview.show()

# 在Houdini中使用
tool = HoudiniTool()
tool.show()
```

---

## Blender集成

### 基础示例

```python
# blender_addon.py
import bpy
from auroraview import WebView

class BlenderWebViewTool:
    def __init__(self):
        self.webview = WebView(
            title="Blender Tool",
            width=800,
            height=600
        )
        self.setup_events()
    
    def setup_events(self):
        @self.webview.on("get_objects")
        def handle_get_objects(data):
            objects = [obj.name for obj in bpy.data.objects]
            self.webview.emit("objects_list", {
                "objects": objects
            })
        
        @self.webview.on("select_object")
        def handle_select(data):
            obj_name = data.get('name')
            obj = bpy.data.objects.get(obj_name)
            if obj:
                bpy.context.view_layer.objects.active = obj
                obj.select_set(True)
    
    def show(self):
        html = """
        <html>
        <body>
            <h1>Blender Tool</h1>
            <button onclick="getObjects()">Get Objects</button>
            <ul id="objects"></ul>
            
            <script>
                function getObjects() {
                    window.dispatchEvent(new CustomEvent('get_objects', {}));
                }
                
                window.addEventListener('objects_list', (e) => {
                    const list = e.detail.objects
                        .map(obj => '<li>' + obj + '</li>')
                        .join('');
                    document.getElementById('objects').innerHTML = list;
                });
            </script>
        </body>
        </html>
        """
        self.webview.load_html(html)
        self.webview.show()

# 在Blender中使用
tool = BlenderWebViewTool()
tool.show()
```

---

## 最佳实践

### 1. 线程安全
```python
# [OK] 正确: 在主线程中调用
webview.emit("update", data)

# [ERROR] 错误: 在后台线程中调用
import threading
threading.Thread(target=lambda: webview.emit("update", data)).start()
```

### 2. 事件处理
```python
# [OK] 正确: 使用装饰器
@webview.on("my_event")
def handle_event(data):
    print(data)

# [OK] 也可以: 使用register_callback
def handle_event(data):
    print(data)
webview.register_callback("my_event", handle_event)
```

### 3. 错误处理
```python
try:
    webview.load_html(html_content)
except Exception as e:
    print(f"Error loading HTML: {e}")
```

### 4. 资源管理
```python
# 使用上下文管理器
with WebView(title="Tool") as webview:
    webview.load_html(html)
    webview.show()
# 自动清理
```

---

## 常见问题

### Q: 如何在DCC中后台运行WebView？
A: 使用线程池或异步任务，但确保所有WebView操作在主线程中。

### Q: 如何处理大量数据传输？
A: 使用分页或流式传输，避免一次性发送大量数据。

### Q: 如何调试JavaScript代码？
A: 启用开发者工具: `WebView(..., dev_tools=True)`

### Q: 如何处理DCC关闭时的清理？
A: 使用上下文管理器或注册清理回调。

---

## 性能优化建议

1. **减少事件频率**: 使用节流和防抖
2. **异步加载**: 不要阻塞DCC主线程
3. **缓存数据**: 避免重复查询
4. **优化HTML/CSS**: 减少DOM操作
5. **使用Web Workers**: 处理复杂计算

---

## 下一步

- 查看 `examples/` 目录中的完整示例
- 阅读 API 文档
- 参与社区讨论

