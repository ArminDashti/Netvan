#Requires -Version 5.1
<#
.SYNOPSIS
  Build and export the Netvan Windows .exe (Tauri release).

.DESCRIPTION
  Runs npm install (optional), then npm run tauri build.
  Copies Netvan.exe (and optional service) to an output folder.

.EXAMPLE
  .\install.ps1
  .\install.ps1 --out=.\Netvan
  .\install.ps1 --with-service
  .\install.ps1 --skip-npm
  .\install.ps1 --help
#>

$ErrorActionPreference = "Stop"
$Root = $PSScriptRoot

function Write-Info([string]$Message)  { Write-Host "[*] $Message" -ForegroundColor Cyan }
function Write-Ok([string]$Message)    { Write-Host "[+] $Message" -ForegroundColor Green }
function Write-Warn([string]$Message)  { Write-Host "[!] $Message" -ForegroundColor Yellow }
function Write-Err([string]$Message)   { Write-Host "[-] $Message" -ForegroundColor Red }
function Write-Step([string]$Message)  { Write-Host "" ; Write-Host "==> $Message" -ForegroundColor Magenta }

function Show-Help {
    Write-Host ""
    Write-Host "Netvan install.ps1 - build and export .exe" -ForegroundColor White
    Write-Host ""
    Write-Host "Usage:" -ForegroundColor Cyan
    Write-Host "  .\install.ps1 [--out=<path>] [--with-service] [--skip-npm] [--help]"
    Write-Host ""
    Write-Host "Parameters:" -ForegroundColor Cyan
    Write-Host "  --out=<path>      Folder to copy built binaries into (default: .\Netvan)"
    Write-Host "  --with-service    Also build netvan-service.exe (release)"
    Write-Host "  --skip-npm        Skip npm install (use existing node_modules)"
    Write-Host "  --help            Show this help"
    Write-Host ""
    Write-Host "Examples:" -ForegroundColor Cyan
    Write-Host "  .\install.ps1"
    Write-Host "  .\install.ps1 --out=.\Netvan"
    Write-Host "  .\install.ps1 --with-service --skip-npm"
    Write-Host ""
    Write-Host "Output:" -ForegroundColor Cyan
    Write-Host "  App exe:     src-tauri\target\release\Netvan.exe"
    Write-Host "  Bundles:     src-tauri\target\release\bundle\"
    Write-Host "  Service exe: target\release\netvan-service.exe  (with --with-service)"
    Write-Host ""
}

function Get-ArgValue {
    param(
        [string[]]$ArgList,
        [string]$Name,
        [string]$Default = $null
    )
    foreach ($a in $ArgList) {
        if ($a -eq $Name -or $a -eq ($Name + "=")) {
            return ""
        }
        if ($a.StartsWith($Name + "=")) {
            return $a.Substring($Name.Length + 1)
        }
    }
    return $Default
}

function Test-HasFlag {
    param(
        [string[]]$ArgList,
        [string]$Name
    )
    foreach ($a in $ArgList) {
        if ($a -eq $Name -or $a.StartsWith($Name + "=")) { return $true }
    }
    return $false
}

function Assert-Command {
    param([string]$Name)
    if (-not (Get-Command $Name -ErrorAction SilentlyContinue)) {
        Write-Err ("Required command not found: " + $Name)
        Write-Host "Install it, then re-run. See README prerequisites." -ForegroundColor Yellow
        Show-Help
        exit 1
    }
}

function Invoke-Step {
    param(
        [string]$Title,
        [scriptblock]$Action
    )
    Write-Step $Title
    & $Action
    if ($LASTEXITCODE -ne 0 -and $null -ne $LASTEXITCODE) {
        Write-Err ("Step failed: " + $Title + " (exit " + $LASTEXITCODE + ")")
        exit $LASTEXITCODE
    }
    Write-Ok ("Done: " + $Title)
}

# --- parse args ---
$RawArgs = @($args)
if (Test-HasFlag -ArgList $RawArgs -Name "--help") {
    Show-Help
    exit 0
}

