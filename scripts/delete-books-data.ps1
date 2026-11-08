# Delete selected __APP_LABEL__ local data for __APP_ID__.
# Installed into the app folder by the MSI; Start Menu shortcut runs this.
# MSI uninstall also runs this with -FromUninstall (one checkbox dialog).
# Placeholders are replaced by build-msi.sh.
param(
    [switch]$FromUninstall
)
$ErrorActionPreference = "Stop"
$DataDir = Join-Path $env:APPDATA "__APP_ID__"
$Exe = Join-Path $PSScriptRoot "__BIN_NAME__.exe"
$PromptScript = Join-Path $PSScriptRoot "windows-wipe-prompt.ps1"

if (-not $FromUninstall) {
    Write-Host "__APP_LABEL__ local data:"
    Write-Host "  $DataDir"
    Write-Host ""
}

if (-not (Test-Path $DataDir)) {
    if (-not $FromUninstall) {
        Write-Host "No local data found - nothing to delete."
        Write-Host ""
        Write-Host "Press Enter to close this window..."
        try { [void][Console]::ReadLine() } catch { Read-Host | Out-Null }
    }
    exit 0
}

if (-not (Test-Path $PromptScript)) {
    Write-Error "Missing windows-wipe-prompt.ps1 beside this script."
    exit 1
}
. $PromptScript
$cats = @(Get-KumkajWipeCategories -AppLabel "__APP_LABEL__" -DataDir $DataDir)
if ($cats.Count -eq 0) {
    if (-not $FromUninstall) {
        Write-Host "Cancelled. Nothing was deleted."
        Write-Host ""
        Write-Host "Press Enter to close this window..."
        try { [void][Console]::ReadLine() } catch { Read-Host | Out-Null }
    }
    exit 0
}

$delSetup = $cats -contains "schema"
$delRecords = $cats -contains "trading"
$delUi = $cats -contains "ui"

if (Test-Path $Exe) {
    if (-not $FromUninstall) {
        Write-Host "Deleting: $($cats -join ', ') ..."
    }
    $argList = @("--uninstall-wipe") + $cats
    $p = Start-Process -FilePath $Exe -ArgumentList $argList -Wait -PassThru -WindowStyle Hidden
    if (-not $FromUninstall) {
        if ($p.ExitCode -ne 0) {
            Write-Host "Wipe finished with exit code $($p.ExitCode)."
        } else {
            Write-Host "Done."
        }
    }
} else {
    if (-not $FromUninstall) {
        Write-Host "App exe not found beside this script; removing folders directly."
    }
    if ($delSetup) {
        Remove-Item -Recurse -Force -ErrorAction SilentlyContinue (Join-Path $DataDir "servers")
        Remove-Item -Recurse -Force -ErrorAction SilentlyContinue (Join-Path $DataDir "client")
        Remove-Item -Recurse -Force -ErrorAction SilentlyContinue (Join-Path $DataDir "backups")
        Remove-Item -Force -ErrorAction SilentlyContinue (Join-Path $DataDir "servers.json")
    }
    if ($delUi) {
        Remove-Item -Force -ErrorAction SilentlyContinue (Join-Path $DataDir "window_geom.json")
        Remove-Item -Force -ErrorAction SilentlyContinue (Join-Path $DataDir "app_language.json")
    }
    if ($delRecords -and -not $delSetup -and -not $FromUninstall) {
        Write-Host "Cannot clear business records without the app exe. Choose shop setup to remove the databases, or reinstall and try again."
    }
}

if (-not $FromUninstall) {
    Write-Host ""
    Write-Host "Press Enter to close this window..."
    try { [void][Console]::ReadLine() } catch { Read-Host | Out-Null }
}
