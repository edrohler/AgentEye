#requires -Version 7
<#
.SYNOPSIS
Stamps a release version into the files the installers read it from.

.DESCRIPTION
The version lives in four places: the Cargo workspace (inherited by every crate),
src-tauri/tauri.conf.json (the installer and the app's About information), and the two
package.json files (the root tooling and the Angular UI), which are stamped so tools reading
them agree with the installers. CI runs this on a throwaway checkout before building, so the
committed files never change and a release candidate and its final release carry the same version.

.PARAMETER Version
A plain major.minor.patch version, for example 0.1.3. Pre-release suffixes are rejected because
Windows installers require a numeric version.
#>
[CmdletBinding()]
param(
    [Parameter(Mandatory)] [string]$Version
)

$ErrorActionPreference = 'Stop'

if ($Version -notmatch '^\d+\.\d+\.\d+$') {
    throw "Version '$Version' must look like 1.2.3."
}

[string]$repositoryRoot = Split-Path -Parent $PSScriptRoot

# Replaces the first match only, and fails if there is none, so a reformatted file cannot be skipped silently.
function Set-FirstMatch([string]$Path, [string]$Pattern, [string]$Replacement) {
    [string]$text = Get-Content -Raw -Path $Path
    [regex]$regex = [regex]::new($Pattern, [System.Text.RegularExpressions.RegexOptions]::Multiline)
    if (-not $regex.IsMatch($text)) {
        throw "No version found in $Path."
    }

    [System.IO.File]::WriteAllText($Path, $regex.Replace($text, $Replacement, 1), [System.Text.UTF8Encoding]::new($false))
    Write-Host "Set version $Version in $Path"
}

# [workspace.package] is the only top-level `version = "..."` line; dependency versions are inline tables.
Set-FirstMatch (Join-Path $repositoryRoot 'Cargo.toml') '^version = "[^"]*"' "version = `"$Version`""

# JSON files: the first `"version"` key is the top-level one (dependencies are `"name": "range"` pairs).
foreach ($jsonFile in @('src-tauri/tauri.conf.json', 'package.json', 'ui/package.json')) {
    Set-FirstMatch (Join-Path $repositoryRoot $jsonFile) '^(\s*)"version": "[^"]*"' "`$1`"version`": `"$Version`""
}
