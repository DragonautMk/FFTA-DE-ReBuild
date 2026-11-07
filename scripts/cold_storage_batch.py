"""Automatic discovery of completed local artifacts for verified cold storage.

This is storage maintenance, not a validation runner. Existing result receipts
are consulted without executing their commands or rewriting their outcomes.
"""
import fnmatch
import hashlib
import json
import re
import time
import uuid
from pathlib import Path

import cold_storage as cold
from artifact_coordinator import Lease, NAME, ordinary


# Current resources and unresolved historical work are deliberately kept hot.
EXCLUDED = (
    'target/debug', 'target/release', 'target/x86_64-*', 'target/aarch64-*',
    'target/cold-storage*', 'target/local-*', 'target/u31-*', 'target/desktop-perf*',
    'target/debug-saves*', 'target/extended-saves*',
    'target/debug-tools-v2-migration', 'target/debug-tools-v2-library',
    'target/rom-data-6e84e0-e132-cargo-target',
    'target/validation/units/e132-*', 'target/validation/units/e131-*',
    'target/validation/units/menu-atlas-linux-docker-final-*',
    'target/validation/units/cold-storage-*',
)
PRESERVE = re.compile(
    r'preimage|snapshot|^source$|compiled.sources|^original|first.enospc|'
    r'failed|failure|interrupted|timeout|attempt|negative', re.I)
BOOKKEEPING = re.compile(r'preserv|guard|hash|cleanup|retire|memory|selection|inventory|source', re.I)
RESULT_NAME = re.compile(r'^(?:result|final-summary|test-results|windows-.+-result|gui.*(?:close|result))\.json$')
EXIT_KEYS = ('exit', 'exit_code', 'returncode', 'normal_exit')


def read_result(path):
    ordinary(path)
    if path.stat().st_size > 2 * cold.BLOCK:
        raise ValueError('Result exceeds the bounded metadata reader')
    return json.loads(path.read_text())


def failed(value):
    """Conflicting failures prevent automatic adoption of a historical scope."""
    if isinstance(value, list):
        return any(failed(v) for v in value)
    if not isinstance(value, dict):
        return False
    status = str(value.get('status', '')).upper()
    if status.startswith(('FAIL', 'PARTIAL', 'ERROR', 'INTERRUPTED', 'TIMEOUT')):
        return True
    if any(k in value and value[k] != 0 for k in EXIT_KEYS):
        return True
    return any(failed(value[k]) for k in ('results', 'tests', 'checks') if k in value)


def passed(value):
    if failed(value):
        return False
    if isinstance(value, list):
        return bool(value) and all(passed(v) for v in value)
    return isinstance(value, dict) and (
        value.get('status') == 'PASS' or value.get('closed') is True
        or any(type(value.get(k)) is int and value[k] == 0 for k in EXIT_KEYS))


def evidence(scope, managed):
    if (scope / NAME).exists():
        raise ValueError('Active or interrupted scope lease')
    if managed:
        p = scope / 'unit-result.json'
        value = read_result(p)
        if not isinstance(value, dict) or value.get('status') != 'PASS' or value.get('cleanup') != 'kept':
            raise ValueError('No successful retained managed work')
        if value.get('keep_hot_reason'):
            raise ValueError('Explicit --keep-hot retention')
        return [p]
    selected = []
    for p in sorted(scope.glob('*.json')):
        if not any(s in p.name for s in ('result', 'summary', 'close')) or BOOKKEEPING.search(p.name):
            continue
        value = read_result(p)
        if failed(value):
            raise ValueError(f'Conflicting or incomplete outcome: {p.name}')
        if RESULT_NAME.fullmatch(p.name) and passed(value) and 'build' not in p.name:
            selected.append(p)
    if not selected:
        raise ValueError('No recognized successful completion receipt')
    return selected


def discover(store, minimum, excluded):
    root = store.root
    catalog = [root / d['source'] for _, d in store.entries()]
    candidates, skipped = [], []

    def skip(p, reason):
        skipped.append({'source': p.relative_to(root).as_posix(), 'reason': reason})

    def exclusion(p):
        rel = p.relative_to(root).as_posix()
        return any(fnmatch.fnmatchcase(rel, pat) for pat in (*EXCLUDED, *excluded))

    def collect(p, scope, results):
        if p.is_symlink() or exclusion(p) or PRESERVE.search(p.name):
            skip(p, 'Preserved path or symbolic-link root'); return
        if p.is_dir() and (p / NAME).exists():
            skip(p, 'Active or interrupted subtree lease'); return
        if any(p == q or q in p.parents for q in catalog):
            skip(p, 'Already cataloged; manual restorations stay available'); return
        # Never consume completion receipts, source preimages, current paths or
        # ancestors of existing archives. Split mixed trees into eligible children.
        split = any(p in q.parents for q in catalog + results)
        if p.is_dir() and not split:
            for child in p.rglob('*'):
                if child.name == NAME:
                    skip(p, 'Nested active or interrupted lease'); return
                if exclusion(child) or PRESERVE.search(child.name):
                    split = True; break
        if not split and p not in results:
            try:
                store.source(p)
                rows = cold.scan(p, hashes=False)
                amount = cold.size(rows, 'allocated')
                if amount >= minimum:
                    candidates.append({'source': p.relative_to(root).as_posix(),
                                       'scope': scope.relative_to(root).as_posix(),
                                       'allocated_bytes': amount, 'logical_bytes': cold.size(rows),
                                       'stat_inventory_sha256': cold.inventory_digest(rows),
                                       'completion': [{'path': q.relative_to(root).as_posix(),
                                                       'sha256': cold.digest(q)} for q in results]})
                return
            except (OSError, ValueError) as error:
                skip(p, str(error))
        if p.is_dir():
            for child in sorted(p.iterdir()):
                collect(child, scope, results)

    scopes = [(p, False) for p in sorted((root / 'target').iterdir())
              if p.is_dir() and p.name != 'validation']
    units = root / 'target/validation/units'
    if units.is_dir():
        scopes.extend((p, True) for p in sorted(units.iterdir()) if p.is_dir())
    for scope, managed in scopes:
        if scope.is_symlink() or exclusion(scope):
            skip(scope, 'Current, shared, private, suspended or maintenance scope'); continue
        try:
            ordinary(scope, directory=True)
            results = evidence(scope, managed)
            if managed:
                if (scope / 'work').is_dir():
                    collect(scope / 'work', scope, results)
            else:
                for p in sorted(scope.iterdir()):
                    collect(p, scope, results)
        except (OSError, ValueError) as error:
            skip(scope, str(error))
    candidates.sort(key=lambda c: c['allocated_bytes'], reverse=True)
    return candidates, skipped


