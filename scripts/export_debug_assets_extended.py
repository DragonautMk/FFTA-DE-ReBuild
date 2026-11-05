#!/usr/bin/env python3
"""Export private debug assets including every consecutive micro-snapshot.

This extended entry point shares the standard exporter's checks and leaves all
source files untouched. It creates a separately named extended archive.
"""
from export_debug_assets import main


if __name__ == '__main__':
    main(extended=True)
