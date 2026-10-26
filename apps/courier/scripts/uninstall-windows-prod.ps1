# Uninstall Kumkaj Courier (production) on Windows.
# Thin wrapper - same as: uninstall-windows.ps1 -Prod
$ErrorActionPreference = "Stop"
& "$PSScriptRoot\uninstall-windows.ps1" -Prod @args
