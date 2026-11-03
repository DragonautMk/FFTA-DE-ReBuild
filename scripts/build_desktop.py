#!/usr/bin/env python3
"""Build a native desktop portable package from Cargo's actual output paths."""
import argparse
import json
import os
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]


def cargo_runtime(platform):
    command = ['cargo', 'build', '--locked', '--release', '-p', 'recomp', '-p',
               'ffta-play', '--message-format=json']
    print('Building desktop runtime with Cargo...', flush=True)
    binaries = {}
    with subprocess.Popen(command, cwd=ROOT, stdout=subprocess.PIPE,
                          text=True, encoding='utf-8', errors='replace') as process:
        for line in process.stdout:
            try:
                event = json.loads(line)
            except json.JSONDecodeError:
                print(line, end='', file=sys.stderr)
                continue
            if event.get('reason') == 'compiler-message':
                print(event['message'].get('rendered') or event['message']['message'],
                      end='', file=sys.stderr)
            if event.get('reason') == 'compiler-artifact' and event.get('executable'):
                target = event.get('target', {})
                if target.get('name') in ('recomp', 'ffta-play') and 'bin' in target.get('kind', []):
                    path = Path(event['executable'])
                    binaries[target['name']] = path if path.is_absolute() else ROOT / path
        status = process.wait()
    if status:
        raise SystemExit(status)
    if set(binaries) != {'recomp', 'ffta-play'}:
        raise SystemExit('Cargo did not report both desktop executable paths; refusing stale outputs')
    suffix = '.exe' if platform == 'windows' else ''
    for name, path in binaries.items():
        if path.name != name + suffix or not path.is_file():
            raise SystemExit(f'Cargo output does not match {platform}: {path}')
        with path.open('rb') as binary:
            header = binary.read(64)
            if platform == 'linux':
                valid = (header[:6] == b'\x7fELF\x02\x01' and header[18:20] == b'\x3e\x00')
            else:
                binary.seek(int.from_bytes(header[60:64], 'little'))
                valid = header[:2] == b'MZ' and binary.read(6) == b'PE\x00\x00\x64\x86'
        if not valid:
            raise SystemExit(f'Expected a {platform} x86_64 executable: {path}')
    parents = {path.parent for path in binaries.values()}
    if len(parents) != 1:
        raise SystemExit('Cargo desktop executables have different output directories')
    return parents.pop()


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--platform', choices=('windows', 'linux'),
                        default='windows' if os.name == 'nt' else 'linux')
    args = parser.parse_args(argv)
    runtime = cargo_runtime(args.platform)
    library = 'libtcc.dll' if args.platform == 'windows' else 'libtcc.so'
    subprocess.run([sys.executable, str(ROOT / 'TinyCC/build.py'), '--platform',
                    args.platform, '--output', str(runtime / 'tcc' / library)],
                   cwd=ROOT, check=True)
    subprocess.run([sys.executable, str(ROOT / 'scripts/package_ffta_portable.py'),
                    '--platform', args.platform, '--runtime-dir', str(runtime)],
                   cwd=ROOT, check=True)


if __name__ == '__main__':
    try:
        main()
    except subprocess.CalledProcessError as error:
        raise SystemExit(error.returncode)
    except OSError as error:
        raise SystemExit(f'Build tool or input unavailable: {error}')
