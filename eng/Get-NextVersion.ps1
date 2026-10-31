#requires -Version 7
<#
.SYNOPSIS
Works out the next release version from version.json and the existing release tags.

.DESCRIPTION
version.json holds major.minor. The patch is one more than the newest release tag of that line, or 0
for a new line. With -Candidate, also prints the next release-candidate number for that version
(counting published tags and unpublished drafts, which reserve their number).

Writes `base=` (and with -Candidate, `candidate=`) to $GITHUB_OUTPUT when it is set, and always to the console.

.PARAMETER Candidate
Also compute the next release-candidate version, for example 0.1.3-rc.4. Needs the gh CLI (GH_TOKEN).
#>
[CmdletBinding()]
param(
    [switch]$Candidate
)

$ErrorActionPreference = 'Stop'

[string]$repositoryRoot = Split-Path -Parent $PSScriptRoot
[version]$line = [version](Get-Content -Raw (Join-Path $repositoryRoot 'version.json') | ConvertFrom-Json).version

git fetch --tags --force --quiet
[version[]]$released = @(
    git tag --list |
        Where-Object { $_ -match '^\d+\.\d+\.\d+$' } |
        ForEach-Object { [version]$_ } |
        Where-Object { $_.Major -eq $line.Major -and $_.Minor -eq $line.Minor }
)
[int]$patch = if ($released.Count -gt 0) { ($released | Sort-Object -Descending)[0].Build + 1 } else { 0 }
[version]$base = [version]::new($line.Major, $line.Minor, $patch)

$outputs = [ordered]@{ base = "$base" }

if ($Candidate) {
    [string[]]$draftTags = @(gh release list --limit 200 --json tagName --jq '.[].tagName')
    [int]$highest = 0
    foreach ($tag in @(git tag --list "$base-rc.*") + $draftTags) {
        if ($tag -match "^$([regex]::Escape("$base"))-rc\.(\d+)$") {
            $highest = [Math]::Max($highest, [int]$Matches[1])
        }
    }

    $outputs['candidate'] = "$base-rc.$($highest + 1)"
}

foreach ($name in $outputs.Keys) {
    Write-Host "$name=$($outputs[$name])"
    if ($env:GITHUB_OUTPUT) {
        "$name=$($outputs[$name])" >> $env:GITHUB_OUTPUT
    }
}
