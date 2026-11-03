# Windows PowerShell 5.1+; Docker Desktop must use Linux containers.
[CmdletBinding()]
param(
    [Parameter(Position = 0)]
    [string]$RomPath = 'rom/1194 - Final Fantasy Tactics Advance (E)(Surplus).gba',
    [Parameter(Position = 1)]
    [string]$OutputDir
)
$ErrorActionPreference = 'Stop'
$repoDir = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '../..')).Path
if (-not [System.IO.Path]::IsPathRooted($RomPath)) {
    $RomPath = Join-Path $repoDir $RomPath
}
if (-not (Test-Path -LiteralPath $RomPath -PathType Leaf)) {
    throw "ROM file does not exist: $RomPath. Pass a different path with -RomPath."
}
$romFile = (Resolve-Path -LiteralPath $RomPath).Path
if (-not $OutputDir) {
    $stem = [System.IO.Path]::GetFileNameWithoutExtension($romFile)
    $OutputDir = Join-Path $repoDir "target/rom-data/$stem"
}
$datasetDir = $ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($OutputDir)
$datasetDir = $datasetDir.TrimEnd([System.IO.Path]::DirectorySeparatorChar)
foreach ($value in @($romFile, $datasetDir)) {
    if ($value.Contains(',') -or $value.Contains("`n") -or $value.Contains("`r")) {
        throw 'ROM/output paths must not contain commas or newlines.'
    }
}
$outputParent = [System.IO.Path]::GetDirectoryName($datasetDir)
$outputName = [System.IO.Path]::GetFileName($datasetDir)
if (-not $outputParent -or -not $outputName) {
    throw 'Choose a named dataset directory, not a filesystem root.'
}
if (-not (Get-Command docker -CommandType Application -ErrorAction SilentlyContinue)) {
    throw 'Docker Desktop with Buildx is required. Start Docker Desktop using Linux containers.'
}
# Mount the parent so the extractor can publish the dataset by atomic rename.
[System.IO.Directory]::CreateDirectory($outputParent) | Out-Null
$image = 'ffta-rom-data-extractor:local'
$buildArgs = @(
    'buildx', 'build', '--platform', 'linux/amd64',
    '--file', (Join-Path $repoDir 'Dockerfile.linux'), '--target', 'extract',
    '--load', '--tag', $image, '--progress', 'plain', $repoDir
)
& docker @buildArgs
if ($LASTEXITCODE -ne 0) { throw "Docker extraction image build failed (exit $LASTEXITCODE)." }
$runArgs = @(
    'run', '--rm', '--platform', 'linux/amd64', '--network', 'none',
    '--mount', "type=bind,source=$romFile,target=/input/source.gba,readonly",
    '--mount', "type=bind,source=$outputParent,target=/output",
    $image, '/input/source.gba', '--output', "/output/$outputName"
)
& docker @runArgs
if ($LASTEXITCODE -ne 0) { throw "Docker data extraction failed (exit $LASTEXITCODE)." }
Write-Host "Extracted dataset on host: $datasetDir"
