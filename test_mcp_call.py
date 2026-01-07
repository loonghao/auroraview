#!/usr/bin/env python
"""Test MCP tool call with streaming response."""
import requests

url = "http://127.0.0.1:27168/mcp"
payload = {
    "jsonrpc": "2.0",
    "method": "tools/call",
    "id": 1,
    "params": {
        "name": "api.open_url",
        "arguments": {"url": "https://www.baidu.com"}
    }
}
headers = {"Accept": "application/json, text/event-stream"}

print("Calling api.open_url...")
r = requests.post(url, json=payload, headers=headers, stream=True, timeout=30)
print(f"Status: {r.status_code}")

for line in r.iter_lines(decode_unicode=True):
    if line:
        print(f"Line: {line}")
        if line.startswith("data:"):
            print("Got data response!")
            break

print("Done!")
