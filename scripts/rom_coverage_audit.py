"""Bounded, metadata-only DBG06 audit import. Never promotes scene acceptance."""
import hashlib
import json
import pathlib
import re

from rom_coverage_ranges import checked_span, complement, intersection, length, union

READERS = ('Bus', 'AudioValue', 'SpritePeek', 'SpriteList', 'ObjSourceTable', 'KeyboardWidths')
SERVICES = ('Typed', 'Raw', 'Unclassified')
SCENE_LAYERS = ('scene_observed', 'scene_typed', 'scene_raw', 'scene_unclassified',
                'scene_shared', 'scene_outside_identified')
MAX_FILE = 32 * 1024 * 1024


def require(ok, message):
    if not ok:
        raise ValueError('ROM audit: ' + message)


def integer(value, minimum=0, maximum=2**64 - 1):
    require(type(value) is int and minimum <= value <= maximum, 'invalid integer')
    return value


def digest(value, optional=False):
    require(optional and value is None or isinstance(value, str) and
            re.fullmatch('[0-9a-f]{64}', value), 'invalid SHA-256')
    return value


def label(value):
    require(isinstance(value, str) and 0 < len(value) <= 128 and
            not any(ord(c) < 32 for c in value), 'invalid case/run label')
    return value


def bounded_json(path):
    with pathlib.Path(path).open('rb') as stream:
        raw = stream.read(MAX_FILE + 1)
    require(len(raw) <= MAX_FILE, 'input exceeds 32 MiB')
    data = json.loads(raw)
    require(isinstance(data, dict), 'expected object')
    return data, hashlib.sha256(raw).hexdigest(), len(raw)


def spans(value, size):
    require(isinstance(value, list) and len(value) <= 250000, 'invalid interval list')
    result = []
    for span in value:
        require(isinstance(span, list) and len(span) == 2, 'invalid interval')
        result.append(checked_span(*span, size))
    require(result == union(result), 'intervals must be sorted, merged and unique')
    return result


def inventory(section, actual, size):
    require(isinstance(section, dict), 'missing interval inventory')
    require(spans(section['physical_intervals'], size) == actual and
            integer(section['distinct_bytes']) == length(actual), 'inventory differs from events')


