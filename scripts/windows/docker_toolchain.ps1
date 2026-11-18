# Return the Dockerfile arguments for the selected builder.
function Get-ToolchainArguments([string]$Platform, [string]$Recipe, [string]$Context, [switch]$Export) {
    $builder = & docker buildx inspect
    if ($LASTEXITCODE -ne 0) { throw 'Cannot inspect the selected Docker builder.' }
    if (-not ($builder -match '^Driver:\s+docker\s*$')) { return }
    $source = [IO.File]::ReadAllText($Recipe).Replace("`r`n", "`n")
    $end = $source.IndexOf("# End of cached toolchain`n", [StringComparison]::Ordinal)
    if ($end -lt 0) { throw "Missing toolchain boundary in $Recipe" }
    $sha = [Security.Cryptography.SHA256]::Create()
    try { $hash = $sha.ComputeHash([Text.Encoding]::UTF8.GetBytes($source.Substring(0, $end))) }
    finally { $sha.Dispose() }
    $digest = [BitConverter]::ToString($hash).Replace('-', '').Substring(0, 20).ToLowerInvariant()
    $image = "ffta-toolchain-${Platform}:$digest"
    # Missing images are expected on a new engine. Avoid PS 5.1 stderr redirection errors.
    $found = & docker image ls --quiet --no-trunc $image
    if ($LASTEXITCODE -ne 0) { throw "Cannot inspect local images for $image" }
    if (-not $found) {
        & docker buildx build --platform linux/amd64 --target toolchain --file $Recipe --output 'type=image,unpack=false' --tag $image $Context | Out-Host
        if ($LASTEXITCODE -ne 0) { throw "Cannot prepare $image" }
    }
    '--build-arg'
    "FFTA_TOOLCHAIN=$image"
    if ($Export) {
        $imageId = & docker image inspect --format '{{.Id}}' $image
        if ($LASTEXITCODE -ne 0) { throw "Cannot read the image identity for $image" }
        '--build-arg'
        "FFTA_TOOLCHAIN_ID=$imageId"
    }
}
