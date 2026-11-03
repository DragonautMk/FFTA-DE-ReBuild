#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../.."
if [[ $(uname -s) != Linux ]]; then
    printf 'This script requires Linux.\n' >&2
    exit 1
fi
python3 scripts/build_desktop.py --platform linux