def apply_candidate(store, item, receipt_dir):
    source, rel = store.source(item['source'])
    scope = store.root / cold.relative(item['scope'])

    def unchanged():
        for result in item['completion']:
            p = store.root / cold.relative(result['path'])
            if cold.digest(p) != result['sha256'] or not passed(read_result(p)):
                raise ValueError(f'Completion evidence changed: {p}')

    unchanged()
    with Lease(scope, 'cold-storage-batch-source'):
        idle = cold.host_activity(scope)
        rows = cold.scan(source, hashes=False)
        if cold.inventory_digest(rows) != item['stat_inventory_sha256']:
            raise ValueError('Source changed since batch discovery')
        # Normalize recognized legacy result formats without modifying evidence.
        # The original receipt hashes remain bound and are rechecked before retire.
        proof = dict(item, status='PASS', idle=idle, batch=receipt_dir.relative_to(store.root).as_posix())
        cold.save(receipt_dir / (hashlib.sha256(rel.encode()).hexdigest() + '.json'), proof)

        def recheck():
            unchanged()
            return cold.host_activity(scope)

        unchanged()
        return store.pack(source, reason='Completed artifact automatically retained in cold storage',
                          proof=proof, before_retire=recheck)


def run(store, args):
    if args.min_mib <= 0:
        raise ValueError('--min-mib must be positive')
    base = store.root / 'target/cold-storage-batches'
    base.mkdir(exist_ok=True)
    ordinary(base, directory=True)
    output = base / (time.strftime('%Y%m%d-%H%M%S') + '-' + uuid.uuid4().hex[:8])
    output.mkdir()
    (output / 'reviews').mkdir()
    report = {'schema': 1, 'status': 'DISCOVERING', 'apply': args.apply,
              'minimum_bytes': args.min_mib * cold.BLOCK,
              'exclusions': [*EXCLUDED, *args.exclude], 'completed': [], 'errors': []}
    report_path = output / 'batch.json'
    cold.save(report_path, report)
    print(f'Batch receipt: {report_path.relative_to(store.root)}', flush=True)
    candidates, skipped = discover(store, report['minimum_bytes'], args.exclude)
    report.update(status='APPLYING' if args.apply else 'PREVIEW', candidates=candidates,
                  skipped=skipped, selected_allocated_bytes=sum(c['allocated_bytes'] for c in candidates))
    cold.save(report_path, report)
    print(f'Selected {len(candidates)} paths, {report["selected_allocated_bytes"] / cold.GIB:.2f} GiB', flush=True)
    if not args.apply:
        return 0
    for i, item in enumerate(candidates, 1):
        print(f'[{i}/{len(candidates)}] {item["source"]}', flush=True)
        try:
            data = apply_candidate(store, item, output / 'reviews')
            entry = store.base / hashlib.sha256(item['source'].encode()).hexdigest()
            metadata = sum(p.stat().st_blocks * 512 for p in entry.rglob('*')
                           if p.is_file() and p.name != 'payload.tar.zst')
            report['completed'].append({k: data[k] for k in (
                'source', 'state', 'source_allocated_bytes', 'archive_allocated_bytes', 'archive_sha256')})
            report['completed'][-1]['metadata_allocated_bytes'] = metadata
        except (OSError, ValueError, cold.tarfile.TarError, cold.subprocess.SubprocessError) as error:
            report['errors'].append({'source': item['source'], 'reason': str(error)})
            print(f'Retained: {error}', flush=True)
            # An interrupted publication must be resolved before another run.
            entry = store.base / hashlib.sha256(item['source'].encode()).hexdigest()
            if entry.exists():
                report['status'] = 'NEEDS_REVIEW'
                cold.save(report_path, report)
                return 1
        cold.save(report_path, report)
    report['status'] = 'PASS' if not report['errors'] else 'PARTIAL'
    report['reclaimed_file_bytes'] = sum(d['source_allocated_bytes'] - d['archive_allocated_bytes']
                                       - d['metadata_allocated_bytes'] for d in report['completed'])
    cold.save(report_path, report)
    print(f'{report["status"]}: {len(report["completed"])} archived; '
          f'{report["reclaimed_file_bytes"] / cold.GIB:.2f} GiB reclaimed', flush=True)
    return 0 if report['status'] == 'PASS' else 1
