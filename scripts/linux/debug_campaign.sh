#!/usr/bin/env bash
# Run host Python or the existing coverage image; paths are relative to repo root.
set -euo pipefail
repo_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)
mode=python
if [[ ${1:-} == --docker ]]; then mode=docker; shift; fi
mkdir -p -- "$repo_dir/target/build-logs"
log=$(mktemp -- "$repo_dir/target/build-logs/debug-campaign-$(date +%Y%m%d-%H%M%S)-XXXXXX.log")
printf 'Campaign log: %s\n' "$log"
cd -- "$repo_dir"
set +e
(
  set -e
  if [[ $mode == docker ]]; then
    if [[ $repo_dir == *','* || $repo_dir == *'"'* || $repo_dir == *$'\n'* || $repo_dir == *$'\r'* ]]; then
      printf 'Unsupported Docker mount path. Use host Python.\n' >&2; exit 2
    fi
    if [[ ${1:-} != report && ${1:-} != --help ]]; then printf "Docker mode supports report only. Use host Python for dispatch.\n" >&2; exit 2; fi
    docker image inspect ffta-debug-tools:local >/dev/null
    docker run --rm --network none --read-only --tmpfs /tmp \
      --user "$(id -u):$(id -g)" --workdir /project --entrypoint python3 \
      --mount "type=bind,source=$repo_dir,target=/project" \
      ffta-debug-tools:local /project/scripts/debug_campaign.py "$@"
  else
    python3 "$repo_dir/scripts/debug_campaign.py" "$@"
  fi
) 2>&1 | tee -- "$log"
statuses=("${PIPESTATUS[@]}")
set -e
if (( statuses[0] != 0 )); then exit "${statuses[0]}"; fi
exit "${statuses[1]}"
