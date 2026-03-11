#!/usr/bin/env pwsh
#Requires -Version 7.0
<#
.SYNOPSIS
    Setup GitHub Secrets for Sentry telemetry configuration

.DESCRIPTION
    This script reads the local .env.sentry file and sets GitHub repository secrets
    using the `gh` CLI. These secrets are used in CI/CD to inject Sentry DSNs
    during build time.

.PARAMETER EnvFile
    Path to the .env.sentry file (default: gallery/.env.sentry)

.PARAMETER DryRun
    Show what would be done without making changes

.EXAMPLE
    ./scripts/setup-sentry-secrets.ps1
    ./scripts/setup-sentry-secrets.ps1 -DryRun
#>

param(
    [string]$EnvFile = "gallery/.env.sentry",
    [switch]$DryRun
)

$ErrorActionPreference = "Stop"

# Check if gh CLI is available
if (-not (Get-Command gh -ErrorAction SilentlyContinue)) {
    Write-Error "GitHub CLI (gh) is not installed. Please install it from https://cli.github.com/"
    exit 1
}

# Check if user is authenticated
$authStatus = gh auth status 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Error "Not authenticated with GitHub CLI. Run 'gh auth login' first."
    exit 1
}

# Check if .env.sentry file exists
if (-not (Test-Path $EnvFile)) {
    Write-Error "Sentry config file not found: $EnvFile"
    Write-Host ""
    Write-Host "Please create $EnvFile with the following content:"
    Write-Host @"
# Sentry Telemetry Configuration
AURORAVIEW_GALLERY_RUST_SENTRY_DSN=https://xxx@xxx.ingest.sentry.io/xxx
AURORAVIEW_GALLERY_RUST_OTLP_ENDPOINT=https://xxx.ingest.sentry.io/api/xxx/integration/otlp
VITE_AURORAVIEW_GALLERY_FRONTEND_SENTRY_DSN=https://xxx@xxx.ingest.sentry.io/xxx
VITE_AURORAVIEW_GALLERY_FRONTEND_OTLP_ENDPOINT=https://xxx.ingest.sentry.io/api/xxx/integration/otlp
"@
    exit 1
}

Write-Host "Reading Sentry configuration from: $EnvFile" -ForegroundColor Cyan
Write-Host ""

# Parse .env.sentry file
$envVars = @{}
Get-Content $EnvFile | ForEach-Object {
    $line = $_.Trim()
    # Skip comments and empty lines
    if ($line -and -not $line.StartsWith('#')) {
        $parts = $line -split '=', 2
        if ($parts.Count -eq 2) {
            $key = $parts[0].Trim()
            $value = $parts[1].Trim()
            $envVars[$key] = $value
        }
    }
}

# Map local env vars to GitHub Secrets names
$secretMappings = @{
    'AURORAVIEW_GALLERY_RUST_SENTRY_DSN' = 'SENTRY_RUST_DSN'
    'AURORAVIEW_GALLERY_RUST_OTLP_ENDPOINT' = 'SENTRY_RUST_OTLP_ENDPOINT'
    'VITE_AURORAVIEW_GALLERY_FRONTEND_SENTRY_DSN' = 'SENTRY_FRONTEND_DSN'
    'VITE_AURORAVIEW_GALLERY_FRONTEND_OTLP_ENDPOINT' = 'SENTRY_FRONTEND_OTLP_ENDPOINT'
}

# Set GitHub Secrets
foreach ($mapping in $secretMappings.GetEnumerator()) {
    $localKey = $mapping.Key
    $secretName = $mapping.Value
    $value = $envVars[$localKey]

    if (-not $value) {
        Write-Warning "Skipping $secretName - not found in $EnvFile"
        continue
    }

    if ($DryRun) {
        Write-Host "[DRY-RUN] Would set secret: $secretName" -ForegroundColor Yellow
    } else {
        Write-Host "Setting secret: $secretName" -ForegroundColor Green
        gh secret set $secretName --body $value
        if ($LASTEXITCODE -ne 0) {
            Write-Error "Failed to set secret: $secretName"
        }
    }
}

# Set GitHub Variables (non-sensitive configuration)
$variableMappings = @{
    'AURORAVIEW_GALLERY_RUST_SENTRY_SAMPLE_RATE' = 'SENTRY_RUST_SAMPLE_RATE'
    'AURORAVIEW_GALLERY_RUST_SENTRY_TRACES_SAMPLE_RATE' = 'SENTRY_RUST_TRACES_SAMPLE_RATE'
    'VITE_AURORAVIEW_GALLERY_FRONTEND_SENTRY_SAMPLE_RATE' = 'SENTRY_FRONTEND_SAMPLE_RATE'
    'VITE_AURORAVIEW_GALLERY_FRONTEND_SENTRY_TRACES_SAMPLE_RATE' = 'SENTRY_FRONTEND_TRACES_SAMPLE_RATE'
}

foreach ($mapping in $variableMappings.GetEnumerator()) {
    $localKey = $mapping.Key
    $varName = $mapping.Value
    $value = $envVars[$localKey]

    if (-not $value) {
        Write-Host "Using default for variable: $varName" -ForegroundColor Gray
        continue
    }

    if ($DryRun) {
        Write-Host "[DRY-RUN] Would set variable: $varName = $value" -ForegroundColor Yellow
    } else {
        Write-Host "Setting variable: $varName = $value" -ForegroundColor Green
        gh variable set $varName --body $value
        if ($LASTEXITCODE -ne 0) {
            Write-Warning "Failed to set variable: $varName (may already exist)"
        }
    }
}

Write-Host ""
if ($DryRun) {
    Write-Host "Dry run complete. Run without -DryRun to apply changes." -ForegroundColor Cyan
} else {
    Write-Host "GitHub Secrets and Variables configured successfully!" -ForegroundColor Green
    Write-Host ""
    Write-Host "Configured secrets:" -ForegroundColor Cyan
    gh secret list
    Write-Host ""
    Write-Host "Configured variables:" -ForegroundColor Cyan
    gh variable list
}
