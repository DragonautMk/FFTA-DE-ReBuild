#!/usr/bin/env python3
"""Compatibility entry point for the Windows portable packager."""
from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from package_ffta_portable import main

if __name__ == '__main__':
    main(default_platform='windows')
