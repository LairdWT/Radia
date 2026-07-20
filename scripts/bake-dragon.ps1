[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'
$repositoryRoot = Split-Path -Parent $PSScriptRoot
$temporaryRoot = Join-Path $repositoryRoot "Temp\dragon-rebuild-$PID"
if (Test-Path -LiteralPath $temporaryRoot) {
    throw "Refusing to reuse existing temporary directory: $temporaryRoot"
}

$archive = Join-Path $temporaryRoot 'dragon.zip'
$expanded = Join-Path $temporaryRoot 'expanded'
$generated = Join-Path $temporaryRoot 'dragon-128.rduf'
$sourceUri = 'https://casual-effects.com/g3d/data10/research/model/dragon/dragon.zip'
$expectedArchive = '111124359A31E4D6B2EEB5398E5BC96A5D9E2D2A130AFB10BB4CFBC011BDB797'
$expectedArtifact = '9A8BABDACDAB6DBC3B8789B5008BBBAEE4C58C7FFEA42183ADA83397D5CB3862'

New-Item -ItemType Directory -Path $temporaryRoot | Out-Null
Invoke-WebRequest -UseBasicParsing $sourceUri -OutFile $archive
$archiveHash = (Get-FileHash -Algorithm SHA256 -LiteralPath $archive).Hash
if ($archiveHash -ne $expectedArchive) {
    throw "Dragon archive SHA-256 mismatch: $archiveHash"
}

Expand-Archive -LiteralPath $archive -DestinationPath $expanded
$sourceObj = Join-Path $expanded 'dragon.obj'
& cargo run -p radia-bake --release -- $sourceObj $generated 128
if ($LASTEXITCODE -ne 0) {
    throw "radia-bake failed with exit code $LASTEXITCODE"
}

$artifactHash = (Get-FileHash -Algorithm SHA256 -LiteralPath $generated).Hash
if ($artifactHash -ne $expectedArtifact) {
    throw "Dragon field SHA-256 mismatch: $artifactHash"
}

Write-Output "Verified deterministic dragon field: $generated"
Write-Output "SHA-256: $artifactHash"
