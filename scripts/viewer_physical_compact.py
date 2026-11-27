"""Reconstruct European compact OBJ banks from authenticated extracted owners."""
import hashlib
import json
import struct

from viewer_localized_obj import SourceBytes, encode_obj_list


BANK_START, BANK_END, WIDE_BANK = 0x6b3428, 0x6cd966, 0x6bb838
SUFFIXES = ('', '-secondary', '-tertiary', '-quaternary', '-quinary', '-senary', '-septenary', '-octonary')
PRIOR_IDS = (('graphics.obj-static-lists', 'graphics.conditional-compact-obj-catalog')
             + tuple('graphics.conditional-compact-obj-animations' + suffix for suffix in SUFFIXES)
             + ('graphics.firaga-compact-obj-bank', 'graphics.firaga-adjacent-compact-obj-bank',
                'graphics.firaga-6c12c8-obj-bank'))


def project_witnessed_links(witnessed, physical):
    """Link only sequences whose roots, frames and referenced objects agree."""
    if not witnessed or not physical:
        return []
    def unique(items, key):
        values, duplicates = {}, set()
        for item in items:
            value = item[key]
            if value in values:
                duplicates.add(value)
            values[value] = item
        return {value: item for value, item in values.items() if value not in duplicates}

    old_banks = unique(witnessed['banks'], 'object_root')
    new_banks = unique(physical['banks'], 'object_root')
    result = []
    for root, old in old_banks.items():
        new = new_banks.get(root)
        if not new or (old['sequence_root'], old['flags']) != (new['sequence_root'], new['flags']):
            continue
        old_sequences = unique(old['sequences'], 'selector')
        new_sequences = unique(new['sequences'], 'selector')
        for selector, left in old_sequences.items():
            right = new_sequences.get(selector)
            if not right or (left['source_offset'], left['relative_offset']) != (right['source_offset'], right['relative_offset']):
                continue
            if len(left['frames']) != len(right['frames']):
                continue
            try:
                for a, b in zip(left['frames'], right['frames']):
                    if (a['list_id'], a['duration']) != (b['list_id'], b['duration']):
                        break
                    if (type(a['object_list']) is not int or type(b['object_list']) is not int
                            or not 0 <= a['object_list'] < len(old['lists'])
                            or not 0 <= b['object_list'] < len(new['lists'])):
                        break
                    x, y = old['lists'][a['object_list']], new['lists'][b['object_list']]
                    if (x['source_index'] != a['list_id'] or y['source_index'] != b['list_id']
                            or x['source_offset'] != root + x['relative_offset']
                            or (x['source_offset'], x['relative_offset'], x['objects'])
                            != (y['source_offset'], y['relative_offset'], y['objects'])):
                        break
                else:
                    result.append({'root': root - 4, 'selector': selector})
            except (KeyError, IndexError, TypeError):
                continue
    return result


def _put(memory, offset, data):
    if offset >= BANK_END or offset + len(data) <= BANK_START:
        return
    if offset < BANK_START or offset + len(data) > BANK_END:
        raise ValueError('Prior compact source crosses bank boundary')
    memory.add(offset, data)


def _sequence_bytes(frames, width):
    if not isinstance(frames, list) or len(frames) > 65535:
        raise ValueError('Invalid prior compact sequence')
    limit = 255 if width == 2 else 65535
    payload = bytearray(struct.pack('<H', len(frames)))
    for frame in frames:
        values = (frame['list'], frame['duration'])
        if any(type(value) is not int or not 0 <= value <= limit for value in values):
            raise ValueError('Invalid prior compact frame')
        payload.extend(struct.pack('<BB' if width == 2 else '<HH', *values))
    return bytes(payload)


