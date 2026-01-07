import requests

r = requests.post(
    'http://127.0.0.1:27168/mcp',
    json={'jsonrpc':'2.0','id':1,'method':'tools/call','params':{'name':'api.get_samples','arguments':{}}},
    headers={'Accept':'application/json, text/event-stream'}
)
print('Status:', r.status_code)
print('Response:', r.text[:2500])
