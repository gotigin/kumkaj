# Uninstall Moklanjo (Windows). Re-runnable: if app files are gone but
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
    if ($Dev -and (Test-Path (Join-Path $bookmarkDir "kumkaj-moklanjo-debug.env"))) {
        return (Read-Manifest (Join-Path $bookmarkDir "kumkaj-moklanjo-debug.env"))
    }
    if ($Prod -and (Test-Path (Join-Path $bookmarkDir "kumkaj-moklanjo.env"))) {
        return (Read-Manifest (Join-Path $bookmarkDir "kumkaj-moklanjo.env"))
    }
    if (Test-Path (Join-Path $bookmarkDir "kumkaj-moklanjo.env")) {
        return (Read-Manifest (Join-Path $bookmarkDir "kumkaj-moklanjo.env"))
    }
    if (Test-Path (Join-Path $bookmarkDir "kumkaj-moklanjo-debug.env")) {
        return (Read-Manifest (Join-Path $bookmarkDir "kumkaj-moklanjo-debug.env"))
    }

    if ($Dev) {
        return @{
            KUMKAJ_ENV = "development"
            KUMKAJ_APP_ID = "com.moklanjo.debug"
            KUMKAJ_APP_LABEL = "Moklanjo Dev"
        }
    }
    return @{
        KUMKAJ_ENV = "production"
        KUMKAJ_APP_ID = "com.moklanjo"
        KUMKAJ_APP_LABEL = "Moklanjo"
    }
}

$m = Resolve-Identity
$AppId = $m["KUMKAJ_APP_ID"]
$Label = $m["KUMKAJ_APP_LABEL"]
$EnvName = $m["KUMKAJ_ENV"]
$IsDebug = $AppId.EndsWith(".debug") -or $EnvName -eq "development"
$ProgramsName = if ($IsDebug) { "Moklanjo Dev" } else { "Moklanjo" }
$slug = if ($IsDebug) { "kumkaj-moklanjo-debug" } else { "kumkaj-moklanjo" }

$InstallDir = $m["INSTALL_DIR"]
if (-not $InstallDir) {
    $InstallDir = Join-Path $env:LOCALAPPDATA "Programs\$ProgramsName"
}
# Prefer the folder we were launched from when it looks like an install tree.
if (Test-Path (Join-Path $ScriptDir "kumkaj-moklanjo.exe")) {
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

$appPresent = Test-Path (Join-Path $InstallDir "kumkaj-moklanjo.exe")
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

Write-Host ""
Write-Host "$Label - local books data:"
Write-Host "  $DataDir"
$removeData = $false
if (Test-Path $DataDir) {
    Write-Host ""
    Write-Host "Remove this data? (books, backups, settings)"
    Write-Host "  Y = delete data forever"
    Write-Host "  N = keep data (run uninstall again later to remove it)"
    $ans = Read-Host "Remove data? [y/N]"
    if ($ans -match '^(y|yes)$') { $removeData = $true }
} else {
    Write-Host "  (no data directory found - nothing to remove)"
}

if ($removeData) {
    Write-Host "Removing data..."
    Remove-Item -Recurse -Force $DataDir
    Write-Host "  Data removed: $DataDir"
} elseif (Test-Path $DataDir) {
    Write-Host "Keeping data at: $DataDir"
    Write-Host "Run this uninstaller again later if you want to delete it."
}

if (Test-Path $Bookmark) { Remove-Item -Force $Bookmark }

Write-Host ""
Write-Host "Done."
if (-not $removeData -and (Test-Path $DataDir)) {
    Write-Host "Reminder: books data is still on disk. Re-run uninstall to remove it."
}

if ($cleanup) {
    Start-Process powershell -WindowStyle Hidden -ArgumentList @(
        "-NoProfile", "-Command",
        "Start-Sleep -Seconds 1; Remove-Item -LiteralPath '$cleanup' -Recurse -Force -ErrorAction SilentlyContinue"
    )
}

Pause-IfInteractive
