#!/usr/bin/env bash
# Build and export native Linux artifacts without installing a host toolchain.
set -euo pipefail
repo_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)

main() {
    if (( $# > 1 )); then
        printf 'Usage: %s [output-directory]\n' "$0" >&2
        exit 2
    fi
    output_dir=${1:-"$repo_dir/dist/docker-linux"}
    # Buildx parses --output as CSV. Do not silently truncate a path at a comma.
    if [[ "$output_dir" == *','* || "$output_dir" == *'"'* || "$output_dir" == *$'\n'* || "$output_dir" == *$'\r'* ]]; then
        printf 'The output directory must not contain commas, double quotes or newlines.\n' >&2
        exit 2
    fi
    command -v docker >/dev/null || { printf 'Docker with Buildx is required.\n' >&2; exit 1; }
    mkdir -p -- "$output_dir"
    docker buildx build \
        --platform linux/amd64 \
        --file "$repo_dir/Dockerfile.linux" \
        --target export \
        --output "type=local,dest=$output_dir" \
        --progress plain \
        "$repo_dir"
    printf 'Portable archive and BUILD-INFO.txt exported to %s\n' "$output_dir"
}

# Preserve diagnostics when a file manager closes its terminal after exit.
log_dir="$repo_dir/target/build-logs"
mkdir -p -- "$log_dir"
log_file=$(mktemp -- "$log_dir/linux-$(date +%Y%m%d-%H%M%S)-XXXXXX.log")
printf 'Linux build log: %s\n' "$log_file"
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
    printf '\nLinux build failed (exit %s). Log: %s\n' "$status" "$log_file" >&2
    if [[ -t 0 && -t 2 ]]; then
        read -r -p 'Premi Invio per chiudere questa finestra...' _ || true
    fi
fi
exit "$status"
