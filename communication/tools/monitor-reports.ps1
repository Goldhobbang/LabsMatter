# REALMATTER Report Monitor (A0)
# Polls agent report directories, validates structure, deduplicates by ID+hash,
# and appends a monitoring log. Run periodically or via Task Scheduler.
# Usage: powershell -File communication/tools/monitor-reports.ps1

param(
    [int]$IntervalSeconds = 0,   # 0 = single pass; >0 = loop
    [string]$LogPath = "communication/supervisor/monitor-log.md"
)

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
Set-Location $root

$stateFile = "communication/supervisor/.monitor-state.json"
if (Test-Path $stateFile) {
    $raw = Get-Content $stateFile | ConvertFrom-Json
    $ht = @{}
    if ($raw.processed) {
        $raw.processed.PSObject.Properties | ForEach-Object { $ht[$_.Name] = $_.Value }
    }
    $state = @{ processed = $ht }
} else {
    $state = @{ processed = @{} }
}

function Get-Hash($path) {
    (Get-FileHash -Path $path -Algorithm SHA256).Hash
}

function Validate-Report($path) {
    $content = Get-Content $path -Raw
    $required = @("Task ID","Summary","Blockers")
    $recommended = @("Agent","Changed","Verification","Known")
    $missing = @()
    foreach ($r in $required) {
        if ($content -notmatch [regex]::Escape($r)) { $missing += $r }
    }
    $warn = @()
    foreach ($r in $recommended) {
        if ($content -notmatch $r) { $warn += $r }
    }
    return @{ missing = $missing; warn = $warn }
}

function Pass {
    $reports = Get-ChildItem -Path "communication/agents" -Recurse -Filter "*.md" |
        Where-Object { $_.FullName -match "\\reports\\" }
    $newFound = 0
    foreach ($r in $reports) {
        $hash = Get-Hash $r.FullName
        $key = "$($r.Name)|$hash"
        if ($state.processed.ContainsKey($key)) { continue }
        $result = Validate-Report $r.FullName
        $rel = $r.FullName.Substring($root.Length+1)
        $entry = "### $rel`n- Hash: $hash`n- Missing required: $(if ($result.missing.Count -eq 0) {'none'} else {$result.missing -join ', '})`n- Missing recommended: $(if ($result.warn.Count -eq 0) {'none'} else {$result.warn -join ', '})`n- Action: $(if ($result.missing.Count -eq 0) {'VALID - request A8 verification'} else {'INVALID - request correction'})`n"
        Add-Content -Path $LogPath -Value $entry
        $state.processed[$key] = (Get-Date).ToString("o")
        $newFound++
    }
    if (-not (Test-Path $LogPath)) {
        "# REALMATTER Report Monitor Log`n" | Set-Content $LogPath
    }
    $state | ConvertTo-Json -Depth 5 | Set-Content $stateFile
    Write-Host "Pass complete. New/changed reports: $newFound"
}

# Ensure log header
if (-not (Test-Path $LogPath)) {
    "# REALMATTER Report Monitor Log`n" | Set-Content $LogPath
    "Started: $(Get-Date -Format o)`n" | Add-Content $LogPath
}

do {
    Pass
    if ($IntervalSeconds -gt 0) { Start-Sleep -Seconds $IntervalSeconds }
} while ($IntervalSeconds -gt 0)
