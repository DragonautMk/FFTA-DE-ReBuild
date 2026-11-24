#!/usr/bin/env bash
# Generate an offline extracted-resource viewer with Docker.
set -euo pipefail
repo_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)
usage() {
    printf 'Usage: %s DATASET [OUTPUT_DIRECTORY]\n' "$0"
}
if (( $# == 1 )) && [[ "$1" == --help || "$1" == -h ]]; then
    usage
    exit 0
fi
if (( $# < 1 || $# > 2 )); then
    usage >&2
    exit 2
fi
if [[ ! -f "$1/manifest.toml" ]]; then
    printf 'Dataset manifest not found: %s\n' "$1/manifest.toml" >&2
    exit 2
fi
dataset=$(cd -- "$1" && pwd -P)
output=${2:-"$repo_dir"}
filename=index.html
if (( $# == 1 )); then
    filename=rom_data_viewer_generated.html
fi
for path in "$dataset" "$output" "$repo_dir"; do
    if [[ "$path" == *','* || "$path" == *$'\n'* || "$path" == *$'\r'* ]]; then
        printf 'Paths must not contain commas or newlines.\n' >&2
        exit 2
    fi
done
output=$(realpath -m -- "$output")
if [[ "$output" == "$dataset" || "$output" == "$dataset/"* ]]; then
    printf 'Output must be outside the dataset.\n' >&2
    exit 2
fi
if [[ "$output" == *','* || "$output" == *$'\n'* || "$output" == *$'\r'* ]]; then
    printf 'Resolved output path must not contain commas or newlines.\n' >&2
    exit 2
fi
command -v docker >/dev/null || { printf 'Docker is required.\n' >&2; exit 1; }
mkdir -p -- "$output"
docker run --rm --network none --user "$(id -u):$(id -g)" \
    --mount "type=bind,source=$repo_dir/scripts,target=/viewer/scripts,readonly" \
    --mount "type=bind,source=$repo_dir/view_rom_data.html,target=/viewer/view_rom_data.html,readonly" \
    --mount "type=bind,source=$dataset,target=/dataset,readonly" \
    --mount "type=bind,source=$output,target=/output" \
    python:3.12-alpine python3 /viewer/scripts/view_rom_data.py /dataset /output --filename "$filename"
printf 'Viewer ready: %s/%s\n' "$output" "$filename"
