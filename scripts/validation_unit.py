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
import math
import os
from pathlib import Path, PurePosixPath
import re
import shutil
import subprocess
import sys
import tempfile

from artifact_coordinator import Lease, NAME, ordinary
import cold_storage
from local_artifacts import ensure_untracked

ROOT = Path(__file__).resolve().parents[1]
PRIVATE = cold_storage.PRIVATE
PREFIX_MARKER = '.ffta-validation-prefix.json'
PREFIX_OWNER = {'schema': 1, 'kind': 'ffta-validation-wine-prefix'}
GIB = 1024 ** 3
DEFAULT_MIN_FREE_GIB = 5
DEFAULT_RESERVE_GIB = 1


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


def retained(rel, paths):
    return any(rel == keep or keep in rel.parents for keep in paths)


def snapshot(path, *, hashes=False, disposable=False, read_only=False,
             owned=False, keep_paths=()):
    """Guard complete trees, including empty directories; refuse links/junctions."""
    path = Path(os.path.abspath(path))
    is_dir = path.is_dir()
    ordinary(path, directory=is_dir, allow_hardlinks=read_only or owned)
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
        st = ordinary(p, directory=directory, allow_hardlinks=read_only or owned)
        rel = p.relative_to(path)
        if (disposable and not directory and p.suffix.lower() in PRIVATE
                and not retained(rel, keep_paths)):
            raise ValueError(f'Private input in scratch; retained for review: {p}')
        row = {'directory': directory, 'identity': [st.st_dev, st.st_ino]}
        if not directory:
            row.update(size=st.st_size, mtime_ns=st.st_mtime_ns)
            if owned:
                row['links'] = st.st_nlink
            if hashes:
                row['sha256'] = digest(p)
        result[str(rel)] = row
    if owned:
        # Cargo links build-script executables inside its own output tree.
        # Accept them only when every name for that inode is in owned scratch.
        counts = {}
        for row in result.values():
            if not row['directory']:
                key = tuple(row['identity'])
                counts[key] = counts.get(key, 0) + 1
        for rel, row in result.items():
            if not row['directory'] and counts[tuple(row['identity'])] != row['links']:
                raise ValueError(f'Nonordinary/shared path refused (external hard link): {path / rel}')
    return result


def check_space(root, args):
    """Admission check, not a reservation or a quota on a running child."""
    floor = args.min_free_gib
    reserve = args.reserve_gib
    if (not all(math.isfinite(n) and n >= 0 for n in (floor, reserve))
            or not math.isfinite((floor + reserve) * GIB)):
        raise ValueError('Space budgets must be finite, nonnegative GiB values')
    reason = (args.space_exception or '').strip()
    if (floor < DEFAULT_MIN_FREE_GIB or reserve < DEFAULT_RESERVE_GIB) and not reason:
        raise ValueError('Lower space budgets require --space-exception with a concrete reason')
    target = root / 'target'
    location = target if target.exists() else root
    ordinary(location, directory=True)
    free = shutil.disk_usage(location).free
    required = math.ceil((floor + reserve) * GIB)
    if free < required:
        raise ValueError(f'Insufficient free space: {free / GIB:.2f} GiB available; '
                         f'{required / GIB:.2f} GiB required '
                         '(free-space floor plus expected growth). Review retained artifacts '
                         'or declare a justified smaller budget; no cleanup was attempted')
    return {'free_bytes': free, 'required_bytes': required, 'min_free_gib': floor,
            'reserve_gib': reserve, 'exception_reason': reason or None}


def inventory_bytes(inventory, keep_paths=None):
    seen = set()
    size = 0
    for rel, row in inventory.items():
        key = tuple(row['identity'])
        if (not row['directory'] and key not in seen
                and (keep_paths is None or retained(Path(rel), keep_paths))):
            size += row['size']
            seen.add(key)
    return size


