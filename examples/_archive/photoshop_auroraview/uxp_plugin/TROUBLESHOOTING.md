# UXP Plugin Troubleshooting Guide

## Common Issues and Solutions

### 1. Plugin Load Failed

**Error Message:**
```
Validate command successful in App with ID PS v26.5.0
Load command failed in App with ID PS v26.5.0
Plugin Load Failed.
Devtools: Failed to load the devtools plugin.
```

**Possible Causes:**

#### A. DOM Access Before Ready
**Problem**: Accessing DOM elements before `DOMContentLoaded` event.

**Solution**: ✅ Fixed in latest version
```javascript
// ❌ Wrong - DOM not ready yet
const statusEl = document.getElementById('status');

// ✅ Correct - Wait for DOM
document.addEventListener('DOMContentLoaded', () => {
    const statusEl = document.getElementById('status');
});
```

#### B. Syntax Errors
**Problem**: JavaScript syntax errors in `index.js`.

**Solution**: Check the UXP Developer Tool console for error messages.

#### C. Missing Files
**Problem**: `manifest.json`, `index.html`, or `index.js` missing.

**Solution**: Ensure all required files exist:
```
uxp_plugin/
├── manifest.json
├── index.html
└── index.js
```

#### D. Invalid manifest.json
**Problem**: JSON syntax errors or invalid configuration.

**Solution**: Validate JSON syntax and check required fields:
```json
{
  "manifestVersion": 5,
  "id": "com.auroraview.photoshop.minimal",
  "name": "AuroraView Bridge (Minimal)",
  "version": "1.0.0",
  "main": "index.html",
  "host": [
    {
      "app": "PS",
      "minVersion": "24.0.0"
    }
  ],
  "entrypoints": [
    {
      "type": "panel",
      "id": "auroraview.minimal",
      "label": {
        "default": "AuroraView (Minimal)"
      }
    }
  ],
  "requiredPermissions": {
    "network": {
      "domains": [
        "ws://localhost:*",
        "wss://localhost:*"
      ]
    }
  }
}
```

### 2. Connection Failed

**Error Message:**
```
WebSocket connection failed
```

**Solutions:**

#### A. Python Backend Not Running
```bash
# Start the Python backend first
python examples/photoshop_layers_demo/photoshop_layers_tool.py
```

#### B. Wrong Port
Check the port in `index.js`:
```javascript
// Should match the Bridge port
socket = new WebSocket('ws://localhost:9001');
```

#### C. Firewall Blocking
- Allow Python through Windows Firewall
- Check antivirus settings

### 3. Commands Not Working

**Problem**: Clicking buttons has no effect.

**Solutions:**

#### A. No Active Document
```
Error: No active document
```
**Solution**: Open a document in Photoshop before using the plugin.

#### B. WebSocket Disconnected
**Solution**: Check connection status and reconnect if needed.

#### C. Python Handler Not Registered
**Solution**: Ensure all handlers are registered in Python:
```python
@bridge.on('layer_created')
async def handle_layer_created(data, client):
    # Handler code
    pass
```

### 4. Debugging Steps

#### Step 1: Check UXP Developer Tool
1. Open Photoshop
2. Plugins → Development → UXP Developer Tool
3. Check the "Logs" tab for errors

#### Step 2: Check Python Logs
Look for these messages:
```
✅ Found free port: 9001
✅ Bridge initialized: localhost:9001
✅ WebView created with Bridge on port 9001
```

#### Step 3: Test WebSocket Connection
Use browser console:
```javascript
const ws = new WebSocket('ws://localhost:9001');
ws.onopen = () => console.log('Connected!');
ws.onerror = (e) => console.error('Error:', e);
```

#### Step 4: Reload Plugin
1. In UXP Developer Tool
2. Click "..." next to plugin name
3. Click "Reload"

### 5. Clean Reinstall

If all else fails:

```bash
# 1. Remove plugin from UXP Developer Tool
# 2. Close Photoshop
# 3. Restart Photoshop
# 4. Re-add plugin in UXP Developer Tool
# 5. Load plugin
```

### 6. Version Compatibility

**Minimum Requirements:**
- Photoshop 24.0.0 or later
- UXP API version 2.0+
- Python 3.7+

**Check Photoshop Version:**
```
Help → About Photoshop
```

### 7. Network Permissions

Ensure `manifest.json` includes network permissions:
```json
"requiredPermissions": {
  "network": {
    "domains": [
      "ws://localhost:*",
      "wss://localhost:*"
    ]
  }
}
```

### 8. Common Error Messages

| Error | Cause | Solution |
|-------|-------|----------|
| `Cannot read property 'getElementById' of null` | DOM not ready | Use DOMContentLoaded |
| `WebSocket is not defined` | Network permissions missing | Add to manifest.json |
| `app is not defined` | Missing require | Add `const { app } = require('photoshop')` |
| `No active document` | No document open | Open a document |
| `Connection refused` | Python not running | Start Python backend |

### 9. Getting Help

If you still have issues:

1. **Check Logs**: Look at both UXP and Python logs
2. **Minimal Test**: Try the minimal example first
3. **Report Issue**: Include:
   - Photoshop version
   - Error messages
   - UXP Developer Tool logs
   - Python logs

### 10. Quick Fix Checklist

- [ ] All files present (manifest.json, index.html, index.js)
- [ ] No syntax errors in JavaScript
- [ ] DOMContentLoaded event used
- [ ] Network permissions in manifest.json
- [ ] Python backend running
- [ ] Correct port number
- [ ] Document open in Photoshop
- [ ] Photoshop version >= 24.0.0

---

## Latest Fix Applied

✅ **Fixed DOM Access Issue** (2025-11-09)

**Problem**: DOM elements accessed before DOMContentLoaded
**Solution**: Wrapped initialization in DOMContentLoaded event

```javascript
// Now using proper initialization
document.addEventListener('DOMContentLoaded', () => {
    statusEl = document.getElementById('status');
    logEl = document.getElementById('log');
    connectBtn = document.getElementById('connectBtn');
    
    connectBtn.addEventListener('click', connect);
    log('AuroraView Bridge initialized');
    updateStatus(false);
});
```

**Please reload the plugin in UXP Developer Tool to apply this fix.**

