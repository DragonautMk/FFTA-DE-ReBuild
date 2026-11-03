# Python 3.11+ or the existing Docker coverage image. Paths are repo-relative.
param()
[string[]]$ToolArguments = $args
$Docker = $false
if ($ToolArguments.Count -gt 0 -and $ToolArguments[0] -eq '-Docker') {
    $Docker = $true
    $ToolArguments = @($ToolArguments | Select-Object -Skip 1)
}
$ErrorActionPreference = 'Stop'
$repo = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../..'))
$logs = Join-Path $repo 'target/build-logs'
[IO.Directory]::CreateDirectory($logs) | Out-Null
$log = Join-Path $logs ('debug-campaign-' + [DateTime]::UtcNow.ToString('yyyyMMdd-HHmmss') + '-' + [Guid]::NewGuid().ToString('N') + '.log')
Write-Host "Campaign log: $log"
Push-Location -LiteralPath $repo
try {
    if ($Docker) {
        if ($ToolArguments.Count -eq 0 -or $ToolArguments[0] -notin @('report', '--help')) { throw 'Docker supports report only; dispatch uses host Python.' }
        if ($repo.IndexOfAny([char[]]",`"`r`n") -ge 0) { throw 'Unsupported Docker mount path; use Python.' }
        & docker image inspect ffta-debug-tools:local *> $log
        if ($LASTEXITCODE -ne 0) { throw 'Existing ffta-debug-tools:local image required. No build started.' }
        & docker run --rm --network none --read-only --tmpfs /tmp --workdir /project --entrypoint python3 --mount "type=bind,source=$repo,target=/project" ffta-debug-tools:local /project/scripts/debug_campaign.py @ToolArguments *>> $log
    } else {
        & python3 (Join-Path $repo 'scripts/debug_campaign.py') @ToolArguments *> $log
    }
    $code = $LASTEXITCODE
    Get-Content -LiteralPath $log | Write-Output
    exit $code
} catch {
    $_ | Out-String | Add-Content -LiteralPath $log
    Write-Error "Campaign tool failed; log: $log"
    exit 1
} finally { Pop-Location }
