# Uninstall Khaynjo (Windows). Re-runnable: if app files are gone but
# books data remains, still offers to delete the data directory.
#
# Usage:
#   powershell -ExecutionPolicy Bypass -File .\uninstall.ps1
#   powershell -ExecutionPolicy Bypass -File .\uninstall.ps1 -Dev
param(
    [switch]$Dev,
    [switch]$Prod,
    [switch]$NoPause
)

$ErrorActionPreference = "Stop"
$ScriptDir = $PSScriptRoot

function Pause-IfInteractive {
    if ($NoPause) { return }
    if ([Environment]::UserInteractive -and -not [Console]::IsInputRedirected) {
        Write-Host ""
        Write-Host "Press Enter to close this window..."
        try { [void][Console]::ReadLine() } catch { Read-Host "Press Enter to close" | Out-Null }
    }
}

function Read-Manifest([string]$path) {
    $map = @{}
    if (-not (Test-Path $path)) { return $map }
    Get-Content $path | ForEach-Object {
        $line = $_.Trim()
        if (-not $line -or $line.StartsWith("#")) { return }
        $i = $line.IndexOf("=")
        if ($i -lt 1) { return }
        $map[$line.Substring(0, $i).Trim()] = $line.Substring($i + 1).Trim().Trim('"')
    }
    return $map
}

function Resolve-Identity {
    $local = Join-Path $ScriptDir "install-manifest.env"
    if (Test-Path $local) { return (Read-Manifest $local) }

    $bookmarkDir = Join-Path $env:LOCALAPPDATA "Kumkaj"
    if ($Dev -and (Test-Path (Join-Path $bookmarkDir "kumkaj-khaynjo-debug.env"))) {
        return (Read-Manifest (Join-Path $bookmarkDir "kumkaj-khaynjo-debug.env"))
    }
    if ($Prod -and (Test-Path (Join-Path $bookmarkDir "kumkaj-khaynjo.env"))) {
        return (Read-Manifest (Join-Path $bookmarkDir "kumkaj-khaynjo.env"))
    }
    if (Test-Path (Join-Path $bookmarkDir "kumkaj-khaynjo.env")) {
        return (Read-Manifest (Join-Path $bookmarkDir "kumkaj-khaynjo.env"))
    }
    if (Test-Path (Join-Path $bookmarkDir "kumkaj-khaynjo-debug.env")) {
        return (Read-Manifest (Join-Path $bookmarkDir "kumkaj-khaynjo-debug.env"))
    }

    if ($Dev) {
        return @{
            KUMKAJ_ENV = "development"
            KUMKAJ_APP_ID = "com.khaynjo.debug"
            KUMKAJ_APP_LABEL = "Khaynjo Dev"
        }
    }
    return @{
        KUMKAJ_ENV = "production"
        KUMKAJ_APP_ID = "com.khaynjo"
        KUMKAJ_APP_LABEL = "Khaynjo"
    }
}

$m = Resolve-Identity
$AppId = $m["KUMKAJ_APP_ID"]
$Label = $m["KUMKAJ_APP_LABEL"]
$EnvName = $m["KUMKAJ_ENV"]
$IsDebug = $AppId.EndsWith(".debug") -or $EnvName -eq "development"
$ProgramsName = if ($IsDebug) { "Khaynjo Dev" } else { "Khaynjo" }
$slug = if ($IsDebug) { "kumkaj-khaynjo-debug" } else { "kumkaj-khaynjo" }

$InstallDir = $m["INSTALL_DIR"]
if (-not $InstallDir) {
    $InstallDir = Join-Path $env:LOCALAPPDATA "Programs\$ProgramsName"
}
# Prefer the folder we were launched from when it looks like an install tree.
if (Test-Path (Join-Path $ScriptDir "kumkaj-khaynjo.exe")) {
    $InstallDir = $ScriptDir
}

$DataDir = Join-Path $env:APPDATA $AppId
$StartMenu = Join-Path $env:APPDATA "Microsoft\Windows\Start Menu\Programs\$ProgramsName"
$UninstallKey = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\$AppId"
$Bookmark = Join-Path $env:LOCALAPPDATA "Kumkaj\$slug.env"

