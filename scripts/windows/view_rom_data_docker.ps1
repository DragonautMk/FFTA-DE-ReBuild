# Windows PowerShell 5.1+ with Docker Desktop using Linux containers.
[CmdletBinding()]
param(
    [Parameter(Mandatory = $true, Position = 0)]
    [string]$Dataset,
    [Parameter(Position = 1)]
    [string]$OutputDir
)
$ErrorActionPreference = 'Stop'
$repoDir = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '../..')).Path
if (-not (Test-Path -LiteralPath (Join-Path $Dataset 'manifest.toml') -PathType Leaf)) {
    throw "Dataset manifest not found in: $Dataset"
}
$datasetDir = (Resolve-Path -LiteralPath $Dataset).Path.TrimEnd('\', '/')
if (-not $OutputDir) {
    $OutputDir = $repoDir
    $outputName = 'rom_data_viewer_generated.html'
} else {
    $outputName = 'index.html'
}
$outputPath = $ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($OutputDir).TrimEnd('\', '/')
foreach ($value in @($repoDir, $datasetDir, $outputPath)) {
    if ($value.Contains(',') -or $value.Contains("`n") -or $value.Contains("`r")) {
        throw 'Paths must not contain commas or newlines.'
    }
}
if ($outputPath.Equals($datasetDir, [StringComparison]::OrdinalIgnoreCase) -or
    $outputPath.StartsWith($datasetDir + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)) {
    throw 'Output must be outside the dataset.'
}
if (-not (Get-Command docker -CommandType Application -ErrorAction SilentlyContinue)) {
    throw 'Docker Desktop using Linux containers is required.'
}
[IO.Directory]::CreateDirectory($outputPath) | Out-Null
$resolvedOutput = (Resolve-Path -LiteralPath $outputPath).Path
if ($resolvedOutput.Equals($datasetDir, [StringComparison]::OrdinalIgnoreCase) -or
    $resolvedOutput.StartsWith($datasetDir + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)) {
    throw 'Output must be outside the dataset.'
}
$runArgs = @(
    'run', '--rm', '--network', 'none',
    '--mount', "type=bind,source=$(Join-Path $repoDir 'scripts'),target=/viewer/scripts,readonly",
    '--mount', "type=bind,source=$(Join-Path $repoDir 'view_rom_data.html'),target=/viewer/view_rom_data.html,readonly",
    '--mount', "type=bind,source=$datasetDir,target=/dataset,readonly",
    '--mount', "type=bind,source=$resolvedOutput,target=/output",
    'python:3.12-alpine', 'python3', '/viewer/scripts/view_rom_data.py', '/dataset', '/output', '--filename', $outputName
)
& docker @runArgs
if ($LASTEXITCODE -ne 0) { throw "Docker viewer generation failed (exit $LASTEXITCODE)." }
Write-Host "Viewer ready: $(Join-Path $resolvedOutput $outputName)"
