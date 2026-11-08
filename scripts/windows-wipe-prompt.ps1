# Shared Windows UI: one dialog, three checkboxes for local data wipe.
# Dot-source and call: $cats = Get-KumkajWipeCategories -AppLabel "Khaynjo" -DataDir $path
# Returns @( ) when the owner cancels or selects nothing.
# Used by MSI uninstall (via delete-books-data.ps1), folder Uninstall.ps1, and
# Start Menu "Delete books data".

function Get-KumkajWipeCategories {
    param(
        [Parameter(Mandatory = $true)][string]$AppLabel,
        [Parameter(Mandatory = $true)][string]$DataDir
    )

    Add-Type -AssemblyName System.Windows.Forms | Out-Null
    Add-Type -AssemblyName System.Drawing | Out-Null

    $form = New-Object System.Windows.Forms.Form
    $form.Text = "Uninstall $AppLabel — local data"
    $form.Size = New-Object System.Drawing.Size(520, 320)
    $form.StartPosition = "CenterScreen"
    $form.FormBorderStyle = "FixedDialog"
    $form.MaximizeBox = $false
    $form.MinimizeBox = $false
    $form.TopMost = $true

    $label = New-Object System.Windows.Forms.Label
    $label.Location = New-Object System.Drawing.Point(16, 12)
    $label.Size = New-Object System.Drawing.Size(470, 56)
    $label.Text = "Choose what to remove from this computer.`r`n$dataDir`r`nNothing is deleted until you confirm. Leave all unchecked to keep everything."
    $form.Controls.Add($label)

    $cbRecords = New-Object System.Windows.Forms.CheckBox
    $cbRecords.Location = New-Object System.Drawing.Point(20, 78)
    $cbRecords.Size = New-Object System.Drawing.Size(460, 40)
    $cbRecords.Text = "Business records — sales, payments, day books (shop setup stays)"
    $form.Controls.Add($cbRecords)

    $cbSetup = New-Object System.Windows.Forms.CheckBox
    $cbSetup.Location = New-Object System.Drawing.Point(20, 122)
    $cbSetup.Size = New-Object System.Drawing.Size(460, 40)
    $cbSetup.Text = "Shop setup — products, rates, staff, brand, empty books (also clears records)"
    $form.Controls.Add($cbSetup)

    $cbUi = New-Object System.Windows.Forms.CheckBox
    $cbUi.Location = New-Object System.Drawing.Point(20, 166)
    $cbUi.Size = New-Object System.Drawing.Size(460, 40)
    $cbUi.Text = "Screen preferences — sidebar, amount visibility, window size, language"
    $form.Controls.Add($cbUi)

    $ok = New-Object System.Windows.Forms.Button
    $ok.Text = "Continue"
    $ok.Location = New-Object System.Drawing.Point(280, 230)
    $ok.Size = New-Object System.Drawing.Size(100, 28)
    $ok.DialogResult = [System.Windows.Forms.DialogResult]::OK
    $form.Controls.Add($ok)
    $form.AcceptButton = $ok

    $cancel = New-Object System.Windows.Forms.Button
    $cancel.Text = "Keep all data"
    $cancel.Location = New-Object System.Drawing.Point(390, 230)
    $cancel.Size = New-Object System.Drawing.Size(100, 28)
    $cancel.DialogResult = [System.Windows.Forms.DialogResult]::Cancel
    $form.Controls.Add($cancel)
    $form.CancelButton = $cancel

    $result = $form.ShowDialog()
    $form.Dispose()
    if ($result -ne [System.Windows.Forms.DialogResult]::OK) {
        return @()
    }

    $cats = @()
    if ($cbSetup.Checked) {
        $cats += "schema"
    } elseif ($cbRecords.Checked) {
        $cats += "trading"
    }
    if ($cbUi.Checked) {
        $cats += "ui"
    }
    return $cats
}
