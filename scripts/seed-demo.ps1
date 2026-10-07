<#
.SYNOPSIS
  Posts a history of demo scans so the dashboard has something to show.
#>
param(
    [string]$Api = "http://127.0.0.1:8787",
    [string]$Key = $env:SHIPCHECK_API_KEY,
    [string]$Project = "eyadstein/demo-app"
)

$ErrorActionPreference = "Stop"
if (-not $Key) { throw "Set SHIPCHECK_API_KEY or pass -Key." }

$report = cargo run -q -p shipcheck-cli -- examples/demo-app --format json | ConvertFrom-Json
$all = @($report.findings)

# Each scan keeps a smaller share of the findings, as if the project was being fixed.
foreach ($share in 1.0, 0.8, 0.6, 0.45, 0.3, 0.2, 0.1) {
    $take = [int][Math]::Ceiling($all.Count * $share)
    $body = @{
        project  = $Project
        git_ref  = "demo-$([int]($share * 100))"
        findings = @($all | Select-Object -First $take)
    } | ConvertTo-Json -Depth 6
    $result = Invoke-RestMethod -Method Post -Uri "$Api/api/v1/scans" `
        -Headers @{ "x-api-key" = $Key } -ContentType "application/json" -Body $body
    "{0,3} findings -> score {1}" -f $take, $result.score
    Start-Sleep -Milliseconds 200
}