def _add_bank(memory, root, flags, sequence_relative, lists, sequences):
    if (type(root) is not int or type(flags) is not int or type(sequence_relative) is not int
            or not BANK_START <= root < BANK_END or flags not in (4, 12)
            or (flags == 4) != (root == WIDE_BANK) or not 4 < sequence_relative <= 65535):
        raise ValueError('Invalid prior compact bank header')
    sequence_root, width = root + sequence_relative, 4 if flags == 4 else 2
    _put(memory, root, struct.pack('<HH', flags, sequence_relative))
    for item in lists:
        index, relative, source = item['index'], item['relative_offset'], item['list']
        if (type(index) is not int or not 0 <= index < 4096 or type(relative) is not int
                or not 0 <= relative <= 65535 or source['source_offset'] != root + 4 + relative):
            raise ValueError('Invalid prior compact list reference')
        _put(memory, root + 4 + index * 2, struct.pack('<H', relative))
        _put(memory, source['source_offset'], encode_obj_list(source))
    for item in sequences:
        relative, sequence = item['relative_offset'], item['sequence']
        selector = sequence['selector']
        if (type(selector) is not int or not 0 <= selector < 4096 or type(relative) is not int
                or not 0 <= relative <= (0xffffffff if width == 4 else 65535)
                or sequence['source_offset'] != sequence_root + relative):
            raise ValueError('Invalid prior compact sequence reference')
        _put(memory, sequence_root + selector * width, struct.pack('<I' if width == 4 else '<H', relative))
        _put(memory, sequence['source_offset'], _sequence_bytes(sequence['frames'], width))


def reconstruct_prior_banks(priors):
    """Recover prior bank spans and authenticate every intersecting source range."""
    if not all(identity in priors for identity in PRIOR_IDS):
        raise ValueError('Missing prior compact OBJ source owners')
    catalogs = {identity: json.loads(priors[identity][0]) for identity in PRIOR_IDS}
    candidate = SourceBytes()
    for source in catalogs['graphics.obj-static-lists']['lists']:
        _put(candidate, source['source_offset'], encode_obj_list(source))
    compact_catalog = catalogs['graphics.conditional-compact-obj-catalog']
    for bank in compact_catalog['banks']:
        _add_bank(candidate, bank['root'], bank['flags'], bank['sequence_relative_offset'], bank['lists'], bank['sequences'])
    soil = compact_catalog['soil_evidence_metadata']
    if soil is not None:
        root, offsets = 0x6c173c, soil['list_offsets']
        if not isinstance(offsets, list) or len(offsets) != 20 or any(type(value) is not int or not 0 <= value <= 65535 for value in offsets):
            raise ValueError('Invalid prior Soil Evidence offsets')
        _put(candidate, root, struct.pack('<HH', soil['flags'], soil['sequence_relative_offset'])
             + struct.pack('<20H', *offsets))
        sequence_root = root + soil['sequence_relative_offset']
        _put(candidate, sequence_root, struct.pack('<H', soil['selector0_relative_offset']))
        _put(candidate, sequence_root + soil['selector0_relative_offset'], _sequence_bytes(soil['frames'], 2))
    for suffix in SUFFIXES:
        catalog = catalogs['graphics.conditional-compact-obj-animations' + suffix]
        if not suffix:
            lists = [{'index': i, 'relative_offset': relative, 'list': item}
                     for i, (relative, item) in enumerate(zip(catalog['list_offsets'], catalog['lists']))]
            sequences = [{'relative_offset': relative, 'sequence': item}
                         for relative, item in zip(catalog['sequence_offsets'], catalog['sequences'])]
            if (len(lists) != len(catalog['list_offsets']) or len(lists) != len(catalog['lists'])
                    or len(sequences) != len(catalog['sequence_offsets']) or len(sequences) != len(catalog['sequences'])):
                raise ValueError('Incomplete prior compact directory')
        else:
            lists, sequences = catalog['lists'], catalog['sequences']
        if not lists or not sequences:
            raise ValueError('Empty prior compact bank')
        root = lists[0]['list']['source_offset'] - lists[0]['relative_offset'] - 4
        sequence_root = sequences[0]['sequence']['source_offset'] - sequences[0]['relative_offset']
        _add_bank(candidate, root, 4 if root == WIDE_BANK else 12, sequence_root - root, lists, sequences)
    for identity in PRIOR_IDS[-3:]:
        catalog, sources = catalogs[identity], priors[identity][1]
        headers = [source for source in sources if source['role'].endswith('header-directory')]
        if len(headers) != 1:
            raise ValueError('Missing Firaga compact directory')
        root, flags, split = headers[0]['offset'], catalog['flags'], catalog['split_offset']
        offsets = catalog['list_offsets']
        if not isinstance(offsets, list) or not offsets or any(type(value) is not int or not 0 <= value <= 65535 for value in offsets):
            raise ValueError('Invalid Firaga compact offsets')
        _put(candidate, root, struct.pack('<HH', flags, split) + struct.pack('<' + str(len(offsets)) + 'H', *offsets))
        sequence_root = root + split
        _put(candidate, sequence_root, struct.pack('<H', catalog['sequence_relative_offset']))
        _put(candidate, sequence_root + catalog['sequence_relative_offset'], _sequence_bytes(catalog['frames'], 2))
    verified = SourceBytes(); source_count = 0
    for identity in PRIOR_IDS:
        for source in priors[identity][1]:
            offset, length = source['offset'], source['length']
            if offset >= BANK_END or offset + length <= BANK_START:
                continue
            if offset < BANK_START or offset + length > BANK_END or length <= 0 or length > 65536:
                raise ValueError('Invalid prior compact source extent')
            content = candidate.read(offset, length)
            if hashlib.sha256(content).hexdigest() != source['sha256']:
                raise ValueError('Prior compact source SHA-256 mismatch at ' + hex(offset))
            verified.add(offset, content); source_count += 1
    return verified, source_count