def retire_work(work, inventory, keep_paths):
    if not keep_paths:
        shutil.rmtree(work)
        return
    # Keep selected paths in place so dataset manifests and consumers keep
    # their original locations. Ancestors survive; unselected siblings do not.
    for name, row in sorted(inventory.items(), key=lambda item: len(Path(item[0]).parts),
                            reverse=True):
        rel = Path(name)
        if rel == Path('.') or retained(rel, keep_paths):
            continue
        if row['directory'] and any(rel in keep.parents for keep in keep_paths):
            continue
        p = work / rel
        ordinary(p, directory=row['directory'], allow_hardlinks=True)
        if row['directory']:
            p.rmdir()
        else:
            p.unlink()


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
    check_space(root, args)
    keep_hot = getattr(args, 'keep_hot', None)
    if keep_hot is not None and not keep_hot.strip():
        raise ValueError('--keep-hot requires a concrete reason')
    if (args.keep_work or args.keep_path) and not keep_hot:
        cold_storage.codec()
    inputs = cold_storage.Inputs(root, [Path(os.path.abspath(p)) for p in args.protect],
                                 floor=int((args.min_free_gib + args.reserve_gib) * GIB),
                                 archive_floor=int(args.min_free_gib * GIB))
    unit = None
    try:
        inputs.open()
        unit = _run_unit(args, root=root, inputs=inputs)
        inputs.close(success=True)
        receipt = unit / 'unit-result.json'
        report = json.loads(receipt.read_text())
        report['cold_inputs_retired'] = [data['source'] for _, data in inputs.restored]
        write_json(receipt, report)
        print(f'PASS; scratch {report["cleanup"]}; storage {report["retention_storage"]}; receipt: {receipt}', flush=True)
        return unit
    except BaseException as error:
        inputs.close(success=False)
        if unit:
            receipt = unit / 'unit-result.json'
            report = json.loads(receipt.read_text())
            report['status'] = 'FAIL'
            report['errors'].append(f'Archived input retirement failed: {error}')
            write_json(receipt, report)
        raise


