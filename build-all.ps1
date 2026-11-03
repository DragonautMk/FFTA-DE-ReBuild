param(
    [string]$AndroidSdk = $env:ANDROID_SDK_ROOT,
    [switch]$SkipAndroid,
    [switch]$SkipLinux
)

# Android build tools are developer prerequisites, not part of the repository/payload.
# Install Android SDK + NDK 28.2.13676358, a JDK, Python, Rust's
# aarch64-linux-android target and a host GCC/Clang. Set JAVA_HOME/PATH as needed.
$ErrorActionPreference = 'Stop'
Set-Location $PSScriptRoot
if (-not $SkipAndroid) {
    if (-not $AndroidSdk) { $AndroidSdk = $env:ANDROID_HOME }
    if (-not $AndroidSdk -or -not (Test-Path -LiteralPath ([IO.Path]::Combine($AndroidSdk, 'ndk/28.2.13676358/toolchains/llvm/prebuilt/windows-x86_64/bin')))) {
        throw 'Android SDK/NDK is required. Install NDK 28.2.13676358 and set ANDROID_SDK_ROOT or pass -AndroidSdk PATH. Also required: JDK, Python, GCC/Clang in PATH and rustup target add aarch64-linux-android. These tools are not bundled. Use -SkipAndroid for desktop builds only.'
    }
}

Write-Host '==> [1/3] Building Windows runtime, TinyCC and portable package...' -ForegroundColor Cyan
$oldErrorAction = $ErrorActionPreference
$ErrorActionPreference = 'Continue'
try {
    & python (Join-Path $PSScriptRoot 'scripts\build_desktop.py') --platform windows
    $buildExit = $LASTEXITCODE
} finally {
    $ErrorActionPreference = $oldErrorAction
}
if ($buildExit -ne 0) { throw "Windows build/package failed (exit $buildExit)" }

if (-not $SkipLinux) {
    Write-Host '==> [2/3] Building Linux x86_64 and Steam Deck installers...' -ForegroundColor Cyan
    $childPowerShell = (Get-Process -Id $PID).Path
    & $childPowerShell -NoProfile -ExecutionPolicy Bypass -File (Join-Path $PSScriptRoot 'scripts\windows\build-linux-windows.ps1') -OutputDir (Join-Path $PSScriptRoot 'Installer\Linux')
    if ($LASTEXITCODE -ne 0) { throw 'Linux installer build failed' }
} else {
    Write-Host '==> [2/3] Linux skipped (-SkipLinux).'
}
if (-not $SkipAndroid) {
    Write-Host '==> [3/3] Building Android ARM64 APK (external SDK/NDK and JDK required)...' -ForegroundColor Cyan
    $previousTargetDir = $env:CARGO_TARGET_DIR
    $oldErrorAction = $ErrorActionPreference
    try {
        # AndroidPort's builder expects its own target/ directory.
        Remove-Item Env:\CARGO_TARGET_DIR -ErrorAction SilentlyContinue
        $ErrorActionPreference = 'Continue'
        & python (Join-Path $PSScriptRoot 'AndroidPort\scripts\build_android.py') --sdk $AndroidSdk
        $androidExit = $LASTEXITCODE
    } finally {
        $ErrorActionPreference = $oldErrorAction
        if ($null -eq $previousTargetDir) { Remove-Item Env:\CARGO_TARGET_DIR -ErrorAction SilentlyContinue }
        else { $env:CARGO_TARGET_DIR = $previousTargetDir }
    }
    if ($androidExit -ne 0) { throw 'Android APK build failed' }
    $androidOutput = Join-Path $PSScriptRoot 'Installer\Android'
    New-Item -ItemType Directory -Force $androidOutput | Out-Null
    Copy-Item -LiteralPath (Join-Path $PSScriptRoot 'AndroidPort\APK\FFTARecomp-android-arm64-preview.apk') -Destination $androidOutput -Force
    Copy-Item -LiteralPath (Join-Path $PSScriptRoot 'AndroidPort\APK\SHA256.txt') -Destination $androidOutput -Force
} else {
    Write-Host '==> [3/3] Android skipped (-SkipAndroid).'
}
Write-Host "Windows package: $PSScriptRoot\dist\FFTARecomp-windows-portable.zip" -ForegroundColor Green
if (-not $SkipLinux) { Write-Host "Linux/Steam Deck installers: $PSScriptRoot\Installer\Linux" }
if (-not $SkipAndroid) { Write-Host "Android APK: $PSScriptRoot\Installer\Android" }