def _object(a0, a1, a2):
    x, y = a1 & 511, a0 & 255
    return {'x': x - 512 if x >= 256 else x, 'y': y - 256 if y >= 128 else y,
            'affine_mode': a0 >> 8 & 3, 'object_mode': a0 >> 10 & 3,
            'mosaic': bool(a0 & 0x1000), 'color_256': bool(a0 & 0x2000), 'shape': a0 >> 14,
            'transform': a1 >> 9 & 31, 'size': a1 >> 14, 'tile': a2 & 1023,
            'priority': a2 >> 10 & 3, 'palette': a2 >> 12}


def _directory(memory, root, limit, width, *, lists, list_count=0):
    first = memory.half(root) if width == 2 else memory.word(root)
    if not first or first % width or first > 4096 or root + first >= limit:
        raise ValueError('Invalid physical compact directory')
    offsets = [memory.half(root + i) if width == 2 else memory.word(root + i) for i in range(0, first, width)]
    records, end = [], root + first
    for index, relative in enumerate(offsets):
        at = (end + width - 1) & ~(width - 1)
        if root + relative != at or at + 2 > limit:
            raise ValueError('Invalid physical compact relative offset')
        count = memory.half(at)
        stride = 6 if lists else width
        end = at + 2 + count * stride
        if end > limit or (lists and count >= 32768):
            raise ValueError('Invalid physical compact record extent')
        raw = memory.read(at + 2, count * stride)
        if lists:
            objects = [_object(*attributes) for attributes in struct.iter_unpack('<HHH', raw)]
            records.append({'source_offset': at, 'source_index': index, 'relative_offset': relative, 'objects': objects})
        else:
            frames = []
            for object_list, duration in struct.iter_unpack('<BB' if width == 2 else '<HH', raw):
                if object_list >= list_count:
                    raise ValueError('Invalid physical compact frame reference')
                frames.append({'object_list': object_list, 'list_id': object_list, 'duration': duration})
            records.append({'selector': index, 'source_offset': at, 'relative_offset': relative, 'frames': frames})
    return records, end


