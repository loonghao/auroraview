# Gallery CDP Start Script
# Starts the Gallery application for CDP testing

param(
    [Parameter(Mandatory=$true)]
    [string]$ExePath,
    
    [Parameter(Mandatory=$true)]
    [string]$WorkDir,
    
    [Parameter(Mandatory=$true)]
    [string]$PidFile
)

# Check if exe exists, if not try to find any exe in the directory
if (-not (Test-Path $ExePath)) {
    Write-Host "[WARN] Expected exe not found: $ExePath"
    # Try to find any exe in pack-output directory
    $exeFiles = Get-ChildItem -Path $WorkDir -Filter "*.exe" -ErrorAction SilentlyContinue
    if ($exeFiles) {
        $ExePath = $exeFiles[0].FullName
        Write-Host "[INFO] Found alternative exe: $ExePath"
    } else {
        Write-Host "[ERROR] No exe found in: $WorkDir"
        exit 1
    }
}

# Start the process (with visible console for debugging)
Write-Host "Starting: $ExePath"
Write-Host "WorkDir: $WorkDir"
$proc = Start-Process -FilePath $ExePath -WorkingDirectory $WorkDir -PassThru -NoNewWindow

# Save PID to file
$proc.Id | Out-File -FilePath $PidFile -NoNewline

Write-Host "[OK] Gallery started (PID: $($proc.Id))"
