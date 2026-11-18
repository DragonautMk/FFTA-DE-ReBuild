#!/usr/bin/env bash
# Build the maintained ARM64 APK with the shared Docker driver and signing cache.
set -euo pipefail
repo_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)
source "$repo_dir/scripts/linux/build_terminal.sh"

if (( $# == 1 )) && [[ $1 != -* ]]; then
    set -- --output "$1"
fi
if python3 "$repo_dir/scripts/android/build_android_docker.py" "$@"; then
    exit 0
else
    status=$?
    report_terminal_failure "ERROR: Android build failed (exit $status)."
    exit "$status"
fi