def read_audit(path, report):
    data, identity, byte_size = bounded_json(path)
    require(type(data['schema']) is int and data['schema'] == 1 and
            data['kind'] == 'ffta-rom-read-audit', 'unsupported schema/kind')
    require(digest(data['rom_sha256']) == report['rom_sha256'], 'ROM identity mismatch')
    size = report['totals']['rom']
    label(data['case']); label(data['run'])
    for name in ('runtime_sha256', 'input_sha256'):
        digest(data[name])
    digest(data['dataset_manifest_sha256'], optional=True)
    window, filters = data['window'], data['filter']
    require(isinstance(window, dict) and isinstance(filters, dict), 'missing window/filter')
    start = integer(window['relative_from']); end = integer(window['relative_to_exclusive'])
    completed = integer(window['replay_completed']); entered = integer(window['frames_entered'])
    require(start < end and type(window['complete']) is bool and
            window['complete'] == (completed >= end) and
            entered == max(0, min(completed, end) - start), 'inconsistent frame window')
    require(window['replay_outcome'] in ('passed', 'diverged', 'invalid', 'timeout',
                                       'cancelled', 'interrupted'), 'invalid replay outcome')
    require(filters['preset'] in ('all', 'bus', 'host'), 'invalid collector preset')
    ranges = spans(filters['physical_ranges'], size)
    limit = integer(filters['unique_event_limit'], 1, 50000)
    collector = data['coverage']
    require(isinstance(collector, dict), 'missing collector coverage')
    require(collector['readers'] == list(READERS), 'unsupported collector inventory')
    for name in ('absence', 'fetches', 'instruction_scope'):
        require(isinstance(collector[name], str) and len(collector[name]) <= 2048,
                'missing collector limitations')
    require(isinstance(collector['not_observed'], list) and
            len(collector['not_observed']) <= 32 and all(isinstance(s, str) and len(s) <= 2048
            for s in collector['not_observed']), 'invalid collector exclusions')
    events = data['events']
    require(isinstance(events, list) and len(events) <= limit and
            integer(data['retained_events']) == len(events), 'invalid retained event count')
    by_reader = {r: [] for r in READERS}; by_service = {s: [] for s in SERVICES}
    seen, requests = set(), 0
    for event in events:
        require(isinstance(event, dict), 'invalid event')
        key = event['key']
        require(isinstance(key, dict) and isinstance(key['read'], dict), 'invalid read key')
        read = key['read']
        reader, service = read['reader'], key['service']
        require(reader in READERS and service in SERVICES and reader in collector['readers'],
                'unknown reader/service')
        guest = integer(read['guest_address'], 0x08000000, 0x0dffffff)
        a = integer(read['physical_offset']); count = integer(read['length'], 1, size)
        span = checked_span(a, a + count, size)
        require((guest & 0x01ffffff) == a, 'guest mirror/physical offset mismatch')
        require(not ranges or intersection([span], ranges), 'event outside physical filter')
        require(filters['preset'] == 'all' or (reader == 'Bus') == (filters['preset'] == 'bus'),
                'event outside collector preset')
        scope = key['instruction']
        scope_key = None
        if scope is not None:
            require(isinstance(scope, dict), 'invalid instruction scope')
            scope_key = (integer(scope['pc'], 0, 0xffffffff), scope['thumb'])
            require(type(scope['thumb']) is bool, 'invalid instruction mode')
        event_key = (reader, service, guest, a, count, scope_key)
        require(event_key not in seen, 'duplicate event key')
        seen.add(event_key)
        first = integer(event['first_guest_frame']); last = integer(event['last_guest_frame'])
        require(first <= last and last - first < entered, 'event outside frame window')
        requests += integer(event['requests'], 1)
        by_reader[reader].append(span); by_service[service].append(span)
    by_reader = {k: union(v) for k, v in by_reader.items()}
    by_service = {k: union(v) for k, v in by_service.items()}
    observed = union([s for values in by_reader.values() for s in values])
    require(spans(data['physical_intervals'], size) == observed and
            integer(data['observed_distinct_bytes']) == length(observed), 'observed union differs from events')
    bus = by_reader['Bus']; host = union([s for r in READERS[1:] for s in by_reader[r]])
    inventory(data['bus'], bus, size); inventory(data['host'], host, size)
    require(integer(data['bus_host_shared_distinct_bytes']) == length(intersection(bus, host)),
            'bus/host intersection differs')
    for service in SERVICES:
        inventory(data['serving'][service.lower()], by_service[service], size)
    dropped = integer(data['dropped_requests']); matched = integer(data['matched_requests'])
    require(requests + dropped == matched and type(data['truncated']) is bool and
            data['truncated'] == (dropped > 0) and (not dropped or len(events) == limit),
            'inconsistent request/drop counters')
    integer(data['fetch_bus_requests_excluded'])
    # Recompute ownership against THIS authenticated atlas, never import a family's
    # entire extent or trust ownership claims from a different manifest.
    resources = []
    for resource in report['resources']:
        overlap = intersection(observed, resource['ranges'])
        if overlap:
            resources.append(dict(id=resource['id'], ranges=overlap, distinct_bytes=length(overlap)))
    known = data['known_plan']
    require(isinstance(known, dict), 'invalid known-plan inventory')
    require(type(known['compared']) is bool, 'invalid known-plan status')
    digest(known.get('manifest_sha256'), optional=True)
    known_matches = known['compared'] and known['manifest_sha256'] == report['manifest_sha256']
    if known_matches:
        require(isinstance(known['resources'], list) and len(known['resources']) <= 4096,
                'invalid known resource inventory')
        expected = {r['id']: r['ranges'] for r in resources if next(
            x['runtime_available'] for x in report['resources'] if x['id'] == r['id'])}
        claimed = {}
        for r in known['resources']:
            require(r['id'] not in claimed, 'duplicate known resource')
            claimed[r['id']] = spans(r['physical_intervals'], size)
            require(integer(r['observed_distinct_bytes']) == length(claimed[r['id']]), 'known resource count differs')
        require(claimed == expected, 'known-plan intersections differ from authenticated atlas')
        within = length([s for v in expected.values() for s in v])
        require(integer(known['within_distinct_bytes']) == within and
                integer(known['outside_distinct_bytes']) == length(observed) - within,
                'known-plan totals differ')
    return dict(id=identity, report_sha256=identity, case=data['case'], run=data['run'],
                runtime_sha256=data['runtime_sha256'], input_sha256=data['input_sha256'],
                dataset_manifest_sha256=data['dataset_manifest_sha256'],
                window={k: window[k] for k in ('relative_from', 'relative_to_exclusive', 'frames_entered',
                        'replay_completed', 'complete', 'replay_outcome')},
                filter={k: filters[k] for k in ('preset', 'physical_ranges', 'unique_event_limit')},
                truncated=data['truncated'], partial=not window['complete'] or window['replay_outcome'] != 'passed',
                matched_requests=matched, dropped_requests=dropped, retained_events=len(events),
                fetch_bus_requests_excluded=data['fetch_bus_requests_excluded'],
                ranges=observed, distinct_bytes=length(observed), readers=by_reader, serving=by_service,
                collector={k: collector[k] for k in ('readers', 'absence', 'fetches', 'instruction_scope', 'not_observed')},
                known_manifest_sha256=known.get('manifest_sha256'),
                known_matches_atlas=known_matches, resources=resources), byte_size


