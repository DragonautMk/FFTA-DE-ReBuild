# PUBLIC fixtures only. Linux PowerShell 7 container: mount scripts/windows at
# /input readonly, this file at /check.ps1 readonly and an empty results dir at
# /output. Use network none, readonly root, /tmp:exec and XDG_CACHE_HOME,
# XDG_CONFIG_HOME, XDG_DATA_HOME, XDG_STATE_HOME beneath /tmp.
# The fake native application is the image's Bash, never a game runtime.
$ErrorActionPreference = 'Stop'
$root = "/tmp/portable tools ' à"
$windows = Join-Path $root 'scripts/windows'
[IO.Directory]::CreateDirectory($windows) | Out-Null
foreach ($name in @('debug_runtime','report_home','debug_artifacts','debug_campaign')) {
    Copy-Item -LiteralPath "/input/$name.ps1" -Destination "$windows/$name.ps1"
}
$bin = '/tmp/public mock bin'
[IO.Directory]::CreateDirectory($bin) | Out-Null
$env:PATH = $bin + ':' + $env:PATH
$env:FIXTURE_ROOT = $root
$stub = @'
#!/bin/bash
printf '%s\0' "$@" > "$FIXTURE_ROOT/python.args"
if [[ "$1" == */report_home.py ]]; then
 if [[ "${TEST_PYTHON_EXIT:-0}" != 0 ]]; then printf 'PUBLIC PYTHON FAILURE\n' >&2; exit "$TEST_PYTHON_EXIT"; fi
 printf '<html>PUBLIC HOST PAGE</html>\n' > "$PWD/index.html"
fi
printf 'PUBLIC PYTHON OK\n'
exit "${TEST_PYTHON_EXIT:-0}"
'@
[IO.File]::WriteAllText("$bin/python3", $stub)
$stub = @'
#!/bin/bash
printf '%s\0' "$@" > "$FIXTURE_ROOT/docker.args"
if [[ "$1" == image ]]; then exit "${TEST_DOCKER_IMAGE_EXIT:-0}"; fi
if [[ "${TEST_DOCKER_RUN_EXIT:-0}" != 0 ]]; then printf 'PUBLIC DOCKER FAILURE\n' >&2; printf '<html>INCOMPLETE</html>\n'; exit "$TEST_DOCKER_RUN_EXIT"; fi
printf '<html>PUBLIC DOCKER PAGE</html>\n'
'@
[IO.File]::WriteAllText("$bin/docker", $stub)
$stub = @'
#!/bin/bash
pwd > "$FIXTURE_ROOT/runtime.cwd"
printf '%s\0' "$@" > "$FIXTURE_ROOT/runtime.args"
printf 'PUBLIC RUNTIME OK\n'
printf 'PUBLIC RUNTIME STDERR\n' >&2
exit "${TEST_RUNTIME_EXIT:-0}"
'@
[IO.File]::WriteAllText("$root/runtime-fixture.sh", $stub)
Copy-Item -LiteralPath /bin/bash -Destination "$root/recomp.exe"
& /bin/chmod +x "$bin/python3" "$bin/docker" "$root/recomp.exe"

$checks = [Collections.Generic.List[string]]::new()
$pwsh = (Get-Process -Id $PID).Path
function Assert($condition, $message) { if (-not $condition) { throw $message } }
function Run($name, [string[]]$arguments, $expected) {
    if ($name -eq 'debug_runtime' -and (Test-Path -LiteralPath "$root/recomp.exe")) {
        $arguments = @("$root/runtime-fixture.sh") + $arguments
    }
    $output = & $pwsh -NoLogo -NoProfile -File "$windows/$name.ps1" @arguments 2>&1 | Out-String
    $code = $LASTEXITCODE
    [IO.File]::WriteAllText("/output/$($checks.Count)-$name.stdout", $output)
    foreach ($folder in @("$root/Data/config/debug-logs", "$root/target/build-logs")) {
        if (Test-Path -LiteralPath $folder) {
            foreach ($log in Get-ChildItem -LiteralPath $folder -Filter '*.log') {
                Copy-Item -LiteralPath $log.FullName -Destination (Join-Path '/output' $log.Name) -Force
            }
        }
    }
    Assert ($code -eq $expected) "$name expected$expected got$code : $output"
    return $output
}

