# Gallery CDP Stop Script
# Stops the Gallery process and cleans up

param(
    [Parameter(Mandatory=$true)]
    [string]$PidFile
)

if (Test-Path $PidFile) {
    $procId = Get-Content $PidFile
    try {
        Stop-Process -Id $procId -Force -ErrorAction Stop
        Write-Host "[OK] Gallery process stopped (PID: $procId)"
    } catch {
        Write-Host "[OK] Process already stopped"
    }
    Remove-Item $PidFile
} else {
    Write-Host "[OK] No PID file found"
}
