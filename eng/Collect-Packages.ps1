#requires -Version 7
<#
.SYNOPSIS
Copies the installers Tauri built into one folder with predictable names.

.DESCRIPTION
Tauri writes installers under target/release/bundle/<format>/ with names that depend on the
format. This gathers the ones we ship and names them AgentEye-<version>-<platform>.<extension> so
the release workflows can rely on one naming scheme.

.PARAMETER Version
The version stamped into the build, for example 0.1.3 (no release-candidate suffix).

.PARAMETER Platform
windows-x64 or linux-x64.

.PARAMETER OutputDirectory
Folder that receives the renamed installers. It is created if needed.
#>
[CmdletBinding()]
param(
    [Parameter(Mandatory)] [string]$Version,
    [Parameter(Mandatory)] [ValidateSet('windows-x64', 'linux-x64')] [string]$Platform,
    [string]$OutputDirectory = 'artifacts/packages'
)

$ErrorActionPreference = 'Stop'

[string]$repositoryRoot = Split-Path -Parent $PSScriptRoot
[string]$bundleRoot = Join-Path $repositoryRoot 'target/release/bundle'

# Folder under bundle/, file pattern, and the suffix after the platform in the final name.
# The outer @() matters: PowerShell unwraps a single result on assignment, which would turn
# Windows' one format into a flat list of three strings instead of one folder/pattern/suffix entry.
$formats = @(switch ($Platform) {
        'windows-x64' { , @('nsis', '*-setup.exe', '-setup.exe') }
        'linux-x64' { @('deb', '*.deb', '.deb'), @('appimage', '*.AppImage', '.AppImage') }
    })

New-Item -ItemType Directory -Force -Path $OutputDirectory | Out-Null

foreach ($format in $formats) {
    $folder, $pattern, $suffix = $format
    [object[]]$found = @(Get-ChildItem -File -Path (Join-Path $bundleRoot $folder) -Filter $pattern -ErrorAction SilentlyContinue)
    if ($found.Count -ne 1) {
        throw "Expected exactly one $pattern in $bundleRoot/$folder, found $($found.Count)."
    }

    [string]$name = "AgentEye-$Version-$Platform$suffix"
    Copy-Item $found[0].FullName (Join-Path $OutputDirectory $name)
    Write-Host "Collected $name"
}
