# Install Kumkaj Courier for the current Windows user.
# Run from a package folder (dist/windows or dist/windows-dev) that contains:
#   kumkaj-courier.exe
#   kumkaj-courier\resources\
#   install-manifest.env
#   uninstall.ps1
#
# Usage (from package folder):
#   powershell -ExecutionPolicy Bypass -File .\install.ps1
param(
    [string]$PackageDir = $PSScriptRoot
)

$ErrorActionPreference = "Stop"
$PackageDir = (Resolve-Path $PackageDir).Path
$ManifestPath = Join-Path $PackageDir "install-manifest.env"
if (-not (Test-Path $ManifestPath)) {
    Write-Error "Missing install-manifest.env in $PackageDir"
}

function Read-Manifest([string]$path) {
    $map = @{}
    Get-Content $path | ForEach-Object {
        $line = $_.Trim()
        if (-not $line -or $line.StartsWith("#")) { return }
        $i = $line.IndexOf("=")
        if ($i -lt 1) { return }
        $map[$line.Substring(0, $i).Trim()] = $line.Substring($i + 1).Trim().Trim('"')
    }
    return $map
}

$m = Read-Manifest $ManifestPath
$AppId = $m["KUMKAJ_APP_ID"]
$Label = $m["KUMKAJ_APP_LABEL"]
$EnvName = $m["KUMKAJ_ENV"]
if (-not $AppId) { Write-Error "install-manifest.env missing KUMKAJ_APP_ID" }

$IsDebug = $AppId.EndsWith(".debug") -or $EnvName -eq "development"
$ProgramsName = if ($IsDebug) { "Kumkaj Courier Dev" } else { "Kumkaj Courier" }
$InstallDir = Join-Path $env:LOCALAPPDATA "Programs\$ProgramsName"
$DataDir = Join-Path $env:APPDATA $AppId
$StartMenu = Join-Path $env:APPDATA "Microsoft\Windows\Start Menu\Programs\$ProgramsName"
$UninstallKey = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\$AppId"

$ExeSrc = Join-Path $PackageDir "kumkaj-courier.exe"
$ResSrc = Join-Path $PackageDir "kumkaj-courier\resources"
if (-not (Test-Path $ExeSrc)) { Write-Error "Missing $ExeSrc" }
if (-not (Test-Path $ResSrc)) { Write-Error "Missing $ResSrc" }

Write-Host "Installing $Label"
Write-Host "  App files:    $InstallDir"
Write-Host "  Data (later): $DataDir"
Write-Host "  App id:       $AppId"

if (Test-Path $InstallDir) {
    Remove-Item -Recurse -Force $InstallDir
}
New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
New-Item -ItemType Directory -Force -Path $StartMenu | Out-Null

Copy-Item $ExeSrc (Join-Path $InstallDir "kumkaj-courier.exe")
Copy-Item (Join-Path $PackageDir "kumkaj-courier") (Join-Path $InstallDir "kumkaj-courier") -Recurse -Force

# Uninstall + manifest beside the app (re-runnable).
Copy-Item (Join-Path $PackageDir "uninstall.ps1") (Join-Path $InstallDir "uninstall.ps1") -Force
Copy-Item $ManifestPath (Join-Path $InstallDir "install-manifest.env") -Force
# Stamp install path into manifest for the uninstaller.
Add-Content (Join-Path $InstallDir "install-manifest.env") "INSTALL_DIR=$InstallDir"

# Bookmark under LocalAppData so uninstall finds leftovers after a partial delete.
$BookmarkDir = Join-Path $env:LOCALAPPDATA "Kumkaj"
New-Item -ItemType Directory -Force -Path $BookmarkDir | Out-Null
$slug = if ($IsDebug) { "kumkaj-courier-debug" } else { "kumkaj-courier" }
Copy-Item (Join-Path $InstallDir "install-manifest.env") (Join-Path $BookmarkDir "$slug.env") -Force

# Start Menu shortcuts
$Wsh = New-Object -ComObject WScript.Shell
$AppLnk = $Wsh.CreateShortcut((Join-Path $StartMenu "$Label.lnk"))
$AppLnk.TargetPath = Join-Path $InstallDir "kumkaj-courier.exe"
$AppLnk.WorkingDirectory = $InstallDir
$AppLnk.Description = $Label
$IconIco = Join-Path $InstallDir "kumkaj-courier\resources\icon.ico"
if (Test-Path $IconIco) { $AppLnk.IconLocation = $IconIco }
$AppLnk.Save()

$UnLnk = $Wsh.CreateShortcut((Join-Path $StartMenu "Uninstall $Label.lnk"))
$UnLnk.TargetPath = "powershell.exe"
$UnLnk.Arguments = "-NoProfile -ExecutionPolicy Bypass -File `"$(Join-Path $InstallDir 'uninstall.ps1')`""
$UnLnk.WorkingDirectory = $InstallDir
$UnLnk.Description = "Uninstall $Label"
$UnLnk.Save()

# Add/Remove Programs (per-user)
New-Item -Path $UninstallKey -Force | Out-Null
Set-ItemProperty $UninstallKey -Name "DisplayName" -Value $Label
Set-ItemProperty $UninstallKey -Name "Publisher" -Value "Kumkaj"
Set-ItemProperty $UninstallKey -Name "InstallLocation" -Value $InstallDir
Set-ItemProperty $UninstallKey -Name "DisplayVersion" -Value "0.1.0"
Set-ItemProperty $UninstallKey -Name "UninstallString" -Value "powershell.exe -NoProfile -ExecutionPolicy Bypass -File `"$(Join-Path $InstallDir 'uninstall.ps1')`""
Set-ItemProperty $UninstallKey -Name "NoModify" -Value 1 -Type DWord
Set-ItemProperty $UninstallKey -Name "NoRepair" -Value 1 -Type DWord
if (Test-Path $IconIco) {
    Set-ItemProperty $UninstallKey -Name "DisplayIcon" -Value $IconIco
}

Write-Host ""
Write-Host "Installed $Label."
Write-Host "  Start:  Start Menu → $Label"
Write-Host "  Remove: Start Menu → Uninstall $Label  (or Settings → Apps)"
Write-Host "  Data:   $DataDir  (created on first run; uninstall asks keep vs delete)"
