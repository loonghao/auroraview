# Troubleshooting Guide

This guide helps you solve common issues with AuroraView.

## White Screen Issue

### Problem

You create a WebView and call `show()`, but you only see a white/blank screen.

### Symptoms

- WebView window opens successfully
- Console shows "Auroraview event bridge initialized"
- But the window content is completely white/blank
- No errors in the console

### Root Causes

#### 1. URL Not Loading (Most Common)

**Cause**: You're trying to load a URL (like `http://localhost:3000`) but:
- The server is not running
- The server is running on a different port
- The URL is incorrect

**Solution**:

```python
# ❌ WRONG - This might show white screen if server is not running
from auroraview import WebView

webview = WebView(
    title="My App",
    width=800,
    height=600
)
webview.load_url("http://localhost:3000")  # Server not running!
webview.show()

# ✅ CORRECT - Load HTML content directly
from auroraview import WebView

webview = WebView(
    title="My App",
    width=800,
    height=600
)

html = """
<!DOCTYPE html>
<html>
<head>
    <title>My App</title>
    <style>
        body {
            font-family: Arial, sans-serif;
            display: flex;
            justify-content: center;
            align-items: center;
            height: 100vh;
            margin: 0;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
        }
    </style>
</head>
<body>
    <div>
        <h1>Hello from AuroraView!</h1>
        <p>This works without any server!</p>
    </div>
</body>
</html>
"""

webview.load_html(html)
webview.show()
```

#### 2. Server Not Started

**Cause**: You're loading `http://localhost:3000` but forgot to start your development server.

**Solution**:

1. **Start your server first**:
   ```bash
   # For React
   cd your-react-app
   npm start
   
   # For Vue
   cd your-vue-app
   npm run dev
   
   # For simple HTTP server
   cd your-html-folder
   python -m http.server 3000
   ```

2. **Then run your Python code**:
   ```python
   from auroraview import WebView
   
   webview = WebView(title="My App", width=800, height=600)
   webview.load_url("http://localhost:3000")  # Now the server is running!
   webview.show()
   ```

#### 3. CORS Policy Issues

**Cause**: Browser security blocks cross-origin requests.

**Solution**:

If you're loading content from a different origin, you might need to:

1. **Configure your server to allow CORS**:
   ```javascript
   // Express.js example
   const cors = require('cors');
   app.use(cors());
   ```

2. **Or load HTML directly**:
   ```python
   # Instead of loading from URL, embed the content
   webview.load_html(your_html_content)
   ```

#### 4. JavaScript Errors

**Cause**: The page has JavaScript errors that prevent rendering.

**Solution**:

1. **Enable DevTools** (enabled by default):
   ```python
   webview = WebView(
       title="My App",
       width=800,
       height=600,
       dev_tools=True  # This is the default
   )
   ```

2. **Open DevTools in the WebView**:
   - Right-click in the WebView window
   - Select "Inspect" or "Inspect Element"
   - Check the Console tab for errors

3. **Fix the JavaScript errors** in your code

### Diagnostic Steps

#### Step 1: Test with Simple HTML

First, verify that AuroraView works at all:

```python
from auroraview import WebView

webview = WebView(title="Test", width=800, height=600)
webview.load_html("<h1>Hello World</h1>")
webview.show()
```

If this works, AuroraView is fine. The problem is with your URL/content.

#### Step 2: Check if Server is Running

```python
import socket

def is_server_running(host="localhost", port=3000):
    try:
        sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        sock.settimeout(1)
        result = sock.connect_ex((host, port))
        sock.close()
        return result == 0
    except:
        return False

if is_server_running():
    print("✅ Server is running")
else:
    print("❌ Server is NOT running - start it first!")
```

#### Step 3: Test with Public URL

Test with a known-good URL:

```python
from auroraview import WebView

webview = WebView(title="Test", width=800, height=600)
webview.load_url("https://www.example.com")
webview.show()
```

If this works, your localhost server is the problem.

#### Step 4: Use Diagnostic Script

Run the diagnostic script included with AuroraView:

```bash
python examples/diagnose_white_screen.py
```

This will:
- Check if your server is running
- Provide specific solutions
- Create a working example

### Quick Fixes

#### Fix 1: Use HTML Instead of URL

```python
# Instead of this:
webview.load_url("http://localhost:3000")

# Do this:
with open("path/to/your/index.html", "r") as f:
    html = f.read()
webview.load_html(html)
```

#### Fix 2: Check Server Before Loading

```python
import socket
from auroraview import WebView

def is_port_open(port):
    try:
        sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        sock.settimeout(1)
        result = sock.connect_ex(("localhost", port))
        sock.close()
        return result == 0
    except:
        return False

webview = WebView(title="My App", width=800, height=600)

if is_port_open(3000):
    webview.load_url("http://localhost:3000")
else:
    webview.load_html("<h1>Error: Server not running on port 3000</h1>")

webview.show()
```

#### Fix 3: Add Error Handling

```python
from auroraview import WebView
import time

webview = WebView(title="My App", width=800, height=600)

try:
    webview.load_url("http://localhost:3000")
    time.sleep(1)  # Give it time to load
    webview.show()
except Exception as e:
    print(f"Error: {e}")
    # Show error page
    webview.load_html(f"<h1>Error: {e}</h1>")
    webview.show()
```

## Other Common Issues

### Issue: WebView Closes Immediately

**Cause**: In embedded mode, the Python object is garbage collected.

**Solution**:

```python
# ❌ WRONG
def create_webview():
    webview = NativeWebView.embedded(parent_hwnd=maya_hwnd)
    webview.show()
    # webview is destroyed when function returns!

# ✅ CORRECT
# Store reference in a global or long-lived object
import __main__
__main__.my_webview = NativeWebView.embedded(parent_hwnd=maya_hwnd)
__main__.my_webview.show()
```

### Issue: Maya Freezes

**Cause**: Using "child" mode instead of "owner" mode.

**Solution**:

```python
# ❌ WRONG - Can cause freezes
webview = NativeWebView.embedded(
    parent_hwnd=maya_hwnd,
    mode="child"  # Don't use this!
)

# ✅ CORRECT - Use owner mode
webview = NativeWebView.embedded(
    parent_hwnd=maya_hwnd,
    mode="owner"  # This prevents freezes
)
```

### Issue: Qt Import Error

**Cause**: Qt dependencies not installed.

**Solution**:

```bash
# Install Qt dependencies
pip install auroraview[qt]
```

Or use the Native backend instead:

```python
from auroraview import NativeWebView  # No Qt required
```

## Getting Help

If you're still having issues:

1. **Run the diagnostic script**:
   ```bash
   python examples/diagnose_white_screen.py
   ```

2. **Check the examples**:
   ```bash
   python examples/debug_webview.py
   ```

3. **Open an issue** on GitHub with:
   - Your operating system
   - Python version
   - AuroraView version
   - Complete error message
   - Minimal code to reproduce the issue

4. **Join our community** for real-time help

