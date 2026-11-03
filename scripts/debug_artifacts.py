#!/usr/bin/env python3
"""Preview/apply bounded cleanup of completed debug caches and ROM fixture copies.

All sessions, inputs, reports, errors, datasets and unknown files are preserved.
--keep protects selected sessions completely. Existing ROM-unit completion retires
only its recognized redundant fixture copies, using local_artifacts.py guards.
Plans/receipts are private. Apply accepts an unchanged reviewed plan, never a
free-form deletion list. Python 3.11+, standard library and Git for ROM units.
"""
import argparse
import contextlib
import datetime
import hashlib
import json
import os
from pathlib import Path
import re
import sys
import tempfile

from artifact_coordinator import Lease, NAME, PROTOCOL, ordinary
import local_artifacts as old

ROOT = Path(__file__).resolve().parents[1]
MAX_SCOPES, MAX_FILES, MAX_JSON = 32, 8192, 16 * 1024 * 1024
CACHE_NAME = re.compile(r'[0-9a-f]{64}(?:-[a-z0-9]+)*\.(?:dll|so|dylib)$')
LIMITS = {
    'campaign': {'folders_per_store': 8, 'cases': 32, 'json_bytes': 2097152, 'guide': 'docs/debug_campaigns.md'},
    'recent': {'folders': 8, 'events': 1024, 'event_bytes': 262144, 'per_event_bytes': 8192, 'json_bytes': 2097152, 'guide': 'docs/debug_recent.md'},
    'graphics': {'folders': 8, 'guide': 'docs/debug_graphics.md'},
    'ui': {'folders': 8, 'events': 1024, 'frames': 600, 'seconds': 30},
    'audio': {'folders': 8, 'frames': 180, 'guide': 'docs/debug_audio.md'},
    'code': {'folders': 8, 'events': 512, 'frames': 60, 'seconds': 30, 'json_bytes': 2097152},
    'gameplay': {'folders': 8, 'events': 512, 'event_bytes': 1048576, 'frames': 60, 'seconds': 30, 'json_bytes': 2097152},
    'issue': {'folders': 8, 'json_bytes': 2097152, 'guide': 'docs/debug_issues.md'},
}


def absolute(path):
    return Path(os.path.abspath(path))


def encoded(value):
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(',', ':')).encode()


def digest(value):
    return hashlib.sha256(encoded(value)).hexdigest()


def read_json(path, limit=1024 * 1024):
    ordinary(path)
    with Path(path).open('rb') as f:
        b = f.read(limit + 1)
    if len(b) > limit:
        raise ValueError(f'JSON exceeds {limit} bytes: {path}')
    value = json.loads(b)
    if not isinstance(value, dict):
        raise ValueError(f'Expected JSON object: {path}')
    return value


def info(path):
    s = ordinary(path)
    return {'path': str(path), 'size': s.st_size, 'mtime_ns': s.st_mtime_ns,
            'ctime_ns': s.st_ctime_ns, 'identity': [s.st_dev, s.st_ino],
            'allocated': s.st_blocks * 512 if hasattr(s, 'st_blocks') else s.st_size,
            'allocation_estimated': not hasattr(s, 'st_blocks')}


def walk(root):
    ordinary(root, directory=True)
    pending, files, count = [root], [], 0
    while pending:
        folder = pending.pop()
        for p in sorted(folder.iterdir()):
            if p == root / NAME:
                continue  # Our lease must not change the protected inventory.
            count += 1
            if count > MAX_FILES:
                raise ValueError(f'Scope exceeds {MAX_FILES} entries: {root}')
            if p.is_dir() and not p.is_symlink():
                ordinary(p, directory=True)
                pending.append(p)
            else:
                files.append(info(p))
    return sorted(files, key=lambda f: f['path'])


def manifest(root):
    m = read_json(root / 'debug-session.json')
    if m.get('schema') != 1 or m.get('kind') != 'ffta-debug-session':
        raise ValueError('Unsupported session manifest')
    if m.get('artifact_coordination') != PROTOCOL:
        raise ValueError('Legacy/unattested runtime coordination; inspect only')
    if not isinstance(m.get('pins'), dict) or not {'ffta.gba', 'portable.txt'} <= m['pins'].keys() or not ({'recomp', 'recomp.exe'} & m['pins'].keys()):
        raise ValueError('Session pins missing')
    for path, sha in m['pins'].items():
        parts = Path(path).parts
        if (Path(path).is_absolute() or not parts or any(p in ('.', '..') for p in parts)
                or Path(path).as_posix() != path or '\\' in path or ':' in path or not re.fullmatch('[0-9a-f]{64}', sha)):
            raise ValueError('Invalid relative session pin')
    result = read_json(root / 'last-result.json', 16 * 1024)
    if (result.get('schema') != 1 or result.get('kind') != 'ffta-debug-session-result'
            or result.get('ok') is not True):
        raise ValueError('Failed/incomplete session: preserve diagnostics')
    return m


