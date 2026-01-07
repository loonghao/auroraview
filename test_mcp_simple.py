"""Simple MCP test script."""
import json
import urllib.request

BASE_URL = "http://127.0.0.1:27168"

def test_health():
    """Test health endpoint."""
    try:
        with urllib.request.urlopen(f"{BASE_URL}/health") as resp:
            data = json.loads(resp.read().decode())
            print(f"Health: {data}")
            return True
    except Exception as e:
        print(f"Health failed: {e}")
        return False

def test_tools_list():
    """Test tools list endpoint."""
    try:
        with urllib.request.urlopen(f"{BASE_URL}/tools") as resp:
            data = json.loads(resp.read().decode())
            print(f"Tools count: {data.get('count', 0)}")
            for tool in data.get("data", [])[:5]:
                print(f"  - {tool['name']}: {tool['description'][:50]}...")
            return True
    except Exception as e:
        print(f"Tools list failed: {e}")
        return False

def call_tool(name: str, args: dict = None):
    """Call a tool via MCP endpoint."""
    body = {
        "jsonrpc": "2.0",
        "method": "tools/call",
        "params": {
            "name": name,
            "arguments": args or {}
        },
        "id": 1
    }
    
    req = urllib.request.Request(
        f"{BASE_URL}/mcp",
        data=json.dumps(body).encode(),
        headers={
            "Content-Type": "application/json",
            "Accept": "application/json, text/event-stream"
        }
    )
    
    try:
        with urllib.request.urlopen(req, timeout=10) as resp:
            content = resp.read().decode()
            print(f"Response for {name}:")
            print(f"  Status: {resp.status}")
            print(f"  Content-Type: {resp.headers.get('Content-Type')}")
            print(f"  Body: {content[:500]}...")
            return content
    except urllib.error.HTTPError as e:
        print(f"HTTP Error for {name}: {e.code} - {e.reason}")
        print(f"  Body: {e.read().decode()[:200]}")
        return None
    except Exception as e:
        print(f"Error for {name}: {e}")
        return None

if __name__ == "__main__":
    print("=" * 50)
    print("Testing MCP Server")
    print("=" * 50)
    
    test_health()
    print()
    test_tools_list()
    print()
    
    print("Testing tool calls:")
    call_tool("mcp.get_logs", {"limit": 5})
    print()
    call_tool("mcp.get_webview_info")
    print()
    call_tool("api.get_samples")
