# Download the CloakBrowser fingerprint kernel (c CloakHQ) for Windows.
# The kernel is NOT redistributed in this repo; it is fetched from the official
# CloakBrowser releases. https://github.com/CloakHQ/CloakBrowser
param(
  [string]$Dest = "$PSScriptRoot\..\cloakbrowser-windows-x64",
  [string]$Version = $(if ($env:CLOAKBROWSER_VERSION) { $env:CLOAKBROWSER_VERSION } else { "chromium-v146.0.7680.177.5" })
)
$ErrorActionPreference = "Stop"
$asset = "cloakbrowser-windows-x64.zip"
$url = "https://github.com/CloakHQ/CloakBrowser/releases/download/$Version/$asset"
$zip = Join-Path $env:TEMP $asset

Write-Host "Downloading $asset ($Version) -> $Dest"
New-Item -ItemType Directory -Force -Path $Dest | Out-Null
Invoke-WebRequest -Uri $url -OutFile $zip
Expand-Archive -Path $zip -DestinationPath $Dest -Force
Remove-Item $zip -Force

Write-Host ""
Write-Host "Done. Point the app at the kernel:"
Write-Host "  setx CLOAKBROWSER_BINARY_PATH `"$Dest\chrome.exe`""
