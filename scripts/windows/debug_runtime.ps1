# Windows PowerShell 5.1+: runtime commands resolve from the portable folder.
param()
[string[]]$ToolArguments = $args
$ErrorActionPreference = 'Stop'
$runtimeRoot = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../..'))
$logs = Join-Path $runtimeRoot 'Data/config/debug-logs'
[IO.Directory]::CreateDirectory($logs) | Out-Null
$log = Join-Path $logs ('runtime-' + [DateTime]::UtcNow.ToString('yyyyMMdd-HHmmss') + '-' + [Guid]::NewGuid().ToString('N') + '.log')
Write-Host "Runtime log: $log"
Push-Location -LiteralPath $runtimeRoot
try {
    $runtime = Join-Path $runtimeRoot 'recomp.exe'
    if (-not (Test-Path -LiteralPath $runtime -PathType Leaf)) { throw 'Missing portable recomp.exe.' }
    if ($ToolArguments.Count -eq 0) { $ToolArguments = @('help') }
    # Native stderr is diagnostic output; preserve it even on successful commands.
    $ErrorActionPreference = 'Continue'
    & $runtime @ToolArguments *> $log
    $code = $LASTEXITCODE
    $ErrorActionPreference = 'Stop'
    if ($null -eq $code) { $code = 1 }
    Get-Content -LiteralPath $log | Write-Output
    exit $code
} catch {
    $_ | Out-String | Add-Content -LiteralPath $log
    Write-Error "Runtime command failed; log: $log"
    exit 1
} finally { Pop-Location }
