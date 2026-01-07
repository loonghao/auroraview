import requests
import sys

port = int(sys.argv[1]) if len(sys.argv) > 1 else 34470

# Test echo tool
r = requests.post(
    f'http://127.0.0.1:{port}/mcp',
    json={
        'jsonrpc':'2.0',
        'id':1,
        'method':'tools/call',
        'params':{
            'name':'mcp.echo',
            'arguments': {'message': 'Hello from AI!'}
        }
    },
    headers={'Accept':'application/json, text/event-stream'}
)
print('Echo Test:')
print('Status:', r.status_code)
print('Response:', r.text[:2000])

# Test calculate tool
r2 = requests.post(
    f'http://127.0.0.1:{port}/mcp',
    json={
        'jsonrpc':'2.0',
        'id':2,
        'method':'tools/call',
        'params':{
            'name':'mcp.calculate',
            'arguments': {'a': 10, 'b': 5, 'operation': 'multiply'}
        }
    },
    headers={'Accept':'application/json, text/event-stream'}
)
print('\nCalculate Test:')
print('Status:', r2.status_code)
print('Response:', r2.text[:2000])
