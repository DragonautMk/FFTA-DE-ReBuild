#!/usr/bin/env python3
"""Explicit targeted campaigns over pinned local replay evidence. No default sweep."""
import argparse
import contextlib
import hashlib
import json
import os
from pathlib import Path
import re
import sys
import time

from artifact_coordinator import Lease, ordinary

MAX_CASES, MAX_MANIFEST, MAX_REPORT = 32, 256 * 1024, 2 * 1024 * 1024
SHA = re.compile(r'[0-9a-f]{64}')
ID = re.compile(r'[A-Za-z0-9][A-Za-z0-9._-]{0,63}')
IDENTITIES = ('runtime_sha256', 'rom_sha256', 'dataset_sha256', 'input_sha256', 'library_sha256')
POSITIVE = {'passed', 'matched', 'captured'}
NEGATIVE = {'diverged', 'error', 'timeout', 'interrupted', 'not-reached'}


def encoded(value):
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(',', ':')).encode()


def read(path, limit):
    ordinary(path)
    with Path(path).open('rb') as f:
        data = f.read(limit + 1)
    if len(data) > limit:
        raise ValueError(f'File exceeds {limit} bytes: {path}')
    return data


def json_object(pairs):
    result = {}
    for key, item in pairs:
        if key in result:
            raise ValueError('Duplicate JSON key: ' + key)
        result[key] = item
    return result


def invalid_constant(name):
    raise ValueError('Nonfinite JSON value: ' + name)


def schema1(data):
    return type(data.get('schema')) is int and data['schema'] == 1


def value(path, limit):
    data = read(path, limit)
    result = json.loads(data, object_pairs_hook=json_object, parse_constant=invalid_constant)
    if not isinstance(result, dict):
        raise ValueError('Expected JSON object')
    return result, hashlib.sha256(data).hexdigest()


def relative(base, name):
    if not isinstance(name, str) or not name or len(name) > 2048 or Path(name).is_absolute() or '\\' in name or ':' in name:
        raise ValueError('Use a portable relative path, with / separators')
    return Path(os.path.abspath(base / name))


def pinned(base, ref, limit):
    if not isinstance(ref, dict) or set(ref) != {'path', 'sha256'} or not SHA.fullmatch(str(ref['sha256'])):
        raise ValueError('Evidence needs a relative path and SHA256')
    path = relative(base, ref['path'])
    data, digest = value(path, limit)
    if digest != ref['sha256']:
        raise ValueError(f'Changed pinned JSON: {ref["path"]}')
    return path, data, digest


def load(path):
    path = Path(os.path.abspath(path))
    manifest, digest = value(path, MAX_MANIFEST)
    if not schema1(manifest) or manifest.get('kind') != 'ffta-debug-campaign':
        raise ValueError('Unsupported campaign schema/kind')
    if not isinstance(manifest.get('label'), str) or not 1 <= len(manifest['label']) <= 200:
        raise ValueError('Campaign label needs 1..200 characters')
    cases = manifest.get('cases')
    if not isinstance(cases, list) or not 1 <= len(cases) <= MAX_CASES:
        raise ValueError('Campaign needs 1..32 explicit cases')
    seen = set()
    for case in cases:
        if not isinstance(case, dict) or not isinstance(case.get('id'), str) or not ID.fullmatch(case['id']) or case['id'] in seen:
            raise ValueError('Case IDs must be unique portable labels (1..64 characters)')
        seen.add(case['id'])
        for key in ('ds', 'families', 'transitions'):
            tags = case.get(key, [])
            if not isinstance(tags, list) or len(tags) > 32 or any(not isinstance(t, str) or not 1 <= len(t) <= 128 for t in tags):
                raise ValueError('Tags need bounded string lists')
        if case.get('profile') is None and case.get('evidence') is not None:
            raise ValueError('Evidence requires a declared pinned profile')
    return path, manifest, digest


def selection(cases, ids=(), ds=(), families=(), transitions=()):
    # OR within each filter, AND between supplied filters. Never default to all.
    groups = [(set(ids), 'id'), (set(ds), 'ds'), (set(families), 'families'), (set(transitions), 'transitions')]
    if not any(wanted for wanted, _ in groups):
        raise ValueError('Select at least one --case / --ds / --family / --transition; no automatic all')
    unknown = set(ids) - {case['id'] for case in cases}
    if unknown:
        raise ValueError('Unknown case IDs: ' + ', '.join(sorted(unknown)))
    return {case['id'] for case in cases if all(not wanted or bool(wanted & ({case[key]} if key == 'id' else set(case.get(key, [])))) for wanted, key in groups)}


def integer(v):
    return isinstance(v, int) and not isinstance(v, bool) and v >= 0


