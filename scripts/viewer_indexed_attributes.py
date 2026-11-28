"""Source-checked packed attributes for indexed graphics archives."""
import hashlib
import struct


ATTRIBUTES_SHA256 = '35a0e8cb8822a4ece9eb085aedcf059640115609c0bba010dc22dc6b0e88378a'
ARCHIVES_SHA256 = '9c0a78930962aca1af0b59a01f130e0ebd38aa8530ddb05e314f65398b96d4c5'
SHARED_BYTE_SHA256 = '067cd9af2584103bc505aeb02951d35b937b3274ed6c4564a646e4c6a85445e7'


def source_binding(sources, offset, role, content):
    if not isinstance(sources, list) or any(not isinstance(source, dict) for source in sources):
        raise ValueError('Invalid indexed attribute source list')
    matches = [source for source in sources if source.get('offset') == offset and source.get('role') == role]
    if (len(matches) != 1 or matches[0].get('length') != len(content)
            or matches[0].get('sha256') != hashlib.sha256(content).hexdigest()):
        raise ValueError('Invalid indexed attribute source binding')


def project_shared_byte(catalog, sources):
    if catalog.get('byte_offset') != 0x3c9460 or type(catalog.get('byte')) is not int or not 0 <= catalog['byte'] <= 255:
        raise ValueError('Invalid shared indexed attribute byte')
    source_binding(sources, 0x3c9460, 'white-flowers-observed-nibble-byte', bytes([catalog['byte']]))
    return catalog['byte']


def table(source_offset, raw, count, tiles, label):
    if (len(raw) != 4 + count * tiles // 2
            or struct.unpack_from('<HH', raw) != (len(raw), count * tiles)):
        raise ValueError('Invalid indexed attribute table header')
    width = tiles // 2
    return {'label': label, 'source_offset': source_offset, 'archive_offset': source_offset + len(raw),
            'tiles_per_program': tiles, 'entries': [list(raw[4 + i * width:4 + (i + 1) * width]) for i in range(count)]}


def project_localized(catalog, sources):
    additional = catalog['additional']
    groups = [group for group in additional['groups'] if group.get('source_offset') == 0x397d94]
    if len(groups) != 1:
        raise ValueError('Invalid localized attribute directory')
    pointers = groups[0]['pointers']
    expected = [0x43d990, 0x44efb8, 0x460b20, 0x471b58, 0x483634]
    if pointers != expected:
        raise ValueError('Invalid localized attribute pointers')
    source_binding(sources, 0x397d94, 'additional-indexed-selector/397d94',
                   struct.pack('<5I', *(0x08000000 + pointer for pointer in pointers)))
    prefixes = additional['embedded_prefixes']
    if not isinstance(prefixes, list) or len(prefixes) != 5:
        raise ValueError('Invalid localized attribute prefixes')
    result = []
    for slot, offset in enumerate(pointers):
        entries = [item for item in prefixes if item.get('source_offset') == offset]
        if len(entries) != 1 or not isinstance(entries[0].get('bytes'), list):
            raise ValueError('Invalid localized attribute prefix')
        values = entries[0]['bytes']
        if len(values) != 180 or any(type(value) is not int or not 0 <= value <= 255 for value in values):
            raise ValueError('Invalid localized attribute bytes')
        raw = bytes(values)
        source_binding(sources, offset, f'additional-indexed-prefix/{offset:x}', raw)
        result.append(table(offset, raw, 44, 8, f'Localized table · language slot {slot}'))
    return result


def project_tables(segments, shared_byte=None, localized=None):
    expected = [(0x0cf3dc, 'reader_literals', 4), (0x0cf494, 'reader_literals', 8),
                (0x3bd70c, 'tile_attributes', 1048), (0x3c943c, 'tile_attributes', 36),
                (0x3c9461, 'tile_attributes', 91)]
    if [(offset, role, len(raw)) for offset, role, raw in segments] != expected:
        raise ValueError('Unexpected indexed attribute source ranges')
    if (struct.unpack('<I', segments[0][2])[0] != 0x083bd70c
            or struct.unpack('<II', segments[1][2]) != (0x02000000, 0x00001f6e)):
        raise ValueError('Invalid indexed attribute reader literals')
    result = [table(0x3bd70c, segments[2][2], 522, 4, 'Direct table · 0x3BD70C')]
    missing = []
    if shared_byte is None:
        missing.append('The second direct table requires its separately owned byte.')
    else:
        raw = segments[3][2] + bytes([shared_byte]) + segments[4][2]
        result.append(table(0x3c943c, raw, 62, 4, 'Direct table · 0x3C943C'))
    if localized is None:
        missing.append('The five localized tables require their indexed-archives source.')
    else:
        result.extend(localized)
    return {'tables': result, 'missing': missing}


def assign_archive_links(attributes, archives):
    if not attributes or not archives:
        return
    for item in attributes['tables']:
        matches = []
        for family_index, family in enumerate(archives['families']):
            for archive_index, archive in enumerate(family['archives']):
                if (archive['source_offset'] == item['archive_offset']
                        and archive['fixed_decoded_bytes'] == item['tiles_per_program'] * 32
                        and len(archive['programs']) == len(item['entries'])):
                    matches.append({'family': family_index, 'archive': archive_index})
        if len(matches) == 1:
            item['archive_link'] = matches[0]
