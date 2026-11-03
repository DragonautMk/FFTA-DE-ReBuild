$ErrorActionPreference = 'Stop'
$repo = (Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
Set-Location -LiteralPath $repo

Write-Host 'Simulating Xbox A (Gilrs South) during gamepad remapping.'
Write-Host 'The test feeds ButtonPressed(South) and ButtonChanged(South, 1.0),'
Write-Host 'then verifies one assignment and zero menu-confirm actions.'
cargo test -p recomp simulated_xbox_a_remap_is_single_capture_and_never_reconfirms -- --nocapture
if ($LASTEXITCODE -ne 0) {
    throw "Gamepad remap simulation failed with exit code $LASTEXITCODE"
}
