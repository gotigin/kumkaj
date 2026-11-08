# Uninstall Tabiyat (Windows). Re-runnable: if app files are gone but
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
    if ($Dev -and (Test-Path (Join-Path $bookmarkDir "kumkaj-tabiyat-debug.env"))) {
        return (Read-Manifest (Join-Path $bookmarkDir "kumkaj-tabiyat-debug.env"))
    }
    if ($Prod -and (Test-Path (Join-Path $bookmarkDir "kumkaj-tabiyat.env"))) {
        return (Read-Manifest (Join-Path $bookmarkDir "kumkaj-tabiyat.env"))
    }
    if (Test-Path (Join-Path $bookmarkDir "kumkaj-tabiyat.env")) {
        return (Read-Manifest (Join-Path $bookmarkDir "kumkaj-tabiyat.env"))
    }
    if (Test-Path (Join-Path $bookmarkDir "kumkaj-tabiyat-debug.env")) {
        return (Read-Manifest (Join-Path $bookmarkDir "kumkaj-tabiyat-debug.env"))
    }

    if ($Dev) {
        return @{
            KUMKAJ_ENV = "development"
            KUMKAJ_APP_ID = "com.tabiyat.debug"
            KUMKAJ_APP_LABEL = "Tabiyat Dev"
        }
    }
    return @{
        KUMKAJ_ENV = "production"
        KUMKAJ_APP_ID = "com.tabiyat"
        KUMKAJ_APP_LABEL = "Tabiyat"
    }
}

$m = Resolve-Identity
$AppId = $m["KUMKAJ_APP_ID"]
$Label = $m["KUMKAJ_APP_LABEL"]
$EnvName = $m["KUMKAJ_ENV"]
$IsDebug = $AppId.EndsWith(".debug") -or $EnvName -eq "development"
$ProgramsName = if ($IsDebug) { "Tabiyat Dev" } else { "Tabiyat" }
$slug = if ($IsDebug) { "kumkaj-tabiyat-debug" } else { "kumkaj-tabiyat" }

$InstallDir = $m["INSTALL_DIR"]
if (-not $InstallDir) {
    $InstallDir = Join-Path $env:LOCALAPPDATA "Programs\$ProgramsName"
}
# Prefer the folder we were launched from when it looks like an install tree.
if (Test-Path (Join-Path $ScriptDir "kumkaj-tabiyat.exe")) {
    $InstallDir = $ScriptDir
}

$DataDir = Join-Path $env:APPDATA $AppId
$StartMenu = Join-Path $env:APPDATA "Microsoft\Windows\Start Menu\Programs\$ProgramsName"
$DesktopLnk = Join-Path ([Environment]::GetFolderPath('Desktop')) "$Label.lnk"
$TaskbarPinLnk = Join-Path $env:APPDATA "Microsoft\Internet Explorer\Quick Launch\User Pinned\TaskBar\$Label.lnk"
$UninstallKey = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\$AppId"
$Bookmark = Join-Path $env:LOCALAPPDATA "Kumkaj\$slug.env"

Write-Host "Uninstall $Label"
Write-Host "  App id:    $AppId"
Write-Host "  App files: $InstallDir"
Write-Host "  Data:      $DataDir"

$exe = Join-Path $InstallDir "kumkaj-tabiyat.exe"
$appPresent = Test-Path $exe

# Ask about data while the exe can still wipe SQLite categories — one dialog,
# three checkboxes (same as MSI uninstall / Delete books data).
Write-Host ""
Write-Host "$Label - local data:"
Write-Host "  $DataDir"
$wiped = $false
$cats = @()
if (Test-Path $DataDir) {
    $prompt = $null
    foreach ($cand in @(
            (Join-Path $ScriptDir "windows-wipe-prompt.ps1"),
            (Join-Path $InstallDir "windows-wipe-prompt.ps1"),
            (Join-Path $ScriptDir "..\..\..\scripts\windows-wipe-prompt.ps1")
        )) {
        if (Test-Path $cand) { $prompt = $cand; break }
    }
    if ($prompt) {
        . $prompt
        $cats = @(Get-KumkajWipeCategories -AppLabel $Label -DataDir $DataDir)
    } else {
        Write-Host "(wipe prompt UI missing — keeping all local data)"
    }
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
    # Remove Start Menu / Desktop / taskbar pin copy + ARP first.
    if (Test-Path $StartMenu) {
        Remove-Item -Recurse -Force $StartMenu
    }
    if (Test-Path $DesktopLnk) {
        Remove-Item -Force $DesktopLnk
    }
    if (Test-Path $TaskbarPinLnk) {
        Remove-Item -Force $TaskbarPinLnk
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
    if (Test-Path $DesktopLnk) { Remove-Item -Force $DesktopLnk }
    if (Test-Path $TaskbarPinLnk) { Remove-Item -Force $TaskbarPinLnk }
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
