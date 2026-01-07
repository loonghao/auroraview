$body = @{
    jsonrpc = "2.0"
    method = "tools/call"
    params = @{
        name = "mcp.get_logs"
        arguments = @{
            limit = 5
        }
    }
    id = 1
} | ConvertTo-Json -Depth 5

Write-Host "=== Testing mcp.get_logs ==="
Write-Host "Request: $body"

$headers = @{
    "Accept" = "application/json, text/event-stream"
}

try {
    $response = Invoke-WebRequest -Uri 'http://127.0.0.1:27168/mcp' -Method POST -ContentType 'application/json' -Headers $headers -Body $body -UseBasicParsing
    Write-Host "Status: $($response.StatusCode)"
    Write-Host "Response: $($response.Content)"
} catch {
    Write-Host "Error: $($_.Exception.Message)"
    if ($_.Exception.Response) {
        $reader = New-Object System.IO.StreamReader($_.Exception.Response.GetResponseStream())
        Write-Host "Response body: $($reader.ReadToEnd())"
    }
}

Write-Host ""
Write-Host "=== Testing mcp.get_webview_info ==="
$body2 = @{
    jsonrpc = "2.0"
    method = "tools/call"
    params = @{
        name = "mcp.get_webview_info"
        arguments = @{}
    }
    id = 2
} | ConvertTo-Json -Depth 5

try {
    $response2 = Invoke-WebRequest -Uri 'http://127.0.0.1:27168/mcp' -Method POST -ContentType 'application/json' -Headers $headers -Body $body2 -UseBasicParsing
    Write-Host "Status: $($response2.StatusCode)"
    Write-Host "Response: $($response2.Content)"
} catch {
    Write-Host "Error: $($_.Exception.Message)"
}

Write-Host ""
Write-Host "=== Testing api.get_samples ==="
$body3 = @{
    jsonrpc = "2.0"
    method = "tools/call"
    params = @{
        name = "api.get_samples"
        arguments = @{}
    }
    id = 3
} | ConvertTo-Json -Depth 5

try {
    $response3 = Invoke-WebRequest -Uri 'http://127.0.0.1:27168/mcp' -Method POST -ContentType 'application/json' -Headers $headers -Body $body3 -UseBasicParsing
    Write-Host "Status: $($response3.StatusCode)"
    Write-Host "Response: $($response3.Content.Substring(0, [Math]::Min(500, $response3.Content.Length)))..."
} catch {
    Write-Host "Error: $($_.Exception.Message)"
}
