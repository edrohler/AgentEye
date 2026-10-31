#requires -Version 7
<#
.SYNOPSIS
Deletes all but the newest release candidates of one version.

.DESCRIPTION
Run by CI after a candidate is published; needs the gh CLI authenticated through GH_TOKEN. Tags that
have no published release (left behind by a failed run) and unpublished drafts are removed too, so
they cannot displace a valid candidate.

.PARAMETER BaseVersion
The version the candidates belong to, for example 0.1.3 (candidates are tagged 0.1.3-rc.N).

.PARAMETER Keep
How many of the newest published candidates to keep.
#>
[CmdletBinding()]
param(
    [Parameter(Mandatory)] [string]$BaseVersion,
    [int]$Keep = 3
)

$ErrorActionPreference = 'Stop'

git fetch --tags --force --quiet

[string]$pattern = "^$([regex]::Escape($BaseVersion))-rc\.(\d+)$"
[string[]]$tags = @(git tag --list "$BaseVersion-rc.*" | Where-Object { $_ -match $pattern })
function Get-CandidateNumber([string]$Tag) { [int][regex]::Match($Tag, $pattern).Groups[1].Value }

[string[]]$releasedTags = @(gh release list --limit 200 --json tagName --jq '.[].tagName')
[string[]]$orphaned = @($tags | Where-Object { $releasedTags -notcontains $_ })
[string[]]$stale = @(
    $tags | Where-Object { $releasedTags -contains $_ } |
        Sort-Object { Get-CandidateNumber $_ } -Descending |
        Select-Object -Skip $Keep
) + $orphaned

# A draft left by a failed run has no tag; it would be replaced when its number is reused, but do not leave it around.
[string[]]$drafts = @(gh release list --limit 200 --json tagName,isDraft --jq '.[] | select(.isDraft) | .tagName' | Where-Object { $_ -match $pattern })
foreach ($draft in $drafts) {
    Write-Host "Removing unpublished draft $draft"
    gh release delete $draft --yes
}

foreach ($tag in $stale) {
    Write-Host "Removing old release candidate $tag"
    gh release delete $tag --cleanup-tag --yes
    if ($LASTEXITCODE -ne 0) {
        # The release may already be gone (for example after a manual delete); remove the tag directly.
        gh api --method DELETE "repos/$env:GITHUB_REPOSITORY/git/refs/tags/$tag"
        if ($LASTEXITCODE -ne 0) {
            throw "Failed to remove release candidate $tag."
        }
    }
}