def attach_audits(report, paths):
    require(len(paths) <= 32, 'at most 32 audit inputs')
    audits, seen, runs, total_bytes = [], set(), set(), 0
    for path in paths:
        audit, byte_size = read_audit(path, report)
        total_bytes += byte_size
        require(total_bytes <= 128 * 1024 * 1024, 'total input exceeds 128 MiB')
        if audit['id'] in seen:
            continue  # Same metadata file selected twice is one observation.
        require(audit['run'] not in runs, 'conflicting reports for the same run')
        seen.add(audit['id']); runs.add(audit['run']); audits.append(audit)
    audits.sort(key=lambda a: (a['case'], a['run'], a['id']))
    observed = union([s for a in audits for s in a['ranges']])
    layers = dict(scene_observed=observed,
                  scene_shared=multiple_scene_ranges(audits),
                  scene_outside_identified=intersection(observed, complement(
                      report['layers']['identified_data'], report['totals']['rom'])))
    for service in SERVICES:
        layers['scene_' + service.lower()] = union([s for a in audits for s in a['serving'][service]])
    report['schema'] = 3
    report['audits'] = audits
    report['layers'].update(layers)
    report['totals'].update({k: length(v) for k, v in layers.items()})
    report['totals']['ordinary_distinct_read_bytes'] = None  # Existing historical proofs have no byte inventory.
    report['totals']['scene_audits'] = len(audits)
    report['totals']['scene_cases'] = len({a['case'] for a in audits})
    for r in report['resources']:
        r['scene_ranges'] = intersection(observed, r['ranges'])
        r['scene_distinct_bytes'] = length(r['scene_ranges'])
        r['scene_audits'] = [a['id'] for a in audits if any(x['id'] == r['id'] for x in a['resources'])]


def multiple_scene_ranges(audits):
    from rom_coverage_ranges import multiple_owners
    # Different runs of ONE case do not establish cross-scene sharing.
    cases = {}
    for a in audits:
        cases.setdefault(a['case'], []).extend(a['ranges'])
    return multiple_owners(cases)


def compare_reports(current, previous, identity):
    require(type(previous['schema']) is int and previous['schema'] in (2, 3), 'unsupported comparison schema')
    require(previous['rom_sha256'] == current['rom_sha256'] and
            previous['totals']['rom'] == current['totals']['rom'], 'comparison ROM identity mismatch')
    size = current['totals']['rom']
    changes = {}
    for name, old in previous['layers'].items():
        old = spans(old, size)
        require(previous['totals'][name] == length(old), 'comparison layer count differs')
        if name in current['layers']:
            new = current['layers'][name]
            changes[name] = dict(before=length(old), after=length(new), delta=length(new)-length(old),
                                 added=intersection(new, complement(old, size)),
                                 removed=intersection(old, complement(new, size)))
    for key in ('generated_at', 'generation', 'source_revision', 'manifest_sha256'):
        require(isinstance(previous[key], str) and len(previous[key]) <= 256, 'invalid comparison identity')
    require(isinstance(previous.get('audits', []), list) and len(previous.get('audits', [])) <= 32,
            'invalid comparison audit inventory')
    old_audits = sorted(digest(a['id']) for a in previous.get('audits', []))
    new_audits = sorted(a['id'] for a in current.get('audits', []))
    return dict(report_sha256=identity, generated_at=previous['generated_at'], generation=previous['generation'],
                source_revision=previous['source_revision'], manifest_sha256=previous['manifest_sha256'],
                rom_sha256=previous['rom_sha256'], audit_ids=old_audits,
                scene_selection_same=old_audits == new_audits, layers=changes)
