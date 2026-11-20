#!/usr/bin/env python3
"""Run a leased Wine validation with a verified local Bluetooth backport.

The overlay exists only in a private mount namespace. It does not replace the
installed Wine module or change the shared prefix configuration.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys

from artifact_coordinator import NAME, ordinary

ROOT = Path(__file__).resolve().parents[1]
WINE_UNIX = Path('/usr/lib/x86_64-linux-gnu/wine/x86_64-unix')


def digest(path):
    h = hashlib.sha256()
    with path.open('rb') as source:
        for block in iter(lambda: source.read(1024 * 1024), b''):
            h.update(block)
    return h.hexdigest()


def checked_owner(root, role):
    ordinary(root, directory=True)
    lease = root / NAME
    ordinary(lease, directory=True)
    owner = lease / 'owner.json'
    ordinary(owner)
    data = json.loads(owner.read_text(encoding='utf-8'))
    if data.get('role') != role or data.get('pid') != os.getppid():
        raise ValueError(f'The parent does not own the {role} lease')
    return data


def run(module, manifest, command):
    unit_value = os.environ.get('FFTA_VALIDATION_UNIT')
    prefix_value = os.environ.get('WINEPREFIX')
    if not unit_value or not prefix_value:
        raise ValueError('Run directly under validation_unit.py --wine-host')
    unit = Path(os.path.abspath(unit_value))
    prefix = Path(os.path.abspath(prefix_value))
    units = ROOT / 'target/validation/units'
    if unit.parent != units or prefix != ROOT / 'target/validation/wine-prefix':
        raise ValueError('Expected the managed validation unit and Wine prefix')
    unit_owner = checked_owner(unit, 'validation-unit')
    prefix_owner = checked_owner(prefix, 'validation-wine-prefix')

    module = Path(os.path.abspath(module))
    manifest = Path(os.path.abspath(manifest))
    for path in (module, manifest, WINE_UNIX / 'winebth.so', WINE_UNIX / 'ntdll.so'):
        ordinary(path, allow_hardlinks=True)
    record = json.loads(manifest.read_text(encoding='utf-8'))
    if record.get('status') != 'PASS':
        raise ValueError('The overlay manifest does not report PASS')
    expected = {
        module: record['module_sha256'],
        WINE_UNIX / 'winebth.so': record['installed_module_sha256'],
        WINE_UNIX / 'ntdll.so': record['installed_ntdll_sha256'],
    }
    for path, expected_hash in expected.items():
        if digest(path) != expected_hash:
            raise ValueError(f'Wine overlay input hash changed: {path}')
    print(f'WINE_BLUETOOTH_OVERLAY module_sha256={expected[module]}', flush=True)
    child = subprocess.run(['bwrap', '--bind', '/', '/', '--ro-bind', str(module),
                            str(WINE_UNIX / 'winebth.so'), '--', *command])
    if checked_owner(unit, 'validation-unit') != unit_owner or checked_owner(prefix, 'validation-wine-prefix') != prefix_owner:
        raise ValueError('A validation lease changed during Wine execution')
    for path, expected_hash in expected.items():
        if digest(path) != expected_hash:
            raise ValueError(f'Wine overlay input changed during execution: {path}')
    return child.returncode


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--module', required=True, help='Verified local winebth.so')
    parser.add_argument('--manifest', required=True, help='PASS build result with module and installed Wine hashes')
    parser.add_argument('command', nargs=argparse.REMAINDER, help='Command to run inside the overlay')
    args = parser.parse_args()
    command = args.command[1:] if args.command[:1] == ['--'] else args.command
    if not command:
        parser.error('A command is required after --')
    try:
        return run(args.module, args.manifest, command)
    except (OSError, ValueError, KeyError, json.JSONDecodeError) as error:
        print(f'Wine Bluetooth overlay refused: {error}', file=sys.stderr)
        return 1


if __name__ == '__main__':
    sys.exit(main())
