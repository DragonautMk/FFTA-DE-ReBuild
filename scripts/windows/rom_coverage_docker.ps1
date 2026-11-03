# Windows PowerShell 5.1+; Docker Desktop must use Linux containers.
[CmdletBinding()]
param(
    [string]$RomPath = 'rom/1194 - Final Fantasy Tactics Advance (E)(Surplus).gba',
    [string]$DatasetDir,
    [string]$TranslationPath,
    [string[]]$AuditPath = @(),
    [string]$ComparePath,
    [string]$OutputDir = 'docs/development/rom-coverage',
    [string]$EvidencePath = 'docs/development/rom-coverage/evidence.json',
    [switch]$OpenReport
)
$ErrorActionPreference = 'Stop'
$repoDir = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '../..')).Path
function Get-ProjectPath([string]$Value) {
    if ([string]::IsNullOrWhiteSpace($Value)) { throw 'Paths must not be empty.' }
    if (-not [System.IO.Path]::IsPathRooted($Value)) { $Value = Join-Path $repoDir $Value }
    $Value = $ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($Value)
    if ($Value.Contains(',') -or $Value.Contains('"') -or $Value.Contains("`n") -or $Value.Contains("`r")) {
        throw 'Paths must not contain commas, double quotes or newlines.'
    }
    return $Value
}
$logDir = Join-Path $repoDir 'target/build-logs'
[IO.Directory]::CreateDirectory($logDir) | Out-Null
$logFile = Join-Path $logDir ('rom-coverage-{0}-{1}.log' -f (Get-Date -Format 'yyyyMMdd-HHmmss'), ([Guid]::NewGuid().ToString('N').Substring(0,8)))
Write-Host "ROM coverage log: $logFile"
Start-Transcript -LiteralPath $logFile | Out-Null
$contextDir = $null
try {
$romFile = Get-ProjectPath $RomPath
$dataset = if ($DatasetDir) { Get-ProjectPath $DatasetDir } else { $null }
$translation = if ($TranslationPath) { Get-ProjectPath $TranslationPath } else { $null }
$evidence = Get-ProjectPath $EvidencePath
$output = Get-ProjectPath $OutputDir
$audits = @($AuditPath | ForEach-Object { Get-ProjectPath $_ })
$comparison = if ($ComparePath) { Get-ProjectPath $ComparePath } else { $null }
$inputs = @($romFile, $evidence) + $audits
if ($comparison) { $inputs += $comparison }
if ($dataset) { $inputs += Join-Path $dataset 'manifest.toml' }
if ($translation) { $inputs += $translation }
foreach ($file in $inputs) {
    if (-not (Test-Path -LiteralPath $file -PathType Leaf)) { throw "Required input file does not exist: $file" }
}
if (-not (Get-Command docker -CommandType Application -ErrorAction SilentlyContinue)) {
    throw 'Docker Desktop with Buildx is required. Start Docker Desktop using Linux containers.'
}
if (-not (Get-Command git -CommandType Application -ErrorAction SilentlyContinue)) {
    throw 'Git is required to isolate committed extraction sources.'
}
[System.IO.Directory]::CreateDirectory($output) | Out-Null
$image = 'ffta-rom-coverage:local'
$sourceRevision = & git -C $repoDir rev-parse --verify HEAD
if ($LASTEXITCODE -ne 0) { throw 'Cannot resolve the committed extraction source.' }
$sourceRevision = ($sourceRevision -join '').Trim()
$contextDir = Join-Path $repoDir ('target/rom-coverage-context-' + [Guid]::NewGuid().ToString('N'))
[IO.Directory]::CreateDirectory($contextDir) | Out-Null
# Rust embeds public tool assets: archive scripts from the same commit, not WIP.
& git -C $repoDir archive --format=tar --output (Join-Path $contextDir 'runtime.tar') $sourceRevision Cargo.toml Cargo.lock crates assets TinyCC Licenses gamedb.sqlite scripts
if ($LASTEXITCODE -ne 0) { throw 'Cannot archive the committed extraction source.' }
foreach ($directory in @('scripts', 'docs/changes', 'docs/development/rom-coverage')) {
    [IO.Directory]::CreateDirectory((Join-Path $contextDir $directory)) | Out-Null
}
foreach ($name in @('rom_coverage.py', 'rom_coverage.html', 'local_artifacts.py', 'artifact_coordinator.py', 'rom_coverage_audit.py', 'rom_coverage_ranges.py')) {
    Copy-Item -LiteralPath (Join-Path $repoDir "scripts/$name") -Destination (Join-Path $contextDir 'scripts')
}
Get-ChildItem -LiteralPath (Join-Path $repoDir 'docs/changes') -Filter '*.json' -File |
    Copy-Item -Destination (Join-Path $contextDir 'docs/changes')
foreach ($name in @('evidence.json', 'code-seeds.json')) {
    Copy-Item -LiteralPath (Join-Path $repoDir "docs/development/rom-coverage/$name") -Destination (Join-Path $contextDir 'docs/development/rom-coverage')
}
$buildArgs = @(
    'buildx', 'build', '--platform', 'linux/amd64',
    '--file', (Join-Path $repoDir 'Dockerfile.coverage'), '--build-arg', "COVERAGE_SOURCE_REVISION=$sourceRevision",
    '--load', '--tag', $image, '--progress', 'plain', $contextDir
)
Write-Host "Building ROM coverage tools inside Docker from commit $sourceRevision..."
& docker @buildArgs
if ($LASTEXITCODE -ne 0) { throw "Docker coverage image build failed (exit $LASTEXITCODE)." }
$runArgs = @(
    'run', '--rm', '--platform', 'linux/amd64', '--network', 'none',
    '--mount', "type=bind,source=$romFile,target=/input/source.gba,readonly",
    '--mount', "type=bind,source=$evidence,target=/input/evidence.json,readonly",
    '--mount', "type=bind,source=$output,target=/output"
)
$generatorArgs = @('--rom', '/input/source.gba', '--evidence', '/input/evidence.json', '--output', '/output')
if ($dataset) {
    $runArgs += @('--mount', "type=bind,source=$dataset,target=/input/dataset,readonly")
    $generatorArgs += @('--dataset', '/input/dataset')
    $datasetInfo = Get-Item -LiteralPath $dataset
    if ($datasetInfo.Name -match '^e[0-9]+-d[0-9]+$' -and $datasetInfo.Parent.Name -match '^[0-9a-f]{64}$') {
        $leaseRoot = $datasetInfo.Parent.Parent.FullName
        $runArgs += @('--mount', "type=bind,source=$leaseRoot,target=/cache-lease")
        $generatorArgs += @('--cache-lease', '/cache-lease')
    }
} else {
    $cache = Get-ProjectPath 'target/rom-data/coverage'
    [IO.Directory]::CreateDirectory($cache) | Out-Null
    $runArgs += @('--mount', "type=bind,source=$cache,target=/cache")
    $generatorArgs += @('--dataset-cache', '/cache', '--cache-keep', '2')
}
if ($translation) {
    $runArgs += @('--mount', "type=bind,source=$translation,target=/input/translation.dll,readonly")
    $generatorArgs += @('--translation', '/input/translation.dll')
}
for ($i = 0; $i -lt $audits.Count; $i++) {
    $runArgs += @('--mount', "type=bind,source=$($audits[$i]),target=/input/audit-$i.json,readonly")
    $generatorArgs += @('--audit', "/input/audit-$i.json")
}
if ($comparison) {
    $runArgs += @('--mount', "type=bind,source=$comparison,target=/input/previous.json,readonly")
    $generatorArgs += @('--compare', '/input/previous.json')
}
$runArgs += @($image) + $generatorArgs
& docker @runArgs
if ($LASTEXITCODE -ne 0) { throw "Docker ROM coverage generation failed (exit $LASTEXITCODE)." }
foreach ($name in @('index.html', 'coverage.json')) {
    $file = Join-Path $output $name
    if (-not (Test-Path -LiteralPath $file -PathType Leaf) -or (Get-Item -LiteralPath $file).Length -eq 0) {
        throw "No report produced: $file"
    }
}
Write-Host "ROM coverage on host: $output/index.html"
Write-Host "JSON data on host: $output/coverage.json"
if ($OpenReport) {
    $reportFile = Get-Item -LiteralPath (Join-Path $output 'index.html')
    $reportUri = ([Uri]::new($reportFile.FullName, [UriKind]::Absolute)).AbsoluteUri + '?v=' + $reportFile.LastWriteTimeUtc.Ticks
    Start-Process -FilePath $reportUri
}
} catch {
    Write-Host "ROM coverage failed: $_"
    throw
} finally {
    if ($contextDir -and (Test-Path -LiteralPath $contextDir)) {
        Remove-Item -LiteralPath $contextDir -Recurse -Force
    }
    Stop-Transcript | Out-Null
    Write-Host "ROM coverage log: $logFile"
}
