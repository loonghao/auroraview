import requests
import sys

port = int(sys.argv[1]) if len(sys.argv) > 1 else 34470
r = requests.post(
    f'http://127.0.0.1:{port}/mcp',
    json={'jsonrpc':'2.0','id':1,'method':'tools/list','params':{}},
    headers={'Accept':'application/json, text/event-stream'}
)
print('Status:', r.status_code)
print('Response:', r.text[:2000])
