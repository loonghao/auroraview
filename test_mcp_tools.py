"""Test MCP tools directly."""
import requests
import json

BASE_URL = "http://127.0.0.1:27168"

def test_health():
    """Test health endpoint."""
    resp = requests.get(f"{BASE_URL}/health")
    print(f"Health: {resp.status_code} - {resp.json()}")

def test_tools_list():
    """Test tools listing."""
    resp = requests.get(f"{BASE_URL}/tools")
    data = resp.json()
    print(f"Tools count: {data.get('count', 0)}")
    for tool in data.get("data", [])[:5]:
        print(f"  - {tool['name']}: {tool['description'][:50]}...")

def test_mcp_call(tool_name: str, args: dict = None):
    """Test MCP tool call via Streamable HTTP."""
    headers = {
        "Content-Type": "application/json",
        "Accept": "application/json, text/event-stream",
    }
    body = {
        "jsonrpc": "2.0",
        "method": "tools/call",
        "params": {
            "name": tool_name,
            "arguments": args or {},
        },
        "id": 1,
    }
    print(f"\nCalling {tool_name}...")
    print(f"Request: {json.dumps(body, indent=2)}")
    
    resp = requests.post(f"{BASE_URL}/mcp", headers=headers, json=body)
    print(f"Status: {resp.status_code}")
    print(f"Headers: {dict(resp.headers)}")
    print(f"Response: {resp.text[:1000]}")

if __name__ == "__main__":
    test_health()
    print()
    test_tools_list()
    print()
    test_mcp_call("mcp.get_logs", {"limit": 5})
    print()
    test_mcp_call("mcp.clear_logs")
    print()
    test_mcp_call("api.get_categories")