def session_row(root, keep, held):
    row = {'root': str(root), 'kind': 'session', 'candidates': []}
    if str(root) in keep:
        return dict(row, status='kept', reason='Explicit --keep selection')
    try:
        if str(root) not in held and (root / NAME).exists():
            return dict(row, status='protected', reason=f'Active or interrupted artifact lease: {root / NAME}')
        m = manifest(root)
        with contextlib.ExitStack() as stack:
            lease = held.get(str(root)) or stack.enter_context(Lease(root, 'artifact-preview'))
            lease.check(root)
            # Recheck eligibility under the same runtime/cleaner coordination.
            m = manifest(root)
            files = walk(root)
            for f in files:
                p = Path(f['path'])
                rel = p.relative_to(root).as_posix()
                parts = p.relative_to(root).parts
                if (len(parts) == 4 and parts[:2] == ('Data', 'cache')
                        and re.fullmatch('t[0-9]+', parts[2]) and CACHE_NAME.fullmatch(parts[3])
                        and rel not in m['pins']):
                    for ancestor in p.parents:
                        if (ancestor / '.git').exists():
                            old.ensure_untracked(p, ancestor)
                            break
                    row['candidates'].append(dict(f, scope=str(root), kind='native-cache'))
            candidate_paths = {f['path'] for f in row['candidates']}
            preserved = [f for f in files if f['path'] not in candidate_paths]
            row.update(status='completed', manifest_sha256=digest(m),
                       preserved_files=len(preserved), preserved_inventory_sha256=digest(preserved),
                       preserved_bytes=sum(f['size'] for f in preserved))
    except (OSError, ValueError, TypeError) as e:
        row.update(status='protected', reason=str(e))
        row['candidates'] = []
    return row


def unit_row(root, held):
    row = {'root': str(root), 'kind': 'rom-unit', 'candidates': []}
    try:
        with contextlib.ExitStack() as stack:
            lease = held.get(str(root)) or stack.enter_context(Lease(root, 'artifact-preview'))
            lease.check(root)
            for name in old.SCRATCH_GROUPS:
                p = root / 'windows-managed-europe' / name
                if p.exists() or p.is_symlink():
                    walk(p)  # Bound the old inventory before allocating it.
            candidates, skipped = old.completion_plan([root], root=ROOT)
            if skipped:
                return dict(row, status='protected', reason='Unexpected/active contents; entire unit retained', skipped=skipped)
            row.update(status='completed', skipped=skipped)
            row['candidates'] = [dict(c, scope=str(root), kind='rom-copy') for c in candidates]
    except (OSError, ValueError, TypeError) as e:
        row.update(status='protected', reason=str(e))
    return row


def build(request, held=None):
    if (set(request) != {'sessions', 'units', 'keep'} or any(not isinstance(request[k], list) for k in request)
            or len(request['sessions']) + len(request['units']) > MAX_SCOPES):
        raise ValueError(f'Expected at most {MAX_SCOPES} explicit session/unit scopes')
    for kind in ('sessions', 'units', 'keep'):
        if request[kind] != sorted(set(request[kind])):
            raise ValueError('Scopes must be distinct and ordered')
        for path in request[kind]:
            if str(absolute(path)) != path:
                raise ValueError('Plan scopes must be absolute')
    roots = [Path(p) for p in request['sessions'] + request['units']]
    if len(set(roots)) != len(roots) or any(a != z and a.is_relative_to(z) for a in roots for z in roots):
        raise ValueError('Overlapping scopes refused')
    if not set(request['keep']) <= set(request['sessions']):
        raise ValueError('--keep must name a selected session')
    held = held or {}
    rows, leaf_files, metadata_bytes = [], 0, 0
    for kind in ('sessions', 'units'):
        for p in request[kind]:
            row = (session_row(Path(p), set(request['keep']), held) if kind == 'sessions'
                   else unit_row(Path(p), held))
            leaf_files += sum(len(c['files']) if c['kind']=='rom-copy' else 1 for c in row['candidates'])
            metadata_bytes += len(encoded(row))
            if leaf_files > MAX_FILES or metadata_bytes > MAX_JSON:
                raise ValueError('Too many candidate files or plan metadata bytes')
            rows.append(row)
    candidates = [c for row in rows for c in row['candidates']]
    result = {'schema': 1, 'kind': 'ffta-artifact-plan', 'request': request, 'scopes': rows,
              'logical_bytes': sum(c.get('bytes', c.get('size', 0)) for c in candidates),
              'allocated_bytes': sum(c['allocated'] for c in candidates),
              'candidate_count': len(candidates), 'candidate_files': leaf_files, 'capture_limits': LIMITS,
              'preservation': 'Inputs, manifests, saves, recordings, datasets, reports, errors, source preimages, unknown files and kept sessions remain. Only generated native libraries and explicit completed ROM fixture copies are eligible.'}
    if len(encoded(result)) > MAX_JSON:
        raise ValueError('Plan metadata exceeds 16 MiB')
    return result


def new_output(path, scopes):
    path = absolute(path)
    if any(path == Path(p) or path.is_relative_to(p) for p in scopes):
        raise ValueError('Plans/receipts must be outside selected scopes')
    path.parent.mkdir(parents=True, exist_ok=True)
    ordinary(path.parent, directory=True)
    return path