def project_physical_compact(segments, priors):
    """Join verified old and new byte spans and expose all 199 physical banks."""
    memory, source_count = reconstruct_prior_banks(priors)
    if source_count != 211 or len(memory.data) != 27454:
        raise ValueError('Incomplete prior physical compact sources')
    for offset, role, content in segments:
        if role not in ('banks', 'descriptors', 'selector_tables', 'reader_literals'):
            raise ValueError('Unexpected physical compact source role')
        if role == 'banks':
            _put(memory, offset, content)
    banks, root = [], BANK_START
    while root < BANK_END:
        flags, relative = memory.half(root), memory.half(root + 2)
        if flags != (4 if root == WIDE_BANK else 12) or not 4 < relative < BANK_END - root:
            raise ValueError('Invalid physical compact bank header')
        object_root, sequence_root = root + 4, root + relative
        lists, list_end = _directory(memory, object_root, sequence_root, 2, lists=True)
        if not 0 <= sequence_root - list_end <= 2:
            raise ValueError('Invalid physical compact list boundary')
        sequences, end = _directory(memory, sequence_root, BANK_END, 4 if flags == 4 else 2,
                                    lists=False, list_count=len(lists))
        banks.append({'object_root': object_root, 'sequence_root': sequence_root, 'flags': flags,
                      'lists': lists, 'sequences': sequences})
        if end == BANK_END:
            break
        root = (end + 3) & ~3
        if root >= BANK_END:
            raise ValueError('Invalid physical compact bank boundary')
    counts = (len(banks), sum(len(bank['lists']) for bank in banks),
              sum(len(item['objects']) for bank in banks for item in bank['lists']),
              sum(len(bank['sequences']) for bank in banks),
              sum(len(item['frames']) for bank in banks for item in bank['sequences']))
    if counts != (199, 3135, 13633, 560, 5045):
        raise ValueError('Incomplete physical compact bank projection')
    return {'banks': banks, 'selector_tables': []}


