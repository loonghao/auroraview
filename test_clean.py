import sys
try:
    from auroraview_mcp import McpConfig, McpServer
    print("import OK")
except Exception as e:
    print(f"import FAILED: {e}")
    sys.exit(1)

# Test McpConfig
cfg = McpConfig(port=7890, host="127.0.0.1", service_name="test",
              enable_mdns=True, enable_oauth=False, max_webviews=None)
assert cfg.port == 7890
assert cfg.host == "127.0.0.1"
assert cfg.service_name == "test"
assert cfg.enable_mdns is True
assert cfg.max_webviews is None
print("McpConfig: OK")

cfg2 = McpConfig(port=7891, host="0.0.0.0", service_name="test",
               enable_mdns=False, enable_oauth=False, max_webviews=5)
assert cfg2.max_webviews == 5
print("McpConfig with max_webviews: OK")

# Test McpServer creation
server = McpServer(port=7890)
assert server.port == 7890
assert server.is_running() is False
print("McpServer create: OK")

# Test from_config
cfg3 = McpConfig(port=7891, host="127.0.0.1", service_name="test",
               enable_mdns=False, enable_oauth=False, max_webviews=None)
server2 = McpServer.from_config(cfg3)
assert server2.port == 7891
assert server2.is_running() is False
print("McpServer.from_config: OK")

# Test start/stop
server3 = McpServer(port=17890)
assert server3.is_running() is False
result = server3.start()
assert result is None
print("McpServer.start(): OK")

# Double-start should fail
try:
    server3.start()
    print("  WARN: double-start did not raise")
except Exception as e:
    print(f"McpServer double-start rejected: OK (error: {e})")

server3.stop()
print("McpServer stop: OK")

print("ALL PASSED")
