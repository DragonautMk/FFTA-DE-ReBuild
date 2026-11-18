#!/usr/bin/env bash
# BuildKit invokes this shell for the desktop build/test/export RUN. Its target
# cache is exclusively locked throughout verification, compilation and publication.
set -euo pipefail
platform=$1
artifacts=$2
shift 2
if [[ ! ${FFTA_TOOLCHAIN_ID:-} =~ ^sha256:[a-f0-9]{64}$ ]]; then
    exec "$@"
fi
case "$platform" in
    linux) archive=FFTARecomp-linux-portable.tar.gz ;;
    windows) archive=FFTARecomp-windows-portable.zip ;;
    *) exit 2 ;;
esac
cache="target/.ffta-verified-export-$platform"
inputs=$(dirname -- "${BASH_SOURCE[0]}")
# Hash everything the Dockerfile copied, including paths, modes, link targets,
# the build recipe and this shell. Ignore the writable Cargo cache and mtimes.
key=$({
    printf '%s\0' "$FFTA_TOOLCHAIN_ID"
    (cd "$inputs" && sha256sum -- "Dockerfile.$platform" docker_cached_export.sh)
    tar --sort=name --format=gnu --mtime=@0 --owner=0 --group=0 \
        --exclude=./target -cf - .
} | sha256sum)
if [[ -f "$cache/key" && $(<"$cache/key") == "$key" ]] &&
    (cd "$cache" && sha256sum -- "$archive" BUILD-INFO.txt | cmp -s checksums -); then
    mkdir -p "$artifacts"
    cp -- "$cache/$archive" "$cache/BUILD-INFO.txt" "$artifacts/"
    echo 'Reused the verified desktop export for these build inputs.'
    exit 0
fi
"$@"
# Publish the key last. An interrupted write becomes a miss on the next build.
mkdir -p "$cache"
rm -f -- "$cache/key"
cp -- "$artifacts/$archive" "$artifacts/BUILD-INFO.txt" "$cache/"
(cd "$cache" && sha256sum -- "$archive" BUILD-INFO.txt > checksums)
printf '%s\n' "$key" > "$cache/key"