def project_physical_descriptors(segments, priors, catalog):
    """Reconstruct the 548 witnessed 24-byte descriptors and link their selectors."""
    memory, candidate = SourceBytes(), SourceBytes()
    for offset, role, content in segments:
        if role not in ('banks', 'descriptors', 'selector_tables', 'reader_literals'):
            raise ValueError('Unexpected physical compact source role')
        if role == 'descriptors':
            memory.add(offset, content)
    catalog_id = 'graphics.conditional-compact-obj-catalog'
    first_id, adjacent_id = 'graphics.firaga-compact-obj-bank', 'graphics.firaga-adjacent-compact-obj-bank'
    if any(identity not in priors for identity in (catalog_id, first_id, adjacent_id)):
        raise ValueError('Missing prior compact descriptor owners')
    old = json.loads(priors[catalog_id][0])
    for group in ('red_spring_records', 'star_cross_records'):
        for item in old[group]:
            fields = item['halfwords']
            if not isinstance(fields, list) or len(fields) != 10 or any(type(v) is not int or not 0 <= v <= 65535 for v in fields):
                raise ValueError('Invalid prior compact descriptor fields')
            candidate.add(item['source_offset'], struct.pack('<I10H', item['bank_pointer'], *fields))
    first, adjacent = (json.loads(priors[identity][0]) for identity in (first_id, adjacent_id))
    for key in ('first_descriptor_fields', 'second_descriptor_fields'):
        values = adjacent[key]
        if not isinstance(values, list) or len(values) != 10 or any(type(v) is not int or not 0 <= v <= 65535 for v in values):
            raise ValueError('Invalid prior Firaga descriptor fields')
    candidate.add(0x39cc3c, struct.pack('<I', first['descriptor_root']))
    candidate.add(0x39cc40, struct.pack('<10H', *adjacent['first_descriptor_fields']))
    candidate.add(0x39cc54, struct.pack('<I10H', adjacent['second_bank_pointer'], *adjacent['second_descriptor_fields']))
    for identity, offset, length in ((catalog_id, 0x39ddf0, 48), (catalog_id, 0x3a4ca0, 72),
                                     (first_id, 0x39cc3c, 4), (adjacent_id, 0x39cc40, 20),
                                     (adjacent_id, 0x39cc54, 24)):
        matches = [source for source in priors[identity][1] if source['offset'] == offset and source['length'] == length]
        raw = candidate.read(offset, length)
        if len(matches) != 1 or hashlib.sha256(raw).hexdigest() != matches[0]['sha256']:
            raise ValueError('Prior compact descriptor SHA-256 mismatch at ' + hex(offset))
        memory.add(offset, raw)
    if len(memory.data) != 548 * 24:
        raise ValueError('Incomplete physical compact descriptor bytes')
    positions = sorted(memory.data)
    ranges, first = [], positions[0]
    for previous, current in zip(positions, positions[1:]):
        if current != previous + 1:
            ranges.append((first, previous + 1)); first = current
    ranges.append((first, positions[-1] + 1))
    if len(ranges) != 177 or any((end - start) % 24 for start, end in ranges):
        raise ValueError('Invalid physical compact descriptor extents')
    roots = {bank['object_root'] - 4: (index, bank) for index, bank in enumerate(catalog['banks'])}
    records, pairs, used = [], set(), set()
    for start, end in ranges:
        for offset in range(start, end, 24):
            pointer, *fields = struct.unpack('<I10H', memory.read(offset, 24)); root = pointer - 0x08000000
            if root not in roots:
                raise ValueError('Compact descriptor references an unavailable bank')
            bank_index, bank = roots[root]; selector = fields[0]
            matches = [i for i, sequence in enumerate(bank['sequences']) if sequence['selector'] == selector]
            if len(matches) != 1:
                raise ValueError('Compact descriptor references an unavailable sequence')
            records.append({'source_offset': offset, 'bank_root': root, 'bank': bank_index,
                            'selector': selector, 'sequence': matches[0], 'fields': fields})
            pairs.add((root, selector)); used.add(root)
    if (len(records), len(pairs), len(used)) != (548, 219, 107):
        raise ValueError('Incomplete physical compact descriptor links')
    return {'records': records}


# These are observed scene contexts, not asserted names for every bank sequence.
OBSERVED_CONTEXTS = {
    0x6bac28: [{'name': 'Soil Evidence', 'selector': None}],
    0x6c0614: [{'name': 'Soil Evidence', 'selector': None}],
    0x6c173c: [{'name': 'Soil Evidence', 'selector': 0}],
    0x6babc4: [{'name': 'Freezeblink', 'selector': None}],
    0x6bb574: [{'name': 'Stardust', 'selector': None}],
    0x6bacd8: [{'name': 'Star Cross', 'selector': 0}, {'name': 'Star Cross', 'selector': 1}],
    0x6c1e78: [{'name': 'Red Spring', 'selector': 5}, {'name': 'Blue Screw', 'selector': 6}],
}


def add_observed_contexts(catalog):
    """Annotate verified physical banks with documented, explicitly scoped observations."""
    for bank in catalog['banks']:
        root = bank['object_root'] - 4
        if root in OBSERVED_CONTEXTS:
            known = {sequence['selector'] for sequence in bank['sequences']}
            contexts = OBSERVED_CONTEXTS[root]
            if any(item['selector'] is not None and item['selector'] not in known for item in contexts):
                raise ValueError('Observed compact selector is missing')
            bank['observed_contexts'] = [dict(item) for item in contexts]


def annotate_authenticated_contexts(catalog, rom_sha, source_sha):
    """Attach scene observations only to the matching authenticated source."""
    if (rom_sha != '8fab1e390cb9d5e695bfdb86e81246892e1aa8c0cb4014db8235ff76538ea632'
            or source_sha != '975eadd350ac4295d2ac32a59251e7c8a1f074401e446168fa978075b8ca861e'):
        return False
    add_observed_contexts(catalog)
    return True
