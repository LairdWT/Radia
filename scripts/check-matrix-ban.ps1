#Requires -Version 5.1

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

[string] $RepoRoot = Split-Path -Parent $PSScriptRoot
[System.IO.FileInfo[]] $Sources = @(
    Get-ChildItem -LiteralPath (Join-Path $RepoRoot 'crates') -Recurse -File |
        Where-Object { $_.Extension -in @('.rs', '.wgsl') }
)

[Microsoft.PowerShell.Commands.MatchInfo[]] $Findings = @(
    $Sources | Select-String -CaseSensitive -Pattern '\b(?:Mat(?:[2-4]|[A-Z])[A-Za-z0-9_]*|mat[2-4]x[2-4])\b'
)

if ($Findings.Count -gt 0) {
    $Findings | ForEach-Object { Write-Error $_.ToString() }
    exit 1
}

Write-Output "matrix-ban: files=$($Sources.Count) findings=0"
