# PowerShell 5.1+: offline homepage, persistent logs, no automatic Docker build.
[CmdletBinding()]
param([switch]$OpenReport, [switch]$Python, [string]$Image = 'ffta-rom-coverage:local')
$ErrorActionPreference = 'Stop'
$repoDir = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../..'))
$logs = Join-Path $repoDir 'target/build-logs'
[IO.Directory]::CreateDirectory($logs) | Out-Null
$log = Join-Path $logs ('report-home-' + [DateTime]::UtcNow.ToString('yyyyMMdd-HHmmss') + '-' + [Guid]::NewGuid().ToString('N') + '.log')
Write-Host "Homepage log: $log"
$temporary = Join-Path $repoDir ('.report-home-' + [Guid]::NewGuid().ToString('N') + '.tmp')
$previousEncoding = [Console]::OutputEncoding
Push-Location -LiteralPath $repoDir
try {
    if ($Python) {
        & python3 (Join-Path $repoDir 'scripts/report_home.py') *> $log
        if ($LASTEXITCODE -ne 0) { throw "Python homepage failed; previous page preserved." }
    } else {
        if ($repoDir.Contains(',') -or $repoDir.Contains('"') -or $repoDir.Contains("`n") -or $repoDir.Contains("`r")) {
            throw 'Docker mount path unsupported; use -Python.'
        }
        & docker image inspect $Image *> $log
        if ($LASTEXITCODE -ne 0) { throw "Existing image '$Image' required. No build started." }
        [Console]::OutputEncoding = [Text.UTF8Encoding]::new($false)
        $arguments = @('run', '--rm', '--network', 'none', '--entrypoint', 'python3',
            '--mount', "type=bind,source=$repoDir,target=/project,readonly", $Image,
            '/project/scripts/report_home.py', '--root', '/project', '--stdout')
        $page = & docker @arguments 2>> $log
        if ($LASTEXITCODE -ne 0) { throw 'Homepage failed; previous page preserved.' }
        [IO.File]::WriteAllText($temporary, (($page -join "`n") + "`n"), [Text.UTF8Encoding]::new($false))
        $destination = Join-Path $repoDir 'index.html'
        if (Test-Path -LiteralPath $destination) { [IO.File]::Replace($temporary, $destination, [NullString]::Value) }
        else { [IO.File]::Move($temporary, $destination) }
    }
    Get-Content -LiteralPath $log | Write-Output
    Write-Host "Homepage: $(Join-Path $repoDir 'index.html')"
    if ($OpenReport) { Start-Process -FilePath (Join-Path $repoDir 'index.html') }
} catch {
    $_ | Out-String | Add-Content -LiteralPath $log
    Write-Error "Homepage failed; log: $log"
    exit 1
} finally {
    [Console]::OutputEncoding = $previousEncoding
    if (Test-Path -LiteralPath $temporary) { Remove-Item -LiteralPath $temporary -Force }
    Pop-Location
}