def profile_valid(profile):
    if not schema1(profile) or profile.get('kind') != 'ffta-debug-replay-profile' or not isinstance(profile.get('case'), str) or not 1 <= len(profile['case']) <= 200:
        raise ValueError('Unsupported replay profile')
    for key in (*IDENTITIES, 'start_sha256', 'end_sha256', 'bios_sha256'):
        if key not in profile or not ((isinstance(profile[key], str) and SHA.fullmatch(profile[key])) or (key in ('dataset_sha256', 'library_sha256') and profile[key] is None)):
            raise ValueError('Invalid profile identity: ' + key)
    if not integer(profile.get('frames')) or not 1 <= profile['frames'] <= 360000 or not integer(profile.get('final_clock')):
        raise ValueError('Invalid profile frame/clock')
    for key in ('audit', 'capture', 'comparison'):
        if profile.get(key) is not None and not isinstance(profile[key], dict):
            raise ValueError('Invalid profile settings: ' + key)


def audit_summary(base, ref, profile, result):
    if ref is None:
        return {'state': 'unavailable', 'meaning': 'No pinned audit; this is not zero reads'}
    path, audit, digest = pinned(base, ref, 16 * 1024 * 1024)
    if not schema1(audit) or audit.get('kind') != 'ffta-rom-read-audit' or audit.get('run') != result['run'] or audit.get('case') != profile['case']:
        raise ValueError('Audit run/case/schema mismatch')
    for key in ('runtime_sha256', 'rom_sha256', 'input_sha256'):
        if audit.get(key) != profile.get(key):
            raise ValueError('Audit identity mismatch: ' + key)
    if audit.get('dataset_manifest_sha256') != profile.get('dataset_sha256'):
        raise ValueError('Audit dataset mismatch')
    settings = profile.get('audit')
    window, filters = audit.get('window', {}), audit.get('filter', {})
    if not isinstance(window, dict) or not isinstance(filters, dict):
        raise ValueError('Invalid audit window/filter')
    if not isinstance(settings, dict) or filters.get('preset') != settings.get('preset') or filters.get('unique_event_limit') != settings.get('limit') or filters.get('physical_ranges') != settings.get('ranges') or window.get('relative_from') != settings.get('from') or window.get('relative_to_exclusive') != settings.get('to'):
        raise ValueError('Audit selected window/filter mismatch')
    if window.get('replay_completed') != result['completed'] or window.get('replay_outcome') != result['outcome']:
        raise ValueError('Audit replay outcome/completion mismatch')
    if result.get('audit_truncated') != audit.get('truncated') or result.get('audit_window_complete') != window.get('complete'):
        raise ValueError('Audit/result completeness mismatch')
    if not integer(window.get('relative_from')) or not integer(window.get('relative_to_exclusive')) or not window['relative_from'] < window['relative_to_exclusive'] <= profile['frames']:
        raise ValueError('Invalid audit frame window')
    matched, dropped = audit.get('matched_requests'), audit.get('dropped_requests')
    if not integer(matched) or not integer(dropped) or dropped > matched or not isinstance(audit.get('truncated'), bool) or not isinstance(window.get('complete'), bool):
        raise ValueError('Invalid audit counters/completeness')
    complete = window['complete'] and not audit['truncated'] and dropped == 0
    state = 'observed' if matched else ('zero-in-selected-window' if complete else 'partial-zero')
    summary = {'path': str(path), 'sha256': digest, 'state': state, 'matched_requests': matched,
               'dropped_requests': dropped, 'truncated': audit['truncated'], 'window': window,
               'filter': filters, 'meaning': 'Only instrumented readers in this window/filter; global feature non-use is UNKNOWN'}
    if len(encoded(summary)) > 64 * 1024:
        raise ValueError('Audit summary exceeds 64 KiB')
    return summary


