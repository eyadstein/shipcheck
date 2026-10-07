<#
.SYNOPSIS
  Starts PostgreSQL, the API (in its own window) and the dashboard dev server.
#>
$ErrorActionPreference = "Stop"
$PSNativeCommandUseErrorActionPreference = $false
Set-Location (Split-Path $PSScriptRoot -Parent)

$env:DATABASE_URL = "postgres://shipcheck:shipcheck@127.0.0.1:5433/shipcheck"
$env:SHIPCHECK_API_KEY = "dev-key-change-me-0123456789"
$env:BIND_ADDR = "127.0.0.1:8787"

docker info *> $null
if ($LASTEXITCODE -ne 0) { docker desktop start }
docker compose up -d --wait

# A direct connection first, then the health check without any proxy,
# so something else answering on the port can never fool the script.
function Test-Api {
    $client = [System.Net.Sockets.TcpClient]::new()
    try {
        $client.Connect("127.0.0.1", 8787)
        Invoke-RestMethod "http://127.0.0.1:8787/health" -NoProxy -TimeoutSec 3 | Out-Null
        return $true
    }
    catch { return $false }
    finally { $client.Dispose() }
}

if (-not (Test-Api)) {
    Write-Host "Starting the API in a new window..." -ForegroundColor Cyan
    Start-Process pwsh -ArgumentList "-NoExit", "-Command", "cargo run -p shipcheck-api"
    for ($i = 0; $i -lt 90 -and -not (Test-Api); $i++) { Start-Sleep -Seconds 2 }
    if (-not (Test-Api)) { throw "The API did not start. Read the error in its window." }
}
Write-Host "API is running at http://127.0.0.1:8787" -ForegroundColor Green

Set-Location web
if (-not (Test-Path node_modules)) { npm install }
npm run dev
