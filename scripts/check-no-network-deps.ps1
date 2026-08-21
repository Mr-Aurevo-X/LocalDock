#Requires -Version 5.1
<#
.SYNOPSIS
  LocalDock security gate: direct network deps + URL grep scan.

.DESCRIPTION
  Fails if LocalDock-owned Cargo.toml files declare HTTP client crates directly,
  or if application source contains disallowed http(s):// literals.
  See docs/SECURITY.md for transitive Tauri exceptions.
#>
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$Root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
Set-Location $Root

$ForbiddenCrates = @(
    'reqwest', 'hyper', 'ureq', 'curl', 'surf', 'isahc', 'awc',
    'attohttpc', 'minreq', 'oauth2', 'hubcaps', 'octocrab'
)

$LocalTomls = @(
    (Join-Path $Root 'crates\localdock-core\Cargo.toml'),
    (Join-Path $Root 'src-tauri\Cargo.toml')
)

function Get-DirectDependencyNames {
    param([string]$TomlPath)
    $names = [System.Collections.Generic.List[string]]::new()
    $section = $null
    foreach ($line in Get-Content -LiteralPath $TomlPath) {
        if ($line -match '^\s*\[(.+)\]\s*$') {
            $section = $Matches[1]
            continue
        }
        if ($null -eq $section) { continue }
        if ($section -notmatch '^(dependencies|dev-dependencies|build-dependencies|target\.[^.]+\.dependencies)$') {
            continue
        }
        if ($line -match '^\s*([A-Za-z0-9_-]+)\s*=') {
            $names.Add($Matches[1].ToLowerInvariant()) | Out-Null
        }
    }
    return $names
}

Write-Host '== LocalDock: direct dependency check =='
$depFailures = @()
foreach ($toml in $LocalTomls) {
    if (-not (Test-Path -LiteralPath $toml)) {
        $depFailures += "Missing Cargo.toml: $toml"
        continue
    }
    $direct = Get-DirectDependencyNames -TomlPath $toml
    foreach ($crate in $ForbiddenCrates) {
        if ($direct -contains $crate) {
            $rel = $toml.Substring($Root.Length + 1)
            $depFailures += "Forbidden direct dependency '$crate' in $rel"
        }
    }
}

if ($depFailures.Count -gt 0) {
    Write-Error ($depFailures -join [Environment]::NewLine)
}

if (Get-Command cargo -ErrorAction SilentlyContinue) {
    Write-Host '== LocalDock: cargo tree depth-1 verification =='
    foreach ($pkg in @('localdock-core', 'localdock')) {
        $lines = & cargo tree --depth 1 -p $pkg --prefix none 2>&1
        if ($LASTEXITCODE -ne 0) {
            Write-Error "cargo tree failed for $pkg`: $($lines -join [Environment]::NewLine)"
        }
        $directDeps = @($lines | Select-Object -Skip 1 | ForEach-Object {
            if ($_ -match '^([A-Za-z0-9_-]+) v') { $Matches[1].ToLowerInvariant() }
        })
        foreach ($crate in $ForbiddenCrates) {
            if ($directDeps -contains $crate) {
                Write-Error "Forbidden crate '$crate' is a direct dependency of $pkg"
            }
        }
    }
} else {
    Write-Warning 'cargo not found; skipped cargo tree verification'
}

Write-Host '== LocalDock: URL grep gate =='
$ScanPaths = @(
    (Join-Path $Root 'crates'),
    (Join-Path $Root 'src-tauri\src'),
    (Join-Path $Root 'ui')
)
$AllowPatterns = @(
    'http://127\.0\.0\.1',
    'http://localhost'
)

$rg = Get-Command rg -ErrorAction SilentlyContinue
$urlFailures = @()
if ($rg) {
    $matches = & rg -n 'https?://' @ScanPaths 2>$null
    if ($LASTEXITCODE -eq 0 -and $matches) {
        foreach ($line in $matches) {
            $allowed = $false
            foreach ($pat in $AllowPatterns) {
                if ($line -match $pat) { $allowed = $true; break }
            }
            if (-not $allowed) { $urlFailures += $line }
        }
    }
} else {
    foreach ($path in $ScanPaths) {
        if (-not (Test-Path -LiteralPath $path)) { continue }
        Get-ChildItem -LiteralPath $path -Recurse -File | ForEach-Object {
            $i = 0
            Get-Content -LiteralPath $_.FullName | ForEach-Object {
                $i++
                if ($_ -match 'https?://') {
                    $allowed = $false
                    foreach ($pat in $AllowPatterns) {
                        if ($_ -match $pat) { $allowed = $true; break }
                    }
                    if (-not $allowed) {
                        $rel = $_.FullName.Substring($Root.Length + 1)
                        $urlFailures += "${rel}:$i`:$_"
                    }
                }
            }
        }
    }
}

if ($urlFailures.Count -gt 0) {
    Write-Error ("Disallowed http(s):// literals:`n" + ($urlFailures -join [Environment]::NewLine))
}

Write-Host '== Optional: Semgrep / gitleaks =='
foreach ($tool in @('semgrep', 'gitleaks')) {
    if (-not (Get-Command $tool -ErrorAction SilentlyContinue)) {
        Write-Warning "$tool not installed; skipped (see docs/SECURITY.md release checklist)"
    }
}

Write-Host 'OK: LocalDock network dependency and URL gates passed.'
