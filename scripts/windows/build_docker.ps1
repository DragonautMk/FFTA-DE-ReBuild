# Requires Docker Desktop using Linux containers; Windows PowerShell 5.1+.
[CmdletBinding()]
param(
    [ValidateSet('windows', 'linux', 'android', 'all')]
    [string]$Platform = 'windows',
    [string]$OutputDir
)
$ErrorActionPreference = 'Stop'
$Platform = $Platform.ToLowerInvariant()
$repoDir = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '../..')).Path
. (Join-Path $PSScriptRoot 'docker_toolchain.ps1')

function Find-AndroidPython {
    foreach ($name in @('py', 'python3', 'python')) {
        $app = Get-Command $name -CommandType Application -ErrorAction SilentlyContinue | Select-Object -First 1
        if (-not $app) { continue }
        $prefix = @()
        if ($name -eq 'py') { $prefix = @('-3') }
        & $app.Source @prefix -c 'import sys; sys.exit(sys.version_info < (3, 9))'
        if ($LASTEXITCODE -eq 0) { return $app }
    }
    throw 'The Android Docker launcher requires Python 3.9+ on the host.'
}

function Build-Desktop([string]$Target, [string]$Destination) {
    [IO.Directory]::CreateDirectory($Destination) | Out-Null
    $recipe = Join-Path $repoDir "Dockerfile.$Target"
    $toolchainArgs = @(Get-ToolchainArguments $Target $recipe $repoDir -Export)
    & docker buildx build @toolchainArgs --platform linux/amd64 --file $recipe --target export `
        --output "type=local,dest=$Destination" --progress plain $repoDir
    if ($LASTEXITCODE -ne 0) { throw "Docker $Target build failed (exit $LASTEXITCODE)." }
    Write-Host "Portable archive and BUILD-INFO.txt exported to $Destination"
}

try {
    if (-not (Get-Command docker -CommandType Application -ErrorAction SilentlyContinue)) {
        throw 'Docker with Buildx is required. On Windows, start Docker Desktop using Linux containers.'
    }
    $targets = @($Platform)
    if ($Platform -eq 'all') { $targets = @('windows', 'linux', 'android') }
    if (-not $OutputDir) {
        $OutputDir = Join-Path $repoDir 'dist'
        if ($Platform -ne 'all') { $OutputDir = Join-Path $OutputDir "docker-$Platform" }
    }
    $baseDir = $ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($OutputDir)
    if ($baseDir.IndexOfAny([char[]]",`"`n`r") -ge 0) {
        throw 'The output directory must not contain commas, double quotes or newlines.'
    }
    if ($targets -contains 'android') { $python = Find-AndroidPython }
    foreach ($target in $targets) {
        $destination = $baseDir
        if ($Platform -eq 'all') { $destination = Join-Path $baseDir "docker-$target" }
        Write-Host "Building $target with Docker..."
        if ($target -eq 'android') {
            $androidArgs = @((Join-Path $repoDir 'scripts/android/build_android_docker.py'), '--output', $destination)
            if ($python.Name -in @('py', 'py.exe')) { $androidArgs = @('-3') + $androidArgs }
            & $python.Source @androidArgs
            if ($LASTEXITCODE -ne 0) { throw "Docker Android build failed (exit $LASTEXITCODE)." }
        } else {
            Build-Desktop $target $destination
        }
    }
} catch {
    if (-not [Console]::IsInputRedirected -and (-not [Console]::IsErrorRedirected -or -not [Console]::IsOutputRedirected) -and [Environment]::UserInteractive) {
        $terminal = if ([Console]::IsErrorRedirected) { [Console]::Out } else { [Console]::Error }
        $terminal.WriteLine("ERROR: $($_.Exception.Message)")
        $terminal.Write('Press Enter to close this window... ')
        [void][Console]::ReadLine()
    }
    throw
}
