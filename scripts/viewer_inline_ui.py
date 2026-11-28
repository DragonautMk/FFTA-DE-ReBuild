"""Source-bound European localized inline UI strings and level glyphs."""
import struct


INLINE_SHA256 = '7379f63cdd80cecce940ed267aff7dc4f9b4629294e810139043b02d97a48e42'
LITERALS = [
    (0x01aaf8, (0x3954a0,)), (0x0392b0, (0x395487,)), (0x039344, (0x395487,)),
    (0x039438, (0x395487,)), (0x03fb1c, (0x395484,)), (0x03fc04, (0x395484,)),
    (0x03ffe4, (0x395484,)), (0x040054, (0x395484, 0x395487)), (0x0400c4, (0x395484,)),
    (0x04b200, (0x395484, 0x395487)), (0x04c24c, (0x39548a, 0x39548d, 0x395484)),
    (0x04c44c, (0x395484,)), (0x055944, (0x395378,)), (0x055950, (0x3953c8,)),
    (0x055c84, (0x395378,)), (0x055e48, (0x395420,)), (0x055e54, (0x395470,)),
    (0x05a604, (0x395184,)), (0x05a6fc, (0x395184, 0x395330)), (0x05a76c, (0x39526c,)),
    (0x05a778, (0x395262,)), (0x05a7a8, (0x3952b4,)),
]
TABLES = [0x39526c, 0x3952b4, 0x395330, 0x395378, 0x3953c8, 0x395420, 0x395470]
STRING_SPANS = [(0x395187, 0x395262), (0x395280, 0x3952b3), (0x3952c8, 0x39532e),
                (0x395344, 0x395375), (0x39538c, 0x3953c5), (0x3953dc, 0x39541d), (0x395434, 0x39546d)]
SHARED_SPANS = [(0x395184, 0x395187), (0x395262, 0x395269), (0x395484, 0x395487),
                (0x395487, 0x39548a), (0x39548a, 0x39548d), (0x39548d, 0x395490)]
READERS = [(0x05a70c,), (0x05a782,), (0x05a6e8,), (0x055866, 0x055c04),
           (0x0558b4,), (0x055d6a,), (0x055db8,)]
DATA_SOURCES = [
    (0x395184, 'shared_strings', 3), (0x395187, 'localized_strings', 219),
    (0x395262, 'shared_strings', 7), (0x39526c, 'language_directories', 20),
    (0x395280, 'localized_strings', 51), (0x3952b4, 'language_directories', 20),
    (0x3952c8, 'localized_strings', 102), (0x395330, 'language_directories', 20),
    (0x395344, 'localized_strings', 49), (0x395378, 'language_directories', 20),
    (0x39538c, 'localized_strings', 57), (0x3953c8, 'language_directories', 20),
    (0x3953dc, 'localized_strings', 65), (0x395420, 'language_directories', 20),
    (0x395434, 'localized_strings', 57), (0x395470, 'language_directories', 20),
    (0x395484, 'shared_strings', 12), (0x395490, 'level_glyphs', 12),
    (0x3954a0, 'level_glyph_pointer', 4),
]


def decode(raw, source):
    """Decode one bounded original 08019d98 string without guessing commands."""
    if not raw:
        raise ValueError('Empty inline text source')
    compact = raw[0] == 1
    index = 1 if compact else 0
    units = []
    while index < len(raw):
        byte = raw[index]; index += 1
        if byte == 0:
            if index != len(raw):
                raise ValueError('Trailing bytes after inline text terminator')
            return {'source_offset': source, 'encoding': 'compact' if compact else 'extended',
                    'units': units, 'bytes': list(raw)}
        if compact:
            units.append({'kind': 'glyph', 'index': byte - 1})
        elif byte & 0x80:
            if index >= len(raw):
                raise ValueError('Truncated inline text glyph')
            units.append({'kind': 'glyph', 'index': ((byte << 8) | raw[index]) & 0x7fff}); index += 1
        else:
            units.append({'kind': 'control_byte', 'value': byte})
    raise ValueError('Unterminated inline text')


def project(segments):
    expected = [(offset, 'reader_literals', 4 * len(values)) for offset, values in LITERALS] + DATA_SOURCES
    if [(offset, role, len(raw)) for offset, role, raw in segments] != expected:
        raise ValueError('Unexpected inline text sources')
    for index, (_, targets) in enumerate(LITERALS):
        if struct.unpack('<' + 'I' * len(targets), segments[index][2]) != tuple(0x08000000 + value for value in targets):
            raise ValueError('Invalid inline text reader literals')
    data = {offset: raw for offset, _, raw in segments[len(LITERALS):]}
    if struct.unpack('<I', data[0x3954a0])[0] != 0x08395490:
        raise ValueError('Invalid inline text glyph pointer')
    directories = []
    for index, (table, (start, end)) in enumerate(zip(TABLES, STRING_SPANS)):
        pointers = [pointer - 0x08000000 for pointer in struct.unpack('<5I', data[table])]
        if pointers[0] != start or any(left >= right for left, right in zip(pointers, pointers[1:])) or pointers[-1] >= end:
            raise ValueError('Invalid inline text directory pointers')
        entries = []
        for left, right in zip(pointers, pointers[1:] + [end]):
            entries.append(decode(data[start][left - start:right - start], left))
        directories.append({'source_offset': table, 'readers': list(READERS[index]), 'entries': entries})
    shared = []
    for start, end in SHARED_SPANS:
        block = 0x395484 if start >= 0x395484 else start
        shared.append(decode(data[block][start - block:end - block], start))
    thresholds = []
    for index in range(6):
        raw = data[0x395490][2 * index:2 * index + 2]
        if not raw[0] & 0x80:
            raise ValueError('Invalid threshold glyph')
        entry = decode(raw + b'\0', 0x395490 + 2 * index)
        if len(entry['units']) != 1 or entry['units'][0]['kind'] != 'glyph':
            raise ValueError('Invalid threshold glyph unit')
        thresholds.append({'source_offset': entry['source_offset'], 'minimum_level': 3 * index,
                           'maximum_level': 3 * index + 2 if index < 5 else None,
                           'units': entry['units'], 'bytes': list(raw)})
    all_texts = [entry for directory in directories for entry in directory['entries']] + shared + thresholds
    glyph_ids = sorted({unit['index'] for entry in all_texts for unit in entry['units'] if unit['kind'] == 'glyph'})
    return {'directories': directories, 'shared': shared, 'thresholds': thresholds, 'glyph_ids': glyph_ids,
            'fallback': {'directory': 1, 'language_id': 3, 'shared_index': 1, 'reader': 0x05a750}}