$OutDir = Get-ArgValue -ArgList $RawArgs -Name "--out" -Default (Join-Path $Root "Netvan")
$WithService = Test-HasFlag -ArgList $RawArgs -Name "--with-service"
$SkipNpm = Test-HasFlag -ArgList $RawArgs -Name "--skip-npm"

# reject unknown flags
foreach ($a in $RawArgs) {
    if (-not $a.StartsWith("--")) {
        Write-Err ("Unknown argument: " + $a)
        Show-Help
        exit 1
    }
    $known = @("--out", "--with-service", "--skip-npm", "--help")
    $ok = $false
    foreach ($k in $known) {
        if ($a -eq $k -or $a.StartsWith($k + "=")) { $ok = $true; break }
    }
    if (-not $ok) {
        Write-Err ("Unknown parameter: " + $a)
        Show-Help
        exit 1
    }
}

if ([string]::IsNullOrWhiteSpace($OutDir)) {
    Write-Err "Missing path for --out. Example: --out=.\Netvan"
    Show-Help
    exit 1
}

if (-not [System.IO.Path]::IsPathRooted($OutDir)) {
    $OutDir = Join-Path $Root $OutDir
}

Write-Host ""
Write-Host "  Netvan release build" -ForegroundColor White
Write-Host "  --------------------" -ForegroundColor DarkGray
Write-Info ("Root:    " + $Root)
Write-Info ("Out:     " + $OutDir)
Write-Info ("Service: " + $WithService)
if ($SkipNpm) {
    Write-Info "npm:     skip"
} else {
    Write-Info "npm:     install"
}
Write-Host ""

Assert-Command "node"
Assert-Command "npm"
Assert-Command "cargo"
Assert-Command "rustc"

Push-Location $Root
try {
    if (-not $SkipNpm) {
        Invoke-Step "npm install" {
            npm install
        }
    } else {
        Write-Warn "Skipping npm install"
        if (-not (Test-Path (Join-Path $Root "node_modules"))) {
            Write-Err "node_modules missing. Run without --skip-npm."
            exit 1
        }
    }

    Write-Step "tauri build (release .exe + installers)"
    Write-Info "This may take several minutes..."
    npm run tauri build
    if ($LASTEXITCODE -ne 0) {
        Write-Err ("tauri build failed (exit " + $LASTEXITCODE + ")")
        exit $LASTEXITCODE
    }
    Write-Ok "Done: tauri build"

    if ($WithService) {
        Invoke-Step "cargo build -p netvan-service --release" {
            cargo build -p netvan-service --release
        }
    }

    $AppExe = Join-Path $Root "src-tauri\target\release\Netvan.exe"
    if (-not (Test-Path $AppExe)) {
        Write-Err ("Expected exe not found: " + $AppExe)
        exit 1
    }

    New-Item -ItemType Directory -Force -Path $OutDir | Out-Null
    Copy-Item -Force $AppExe (Join-Path $OutDir "Netvan.exe")
    Write-Ok ("Copied Netvan.exe -> " + $OutDir)

    if ($WithService) {
        $SvcExe = Join-Path $Root "target\release\netvan-service.exe"
        if (-not (Test-Path $SvcExe)) {
            Write-Err ("Service exe not found: " + $SvcExe)
            exit 1
        }
        Copy-Item -Force $SvcExe (Join-Path $OutDir "netvan-service.exe")
        Write-Ok ("Copied netvan-service.exe -> " + $OutDir)
    }

    $BundleDir = Join-Path $Root "src-tauri\target\release\bundle"
    Write-Host ""
    Write-Ok "Build complete"
    Write-Host ("  App:     " + $AppExe) -ForegroundColor Green
    Write-Host ("  Export:  " + (Join-Path $OutDir "Netvan.exe")) -ForegroundColor Green
    if (Test-Path $BundleDir) {
        Write-Host ("  Bundles: " + $BundleDir) -ForegroundColor Green
    }
    if ($WithService) {
        Write-Host ("  Service: " + (Join-Path $OutDir "netvan-service.exe")) -ForegroundColor Green
    }
    Write-Host ""
}
finally {
    Pop-Location
}
