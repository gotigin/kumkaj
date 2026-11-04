# Install Moklanjo for the current Windows user.
# Prefer double-click Setup.bat from the package folder (do not open .ps1 in Notepad).
#
# Package layout required:
#   kumkaj-moklanjo.exe
#   kumkaj_moklanjo\resources\     (underscores - Makepad crate name)
#   install-manifest.env
#   uninstall.ps1
#
# Usage:
#   .\Setup.bat
#   powershell -ExecutionPolicy Bypass -File .\install.ps1
param(
    [string]$PackageDir = $PSScriptRoot,
    [switch]$NoPause
)

$ErrorActionPreference = "Stop"

function Pause-IfInteractive {
    if ($NoPause) { return }
    if ([Environment]::UserInteractive -and -not [Console]::IsInputRedirected) {
        Write-Host ""
        Write-Host "Press Enter to close this window..."
        try { [void][Console]::ReadLine() } catch { Read-Host "Press Enter to close" | Out-Null }
    }
}

try {
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
    $ProgramsName = if ($IsDebug) { "Moklanjo Dev" } else { "Moklanjo" }
    $InstallDir = Join-Path $env:LOCALAPPDATA "Programs\$ProgramsName"
    $DataDir = Join-Path $env:APPDATA $AppId
    $StartMenu = Join-Path $env:APPDATA "Microsoft\Windows\Start Menu\Programs\$ProgramsName"
    $UninstallKey = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\$AppId"

    # Makepad resolves self:resources via crate name with hyphens -> underscores.
    $ResDirName = "kumkaj_moklanjo"
    $ExeSrc = Join-Path $PackageDir "kumkaj-moklanjo.exe"
    $ResSrc = Join-Path $PackageDir "$ResDirName\resources"
    $LegacyRes = Join-Path $PackageDir "kumkaj-moklanjo\resources"
    if (-not (Test-Path $ExeSrc)) { Write-Error "Missing $ExeSrc" }
    if (-not (Test-Path $ResSrc)) {
        if (Test-Path $LegacyRes) {
            Write-Error @"
Found legacy resources at kumkaj-moklanjo\resources (hyphens).
Makepad needs kumkaj_moklanjo\resources (underscores). Re-run the package script.
"@
        }
        Write-Error "Missing $ResSrc"
    }
    $FontCheck = Join-Path $ResSrc "fonts\NotoSans-Regular.ttf"
    if (-not (Test-Path $FontCheck)) {
        Write-Error "Missing UI font: $FontCheck - package is incomplete. Re-run scripts/fetch-ui-fonts.sh apps/moklanjo then apps/moklanjo/scripts/package-windows.sh"
    }
    $FaCheck = Join-Path $ResSrc "fonts\fa-solid-900.ttf"
    if (-not (Test-Path $FaCheck)) {
        Write-Error "Missing icon font: $FaCheck - action icons will be empty boxes. Re-run scripts/fetch-ui-fonts.sh apps/moklanjo then re-package."
    }

    Write-Host "Installing $Label"
    Write-Host "  App files:    $InstallDir"
    Write-Host "  Data (later): $DataDir"
    Write-Host "  App id:       $AppId"

    if (Test-Path $InstallDir) {
        Remove-Item -Recurse -Force $InstallDir
    }
    New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
    New-Item -ItemType Directory -Force -Path $StartMenu | Out-Null

    Copy-Item $ExeSrc (Join-Path $InstallDir "kumkaj-moklanjo.exe")
    Copy-Item (Join-Path $PackageDir $ResDirName) (Join-Path $InstallDir $ResDirName) -Recurse -Force

    # Uninstall + manifest beside the app (re-runnable).
    Copy-Item (Join-Path $PackageDir "uninstall.ps1") (Join-Path $InstallDir "uninstall.ps1") -Force
    if (Test-Path (Join-Path $PackageDir "Uninstall.bat")) {
        Copy-Item (Join-Path $PackageDir "Uninstall.bat") (Join-Path $InstallDir "Uninstall.bat") -Force
    }
    Copy-Item $ManifestPath (Join-Path $InstallDir "install-manifest.env") -Force
    Add-Content (Join-Path $InstallDir "install-manifest.env") "INSTALL_DIR=$InstallDir"

    $BookmarkDir = Join-Path $env:LOCALAPPDATA "Kumkaj"
    New-Item -ItemType Directory -Force -Path $BookmarkDir | Out-Null
    $slug = if ($IsDebug) { "kumkaj-moklanjo-debug" } else { "kumkaj-moklanjo" }
    Copy-Item (Join-Path $InstallDir "install-manifest.env") (Join-Path $BookmarkDir "$slug.env") -Force

    $ExePath = Join-Path $InstallDir "kumkaj-moklanjo.exe"
    $IconIco = Join-Path $InstallDir "$ResDirName\resources\icon.ico"
    $Wsh = New-Object -ComObject WScript.Shell

    function New-AppShortcut([string]$LnkPath, [string]$Target, [string]$WorkDir, [string]$Desc) {
        $lnk = $Wsh.CreateShortcut($LnkPath)
        $lnk.TargetPath = $Target
        $lnk.WorkingDirectory = $WorkDir
        $lnk.Description = $Desc
        if (Test-Path $IconIco) { $lnk.IconLocation = $IconIco }
        $lnk.Save()
        return $LnkPath
    }

    function Pin-ToTaskbar([string]$LnkPath) {
        try {
            $shellApp = New-Object -ComObject Shell.Application
            $dir = Split-Path -Parent $LnkPath
            $name = Split-Path -Leaf $LnkPath
            $folder = $shellApp.Namespace($dir)
            if (-not $folder) { return }
            $item = $folder.ParseName($name)
            if (-not $item) { return }
            foreach ($verb in $item.Verbs()) {
                $v = ($verb.Name -replace '&', '').ToLowerInvariant()
                if ($v -match 'pin to taskbar' -or ($v -match 'taskbar' -and $v -match 'pin') -or ($v -match 'taskleiste' -and $v -match 'anheft')) {
                    $verb.DoIt()
                    return
                }
            }
        } catch {}
        try {
            $pinDir = Join-Path $env:APPDATA 'Microsoft\Internet Explorer\Quick Launch\User Pinned\TaskBar'
            New-Item -ItemType Directory -Force -Path $pinDir | Out-Null
            Copy-Item $LnkPath (Join-Path $pinDir (Split-Path -Leaf $LnkPath)) -Force
        } catch {}
    }

    $StartLnk = New-AppShortcut (Join-Path $StartMenu "$Label.lnk") $ExePath $InstallDir $Label
    $Desktop = [Environment]::GetFolderPath('Desktop')
    New-AppShortcut (Join-Path $Desktop "$Label.lnk") $ExePath $InstallDir $Label | Out-Null
    Pin-ToTaskbar $StartLnk

    $UninstallPs1 = Join-Path $InstallDir "uninstall.ps1"
    $UnLnk = $Wsh.CreateShortcut((Join-Path $StartMenu "Uninstall $Label.lnk"))
    $UnLnk.TargetPath = "powershell.exe"
    $UnLnk.Arguments = "-NoProfile -ExecutionPolicy Bypass -File `"$UninstallPs1`""
    $UnLnk.WorkingDirectory = $InstallDir
    $UnLnk.Description = "Uninstall $Label"
    $UnLnk.Save()

    New-Item -Path $UninstallKey -Force | Out-Null
    Set-ItemProperty $UninstallKey -Name "DisplayName" -Value $Label
    Set-ItemProperty $UninstallKey -Name "Publisher" -Value "Kumkaj"
    Set-ItemProperty $UninstallKey -Name "InstallLocation" -Value $InstallDir
    Set-ItemProperty $UninstallKey -Name "DisplayVersion" -Value "0.1.0"
    Set-ItemProperty $UninstallKey -Name "UninstallString" -Value "powershell.exe -NoProfile -ExecutionPolicy Bypass -File `"$UninstallPs1`""
    Set-ItemProperty $UninstallKey -Name "NoModify" -Value 1 -Type DWord
    Set-ItemProperty $UninstallKey -Name "NoRepair" -Value 1 -Type DWord
    if (Test-Path $IconIco) {
        Set-ItemProperty $UninstallKey -Name "DisplayIcon" -Value $IconIco
    }

    $InstalledFont = Join-Path $InstallDir "$ResDirName\resources\fonts\NotoSans-Regular.ttf"
    if (-not (Test-Path $InstalledFont)) {
        Write-Error "Install copied incomplete resources (missing $InstalledFont)"
    }

    Write-Host ""
    Write-Host "Installed $Label."
    Write-Host "  Start:  Start Menu / Desktop / taskbar -> $Label"
    Write-Host "  Remove: Start Menu -> Uninstall $Label  (or Settings -> Apps)"
    Write-Host "  Data:   $DataDir  (created on first run; uninstall asks keep vs delete)"

    try {
        Start-Process -FilePath $ExePath -WorkingDirectory $InstallDir
    } catch {
        Write-Host "Could not start $Label automatically: $($_.Exception.Message)"
    }

    Pause-IfInteractive
    exit 0
}
catch {
    Write-Host ""
    Write-Host "ERROR: $($_.Exception.Message)" -ForegroundColor Red
    Pause-IfInteractive
    exit 1
}
