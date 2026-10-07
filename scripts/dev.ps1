<#
.SYNOPSIS
  Starts PostgreSQL, the API (hidden, with log files) and the dashboard dev server.
  Press Ctrl+C to stop the dashboard. The API is stopped with it.
#>
param([int]$ApiPort = 3001)

$ErrorActionPreference = "Stop"
$PSNativeCommandUseErrorActionPreference = $false
$root = Split-Path $PSScriptRoot -Parent
Set-Location $root

$api = "http://127.0.0.1:$ApiPort"
$env:DATABASE_URL = "postgres://shipcheck:shipcheck@127.0.0.1:5433/shipcheck"
$env:SHIPCHECK_API_KEY = "dev-key-change-me-0123456789"
$env:BIND_ADDR = "127.0.0.1:$ApiPort"
$env:CORS_ORIGINS = "http://localhost:5173,http://127.0.0.1:5173,http://localhost:5174,http://127.0.0.1:5174"
# The browser calls the API directly, so the Vite proxy is not used.
$env:VITE_API_URL = $api

docker info *> $null
if ($LASTEXITCODE -ne 0) { docker desktop start }
docker compose up -d --wait

# A running copy of the API locks its own .exe, so stop old ones before building.
Get-Process shipcheck-api -ErrorAction SilentlyContinue | Stop-Process -Force
cargo build -q -p shipcheck-api
if ($LASTEXITCODE -ne 0) { throw "The API failed to build. The error is shown above." }

$out = Join-Path $env:TEMP "shipcheck-api.out.log"
$err = Join-Path $env:TEMP "shipcheck-api.err.log"
$server = Start-Process (Join-Path $root "target\debug\shipcheck-api.exe") -PassThru -WindowStyle Hidden `
    -RedirectStandardOutput $out -RedirectStandardError $err

function Test-Api {
    try { Invoke-RestMethod "$api/health" -NoProxy -TimeoutSec 3 | Out-Null; return $true }
    catch { return $false }
}

try {
    for ($i = 0; $i -lt 30 -and -not (Test-Api); $i++) {
        if ($server.HasExited) { break }
        Start-Sleep -Seconds 1
    }
    if (-not (Test-Api)) {
        Write-Host "The API did not start. Its log:" -ForegroundColor Red
        Get-Content $out, $err -ErrorAction SilentlyContinue
        throw "The API failed to start."
    }
    $projects = Invoke-RestMethod "$api/api/v1/projects" -NoProxy
    $count = @($projects | Where-Object { $_ }).Count
    Write-Host "API is running at $api with $count project(s)" -ForegroundColor Green

    Set-Location (Join-Path $root "web")
    if (-not (Test-Path node_modules)) { npm install }
    npm run dev
}
finally {
    Stop-Process -Id $server.Id -Force -ErrorAction SilentlyContinue
}
