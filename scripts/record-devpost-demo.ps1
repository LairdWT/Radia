#Requires -Version 7.4

[CmdletBinding()]
param(
    [Parameter(Mandatory)]
    [ValidateSet('SaveCredential', 'Setup', 'Narration', 'DryRun', 'Rehearse', 'Record', 'Validate')]
    [string] $Action,
    [string] $NarrationPath = '',
    [string] $VideoPath = '',
    [string] $OutputDirectory = '',
    [uri] $ObsUri = 'ws://127.0.0.1:4455',
    [pscredential] $ObsCredential
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$modulePath = Join-Path -Path $PSScriptRoot -ChildPath 'record-devpost-demo.psm1'
Import-Module -Name $modulePath -Force

$arguments = @{
    Action = $Action
    NarrationPath = $NarrationPath
    VideoPath = $VideoPath
    OutputDirectory = $OutputDirectory
    ObsUri = $ObsUri
    ObsCredential = $ObsCredential
}
Invoke-RadiaDemoVideo @arguments
