# Gallery CDP Wait Script
# Waits for CDP port to be ready

param(
    [int]$Port = 9222,
    [int]$Timeout = 30
)

$url = "http://127.0.0.1:$Port/json/version"

for ($i = 0; $i -lt $Timeout; $i++) {
    try {
        $null = Invoke-WebRequest -Uri $url -TimeoutSec 1 -ErrorAction Stop
        Write-Host "[OK] CDP ready on port $Port"
        exit 0
    } catch {
        Start-Sleep -Seconds 1
    }
}

Write-Host "[ERROR] CDP timeout after $Timeout seconds"
exit 1
