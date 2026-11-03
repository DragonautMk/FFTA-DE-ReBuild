#!/usr/bin/env python3
"""Run foreground validation with disposable scratch and durable evidence.

Successful work is removed only after explicit PASS results and input guards.
Failures and interruptions retain their work. Wine runs share one leased prefix;
this tool never adopts or deletes an existing unowned Wine environment.
"""
import argparse
import contextlib
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import re
import shutil
import subprocess
import sys
import tempfile

from artifact_coordinator import Lease, NAME, ordinary
from local_artifacts import ensure_untracked

ROOT = Path(__file__).resolve().parents[1]
PRIVATE = {'.gba', '.bios', '.sav', '.ffstate', '.input', '.rec', '.recording'}
PREFIX_MARKER = '.ffta-validation-prefix.json'
PREFIX_OWNER = {'schema': 1, 'kind': 'ffta-validation-wine-prefix'}


def write_json(path, value):
    with tempfile.NamedTemporaryFile(dir=path.parent, delete=False) as f:
        temporary = Path(f.name)
        try:
            f.write((json.dumps(value, indent=2) + '\n').encode('utf-8'))
            f.flush()
            os.fsync(f.fileno())
            f.close()
            os.replace(temporary, path)
        finally:
            temporary.unlink(missing_ok=True)


def digest(path):
    h = hashlib.sha256()
    with path.open('rb') as f:
        for block in iter(lambda: f.read(1024 * 1024), b''):
            h.update(block)
    return h.hexdigest()


def snapshot(path, *, hashes=False, disposable=False, read_only=False):
    """Guard complete trees, including empty directories; refuse links/junctions."""
    path = Path(os.path.abspath(path))
    is_dir = path.is_dir()
    ordinary(path, directory=is_dir, allow_hardlinks=read_only)
    entries = [path]
    if is_dir:
        for parent, dirs, files in os.walk(path, followlinks=False):
            if NAME in dirs or NAME in files:
                raise ValueError(f'Activity marker inside guarded tree: {parent}')
            for name in dirs:
                ordinary(Path(parent) / name, directory=True)
            entries.extend(Path(parent) / name for name in dirs + files)
    result = {}
    for p in entries:
        directory = p.is_dir()
        st = ordinary(p, directory=directory, allow_hardlinks=read_only)
        if disposable and not directory and p.suffix.lower() in PRIVATE:
            raise ValueError(f'Private input in scratch; retained for review: {p}')
        row = {'directory': directory, 'identity': [st.st_dev, st.st_ino]}
        if not directory:
            row.update(size=st.st_size, mtime_ns=st.st_mtime_ns)
            if hashes:
                row['sha256'] = digest(p)
        result[str(p.relative_to(path))] = row
    return result


def relative_path(value):
    if not value or '\\' in value or ':' in value:
        raise ValueError(f'Expected a relative path with forward slashes: {value}')
    p = PurePosixPath(value)
    if p.is_absolute() or '..' in p.parts or str(p) == '.':
        raise ValueError(f'Path must stay inside scratch: {value}')
    return Path(*p.parts)


def wine_path(path):
    # Wine's normal Z: mapping is required; no winepath subprocess starts early.
    return 'Z:' + str(path).replace('/', '\\')


def managed_prefix(base):
    prefix = base / 'wine-prefix'
    try:
        prefix.mkdir()
    except FileExistsError:
        ordinary(prefix, directory=True)
        ordinary(prefix / PREFIX_MARKER)
        if json.loads((prefix / PREFIX_MARKER).read_text()) != PREFIX_OWNER:
            raise ValueError('Refusing an unowned Wine prefix')
    else:
        write_json(prefix / PREFIX_MARKER, PREFIX_OWNER)
    return prefix


