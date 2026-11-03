#!/usr/bin/env bash
# Resolve the selected portable runtime and preserve each command's full log.
set -euo pipefail
runtime_root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)
mkdir -p -- "$runtime_root/Data/config/debug-logs"
runtime_log=$(mktemp -- "$runtime_root/Data/config/debug-logs/runtime-$(date +%Y%m%d-%H%M%S)-XXXXXX.log")
printf 'Runtime log: %s\n' "$runtime_log"
cd -- "$runtime_root"
set +e
(
  set -e
  wine_runner=
  if [[ ${1:-} == --wine ]]; then
    if (( $# < 2 )); then printf 'Use --wine /path/to/wine64 COMMAND...\n' >&2; exit 2; fi
    wine_runner=$2; shift 2
  fi
  if [[ ${1:-} == --help || $# == 0 ]]; then
    printf 'Usage: bash scripts/linux/debug_runtime.sh [--wine /path/to/wine64] COMMAND...\nRelative runtime arguments resolve from this portable folder. Logs: Data/config/debug-logs.\n'
    exit 0
  fi
  if [[ -n $wine_runner ]]; then
    [[ -f "$runtime_root/recomp.exe" ]] || { printf 'Missing portable recomp.exe\n' >&2; exit 1; }
    "$wine_runner" "$runtime_root/recomp.exe" "$@"
  elif [[ -f "$runtime_root/recomp" ]]; then
    "$runtime_root/recomp" "$@"
  else
    printf 'Missing native recomp; Windows package on POSIX requires explicit --wine /path/to/wine64.\n' >&2
    exit 1
  fi
) 2>&1 | tee -- "$runtime_log"
runtime_statuses=("${PIPESTATUS[@]}")
set -e
if (( runtime_statuses[0] != 0 )); then exit "${runtime_statuses[0]}"; fi
exit "${runtime_statuses[1]}"
