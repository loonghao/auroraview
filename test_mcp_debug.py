#!/usr/bin/env python
"""Debug MCP tool calls."""
import requests
import json

BASE_URL = "http://127.0.0.1:27168"

def test_health():
    r = requests.get(f"{BASE_URL}/health")
    print(f"Health: {r.status_code} - {r.text}")

def test_tools_list():
    r = requests.get(f"{BASE_URL}/tools")
    data = r.json()
    print(f"Tools count: {data.get('count', 0)}")
    for tool in data.get('data', [])[:5]:
        print(f"  - {tool['name']}")

def test_mcp_call(tool_name, args=None):
    """Call MCP tool via JSON-RPC."""
    payload = {
        'jsonrpc': '2.0',
        'id': 1,
        'method': 'tools/call',
        'params': {
            'name': tool_name,
            'arguments': args or {}
        }
    }
    headers = {
        'Content-Type': 'application/json',
        'Accept': 'application/json, text/event-stream'
    }
    
    print(f"\nCalling {tool_name}...")
    r = requests.post(f"{BASE_URL}/mcp", json=payload, headers=headers)
    print(f"Status: {r.status_code}")
    print(f"Headers: {dict(r.headers)}")
    print(f"Response: {r.text[:2000]}")
    return r

if __name__ == "__main__":
    test_health()
    test_tools_list()
    test_mcp_call("api.get_samples")
    test_mcp_call("api.get_categories")
    test_mcp_call("api.list_processes")
