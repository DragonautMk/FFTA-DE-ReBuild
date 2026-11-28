"""Source-bound European screen curve and position tables."""
import hashlib
import struct


CURVES_SHA256 = '2ef65da269981bce0454029b2118cf5bd886cd4151b29ff89259649e28133560'
OWNER_SHA256 = '9cb5b35e8ade7a14fe3ee6d2063ed32c1b465d6e5c4a39e1c5bd7f4c06db6d12'


def prior_positions(catalog, sources):
    table = catalog['selector_tables']['motion']
    if table.get('source_offset') != 0x3a80c8 or not isinstance(table.get('rows'), list) or len(table['rows']) != 14:
        raise ValueError('Invalid prior screen position table')
    values = bytearray()
    for row in table['rows']:
        head = [row[key] for key in ('selector', 'x', 'y')]
        targets = [row[key] for key in ('target_x', 'target_y')]
        if (any(type(x) is not int or not 0 <= x <= 255 for x in head)
                or any(type(x) is not int or not -32768 <= x <= 32767 for x in targets)):
            raise ValueError('Invalid prior screen position row')
        values.extend(head); values.extend(struct.pack('<hh', *targets))
    expected = [(0x3a80c8, 3)] + [(0x3a80cc + index * 8, 7) for index in range(13)] + [(0x3a8134, 4)]
    if not isinstance(sources, list) or any(not isinstance(item, dict) for item in sources):
        raise ValueError('Invalid prior screen position sources')
    position = 0
    for offset, length in expected:
        matches = [item for item in sources if item.get('offset') == offset and item.get('role') == 'conditional-compact-obj-catalog']
        raw = values[position:position + length]; position += length
        if (len(matches) != 1 or matches[0].get('length') != length
                or matches[0].get('sha256') != hashlib.sha256(raw).hexdigest()):
            raise ValueError('Invalid prior screen position binding')
    return [list(values[index * 7:index * 7 + 7]) for index in range(14)]


def project_bank(segments, prior=None):
    expected = ([(offset, 'reader_literals', 4) for offset in (0x114200, 0x114208, 0x1142c0, 0x1152c0, 0x1152cc, 0x1154ac, 0x1193d8)]
                + [(0x3a7fb0, 'curve_records', 220)]
                + [(0x3a80cb + index * 8, 'position_records', 1) for index in range(14)]
                + [(0x3a823c, 'curve_records', 200), (0x3a8304, 'position_records', 128),
                   (0x3a89b0, 'curve_samples', 840), (0x3a8cf8, 'terminal_copy', 20)])
    if [(offset, role, len(raw)) for offset, role, raw in segments] != expected:
        raise ValueError('Unexpected screen curve source ranges')
    literals = (0x3a7fb0, 0x3a80c8, 0x3a7fb0, 0x3a823c, 0x3a8304, 0x3a823c, 0x3a8cf8)
    if any(struct.unpack('<I', segments[index][2])[0] != 0x08000000 + target for index, target in enumerate(literals)):
        raise ValueError('Invalid screen curve reader literals')
    samples, terminal = segments[-2][2], segments[-1][2]
    curves = []
    for index in range(7):
        raw = samples[index * 120:(index + 1) * 120]
        extra = samples[(index + 1) * 120:(index + 1) * 120 + 2] if index < 6 else terminal[:2]
        curves.append({'source_offset': 0x3a89b0 + index * 120,
                       'values': list(struct.unpack('<60H', raw)) + [struct.unpack('<H', extra)[0]]})
    roots = {0x08000000 + curve['source_offset']: index for index, curve in enumerate(curves)}
    controllers = []
    for group, record_segment in enumerate((segments[7], segments[22])):
        offset, _, raw = record_segment; records = []
        for index in range(len(raw) // 20):
            row = raw[index * 20:(index + 1) * 20]; pointers = struct.unpack_from('<II', row, 8)
            if any(pointer and pointer not in roots for pointer in pointers):
                raise ValueError('Invalid screen curve record pointer')
            records.append({'source_offset': offset + index * 20, 'prefix': list(row[:8]), 'suffix': list(row[16:]),
                            'curves': [roots[pointer] if pointer else None for pointer in pointers]})
        if not group:
            positions = []
            for index in range(14):
                owned = prior[index] if prior is not None else [None] * 7
                positions.append({'source_offset': 0x3a80c8 + index * 8,
                                  'bytes': owned[:3] + [segments[8 + index][2][0]] + owned[3:]})
        else:
            positions = [{'source_offset': 0x3a8304 + index * 8,
                          'bytes': list(segments[23][2][index * 8:index * 8 + 8])} for index in range(16)]
        controllers.append({'source_offset': offset, 'records': records, 'positions': positions})
    return {'curves': curves, 'controllers': controllers, 'prior_missing': prior is None}
