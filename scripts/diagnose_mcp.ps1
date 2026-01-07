#!/usr/bin/env pwsh
# MCP Server Diagnostic Script
# Run this while Gallery is running to diagnose connection issues

param(
    [int]$Port = 27168
)

Write-Host "=" * 60 -ForegroundColor Cyan
Write-Host "MCP Server Diagnostic Tool" -ForegroundColor Cyan
Write-Host "=" * 60 -ForegroundColor Cyan
Write-Host ""

# Step 1: Check if port is listening
Write-Host "[1] Checking if port $Port is listening..." -ForegroundColor Yellow
$listening = netstat -ano | Select-String ":$Port.*LISTENING"
if ($listening) {
    Write-Host "    [OK] Port $Port is listening" -ForegroundColor Green
    $pid = ($listening -split '\s+')[-1]
    Write-Host "    Process PID: $pid" -ForegroundColor Gray
    
    # Get process info
    $process = Get-Process -Id $pid -ErrorAction SilentlyContinue
    if ($process) {
        Write-Host "    Process Name: $($process.Name)" -ForegroundColor Gray
        Write-Host "    Process Path: $($process.Path)" -ForegroundColor Gray
    }
} else {
    Write-Host "    [FAIL] Port $Port is NOT listening" -ForegroundColor Red
    Write-Host "    Make sure Gallery is running and MCP sidecar started" -ForegroundColor Red
    exit 1
}
Write-Host ""

# Step 2: Test TCP connection
Write-Host "[2] Testing TCP connection to 127.0.0.1:$Port..." -ForegroundColor Yellow
try {
    $tcp = New-Object System.Net.Sockets.TcpClient
    $tcp.Connect("127.0.0.1", $Port)
    if ($tcp.Connected) {
        Write-Host "    [OK] TCP connection successful" -ForegroundColor Green
        $tcp.Close()
    }
} catch {
    Write-Host "    [FAIL] TCP connection failed: $_" -ForegroundColor Red
    exit 1
}
Write-Host ""

# Step 3: Test health endpoint with timeout
Write-Host "[3] Testing /health endpoint..." -ForegroundColor Yellow
try {
    $stopwatch = [System.Diagnostics.Stopwatch]::StartNew()
    $response = Invoke-WebRequest -Uri "http://127.0.0.1:$Port/health" -TimeoutSec 5 -UseBasicParsing
    $stopwatch.Stop()
    Write-Host "    [OK] Health check passed (${$stopwatch.ElapsedMilliseconds}ms)" -ForegroundColor Green
    Write-Host "    Response: $($response.Content)" -ForegroundColor Gray
} catch [System.Net.WebException] {
    $stopwatch.Stop()
    Write-Host "    [FAIL] Health check failed after $($stopwatch.ElapsedMilliseconds)ms" -ForegroundColor Red
    Write-Host "    Error: $_" -ForegroundColor Red
} catch {
    Write-Host "    [FAIL] Health check error: $_" -ForegroundColor Red
}
Write-Host ""

# Step 4: Test MCP initialize
Write-Host "[4] Testing MCP /mcp endpoint..." -ForegroundColor Yellow
$body = @{
    jsonrpc = "2.0"
    id = 1
    method = "initialize"
    params = @{
        protocolVersion = "2024-11-05"
        capabilities = @{}
        clientInfo = @{
            name = "diagnostic-client"
            version = "1.0.0"
        }
    }
} | ConvertTo-Json -Depth 10

try {
    $stopwatch = [System.Diagnostics.Stopwatch]::StartNew()
    $response = Invoke-WebRequest -Uri "http://127.0.0.1:$Port/mcp" `
        -Method POST `
        -Body $body `
        -ContentType "application/json" `
        -Headers @{ "Accept" = "application/json, text/event-stream" } `
        -TimeoutSec 10 `
        -UseBasicParsing
    $stopwatch.Stop()
    Write-Host "    [OK] MCP initialize succeeded ($($stopwatch.ElapsedMilliseconds)ms)" -ForegroundColor Green
    Write-Host "    Content-Type: $($response.Headers['Content-Type'])" -ForegroundColor Gray
    Write-Host "    Response (first 500 chars):" -ForegroundColor Gray
    Write-Host "    $($response.Content.Substring(0, [Math]::Min(500, $response.Content.Length)))" -ForegroundColor Gray
} catch {
    Write-Host "    [FAIL] MCP initialize failed: $_" -ForegroundColor Red
}
Write-Host ""

Write-Host "=" * 60 -ForegroundColor Cyan
Write-Host "Diagnostic Complete" -ForegroundColor Cyan
Write-Host "=" * 60 -ForegroundColor Cyan

