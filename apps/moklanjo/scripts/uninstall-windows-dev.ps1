# Uninstall Moklanjo Dev (debug app id) on Windows.
# Thin wrapper - same as: uninstall-windows.ps1 -Dev
$ErrorActionPreference = "Stop"
& "$PSScriptRoot\uninstall-windows.ps1" -Dev @args