def inspect(base, case, selected):
    row = {key: case.get(key, []) for key in ('id', 'ds', 'families', 'transitions')}
    row.update(selected=selected, outcome='not-executed', evidence='none', reads={'state': 'unavailable', 'meaning': 'Not observed; never zero/global non-use'})
    if not selected:
        return dict(row, reason='Outside explicit selection; evidence not opened')
    if case.get('profile') is None:
        return dict(row, reason='No prepared profile; preserve missing-case status')
    try:
        profile_path, profile, digest = pinned(base, case['profile'], 128 * 1024)
        profile_valid(profile)
        row.update(profile=str(profile_path), profile_sha256=digest, profile_case=profile['case'])
        evidence = case.get('evidence')
        if evidence is None:
            return dict(row, reason='Prepared profile; not executed by this report')
        if not isinstance(evidence, dict) or set(evidence) - {'result', 'audit'} or 'result' not in evidence:
            raise ValueError('Evidence needs result and optional audit pins')
        # New profiles coordinate with cleanup/runtime. Old retained metadata has
        # pinned bytes, but no retroactive claim that its producer used the lease.
        session_path = profile_path.parent / 'debug-session.json'
        session, _ = value(session_path, 1024 * 1024) if session_path.exists() else ({}, None)
        coordinated = session.get('artifact_coordination') == 'directory-v1'
        with (Lease(profile_path.parent, 'campaign-evidence-reader') if coordinated else contextlib.nullcontext()):
            result_path, result, result_digest = pinned(base, evidence['result'], 128 * 1024)
            if not schema1(result) or result.get('kind') != 'ffta-debug-replay-result' or result.get('case') != profile['case'] or not isinstance(result.get('run'), str) or not result['run']:
                raise ValueError('Replay result schema/case/run mismatch')
            for key in IDENTITIES:
                if key not in result or result[key] != profile.get(key):
                    raise ValueError('Result identity mismatch: ' + key)
            frames = (profile.get('comparison') or {}).get('frames', (profile.get('capture') or {}).get('limit_frames', profile.get('frames')))
            if not integer(frames) or not 1 <= frames <= profile['frames']:
                raise ValueError('Invalid selected profile frame count')
            if not integer(result.get('completed')) or not integer(result.get('frames')) or result['frames'] != frames or result['completed'] > result['frames']:
                raise ValueError('Result selected/completed frame mismatch')
            terminal = result.get('outcome')
            if terminal not in POSITIVE | NEGATIVE | {'incompatible'}:
                raise ValueError('Result is not a supported terminal outcome')
            if terminal in POSITIVE and result['completed'] != frames:
                raise ValueError('Incomplete positive result')
            if terminal == 'passed' and not profile.get('comparison') and not profile.get('capture'):
                if result.get('state_sha256') != profile.get('end_sha256') or result.get('clock') != profile.get('final_clock'):
                    raise ValueError('Passed endpoint/clock mismatch')
            row['reads'] = audit_summary(base, evidence.get('audit'), profile, result)
            # Profile may have changed while acquiring the evidence scope.
            pinned(base, case['profile'], 128 * 1024)
        return dict(row, outcome='positive' if terminal in POSITIVE else ('incompatible' if terminal == 'incompatible' else 'negative'),
                    detail_outcome=terminal, evidence='reused-pinned', result=str(result_path), result_sha256=result_digest,
                    run=result['run'], completed=result['completed'], frames=frames,
                    reason=str(result.get('message', ''))[:2048],
                    coordination='directory-v1' if coordinated else 'legacy pinned metadata; active-run exclusion unknown',
                    authentication='Pinned metadata and reported identities match; payload availability/current runtime compatibility not revalidated')
    except (ValueError, OSError, TypeError, KeyError, RecursionError) as error:
        return dict(row, outcome='incompatible', reason=str(error)[:2048])


def output_dir(path):
    path = Path(os.path.abspath(path))
    for parent in reversed([path.parent, *path.parent.parents]):
        if not parent.exists():
            parent.mkdir()
        ordinary(parent, directory=True)
    for count, _ in enumerate(path.parent.iterdir(), 1):
        if count >= 8:
            raise ValueError('Eight campaign outputs already retained here; archive explicitly')
    path.mkdir()
    (path / '.gitignore').write_text('*\n')
    return path


def publish(path, value, name):
    data = encoded(value)
    if len(data) > MAX_REPORT:
        raise ValueError('Campaign JSON exceeds 2 MiB')
    (path / name).write_bytes(data)


def report(args):
    path, manifest, digest = load(args.manifest)
    chosen = selection(manifest['cases'], args.case, args.ds, args.family, args.transition)
    rows = [inspect(path.parent, case, case['id'] in chosen) for case in manifest['cases']]
    result = {'schema': 1, 'kind': 'ffta-debug-campaign-report', 'label': manifest['label'], 'manifest_sha256': digest,
              'manifest': str(path), 'created_unix_ms': time.time_ns() // 1000000,
              'selection': {'case': args.case, 'ds': args.ds, 'family': args.family, 'transition': args.transition},
              'selected': len(chosen), 'cases': rows,
              'scope': 'Pinned retained evidence and explicit worklist only. Report executes zero guest frames. Tags are declared labels, not verified reachability. Global feature non-use is UNKNOWN. No DS/ROM/native acceptance promotion.',
              'limits': {'cases': MAX_CASES, 'folders_per_store': 8, 'json_bytes': MAX_REPORT, 'audit_input_bytes': 16 * 1024 * 1024}}
    # Use relative report links, preserving whole-workspace relocation.
    out = output_dir(args.out)
    for row in rows:
        for key in ('profile', 'result'):
            if key in row:
                row[key] = os.path.relpath(row[key], out).replace(os.sep, '/')
        if 'path' in row['reads']:
            row['reads']['path'] = os.path.relpath(row['reads']['path'], out).replace(os.sep, '/')
    result['manifest'] = os.path.relpath(path, out).replace(os.sep, '/')
    publish(out, result, 'campaign.json')
    html = Path(__file__).with_suffix('.html').read_text()
    (out / 'index.html').write_text(html.replace('/*CAMPAIGN*/', encoded(result).decode().replace('<', '\\u003c'), 1))
    return result


