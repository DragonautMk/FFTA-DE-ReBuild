#!/usr/bin/env bash
# Host Python or existing Docker image; preserve the previous page on errors.
set -euo pipefail
repo_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)
mkdir -p -- "$repo_dir/target/build-logs"
home_log=$(mktemp -- "$repo_dir/target/build-logs/report-home-$(date +%Y%m%d-%H%M%S)-XXXXXX.log")
printf 'Homepage log: %s\n' "$home_log"
cd -- "$repo_dir"
set +e
(
  set -e
  if [[ ${1:-} == --python && $# == 1 ]]; then
    python3 "$repo_dir/scripts/report_home.py"
  elif (( $# == 0 )); then
    if [[ "$repo_dir" == *','* || "$repo_dir" == *'"'* || "$repo_dir" == *$'\n'* || "$repo_dir" == *$'\r'* ]]; then
      printf 'Unsupported Docker mount path. Use --python.\n' >&2; exit 2
    fi
    docker image inspect ffta-rom-coverage:local >/dev/null
    home_tmp=$(mktemp -- "$repo_dir/.report-home-XXXXXXXX.tmp")
    trap 'rm -f -- "$home_tmp"' EXIT
    docker run --rm --network none --entrypoint python3 \
      --mount "type=bind,source=$repo_dir,target=/project,readonly" \
      ffta-rom-coverage:local /project/scripts/report_home.py --root /project --stdout > "$home_tmp"
    mv -f -- "$home_tmp" "$repo_dir/index.html"
    printf 'Homepage: %s/index.html\n' "$repo_dir"
  else
    printf 'Usage: bash scripts/linux/report_home.sh [--python]\nDefault: existing ffta-rom-coverage:local image; no build or extraction.\n' >&2
    exit 2
  fi
) 2>&1 | tee -- "$home_log"
home_statuses=("${PIPESTATUS[@]}")
set -e
if (( home_statuses[0] != 0 )); then exit "${home_statuses[0]}"; fi
exit "${home_statuses[1]}"