def _run_unit(args, *, root, inputs):
    root = root.resolve()
    if not re.fullmatch(r'[A-Za-z0-9][A-Za-z0-9._-]{0,95}', args.name):
        raise ValueError('Unit name must be a short filename, starting with a letter or digit')
    results = [relative_path(p) for p in args.result]
    evidence = [relative_path(p) for p in args.evidence]
    keep_paths = [relative_path(p) for p in args.keep_path]
    if args.keep_work is not None and (not args.keep_work.strip() or keep_paths or args.keep_reason):
        raise ValueError('--keep-work needs a reason and cannot be combined with selective retention')
    if (bool(keep_paths) != (args.keep_reason is not None)
            or args.keep_reason is not None and not args.keep_reason.strip()):
        raise ValueError('--keep-path and a nonempty --keep-reason must be supplied together')
    protected = [Path(os.path.abspath(p)) for p in args.protect]
    before = {str(p): snapshot(p, hashes=True, read_only=True) for p in protected}
    command = args.command[1:] if args.command[:1] == ['--'] else args.command
    if not command:
        raise ValueError('A foreground validation command is required after --')
    wine_host = getattr(args, 'wine_host', False)
    if args.wine and wine_host:
        raise ValueError('--wine and --wine-host are mutually exclusive')
    uses_wine = args.wine or wine_host
    wineserver = shutil.which(args.wineserver or os.environ.get('WINESERVER', 'wineserver'))
    if uses_wine and (os.name != 'posix' or not shutil.which('wine') or not wineserver):
        raise ValueError('--wine/--wine-host require Wine and wineserver on a POSIX host')
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
              'protected': list(before), 'evidence': [],
              'keep_reason': args.keep_work or args.keep_reason,
              'retention_mode': 'all' if args.keep_work else 'selected' if keep_paths else 'none',
              'kept_paths': [str(p) for p in keep_paths],
              'keep_hot_reason': getattr(args, 'keep_hot', None),
              'cold_inputs': [data['source'] for _, data in inputs.selected],
              'cold_archives': [], 'cold_skipped': []}
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
        if uses_wine:
            prefix = managed_prefix(base)
            wine_lease = leases.enter_context(Lease(prefix, 'validation-wine-prefix'))
            held.append(wine_lease)
            env.update(WINEPREFIX=str(prefix), WINEDEBUG='-all', WINESERVER=wineserver,
                       WINEDLLOVERRIDES='winemenubuilder.exe=d')
            report.update(wine_prefix=str(prefix), wine_mode='host' if wine_host else 'direct',
                          wine_lease=wine_lease.owner, wine_server_drained=False)
        env['FFTA_VALIDATION_WORK'] = wine_path(work) if args.wine else str(work)
        env['FFTA_VALIDATION_UNIT'] = wine_path(unit) if args.wine else str(unit)
        command = (['wine'] if args.wine else []) + [expand(p) for p in command]
        report['command'] = command
        report['space_check'] = check_space(root, args)
        write_json(receipt_path, report)
        with (unit / 'command.log').open('wb') as log:
            # A timed-out/interrupting command may have descendants. Keep both
            # work and protective leases unless foreground completion is known.
            safe_to_release = False
            inputs.safe_to_release = False
            child = subprocess.run(command, cwd=work, env=env, stdout=log, stderr=log,
                                   timeout=args.timeout)
            if uses_wine:
                # Native drivers inherit the leased environment for every Wine
                # child. Keep ownership across restarts and the final server drain.
                wine_lease.check()
                subprocess.run([wineserver, '-w'], env=env, stdout=log, stderr=log,
                               timeout=120, check=True)
                report['wine_server_drained'] = True
            safe_to_release = True
            inputs.safe_to_release = True
        report['exit_code'] = child.returncode
        if child.returncode:
            raise ValueError(f'Validation command exited {child.returncode}')
        for p, saved in before.items():
            if snapshot(Path(p), hashes=True, read_only=True) != saved:
                raise ValueError(f'Protected input changed: {p}')
        for rel in results:
            p = work / rel
            ordinary(p, allow_hardlinks=True)
            if json.loads(p.read_text(encoding='utf-8')).get('status') != 'PASS':
                raise ValueError(f'Result must explicitly report status PASS: {rel}')
        inventory = snapshot(work, hashes=True, disposable=not args.keep_work,
                             owned=True, keep_paths=keep_paths)
        for rel in keep_paths:
            if str(rel) not in inventory:
                raise ValueError(f'Selected retention path is missing: {rel}')
        report['work_bytes_before'] = inventory_bytes(inventory)
        # Manifests and small result identities survive retirement automatically.
        selected = set(results + evidence)
        selected.update(Path(rel) for rel, row in inventory.items()
                        if not row['directory'] and
                        (Path(rel).name == 'manifest.toml' or
                         Path(rel).name.endswith(('result.json', 'summary.json'))))
        for rel in sorted(selected):
            source = work / rel
            ordinary(source, allow_hardlinks=True)
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
            if snapshot(work, hashes=True, disposable=True, owned=True,
                        keep_paths=keep_paths) != inventory:
                raise ValueError('Scratch changed before retirement')
            report['cleanup'] = 'removing'
            write_json(receipt_path, report)
            retire_work(work, inventory, keep_paths)
        report.update(status='PASS', cleanup='kept' if args.keep_work or keep_paths else 'removed',
                      work_bytes_retained=inventory_bytes(inventory, None if args.keep_work else keep_paths))
        if (args.keep_work or keep_paths) and not getattr(args, 'keep_hot', None):
            proof = {'kind': 'foreground-validation', 'unit': str(unit.relative_to(root)),
                     'status': 'PASS', 'exit_code': child.returncode,
                     'results': report['evidence']}
            report['cold_archives'], report['cold_skipped'] = inputs.archive_outputs(
                work, reason=report['keep_reason'], proof=proof)
        report['retention_storage'] = ('mixed' if report['cold_archives'] and work.exists()
                                       else 'cold' if report['cold_archives']
                                       else 'hot' if work.exists() else 'none')
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
    return unit


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--name', required=True, help='short validation label; each run gets a unique unit')
    p.add_argument('--protect', action='append', default=[], help='read-only input file/tree to hash')
    p.add_argument('--result', action='append', required=True,
                   help='required PASS JSON relative to work (repeatable)')
    p.add_argument('--evidence', action='append', default=[], help='additional small work file to retain')
    p.add_argument('--env', action='append', default=[], help='child NAME=value; supports path placeholders')
    wine_mode = p.add_mutually_exclusive_group()
    wine_mode.add_argument('--wine', action='store_true',
                           help='run a Windows command through Wine with the shared leased prefix')
    wine_mode.add_argument('--wine-host', action='store_true',
                           help='run a native foreground driver with the shared leased Wine prefix')
    p.add_argument('--wineserver', help='matching wineserver executable if absent from PATH')
    p.add_argument('--timeout', type=float, default=900, help='foreground timeout in seconds')
    p.add_argument('--keep-hot', metavar='REASON',
                   help='keep retained outputs directly available instead of default cold archival')
    p.add_argument('--keep-work', metavar='REASON', help='explicit current-input or diagnostic exception')
    p.add_argument('--keep-path', action='append', default=[], metavar='REL',
                   help='retain only this work file/subtree in place after PASS (repeatable)')
    p.add_argument('--keep-reason', metavar='REASON', help='consumer and lifetime of --keep-path outputs')
    p.add_argument('--min-free-gib', type=float, default=DEFAULT_MIN_FREE_GIB,
                   help='free-space floor after expected growth (default: 5 GiB)')
    p.add_argument('--reserve-gib', type=float, default=DEFAULT_RESERVE_GIB,
                   help='expected peak growth, including builds, datasets and logs (default: 1 GiB)')
    p.add_argument('--space-exception', metavar='REASON',
                   help='record why a bounded check can use a lower space budget')
    p.add_argument('command', nargs=argparse.REMAINDER)
    args = p.parse_args()
    try:
        run_unit(args)
        return 0
    except (OSError, ValueError, subprocess.SubprocessError) as error:
        print(f'Validation refused or retained for review: {error}', file=sys.stderr)
        return 1


if __name__ == '__main__':
    raise SystemExit(main())
