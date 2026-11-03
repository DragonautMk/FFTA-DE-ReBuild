#!/usr/bin/env bash
# Generate the atlas with Python, Cargo and nm inside Docker.
set -euo pipefail
repo_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)
rom_path='rom/1194 - Final Fantasy Tactics Advance (E)(Surplus).gba'
dataset_dir=''
translation_path=''
audit_paths=()
comparison_path=''
output_dir='docs/development/rom-coverage'
evidence_path='docs/development/rom-coverage/evidence.json'
usage() {
    printf 'Usage: %s [--rom FILE] [--dataset DIR] [--translation FILE] [--output DIR] [--evidence FILE] [--audit REPORT ...] [--compare COVERAGE_JSON]\n' "$0"
    printf 'Default: prepare the latest committed extractor dataset and use versioned code-entry metadata.\n'
    printf 'All relative paths start at the project root. Logs: target/build-logs/rom-coverage-*.log\n'
}
main() {
while (( $# )); do
    case "$1" in
        --help|-h) usage; exit 0 ;;
        --rom|--dataset|--translation|--output|--evidence|--audit|--compare)
            if (( $# < 2 )) || [[ -z "$2" ]]; then
                printf 'Missing value for %s\n' "$1" >&2; exit 2
            fi
            case "$1" in
                --rom) rom_path=$2 ;;
                --dataset) dataset_dir=$2 ;;
                --translation) translation_path=$2 ;;
                --output) output_dir=$2 ;;
                --evidence) evidence_path=$2 ;;
                --audit) audit_paths+=("$2") ;;
                --compare) comparison_path=$2 ;;
            esac
            shift 2 ;;
        *) printf 'Unknown argument: %s\n' "$1" >&2; usage >&2; exit 2 ;;
    esac
