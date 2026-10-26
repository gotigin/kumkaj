# Delete __APP_LABEL__ books data for __APP_ID__.
# Installed into the app folder by the MSI; Start Menu shortcut runs this.
# __APP_ID__ is replaced by build-msi.sh. Prompts before deleting anything.
$ErrorActionPreference = "Stop"
$DataDir = Join-Path $env:APPDATA "__APP_ID__"

Write-Host "__APP_LABEL__ books data:"
Write-Host "  $DataDir"
Write-Host ""
if (-not (Test-Path $DataDir)) {
    Write-Host "No books data found - nothing to delete."
} else {
    Write-Host "This deletes your local books, backups and settings forever."
    $ans = Read-Host "Delete this data? [y/N]"
    if ($ans -match '^(y|yes)$') {
        Remove-Item -Recurse -Force $DataDir
        Write-Host "Books data deleted."
    } else {
        Write-Host "Kept. Nothing was deleted."
    }
}
Write-Host ""
Write-Host "Press Enter to close this window..."
try { [void][Console]::ReadLine() } catch { Read-Host | Out-Null }
