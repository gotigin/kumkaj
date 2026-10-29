# Delete selected __APP_LABEL__ local data for __APP_ID__.
# Installed into the app folder by the MSI; Start Menu shortcut runs this.
# Placeholders are replaced by build-msi.sh. Prompts before deleting anything.
$ErrorActionPreference = "Stop"
$DataDir = Join-Path $env:APPDATA "__APP_ID__"
$Exe = Join-Path $PSScriptRoot "__BIN_NAME__.exe"

Write-Host "__APP_LABEL__ local data:"
Write-Host "  $DataDir"
Write-Host ""
if (-not (Test-Path $DataDir)) {
    Write-Host "No local data found - nothing to delete."
    Write-Host ""
    Write-Host "Press Enter to close this window..."
    try { [void][Console]::ReadLine() } catch { Read-Host | Out-Null }
    exit 0
}

Add-Type -AssemblyName System.Windows.Forms | Out-Null
Add-Type -AssemblyName System.Drawing | Out-Null

$form = New-Object System.Windows.Forms.Form
$form.Text = "Delete __APP_LABEL__ data"
$form.Size = New-Object System.Drawing.Size(480, 280)
$form.StartPosition = "CenterScreen"
$form.FormBorderStyle = "FixedDialog"
$form.MaximizeBox = $false
$form.MinimizeBox = $false

$label = New-Object System.Windows.Forms.Label
$label.Location = New-Object System.Drawing.Point(16, 12)
$label.Size = New-Object System.Drawing.Size(440, 40)
$label.Text = "Choose what to remove from this computer. Nothing is deleted until you confirm."
$form.Controls.Add($label)

$cbRecords = New-Object System.Windows.Forms.CheckBox
$cbRecords.Location = New-Object System.Drawing.Point(20, 60)
$cbRecords.Size = New-Object System.Drawing.Size(430, 36)
$cbRecords.Text = "Business records (sales, payments, day books) — keep shop setup"
$form.Controls.Add($cbRecords)

$cbSetup = New-Object System.Windows.Forms.CheckBox
$cbSetup.Location = New-Object System.Drawing.Point(20, 100)
$cbSetup.Size = New-Object System.Drawing.Size(430, 36)
$cbSetup.Text = "Shop setup (products, rates, staff, brand, empty books)"
$form.Controls.Add($cbSetup)

$cbUi = New-Object System.Windows.Forms.CheckBox
$cbUi.Location = New-Object System.Drawing.Point(20, 140)
$cbUi.Size = New-Object System.Drawing.Size(430, 36)
$cbUi.Text = "Screen preferences (sidebar, amount visibility, window, language)"
$form.Controls.Add($cbUi)

$ok = New-Object System.Windows.Forms.Button
$ok.Text = "Delete selected"
$ok.Location = New-Object System.Drawing.Point(240, 200)
$ok.Size = New-Object System.Drawing.Size(110, 28)
$ok.DialogResult = [System.Windows.Forms.DialogResult]::OK
$form.Controls.Add($ok)
$form.AcceptButton = $ok

$cancel = New-Object System.Windows.Forms.Button
$cancel.Text = "Cancel"
$cancel.Location = New-Object System.Drawing.Point(360, 200)
$cancel.Size = New-Object System.Drawing.Size(90, 28)
$cancel.DialogResult = [System.Windows.Forms.DialogResult]::Cancel
$form.Controls.Add($cancel)
$form.CancelButton = $cancel

$result = $form.ShowDialog()
if ($result -ne [System.Windows.Forms.DialogResult]::OK) {
    Write-Host "Cancelled. Nothing was deleted."
    Write-Host ""
    Write-Host "Press Enter to close this window..."
    try { [void][Console]::ReadLine() } catch { Read-Host | Out-Null }
    exit 0
}

$delRecords = [bool]$cbRecords.Checked
$delSetup = [bool]$cbSetup.Checked
$delUi = [bool]$cbUi.Checked
if (-not $delRecords -and -not $delSetup -and -not $delUi) {
    Write-Host "Nothing selected. Nothing was deleted."
    Write-Host ""
    Write-Host "Press Enter to close this window..."
    try { [void][Console]::ReadLine() } catch { Read-Host | Out-Null }
    exit 0
}

$cats = @()
if ($delSetup) {
    $cats += "schema"
} elseif ($delRecords) {
    $cats += "trading"
}
if ($delUi) { $cats += "ui" }

if (Test-Path $Exe) {
    Write-Host "Deleting: $($cats -join ', ') ..."
    $argList = @("--uninstall-wipe") + $cats
    $p = Start-Process -FilePath $Exe -ArgumentList $argList -Wait -PassThru -WindowStyle Hidden
    if ($p.ExitCode -ne 0) {
        Write-Host "Wipe finished with exit code $($p.ExitCode)."
    } else {
        Write-Host "Done."
    }
} else {
    Write-Host "App exe not found beside this script; removing folders directly."
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
    if ($delRecords -and -not $delSetup) {
        Write-Host "Cannot clear business records without the app exe. Choose shop setup to remove the databases, or reinstall and try again."
    }
}

Write-Host ""
Write-Host "Press Enter to close this window..."
try { [void][Console]::ReadLine() } catch { Read-Host | Out-Null }
