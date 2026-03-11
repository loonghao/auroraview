param(
    [string]$ProjectRoot = (Resolve-Path "$PSScriptRoot\..").Path,
    [string]$PidFile = "$((Resolve-Path "$PSScriptRoot\..").Path)\.gallery-pid.tmp",
    [string]$SpecFile = "",
    [switch]$EnableScreenshots
)

$ErrorActionPreference = 'Stop'
$testExit = 1

try {
    Set-Location (Join-Path $ProjectRoot 'tests\e2e')

    if ($EnableScreenshots) {
        $env:AURORAVIEW_SCREENSHOTS = '1'
    }

    $playwrightArgs = @('playwright', 'test', '--config', 'playwright.config.ts')
    if (-not [string]::IsNullOrWhiteSpace($SpecFile)) {
        $playwrightArgs += $SpecFile
    }

    & vx npx @playwrightArgs
    $testExit = $LASTEXITCODE
}
finally {
    & (Join-Path $ProjectRoot 'scripts\gallery_cdp_stop.ps1') -PidFile $PidFile
}

exit $testExit
