#!/usr/bin/env bash
# Private extended export, including consecutive micro-snapshots; source files stay intact.
set -euo pipefail
repo_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"
exec python3 "$repo_dir/scripts/export_debug_assets_extended.py" "$@"
