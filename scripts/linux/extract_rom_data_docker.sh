#!/usr/bin/env bash
# Build and run offline extraction with all dependencies inside Docker.
set -euo pipefail
repo_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)
default_rom='rom/1194 - Final Fantasy Tactics Advance (E)(Surplus).gba'
usage() {
    printf 'Usage: %s [ROM.gba] [dataset-directory]\n' "$0"
    printf 'ROM paths are relative to the project root; default: %s\n' "$default_rom"
}
if (( $# == 1 )) && [[ "$1" == --help || "$1" == -h ]]; then
    usage
    exit 0
fi
if (( $# > 2 )); then
    usage >&2
    exit 2
fi
rom_path=${1:-"$default_rom"}
if [[ "$rom_path" != /* ]]; then
    rom_path="$repo_dir/$rom_path"
fi
if [[ ! -f "$rom_path" ]]; then
    printf 'ROM file does not exist: %s\n' "$rom_path" >&2
    printf 'Pass a different ROM path as the first argument.\n' >&2
    exit 2
fi
rom_dir=$(cd -- "$(dirname -- "$rom_path")" && pwd)
rom_name=$(basename -- "$rom_path")
rom_path="$rom_dir/$rom_name"
output_dir=${2:-"$repo_dir/target/rom-data/${rom_name%.*}"}
# --mount uses CSV; reject unsupported path characters before invoking Docker.
for path in "$rom_path" "$output_dir"; do
    if [[ "$path" == *','* || "$path" == *$'\n'* || "$path" == *$'\r'* ]]; then
        printf 'ROM/output paths must not contain commas or newlines.\n' >&2
        exit 2
    fi
done
output_name=$(basename -- "$output_dir")
if [[ "$output_name" == . || "$output_name" == .. || "$output_name" == / ]]; then
    printf 'Choose a named dataset directory, not a filesystem root or dot path.\n' >&2
    exit 2
fi
command -v docker >/dev/null || { printf 'Docker with Buildx is required.\n' >&2; exit 1; }
# Mount the parent: the extractor atomically renames a sibling into the dataset.
mkdir -p -- "$(dirname -- "$output_dir")"
output_parent=$(cd -- "$(dirname -- "$output_dir")" && pwd)
if [[ "$output_parent" == *','* || "$output_parent" == *$'\n'* || "$output_parent" == *$'\r'* ]]; then
    printf 'The resolved output path must not contain commas or newlines.\n' >&2
    exit 2
fi
image=ffta-rom-data-extractor:local
docker buildx build --platform linux/amd64 \
    --file "$repo_dir/Dockerfile.linux" --target extract \
    --load --tag "$image" --progress plain "$repo_dir"
docker run --rm --platform linux/amd64 --network none \
    --user "$(id -u):$(id -g)" --env HOME=/tmp \
    --mount "type=bind,source=$rom_path,target=/input/source.gba,readonly" \
    --mount "type=bind,source=$output_parent,target=/output" \
    "$image" /input/source.gba --output "/output/$output_name"
printf '\nExtracted dataset on host: %s/%s\n' "$output_parent" "$output_name"
