# 自定义右键菜单

AuroraView 支持禁用原生浏览器右键菜单，并通过 JavaScript 实现自定义菜单。

## 禁用原生右键菜单

### WebView

```python
from auroraview import WebView

# 禁用原生右键菜单
webview = WebView.create(
    title="My Tool",
    url="http://localhost:3000",
    context_menu=False  # 禁用原生右键菜单
)
webview.show()
```

### QtWebView

```python
from auroraview import QtWebView

# 在 Qt 组件中禁用原生右键菜单
webview = QtWebView(
    parent=maya_main_window(),
    title="My Tool",
    width=800,
    height=600,
    context_menu=False  # 禁用原生右键菜单
)
webview.show()
```

## 实现自定义右键菜单

禁用原生右键菜单后，你可以使用 JavaScript 实现自定义菜单：

### HTML/JavaScript 示例

```html
<!DOCTYPE html>
<html>
<head>
    <style>
        .custom-menu {
            display: none;
            position: absolute;
            background: white;
            border: 1px solid #ccc;
            box-shadow: 2px 2px 5px rgba(0,0,0,0.2);
            z-index: 1000;
        }
        
        .custom-menu ul {
            list-style: none;
            margin: 0;
            padding: 5px 0;
        }
        
        .custom-menu li {
            padding: 8px 20px;
            cursor: pointer;
        }
        
        .custom-menu li:hover {
            background: #f0f0f0;
        }
    </style>
</head>
<body>
    <div id="customMenu" class="custom-menu">
        <ul>
            <li onclick="handleMenuAction('export')">导出场景</li>
            <li onclick="handleMenuAction('import')">导入资产</li>
            <li onclick="handleMenuAction('settings')">设置</li>
        </ul>
    </div>

    <script>
        const menu = document.getElementById('customMenu');
        
        // 右键点击时显示自定义菜单
        document.addEventListener('contextmenu', (e) => {
            e.preventDefault();
            menu.style.display = 'block';
            menu.style.left = e.pageX + 'px';
            menu.style.top = e.pageY + 'px';
        });
        
        // 点击其他地方时隐藏菜单
        document.addEventListener('click', () => {
            menu.style.display = 'none';
        });
        
        // 处理菜单操作
        function handleMenuAction(action) {
            // 发送操作到 Python
            window.auroraview.send_event('menu_action', { action: action });
            menu.style.display = 'none';
        }
    </script>
</body>
</html>
```

### Python 处理器

```python
from auroraview import WebView

webview = WebView.create(
    title="自定义菜单演示",
    context_menu=False
)

@webview.on('menu_action')
def handle_menu_action(data):
    action = data.get('action')
    print(f"菜单操作: {action}")
    
    if action == 'export':
        # 处理导出
        pass
    elif action == 'import':
        # 处理导入
        pass
    elif action == 'settings':
        # 显示设置
        pass

webview.load_file('custom_menu.html')
webview.show()
```

## 开发者控制台配置

开发者控制台（DevTools）也可以配置：

```python
from auroraview import WebView

# 生产环境禁用右键菜单和开发者工具
webview = WebView.create(
    title="生产工具",
    url="http://localhost:3000",
    debug=False,  # 禁用开发者工具
    context_menu=False  # 禁用右键菜单
)
webview.show()
```

## React 示例

```jsx
import { useState, useEffect } from 'react';

function ContextMenu({ x, y, onClose, onAction }) {
  return (
    <div 
      className="context-menu"
      style={{ left: x, top: y }}
    >
      <ul>
        <li onClick={() => onAction('export')}>导出场景</li>
        <li onClick={() => onAction('import')}>导入资产</li>
        <li onClick={() => onAction('settings')}>设置</li>
      </ul>
    </div>
  );
}

function App() {
  const [menu, setMenu] = useState(null);

  useEffect(() => {
    const handleContextMenu = (e) => {
      e.preventDefault();
      setMenu({ x: e.pageX, y: e.pageY });
    };

    const handleClick = () => setMenu(null);

    document.addEventListener('contextmenu', handleContextMenu);
    document.addEventListener('click', handleClick);

    return () => {
      document.removeEventListener('contextmenu', handleContextMenu);
      document.removeEventListener('click', handleClick);
    };
  }, []);

  const handleAction = (action) => {
    window.auroraview.send_event('menu_action', { action });
    setMenu(null);
  };

  return (
    <div>
      {/* 你的应用内容 */}
      {menu && (
        <ContextMenu 
          x={menu.x} 
          y={menu.y} 
          onClose={() => setMenu(null)}
          onAction={handleAction}
        />
      )}
    </div>
  );
}
```

## 最佳实践

1. **始终提供替代功能**：禁用原生右键菜单时，确保用户可以通过自定义 UI 访问相同功能。

2. **键盘快捷键**：为常用操作实现键盘快捷键作为菜单项的替代。

3. **无障碍访问**：确保自定义菜单可通过键盘导航访问。

4. **一致的用户体验**：将自定义菜单样式与宿主 DCC 应用程序匹配，以获得无缝体验。

5. **上下文感知菜单**：根据用户右键点击的内容显示不同的菜单项：

```javascript
document.addEventListener('contextmenu', (e) => {
    e.preventDefault();
    
    const target = e.target;
    let menuItems = [];
    
    if (target.classList.contains('node')) {
        menuItems = ['删除节点', '复制节点', '编辑属性'];
    } else if (target.classList.contains('connection')) {
        menuItems = ['删除连接', '编辑连接'];
    } else {
        menuItems = ['创建节点', '粘贴', '设置'];
    }
    
    showMenu(e.pageX, e.pageY, menuItems);
});
```

## 平台说明

- **Windows**：通过 WebView2 设置完全支持禁用右键菜单
- **macOS/Linux**：右键菜单行为可能因 WebView 后端而异