Write-Host "Uninstall $Label"
Write-Host "  App id:    $AppId"
Write-Host "  App files: $InstallDir"
Write-Host "  Data:      $DataDir"

$exe = Join-Path $InstallDir "kumkaj-khaynjo.exe"
$appPresent = Test-Path $exe

# Ask about data while the exe can still wipe SQLite categories.
Write-Host ""
Write-Host "$Label - local data:"
Write-Host "  $DataDir"
$wiped = $false
$cats = @()
if (Test-Path $DataDir) {
    Write-Host ""
    Write-Host "Delete business records (sales, payments, day books)? Shop setup stays."
    $ansRecords = Read-Host "[y/N]"
    Write-Host "Delete shop setup (products, rates, staff, brand, empty books)?"
    $ansSetup = Read-Host "[y/N]"
    Write-Host "Delete screen preferences (sidebar, amounts, window, language)?"
    $ansUi = Read-Host "[y/N]"
    if ($ansSetup -match '^(y|yes)$') {
        $cats += "schema"
    } elseif ($ansRecords -match '^(y|yes)$') {
        $cats += "trading"
    }
    if ($ansUi -match '^(y|yes)$') { $cats += "ui" }
    if ($cats.Count -gt 0) {
        if (Test-Path $exe) {
            Write-Host "Wiping: $($cats -join ', ') ..."
            Start-Process -FilePath $exe -ArgumentList (@("--uninstall-wipe") + $cats) -Wait -WindowStyle Hidden
        } else {
            Write-Host "App exe missing; applying folder deletes..."
            if ($cats -contains "schema") {
                Remove-Item -Recurse -Force -ErrorAction SilentlyContinue (Join-Path $DataDir "servers")
                Remove-Item -Recurse -Force -ErrorAction SilentlyContinue (Join-Path $DataDir "client")
                Remove-Item -Recurse -Force -ErrorAction SilentlyContinue (Join-Path $DataDir "backups")
                Remove-Item -Force -ErrorAction SilentlyContinue (Join-Path $DataDir "servers.json")
            }
            if ($cats -contains "ui") {
                Remove-Item -Force -ErrorAction SilentlyContinue (Join-Path $DataDir "window_geom.json")
                Remove-Item -Force -ErrorAction SilentlyContinue (Join-Path $DataDir "app_language.json")
            }
        }
        $wiped = $true
    } else {
        Write-Host "Keeping local data at: $DataDir"
    }
} else {
    Write-Host "  (no data directory found - nothing to remove)"
}

if ($appPresent) {
    Write-Host ""
    Write-Host "Removing app files..."
    # Remove Start Menu + ARP first.
    if (Test-Path $StartMenu) {
        Remove-Item -Recurse -Force $StartMenu
    }
    if (Test-Path $UninstallKey) {
        Remove-Item -Recurse -Force $UninstallKey
    }
    if ($ScriptDir -eq $InstallDir) {
        # Delete after process exits.
        $cleanup = $InstallDir
    } else {
        Remove-Item -Recurse -Force $InstallDir
        $cleanup = $null
    }
    Write-Host "  App files removed."
} else {
    Write-Host ""
    Write-Host "App files already gone (or never installed here)."
    if (Test-Path $StartMenu) { Remove-Item -Recurse -Force $StartMenu }
    if (Test-Path $UninstallKey) { Remove-Item -Recurse -Force $UninstallKey }
    $cleanup = $null
}

if (Test-Path $Bookmark) { Remove-Item -Force $Bookmark }

Write-Host ""
Write-Host "Done."
if (-not $wiped -and (Test-Path $DataDir)) {
    Write-Host "Reminder: local data is still on disk. Re-run uninstall to remove it."
}

if ($cleanup) {
    Start-Process powershell -WindowStyle Hidden -ArgumentList @(
        "-NoProfile", "-Command",
        "Start-Sleep -Seconds 1; Remove-Item -LiteralPath '$cleanup' -Recurse -Force -ErrorAction SilentlyContinue"
    )
}

Pause-IfInteractive