done
resolve_path() {
    local value=$1
    if [[ "$value" != /* ]]; then value="$repo_dir/$value"; fi
    # Docker --mount parses CSV, including double quotes.
    if [[ "$value" == *','* || "$value" == *'"'* || "$value" == *$'\n'* || "$value" == *$'\r'* ]]; then
        printf 'Paths must not contain commas, double quotes or newlines.\n' >&2; return 2
    fi
    printf '%s' "$value"
}
rom_path=$(resolve_path "$rom_path")
output_dir=$(resolve_path "$output_dir")
evidence_path=$(resolve_path "$evidence_path")
inputs=("$rom_path" "$evidence_path")
if [[ -n "$dataset_dir" ]]; then
    dataset_dir=$(resolve_path "$dataset_dir")
    inputs+=("$dataset_dir/manifest.toml")
fi
if [[ -n "$translation_path" ]]; then
    translation_path=$(resolve_path "$translation_path")
    inputs+=("$translation_path")
fi
for i in "${!audit_paths[@]}"; do
    audit_paths[$i]=$(resolve_path "${audit_paths[$i]}")
    inputs+=("${audit_paths[$i]}")
done
if [[ -n "$comparison_path" ]]; then
    comparison_path=$(resolve_path "$comparison_path")
    inputs+=("$comparison_path")
fi
for path in "${inputs[@]}"; do
    if [[ ! -f "$path" ]]; then printf 'Required input file does not exist: %s\n' "$path" >&2; exit 2; fi
done
command -v docker >/dev/null || { printf 'Docker with Buildx is required.\n' >&2; exit 1; }
command -v git >/dev/null || { printf 'Git is required to isolate committed extraction sources.\n' >&2; exit 1; }
mkdir -p -- "$output_dir"
# Canonical paths also catch CSV-unsafe characters introduced by symlinks.
rom_path=$(resolve_path "$(cd -- "$(dirname -- "$rom_path")" && pwd -P)/$(basename -- "$rom_path")")
evidence_path=$(resolve_path "$(cd -- "$(dirname -- "$evidence_path")" && pwd -P)/$(basename -- "$evidence_path")")
output_dir=$(resolve_path "$(cd -- "$output_dir" && pwd -P)")
mounts=(--mount "type=bind,source=$rom_path,target=/input/source.gba,readonly"
    --mount "type=bind,source=$evidence_path,target=/input/evidence.json,readonly"
    --mount "type=bind,source=$output_dir,target=/output")
args=(--rom /input/source.gba --evidence /input/evidence.json --output /output)
if [[ -n "$dataset_dir" ]]; then
    dataset_dir=$(resolve_path "$(cd -- "$dataset_dir" && pwd -P)")
    mounts+=(--mount "type=bind,source=$dataset_dir,target=/input/dataset,readonly")
    args+=(--dataset /input/dataset)
    dataset_parent=$(dirname -- "$dataset_dir")
    if [[ $(basename -- "$dataset_dir") =~ ^e[0-9]+-d[0-9]+$ && $(basename -- "$dataset_parent") =~ ^[0-9a-f]{64}$ ]]; then
        lease_root=$(resolve_path "$(dirname -- "$dataset_parent")")
        mounts+=(--mount "type=bind,source=$lease_root,target=/cache-lease")
        args+=(--cache-lease /cache-lease)
    fi
else
    cache_dir=$(resolve_path "$repo_dir/target/rom-data/coverage")
    mkdir -p -- "$cache_dir"
    cache_dir=$(resolve_path "$(cd -- "$cache_dir" && pwd -P)")
    mounts+=(--mount "type=bind,source=$cache_dir,target=/cache")
    args+=(--dataset-cache /cache --cache-keep 2)
fi
if [[ -n "$translation_path" ]]; then
    translation_path=$(resolve_path "$(cd -- "$(dirname -- "$translation_path")" && pwd -P)/$(basename -- "$translation_path")")
    mounts+=(--mount "type=bind,source=$translation_path,target=/input/translation.dll,readonly")
    args+=(--translation /input/translation.dll)
fi
for i in "${!audit_paths[@]}"; do
    path=${audit_paths[$i]}
    path=$(resolve_path "$(cd -- "$(dirname -- "$path")" && pwd -P)/$(basename -- "$path")")
    mounts+=(--mount "type=bind,source=$path,target=/input/audit-$i.json,readonly")
    args+=(--audit "/input/audit-$i.json")
done
if [[ -n "$comparison_path" ]]; then
    comparison_path=$(resolve_path "$(cd -- "$(dirname -- "$comparison_path")" && pwd -P)/$(basename -- "$comparison_path")")
    mounts+=(--mount "type=bind,source=$comparison_path,target=/input/previous.json,readonly")
    args+=(--compare /input/previous.json)
fi
image=ffta-rom-coverage:local
source_revision=$(git -C "$repo_dir" rev-parse --verify HEAD)
context_dir=$(mktemp -d -- "$repo_dir/target/rom-coverage-context-XXXXXX")
trap 'rm -rf -- "$context_dir"' EXIT
# Include committed include_bytes! tool assets; WIP scripts must not enter the runtime.
git -C "$repo_dir" archive --format=tar --output "$context_dir/runtime.tar" "$source_revision" \
    Cargo.toml Cargo.lock crates assets TinyCC Licenses gamedb.sqlite scripts
mkdir -p -- "$context_dir/scripts" "$context_dir/docs/changes" "$context_dir/docs/development/rom-coverage"
cp -- "$repo_dir/scripts/rom_coverage.py" "$repo_dir/scripts/rom_coverage.html" \
    "$repo_dir/scripts/artifact_coordinator.py" "$repo_dir/scripts/local_artifacts.py" "$repo_dir/scripts/rom_coverage_audit.py" \
    "$repo_dir/scripts/rom_coverage_ranges.py" "$context_dir/scripts/"
cp -- "$repo_dir/docs/changes/"*.json "$context_dir/docs/changes/"
cp -- "$repo_dir/docs/development/rom-coverage/evidence.json" \
    "$repo_dir/docs/development/rom-coverage/code-seeds.json" "$context_dir/docs/development/rom-coverage/"
printf 'Building ROM coverage tools inside Docker from commit %s...\n' "$source_revision"
docker buildx build --platform linux/amd64 \
    --file "$repo_dir/Dockerfile.coverage" --build-arg "COVERAGE_SOURCE_REVISION=$source_revision" \
    --load --tag "$image" --progress plain "$context_dir"
docker run --rm --platform linux/amd64 --network none \
    --user "$(id -u):$(id -g)" --env HOME=/tmp \
    "${mounts[@]}" "$image" "${args[@]}"
for path in "$output_dir/index.html" "$output_dir/coverage.json"; do
    if [[ ! -s "$path" ]]; then printf 'No report produced: %s\n' "$path" >&2; return 1; fi
done
printf '\nROM coverage on host: %s/index.html\n' "$output_dir"
printf 'JSON data on host: %s/coverage.json\n' "$output_dir"
}

log_dir="$repo_dir/target/build-logs"
mkdir -p -- "$log_dir"
log_file=$(mktemp -- "$log_dir/rom-coverage-$(date +%Y%m%d-%H%M%S)-XXXXXX.log")
printf 'ROM coverage log: %s\n' "$log_file"
set +e
(
    set -e
    main "$@"
) 2>&1 | tee -- "$log_file"
statuses=("${PIPESTATUS[@]}")
set -e
status=${statuses[0]}
if (( status == 0 )); then status=${statuses[1]}; fi
if (( status != 0 )); then
    printf '\nROM coverage failed (exit %s). Log: %s\n' "$status" "$log_file" | tee -a -- "$log_file" >&2
    if [[ -t 0 && -t 2 ]]; then read -r -p 'Premi Invio per chiudere...' _ || true; fi
fi
exit "$status"