Push-Location /tmp
try {
    $text = Run 'debug_runtime' @('help','debug-control','a path with spaces', "quote ' à") 0
    $argsRead = [Text.Encoding]::UTF8.GetString([IO.File]::ReadAllBytes("$root/runtime.args")).Split([char]0)
    Assert ($argsRead[0] -eq 'help' -and $argsRead[2] -eq 'a path with spaces' -and $argsRead[3] -eq "quote ' à") 'Runtime argv corruption'
    Assert (([IO.File]::ReadAllText("$root/runtime.cwd")).Trim() -eq $root) 'Runtime cwd not portable'
    $runtimeLogs = @(Get-ChildItem -LiteralPath "$root/Data/config/debug-logs" -Filter '*.log')
    Assert ($runtimeLogs.Count -eq 1 -and ([IO.File]::ReadAllText($runtimeLogs[0].FullName)).Contains('PUBLIC RUNTIME STDERR')) 'Successful stderr log lost'
    $checks.Add('Runtime root/cwd/spaced Unicode arguments/stderr')
    Run 'debug_runtime' @('diagnose','rom with spaces','--out',"Data/diagnosis ' à.json",'--json') 0 | Out-Null
    $argsRead = [Text.Encoding]::UTF8.GetString([IO.File]::ReadAllBytes("$root/runtime.args")).Split([char]0)
    Assert ($argsRead[2] -eq '--out' -and $argsRead[3] -eq "Data/diagnosis ' à.json" -and $argsRead[4] -eq '--json') 'Runtime named option interception'
    $checks.Add('Runtime --out/--json forwarded without common-parameter interception')
    $env:TEST_RUNTIME_EXIT='7'
    Run 'debug_runtime' @('debug-control','status','case with spaces') 7 | Out-Null
    Remove-Item Env:TEST_RUNTIME_EXIT
    $checks.Add('Runtime nonzero exit/log')
    Move-Item -LiteralPath "$root/recomp.exe" -Destination "$root/recomp.backup"
    Run 'debug_runtime' @('help') 1 | Out-Null
    Move-Item -LiteralPath "$root/recomp.backup" -Destination "$root/recomp.exe"
    $checks.Add('Missing runtime logged')

    Run 'report_home' @('-Python') 0 | Out-Null
    Assert (([IO.File]::ReadAllText("$root/index.html")).Contains('PUBLIC HOST PAGE')) 'Host homepage not portable'
    $checks.Add('Home Python mode root/log')
    $old = [IO.File]::ReadAllText("$root/index.html")
    $env:TEST_PYTHON_EXIT = '5'
    Run 'report_home' @('-Python') 1 | Out-Null
    Assert ([IO.File]::ReadAllText("$root/index.html") -eq $old) 'Python error replaced old page'
    $checks.Add('Home Python failure preserves page and log')
    Remove-Item Env:TEST_PYTHON_EXIT
    Run 'report_home' @() 0 | Out-Null
    Assert (([IO.File]::ReadAllText("$root/index.html")).Contains('PUBLIC DOCKER PAGE')) 'Docker page not published'
    $dockerArgs = [Text.Encoding]::UTF8.GetString([IO.File]::ReadAllBytes("$root/docker.args"))
    Assert ($dockerArgs.Contains('--network') -and $dockerArgs.Contains('none') -and $dockerArgs.Contains('readonly')) 'Home readonly/network flags missing'
    $checks.Add('Home Docker atomic publication/readonly/network/log')
    $old = [IO.File]::ReadAllText("$root/index.html")
    $env:TEST_DOCKER_RUN_EXIT = '6'
    Run 'report_home' @() 1 | Out-Null
    Assert ([IO.File]::ReadAllText("$root/index.html") -eq $old) 'Docker error replaced old page'
    Assert (@(Get-ChildItem -LiteralPath $root -Filter '.report-home-*.tmp').Count -eq 0) 'Temporary page retained'
    $checks.Add('Home Docker failure preserves page/log/removes temporary')
    Remove-Item Env:TEST_DOCKER_RUN_EXIT
    Run 'debug_artifacts' @('preview','--session', "Data/case ' à",'--out','Data/new preview') 0 | Out-Null
    $pythonArgs = [Text.Encoding]::UTF8.GetString([IO.File]::ReadAllBytes("$root/python.args")).Split([char]0)
    Assert ($pythonArgs[1] -eq 'preview' -and $pythonArgs[2] -eq '--session' -and $pythonArgs[3] -eq "Data/case ' à") 'Artifact forwarding failed'
    $checks.Add('Artifact host named flags/spaced paths/log')
    Run 'debug_campaign' @('report','Data/campaign.json','--case','guide-A','--out','Data/report') 0 | Out-Null
    $checks.Add('Campaign host named flags/log')
    Run 'debug_campaign' @('-Docker','report','Data/campaign.json','--case','guide-A','--out','Data/report') 0 | Out-Null
    $dockerArgs = [Text.Encoding]::UTF8.GetString([IO.File]::ReadAllBytes("$root/docker.args"))
    Assert ($dockerArgs.Contains('debug_campaign.py') -and $dockerArgs.Contains('--read-only') -and $dockerArgs.Contains('none')) 'Campaign Docker args lost'
    $checks.Add('Campaign Docker report/network/readonly/log')
    Run 'debug_campaign' @('-Docker','run-case','Data/campaign.json','guide-A','--reason','test','--out','Data/intent') 1 | Out-Null
    $checks.Add('Campaign Docker dispatch refusal logged')
    Run 'debug_artifacts' @('-Docker','preview','--session','Data/case','--out','Data/preview') 0 | Out-Null
    $checks.Add('Artifact Docker forwarding/log')
} finally { Pop-Location }
$report = @{passed=$checks.Count; asserted_this_run=$checks.Count; checks=$checks; powershell=$PSVersionTable.PSVersion.ToString(); native_windows=$false; public_fixtures_only=$true; scope='Complete standalone public-fixture launcher validation in Linux PowerShell; no native Windows or game execution'}
[IO.File]::WriteAllText('/output/powershell-results.json', ($report | ConvertTo-Json -Depth 6))
Write-Output ($report | ConvertTo-Json -Depth 6)
