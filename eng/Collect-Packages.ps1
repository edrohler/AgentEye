#requires -Version 7
<#
.SYNOPSIS
Copies the installers Tauri built into one folder with predictable names.

.DESCRIPTION
Tauri writes installers under target/release/bundle/<format>/ with names that depend on the
format. This gathers the ones we ship, names them AgentEye-<version>-<platform>.<extension> so
the release workflows can rely on one naming scheme, and writes a .sha256 file next to each.
Renaming a file later (the Release workflow drops the -rc.N part) does not change its bytes, so
the checksum stays valid.

.PARAMETER Version
The version to put in the file names: the release candidate, for example 0.1.3-rc.2. This is not
the version stamped into the app, which never carries a release-candidate suffix.

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
    [string]$destination = Join-Path $OutputDirectory $name
    Copy-Item $found[0].FullName $destination

    [string]$hash = (Get-FileHash -Algorithm SHA256 $destination).Hash.ToLowerInvariant()
    "$hash  $name" | Set-Content -NoNewline "$destination.sha256"
    Write-Host "Collected $name"
}
