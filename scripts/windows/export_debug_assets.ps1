# Windows PowerShell 5.1+: a private local ZIP, never the distributable release.
$ErrorActionPreference = 'Stop'
$script = Join-Path $PSScriptRoot '../export_debug_assets.py'
if (Get-Command py -CommandType Application -ErrorAction SilentlyContinue) {
    & py -3 $script @args
} elseif (Get-Command python3 -CommandType Application -ErrorAction SilentlyContinue) {
    & python3 $script @args
} elseif (Get-Command python -CommandType Application -ErrorAction SilentlyContinue) {
    & python $script @args
} else { throw 'Python 3 is required for the private debug assets export.' }
exit $LASTEXITCODE
