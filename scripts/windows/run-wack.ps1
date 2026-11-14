<#
.SYNOPSIS
  Run the Windows App Certification Kit (WACK) on a Kumkaj Windows package.

.DESCRIPTION
  Store path (the product goal): an .msix from apps/<app>/scripts/msix.sh.
  WACK installs the package, so an unsigned Store upload is first test-signed
  with a throw-away certificate whose subject is the manifest's Publisher
  (trusted only on this machine, removed again with -Cleanup).

  Sideload path: an .msi from apps/<app>/scripts/msi.sh, run through WACK's
  desktop-app tests with msiexec /qn. Unsigned binaries fail the
  "digitally signed files" test there until the .exe and .msi carry a real
  code-signing certificate — expected for an internal build.

  Needs: Windows 10/11, an elevated PowerShell, the Windows SDK's
  "Windows App Certification Kit" and "Signing Tools for Desktop Apps"
  (https://developer.microsoft.com/windows/downloads/windows-sdk/).
  Docs: docs/DEV.md § Windows App Certification Kit.

.EXAMPLE
  .\run-wack.ps1 -Package Z:\dist\Khaynjo-1.0.9.msix
  .\run-wack.ps1 -Package Z:\dist\Khaynjo-1.0.9.msi
  .\run-wack.ps1 -Cleanup       # remove the test certificate again
#>
[CmdletBinding()]
param(
  [string]$Package,
  [string]$ReportDir = "$env:USERPROFILE\Documents\kumkaj-wack",
  [switch]$Cleanup
)
$ErrorActionPreference = 'Stop'
$CertName = 'Kumkaj WACK test signing'

function Find-SdkTool([string]$name) {
  $roots = @("${env:ProgramFiles(x86)}\Windows Kits\10", "$env:ProgramFiles\Windows Kits\10")
  foreach ($r in $roots) {
    $hit = Get-ChildItem -Path $r -Recurse -Filter $name -ErrorAction SilentlyContinue |
      Where-Object { $_.FullName -match 'x64|App Certification Kit' } |
      Sort-Object FullName -Descending | Select-Object -First 1
    if ($hit) { return $hit.FullName }
  }
  $cmd = Get-Command $name -ErrorAction SilentlyContinue
  if ($cmd) { return $cmd.Source }
  throw "$name not found — install the Windows SDK component that provides it."
}

function Remove-TestCert {
  foreach ($store in 'Cert:\CurrentUser\My', 'Cert:\LocalMachine\TrustedPeople') {
    Get-ChildItem $store | Where-Object { $_.FriendlyName -eq $CertName } | ForEach-Object {
      Write-Host "Removing test certificate $($_.Thumbprint) from $store"
      Remove-Item $_.PSPath
    }
  }
}

$admin = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole(
  [Security.Principal.WindowsBuiltInRole]::Administrator)
if (-not $admin) { throw 'Run this from an elevated PowerShell (WACK and the certificate store need admin).' }

if ($Cleanup) { Remove-TestCert; return }
if (-not $Package) { throw 'Pass -Package <path to .msix or .msi>.' }
$Package = (Resolve-Path $Package).Path
New-Item -ItemType Directory -Force -Path $ReportDir | Out-Null
$stamp = Get-Date -Format 'yyyyMMdd-HHmmss'
$name = [IO.Path]::GetFileNameWithoutExtension($Package)
$report = Join-Path $ReportDir "$name-$stamp.xml"
$appcert = Find-SdkTool 'appcert.exe'
Write-Host "appcert: $appcert"

switch ([IO.Path]::GetExtension($Package).ToLowerInvariant()) {
  '.msix' {
    # Publisher from the package's own manifest: the signing subject must match it.
    Add-Type -AssemblyName System.IO.Compression.FileSystem
    $zip = [IO.Compression.ZipFile]::OpenRead($Package)
    try {
      $entry = $zip.Entries | Where-Object { $_.FullName -eq 'AppxManifest.xml' }
      $reader = New-Object IO.StreamReader($entry.Open())
      [xml]$manifest = $reader.ReadToEnd(); $reader.Close()
    } finally { $zip.Dispose() }
    $publisher = $manifest.Package.Identity.Publisher
    $identity = $manifest.Package.Identity.Name
    Write-Host "Identity: $identity  Publisher: $publisher"

    $signtool = Find-SdkTool 'signtool.exe'
    $cert = Get-ChildItem Cert:\CurrentUser\My | Where-Object { $_.FriendlyName -eq $CertName -and $_.Subject -eq $publisher } | Select-Object -First 1
    if (-not $cert) {
      $cert = New-SelfSignedCertificate -Type Custom -Subject $publisher -KeyUsage DigitalSignature `
        -FriendlyName $CertName -CertStoreLocation 'Cert:\CurrentUser\My' `
        -TextExtension @('2.5.29.37={text}1.3.6.1.5.5.7.3.3', '2.5.29.19={text}')
    }
    $pfx = Join-Path $env:TEMP 'kumkaj-wack-test.pfx'
    $pfxPassword = ConvertTo-SecureString -String 'kumkaj-wack' -Force -AsPlainText
    Export-PfxCertificate -Cert $cert -FilePath $pfx -Password $pfxPassword | Out-Null
    $cer = Join-Path $env:TEMP 'kumkaj-wack-test.cer'
    Export-Certificate -Cert $cert -FilePath $cer | Out-Null
    Import-Certificate -FilePath $cer -CertStoreLocation 'Cert:\LocalMachine\TrustedPeople' | Out-Null

    # Sign a copy; the original stays the unsigned Store upload.
    $signed = Join-Path $ReportDir "$name-wack-signed.msix"
    Copy-Item $Package $signed -Force
    & $signtool sign /fd SHA256 /f $pfx /p 'kumkaj-wack' $signed
    if ($LASTEXITCODE -ne 0) { throw 'signtool failed' }
    # A left-over install of the same identity makes WACK's install step fail.
    Get-AppxPackage -Name $identity -ErrorAction SilentlyContinue | Remove-AppxPackage

    & $appcert reset
    & $appcert test -appxpackagepath $signed -reportoutputpath $report
  }
  '.msi' {
    & $appcert reset
    $msiexec = Join-Path $env:WINDIR 'System32\msiexec.exe'
    # Flags as in `appcert.exe test /?` (desktop apps). If your WACK version
    # names them differently, run that and adjust here.
    & $appcert test -setuppath $msiexec -setupcommandline "/i `"$Package`" /qn" `
      -uninstallpath $msiexec -uninstallcommandline "/x `"$Package`" /qn" `
      -reportoutputpath $report
  }
  default { throw 'Package must be an .msix or an .msi' }
}

if (-not (Test-Path $report)) { throw "WACK wrote no report ($report)." }
[xml]$xml = Get-Content $report
$overall = $xml.REPORT.OVERALL_RESULT
Write-Host ''
Write-Host "Report: $report"
Write-Host "Overall: $overall"
# Every test that did not pass, with its message.
$xml.SelectNodes('//TEST') | Where-Object { $_.RESULT -and $_.RESULT -ne 'PASS' } | ForEach-Object {
  $msg = ($_.SelectNodes('.//MESSAGE') | ForEach-Object { $_.TEXT }) -join ' | '
  Write-Host ("{0,-8} {1}  {2}" -f $_.RESULT, $_.NAME, $msg)
}
if ($overall -ne 'PASS') { exit 1 }
