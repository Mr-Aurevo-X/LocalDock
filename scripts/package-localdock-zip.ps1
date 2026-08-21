#Requires -Version 5.1
<#
.SYNOPSIS
  Pack LocalDock portable zip for GitHub Releases on Mr-Aurevo-X/LocalDock.

.EXAMPLE
  .\scripts\package-localdock-zip.ps1
#>
param(
  [string]$OutDir = ""
)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
Set-Location -LiteralPath $Root

if (-not $OutDir) {
  $OutDir = Join-Path $Root "_artifacts"
}
New-Item -ItemType Directory -Force -Path $OutDir | Out-Null

Write-Host "== cargo build --release -p localdock =="
& cargo build --release -p localdock
if ($LASTEXITCODE -ne 0) {
  throw "cargo build --release failed"
}

$exe = Join-Path $Root "target\release\localdock.exe"
if (-not (Test-Path -LiteralPath $exe)) {
  throw "missing $exe"
}

$stage = Join-Path $env:TEMP ("localdock-zip-" + [guid]::NewGuid().ToString("n"))
$destRoot = Join-Path $stage "LocalDock"
New-Item -ItemType Directory -Force -Path $destRoot | Out-Null
try {
  Copy-Item -LiteralPath $exe -Destination (Join-Path $destRoot "localdock.exe")
  foreach ($item in @(
      "Lancer.cmd",
      "README.md",
      "LICENSE",
      "LICENSE.fr.md",
      "TERMS.md",
      "PRIVACY.md",
      "RELEASES.md"
    )) {
    $src = Join-Path $Root $item
    if (Test-Path -LiteralPath $src) {
      Copy-Item -LiteralPath $src -Destination (Join-Path $destRoot $item)
    }
  }
  if (-not (Test-Path (Join-Path $destRoot "Lancer.cmd"))) {
    throw "Lancer.cmd missing in pack"
  }
  $zipPath = Join-Path $OutDir "LocalDock.zip"
  if (Test-Path $zipPath) { Remove-Item -Force $zipPath }
  Compress-Archive -Path $destRoot -DestinationPath $zipPath -CompressionLevel Optimal
  Write-Host ("OK  LocalDock.zip  {0:N0} bytes -> {1}" -f (Get-Item $zipPath).Length, $zipPath)
}
finally {
  Remove-Item -Recurse -Force $stage -ErrorAction SilentlyContinue
}