def run_unit(args, *, root=ROOT):
    root = root.resolve()
    if not re.fullmatch(r'[A-Za-z0-9][A-Za-z0-9._-]{0,95}', args.name):
        raise ValueError('Unit name must be a short filename, starting with a letter or digit')
    results = [relative_path(p) for p in args.result]
    evidence = [relative_path(p) for p in args.evidence]
    protected = [Path(os.path.abspath(p)) for p in args.protect]
    before = {str(p): snapshot(p, hashes=True, read_only=True) for p in protected}
    command = args.command[1:] if args.command[:1] == ['--'] else args.command
    if not command:
        raise ValueError('A foreground validation command is required after --')
    wineserver = shutil.which(args.wineserver or os.environ.get('WINESERVER', 'wineserver'))
    if args.wine and (os.name != 'posix' or not shutil.which('wine') or not wineserver):
        raise ValueError('--wine requires Wine and wineserver on a POSIX host')
    if args.timeout <= 0:
        raise ValueError('Timeout must be positive')
    target = root / 'target'
    target.mkdir(exist_ok=True)
    ordinary(target, directory=True)
    base = target / 'validation'
    base.mkdir(exist_ok=True)
    ordinary(base, directory=True)
    units = base / 'units'
    units.mkdir(exist_ok=True)
    ordinary(units, directory=True)
    unit = Path(tempfile.mkdtemp(prefix=args.name + '-', dir=units))
    work = unit / 'work'
    work.mkdir()
    receipt_path = unit / 'unit-result.json'
    report = {'schema': 1, 'kind': 'ffta-validation-unit', 'status': 'running',
              'work': str(work), 'cleanup': 'not_started', 'errors': [],
              'protected': list(before), 'evidence': [], 'keep_reason': args.keep_work}
    write_json(unit / 'protected-before.json', before)
    print(f'Validation unit: {unit}', flush=True)
    leases = contextlib.ExitStack()
    safe_to_release = True
    held = []
    try:
        held.append(leases.enter_context(Lease(unit, 'validation-unit')))
        env = {k: v for k, v in os.environ.items()
               if not k.startswith(('FFTA_', 'RECOMP_', 'GBA_'))}
        replacements = {'{repo}': str(root), '{unit}': str(unit), '{work}': str(work),
                        '{wine_repo}': wine_path(root), '{wine_unit}': wine_path(unit),
                        '{wine_work}': wine_path(work)}

        def expand(value):
            for token, replacement in replacements.items():
                value = value.replace(token, replacement)
            return value

        for value in args.env:
            key, sep, val = value.partition('=')
            if not sep or not re.fullmatch('[A-Za-z_][A-Za-z0-9_]*', key):
                raise ValueError(f'Expected NAME=value environment argument: {value}')
            if key.upper() == 'WINEPREFIX':
                raise ValueError('WINEPREFIX is managed by the runner')
            env[key] = expand(val)
        if args.wine:
            prefix = managed_prefix(base)
            held.append(leases.enter_context(Lease(prefix, 'validation-wine-prefix')))
            env.update(WINEPREFIX=str(prefix), WINEDEBUG='-all', WINESERVER=wineserver,
                       WINEDLLOVERRIDES='winemenubuilder.exe=d')
            report['wine_prefix'] = str(prefix)
        env['FFTA_VALIDATION_WORK'] = wine_path(work) if args.wine else str(work)
        env['FFTA_VALIDATION_UNIT'] = wine_path(unit) if args.wine else str(unit)
        command = (['wine'] if args.wine else []) + [expand(p) for p in command]
        report['command'] = command
        write_json(receipt_path, report)
        with (unit / 'command.log').open('wb') as log:
            # A timed-out/interrupting command may have descendants. Keep both
            # work and protective leases unless foreground completion is known.
            safe_to_release = False
            child = subprocess.run(command, cwd=work, env=env, stdout=log, stderr=log,
                                   timeout=args.timeout)
            if args.wine:
                subprocess.run([wineserver, '-w'], env=env, stdout=log, stderr=log,
                               timeout=120, check=True)
            safe_to_release = True
        report['exit_code'] = child.returncode
        if child.returncode:
            raise ValueError(f'Validation command exited {child.returncode}')
        for p, saved in before.items():
            if snapshot(Path(p), hashes=True, read_only=True) != saved:
                raise ValueError(f'Protected input changed: {p}')
        for rel in results:
            p = work / rel
            ordinary(p)
            if json.loads(p.read_text(encoding='utf-8')).get('status') != 'PASS':
                raise ValueError(f'Result must explicitly report status PASS: {rel}')
        inventory = snapshot(work, hashes=True, disposable=not args.keep_work)
        # Manifests and small result identities survive retirement automatically.
        selected = set(results + evidence)
        selected.update(Path(rel) for rel, row in inventory.items()
                        if not row['directory'] and
                        (Path(rel).name == 'manifest.toml' or
                         Path(rel).name.endswith(('result.json', 'summary.json'))))
        for rel in sorted(selected):
            source = work / rel
            ordinary(source)
            if source.stat().st_size > 16 * 1024 * 1024:
                raise ValueError(f'Evidence exceeds 16 MiB; keep work explicitly: {rel}')
            dest = unit / 'evidence' / rel
            dest.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(source, dest)
            if digest(dest) != inventory[str(rel)]['sha256']:
                raise ValueError(f'Evidence copy changed: {rel}')
            report['evidence'].append(str(rel))
        write_json(unit / 'work-inventory.json', inventory)
        ensure_untracked(work, root)
        for lease in held:
            lease.check()
        if not args.keep_work:
            # Recheck the entire owned tree immediately before removal. No
            # legacy unit or external dataset is selected by this runner.
            if snapshot(work, hashes=True, disposable=True) != inventory:
                raise ValueError('Scratch changed before retirement')
            report['cleanup'] = 'removing'
            write_json(receipt_path, report)
            shutil.rmtree(work)
        report.update(status='PASS', cleanup='kept' if args.keep_work else 'removed')
    except BaseException as error:
        report.update(status='FAIL',
                      cleanup='incomplete' if report['cleanup'] == 'removing' else 'retained',
                      leases_retained=not safe_to_release)
        report['errors'].append(str(error) or type(error).__name__)
        raise
    finally:
        try:
            if safe_to_release:
                leases.close()
            else:
                leases.pop_all()  # Intentional diagnostic protection; never reclaim automatically.
        except (OSError, ValueError) as error:
            report.update(status='FAIL', leases_retained=True)
            report['errors'].append(str(error))
            raise
        finally:
            write_json(receipt_path, report)
    print(f'PASS; scratch {report["cleanup"]}; receipt: {receipt_path}', flush=True)
    return unit


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--name', required=True, help='short validation label; each run gets a unique unit')
    p.add_argument('--protect', action='append', default=[], help='read-only input file/tree to hash')
    p.add_argument('--result', action='append', required=True,
                   help='required PASS JSON relative to work (repeatable)')
    p.add_argument('--evidence', action='append', default=[], help='additional small work file to retain')
    p.add_argument('--env', action='append', default=[], help='child NAME=value; supports path placeholders')
    p.add_argument('--wine', action='store_true', help='run through Wine with the shared leased prefix')
    p.add_argument('--wineserver', help='matching wineserver executable if absent from PATH')
    p.add_argument('--timeout', type=float, default=900, help='foreground timeout in seconds')
    p.add_argument('--keep-work', metavar='REASON', help='explicit current-input or diagnostic exception')
    p.add_argument('command', nargs=argparse.REMAINDER)
    args = p.parse_args()
    try:
        run_unit(args)
        return 0
    except (OSError, ValueError, subprocess.SubprocessError) as error:
        print(f'Validation retained for review: {error}', file=sys.stderr)
        return 1


if __name__ == '__main__':
    raise SystemExit(main())