def run_case(args, executor=os.execv):
    path, manifest, digest = load(args.manifest)
    selection(manifest['cases'], [args.case])
    case = next(c for c in manifest['cases'] if c['id'] == args.case)
    if not args.reason.strip() or len(args.reason) > 2048:
        raise ValueError('Record a concrete 1..2048 character selection reason')
    has_result = case.get('profile') is not None and (relative(path.parent, case['profile']['path']).parent / 'Data/config/replay/result.json').exists()
    if (case.get('evidence') is not None or has_result) and not (args.reopen_reason and args.reopen_reason.strip()):
        raise ValueError('Existing proof is reused by default; --reopen-reason is required for an explicit repeat')
    if args.reopen_reason and len(args.reopen_reason) > 2048:
        raise ValueError('Reopening reason exceeds 2048 characters')
    profile_path, profile, _ = pinned(path.parent, case.get('profile'), 128 * 1024)
    profile_valid(profile)
    names = [p for p in [profile_path.parent / 'recomp.exe', profile_path.parent / 'recomp'] if p.exists()]
    if len(names) != 1:
        raise ValueError('Profile needs exactly one pinned runtime')
    runtime = names[0]
    if hashlib.sha256(read(runtime, 256 * 1024 * 1024)).hexdigest() != profile.get('runtime_sha256'):
        raise ValueError('Changed profile runtime')
    if args.wine:
        if os.name == 'nt' or runtime.suffix.lower() != '.exe':
            raise ValueError('--wine is for Windows profiles on a POSIX host')
        launcher = Path(os.path.abspath(args.wine)); ordinary(launcher)
        command = [str(launcher), str(runtime), 'debug-replay', 'run', 'Z:' + str(profile_path).replace('/', '\\'), '--json']
    else:
        if os.name != 'nt' and runtime.suffix.lower() == '.exe':
            raise ValueError('Windows profile needs explicit --wine /path/to/wine64')
        command = [str(runtime), 'debug-replay', 'run', str(profile_path), '--json']
    out = output_dir(args.out)
    intent = {'schema': 1, 'kind': 'ffta-debug-campaign-intent', 'case': case['id'], 'manifest_sha256': digest,
              'profile_sha256': case['profile']['sha256'], 'runtime_sha256': profile['runtime_sha256'],
              'reason': args.reason, 'reopen_reason': args.reopen_reason, 'command': command,
              'state': 'dispatch-intent, not a completed result', 'created_unix_ms': time.time_ns() // 1000000}
    publish(out, intent, 'intent.json')
    # Reuse the existing supervised runner for leases, bounds, progress, timeout
    # and cancellation. No new parent/worker supervision or batch sweep.
    print('Campaign intent receipt: ' + str(out / 'intent.json'), file=sys.stderr, flush=True)
    try:
        executor(command[0], command)
    except OSError as error:
        intent.update(state='dispatch-error', error=str(error)[:2048]);publish(out, intent, 'intent.json')
        raise
    return intent


def parser():
    p = argparse.ArgumentParser(description=__doc__)
    sub = p.add_subparsers(dest='action', required=True)
    q = sub.add_parser('report', help='Read selected pinned evidence only; never replay')
    q.add_argument('manifest', type=Path)
    for flag in ('case', 'ds', 'family', 'transition'):
        q.add_argument('--' + flag, action='append', default=[])
    q.add_argument('--out', type=Path, required=True)
    q = sub.add_parser('run-case', help='Explicitly dispatch ONE pinned profile through existing supervisor')
    q.add_argument('manifest', type=Path);q.add_argument('case')
    q.add_argument('--reason', required=True);q.add_argument('--reopen-reason')
    q.add_argument('--wine', type=Path);q.add_argument('--out', type=Path, required=True)
    return p


def main():
    try:
        args = parser().parse_args()
        result = report(args) if args.action == 'report' else run_case(args)
        print(json.dumps({'schema': 1, 'ok': True, 'out': str(args.out), 'selected': result.get('selected')}, ensure_ascii=False))
        return 0
    except (ValueError, OSError, TypeError, KeyError, RecursionError) as error:
        print(json.dumps({'schema': 1, 'ok': False, 'error': str(error)}, ensure_ascii=False), file=sys.stderr)
        return 1


if __name__ == '__main__':
    sys.exit(main())