def publish(path, value):
    data = encoded(value) + b'\n'
    if len(data) > MAX_JSON:
        raise ValueError('Receipt exceeds 16 MiB')
    with tempfile.NamedTemporaryFile(dir=path.parent, delete=False) as f:
        temporary = Path(f.name)
        try:
            f.write(data); f.flush(); os.fsync(f.fileno())
            f.close(); os.replace(temporary, path)
        finally:
            temporary.unlink(missing_ok=True)


def preview(request, output):
    result = build(request)
    out = new_output(output, request['sessions'] + request['units'])
    out.mkdir()
    (out / '.gitignore').write_text('*\n')
    publish(out / 'plan.json', result)
    text = encoded(result).decode().replace('<', '\\u003c')
    (out / 'index.html').write_text(Path(__file__).with_suffix('.html').read_text().replace('/*PLAN*/', text, 1))
    return result


def apply(plan_path, receipt_path):
    plan = read_json(plan_path, MAX_JSON)
    if plan.get('schema') != 1 or plan.get('kind') != 'ffta-artifact-plan':
        raise ValueError('Unsupported artifact plan')
    request = plan['request']
    receipt_path = new_output(receipt_path, request['sessions'] + request['units'])
    # Exclusive creation preserves every old receipt, including failed attempts.
    with receipt_path.open('x') as f:
        f.write('{}\n'); f.flush(); os.fsync(f.fileno())
    receipt = {'schema': 1, 'kind': 'ffta-artifact-receipt', 'plan_sha256': digest(plan),
               'started_at': datetime.datetime.now(datetime.timezone.utc).isoformat(),
               'status': 'preflight', 'removed': [], 'pending': None}
    publish(receipt_path, receipt)
    try:
        validated = build(request)
        if validated != plan:
            raise ValueError('Plan changed since preview; no removal started. Create a new preview')
        with contextlib.ExitStack() as stack:
            held = {}
            for scope in sorted(validated['scopes'], key=lambda s: s['root']):
                if scope['status'] == 'completed':
                    held[scope['root']] = stack.enter_context(Lease(scope['root'], 'artifact-apply'))
            fresh = build(request, held)
            if fresh != plan:
                raise ValueError('Plan changed since preview; no removal started. Create a new preview')
            for scope in plan['scopes']:
                for item in scope['candidates']:
                    held[item['scope']].check(item['scope'])
                    receipt.update(status='removing', pending=item['path'])
                    publish(receipt_path, receipt)
                    if item['kind'] == 'native-cache':
                        if info(Path(item['path'])) != {k:v for k,v in item.items() if k not in ('scope', 'kind')}:
                            raise ValueError('Candidate changed before removal')
                        Path(item['path']).unlink()
                    else:
                        old.ensure_untracked(Path(item['path']), ROOT)
                        old.remove_candidate(dict(item, kind='fixture'))
                    receipt['removed'].append(item['path'])
                    receipt.update(status='applying', pending=None)
                    publish(receipt_path, receipt)
        receipt.update(status='complete', finished_at=datetime.datetime.now(datetime.timezone.utc).isoformat())
        publish(receipt_path, receipt)
        return receipt
    except BaseException as error:
        receipt.update(status='failed', error=str(error))
        publish(receipt_path, receipt)
        raise


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest='command', required=True)
    p = sub.add_parser('preview')
    p.add_argument('--session', type=Path, action='append', default=[])
    p.add_argument('--sessions-root', type=Path)
    p.add_argument('--keep', type=Path, action='append', default=[])
    p.add_argument('--unit', type=Path, action='append', default=[])
    p.add_argument('--out', type=Path, required=True, help='new private report directory')
    p = sub.add_parser('apply')
    p.add_argument('plan', type=Path)
    p.add_argument('--receipt', type=Path, required=True, help='new durable receipt file')
    args = parser.parse_args(argv)
    try:
        if args.command == 'preview':
            if args.sessions_root:
                ordinary(args.sessions_root, directory=True)
                children = sorted(args.sessions_root.iterdir())
                if len(children) > MAX_SCOPES:
                    raise ValueError('Too many sessions; select explicit --session paths')
                args.session += [p for p in children if p.is_dir()]
            request = {k: sorted(set(str(absolute(p)) for p in paths)) for k, paths in
                       [('sessions', args.session), ('units', args.unit), ('keep', args.keep)]}
            result = preview(request, args.out)
            print(json.dumps({'schema':1,'kind':'ffta-artifact-preview','candidates':result['candidate_count'],
                              'logical_bytes':result['logical_bytes'],'allocated_bytes':result['allocated_bytes'],
                              'report':str(absolute(args.out)/'index.html')}))
        else:
            result = apply(args.plan, args.receipt)
            print(json.dumps({'schema':1,'kind':'ffta-artifact-applied','removed':len(result['removed']),
                              'receipt':str(absolute(args.receipt))}))
        return 0
    except (OSError, ValueError, TypeError, KeyError, old.subprocess.CalledProcessError) as error:
        print(f'Artifact manager: {error}', file=sys.stderr)
        return 1


if __name__ == '__main__':
    raise SystemExit(main())
