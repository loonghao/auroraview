# Quick Test Guide

## 5-Minute Integration Test

### Prerequisites Check
- [ ] Photoshop 2024+ installed
- [ ] Rust installed (`rustc --version`)
- [ ] UXP Developer Tool installed

### Step 1: Start Server (1 minute)

**Windows**:
```powershell
cd examples/photoshop_examples
.\run_server.ps1
```

**macOS/Linux**:
```bash
cd examples/photoshop_examples
chmod +x run_server.sh
./run_server.sh
```

**Expected Output**:
```
🚀 AuroraView WebSocket Server listening on: 127.0.0.1:9001
📡 Waiting for Photoshop UXP plugin to connect...
```

### Step 2: Load Plugin (2 minutes)

1. Open **UXP Developer Tool**
2. Click **Add Plugin**
3. Select: `examples/photoshop_examples/uxp_plugin/manifest.json`
4. Click **Load**
5. Open Photoshop → **Plugins → AuroraView**

### Step 3: Connect (30 seconds)

1. In AuroraView panel, click **Connect**
2. Status should turn green: "Connected"

**Server Console Should Show**:
```
✅ New connection from: 127.0.0.1:xxxxx
🔗 WebSocket connection established
🤝 Handshake from Photoshop
```

### Step 4: Test Actions (1.5 minutes)

#### Test 1: Create Layer
1. Create/open a document in Photoshop
2. Click **Create New Layer**
3. ✅ New layer appears in Photoshop
4. ✅ Server shows: `🎨 Layer created`

#### Test 2: Get Document Info
1. Click **Get Document Info**
2. ✅ Server shows document details

#### Test 3: Get Selection
1. Make a selection in Photoshop
2. Click **Get Selection Info**
3. ✅ Server shows selection bounds

## Success Criteria

✅ All 3 tests pass  
✅ No errors in server console  
✅ No errors in plugin log  
✅ Connection remains stable  

## Troubleshooting

### Server won't start
```bash
# Check if port is in use
netstat -an | findstr 9001  # Windows
lsof -i :9001              # macOS/Linux

# Kill process if needed
# Then restart server
```

### Plugin won't connect
1. Verify server is running
2. Check URL: `ws://localhost:9001`
3. Restart Photoshop
4. Reload plugin in UXP Developer Tool

### No layer created
1. Ensure document is open in Photoshop
2. Check plugin log for errors
3. Try creating layer manually first

## Next Steps

After successful test:
- Read `README.md` for detailed documentation
- Explore `docs/PHOTOSHOP_INTEGRATION_GUIDE.md`
- Customize the code for your needs
- Integrate with AuroraView core

## Support

- Check `docs/PHOTOSHOP_INTEGRATION_GUIDE.md` FAQ section
- Review server console logs
- Check UXP Developer Tool debug console

