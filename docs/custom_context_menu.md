# Custom Context Menu Configuration

AuroraView supports disabling the native browser context menu and implementing custom menus through JavaScript.

## Disabling Native Context Menu

### WebView

```python
from auroraview import WebView

# Disable native context menu
webview = WebView.create(
    title="My Tool",
    url="http://localhost:3000",
    context_menu=False  # Disable native right-click menu
)
webview.show()
```

### QtWebView

```python
from auroraview import QtWebView

# Disable native context menu in Qt widget
webview = QtWebView(
    parent=maya_main_window(),
    title="My Tool",
    width=800,
    height=600,
    context_menu=False  # Disable native right-click menu
)
webview.show()
```

## Implementing Custom Context Menu

Once the native context menu is disabled, you can implement your own custom menu using JavaScript:

### HTML/JavaScript Example

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
            <li onclick="handleMenuAction('export')">Export Scene</li>
            <li onclick="handleMenuAction('import')">Import Assets</li>
            <li onclick="handleMenuAction('settings')">Settings</li>
        </ul>
    </div>

    <script>
        const menu = document.getElementById('customMenu');
        
        // Show custom menu on right-click
        document.addEventListener('contextmenu', (e) => {
            e.preventDefault();
            menu.style.display = 'block';
            menu.style.left = e.pageX + 'px';
            menu.style.top = e.pageY + 'px';
        });
        
        // Hide menu on click elsewhere
        document.addEventListener('click', () => {
            menu.style.display = 'none';
        });
        
        // Handle menu actions
        function handleMenuAction(action) {
            // Send action to Python
            window.auroraview.send_event('menu_action', { action: action });
            menu.style.display = 'none';
        }
    </script>
</body>
</html>
```

### Python Handler

```python
from auroraview import WebView

webview = WebView.create(
    title="Custom Menu Demo",
    context_menu=False
)

@webview.on('menu_action')
def handle_menu_action(data):
    action = data.get('action')
    print(f"Menu action: {action}")
    
    if action == 'export':
        # Handle export
        pass
    elif action == 'import':
        # Handle import
        pass
    elif action == 'settings':
        # Show settings
        pass

webview.load_file('custom_menu.html')
webview.show()
```

## Developer Console Configuration

The developer console (DevTools) can also be configured:

```python
from auroraview import WebView

# Disable both context menu and dev tools for production
webview = WebView.create(
    title="Production Tool",
    url="http://localhost:3000",
    debug=False,  # Disable developer tools
    context_menu=False  # Disable context menu
)
webview.show()
```

## Best Practices

1. **Always provide alternative functionality**: When disabling the native context menu, ensure users can access the same functionality through your custom UI.

2. **Keyboard shortcuts**: Implement keyboard shortcuts for common actions as an alternative to menu items.

3. **Accessibility**: Ensure your custom menu is accessible via keyboard navigation.

4. **Consistent UX**: Match your custom menu style to the host DCC application for a seamless experience.

## Platform Notes

- **Windows**: Context menu disabling is fully supported via WebView2 settings
- **macOS/Linux**: Context menu behavior may vary depending on the WebView backend

