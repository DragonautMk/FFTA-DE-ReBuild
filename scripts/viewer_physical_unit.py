"""Decode the bounded, independently owned EUR physical unit OBJ spans."""
import bisect
import hashlib
import struct


EUROPE_PHYSICAL_UNIT_SHA256 = 'd339863e1fa34f00ed9c13de3a01663e7895f79bc7ce934b8a0596b1a724d14a'
ROM_BASE = 0x08000000
INDEX_START, DIRECTORY, DIRECTORY_END = 0x371494, 0x394834, 0x394c14
PIXELS, OBJECTS, OBJECT_END = 0x898a40, 0xa50020, 0xa5129c
SEQUENCES, SEQUENCE_END = 0xa512a0, 0xae6dac
ROLE_RANGES = {'pose_index': (INDEX_START, DIRECTORY), 'sprite_directory': (DIRECTORY, DIRECTORY_END),
               'pixel_tiles': (PIXELS, OBJECTS), 'object_lists': (OBJECTS, OBJECT_END),
               'pose_sequences': (SEQUENCES, SEQUENCE_END)}


def decode_object(a0, a1, a2):
    """Preserve the original OBJ attribute bits, including unsupported modes."""
    x, y = a1 & 511, a0 & 255
    return {'x': x if x < 256 else x - 512, 'y': y if y < 128 else y - 256,
            'affine_mode': (a0 >> 8) & 3, 'object_mode': (a0 >> 10) & 3,
            'mosaic': bool(a0 & 0x1000), 'color_256': bool(a0 & 0x2000),
            'shape': a0 >> 14, 'transform': (a1 >> 9) & 31, 'size': a1 >> 14,
            'tile': a2 & 1023, 'priority': (a2 >> 10) & 3, 'palette': a2 >> 12}


def decode_tile(data):
    return {'indices': [[(data[y * 4 + x // 2] >> (4 * (x % 2))) & 15 for x in range(8)] for y in range(8)]}


def parse_physical_unit_bank(content, sources):
    """Read only manifest-owned spans and verify each source digest and extent."""
    if not isinstance(content, bytes) or not isinstance(sources, list) or not sources or len(sources) > 20000:
        raise ValueError('Invalid physical unit bank or source list')
    result = {'cells': [], 'sequences': [], 'objects': [], 'pixels': [], 'directory_roots': {}}
    intervals, position = [], 0
    for source in sources:
        offset, length, role, digest = (source[key] for key in ('offset', 'length', 'role', 'sha256'))
        if (type(offset) is not int or type(length) is not int or length <= 0 or role not in ROLE_RANGES
                or not isinstance(digest, str) or len(digest) != 64):
            raise ValueError('Invalid physical unit source')
        begin, end = ROLE_RANGES[role]
        if offset < begin or offset + length > end or position + length > len(content):
            raise ValueError('Physical unit source outside its bank')
        block = content[position:position + length]; position += length
        if hashlib.sha256(block).hexdigest() != digest:
            raise ValueError('Physical unit source digest mismatch')
        intervals.append((offset, offset + length))
        if role in ('pose_index', 'sprite_directory'):
            if offset % 4 or length % 4:
                raise ValueError('Unaligned physical unit pointer')
            for at in range(0, length, 4):
                value = struct.unpack_from('<I', block, at)[0]
                if role == 'pose_index':
                    target = value - ROM_BASE
                    if value and (not SEQUENCES <= target < SEQUENCE_END or target % 16):
                        raise ValueError('Invalid physical unit sequence pointer')
                    result['cells'].append({'source_offset': offset + at,
                        'target': {'kind': 'sequence', 'source_offset': target} if value else {'kind': 'null'}})
                else:
                    target = value - ROM_BASE
                    if not INDEX_START <= target < DIRECTORY or target % 4:
                        raise ValueError('Invalid physical unit directory pointer')
                    result['directory_roots'][(offset + at - DIRECTORY) // 4] = target
        elif role == 'pixel_tiles':
            if offset % 32 or length % 32:
                raise ValueError('Unaligned physical unit pixel source')
            result['pixels'].append({'source_offset': offset,
                                     'tiles': [decode_tile(block[at:at + 32]) for at in range(0, length, 32)]})
        elif role == 'object_lists':
            at = 0
            while at < length:
                if at + 2 > length or (offset + at) % 2:
                    raise ValueError('Invalid physical unit object list')
                count = struct.unpack_from('<H', block, at)[0]; end_at = at + 2 + count * 6
                if count >= 32768 or end_at > length:
                    raise ValueError('Invalid physical unit object count')
                objects = [decode_object(*struct.unpack_from('<HHH', block, item))
                           for item in range(at + 2, end_at, 6)]
                result['objects'].append({'source_offset': offset + at, 'objects': objects}); at = end_at
        elif role == 'pose_sequences':
            at = 0
            while at < length:
                if (offset + at) % 16 or at + 4 > length:
                    raise ValueError('Invalid physical unit sequence alignment')
                count, header = struct.unpack_from('<HH', block, at); end_at = at + 4 + count * 20
                if not count or count >= 32768 or end_at > length:
                    raise ValueError('Invalid physical unit sequence length')
                frames = []
                for item in range(at + 4, end_at, 20):
                    word00, word04, duration, operation = struct.unpack_from('<IIBB', block, item)
                    frames.append({'word00': word00, 'word04': word04, 'duration': duration,
                                   'operation': operation, 'bytes_0a_13': list(block[item + 10:item + 20])})
                result['sequences'].append({'source_offset': offset + at, 'opaque_header': header, 'frames': frames})
                at = end_at
    if position != len(content) or any(a[1] > b[0] for a, b in zip(sorted(intervals), sorted(intervals)[1:])):
        raise ValueError('Overlapping or incomplete physical unit sources')
    if not result['cells'] or not result['sequences']:
        raise ValueError('Physical unit index or sequences are missing')
    return result


def assign_physical_unit_bindings(projected, cells, directory_roots, catalogs):
    """Map each physical index cell to every documented sprite-directory alias."""
    roots = {}
    for catalog in catalogs:
        for binding in catalog.get('bindings', []):
            sid, index, animation, part = (binding[key] for key in ('sprite_id', 'index_source', 'animation', 'part'))
            if index is None or not 0 <= sid < 248:
                continue
            root = index - (animation & ~3) * 6 - (12 if animation & 3 in (1, 2) else 0) - part * 4
            if not INDEX_START <= root < DIRECTORY or root % 4:
                continue
            if sid in roots and roots[sid] != root:
                raise ValueError('Conflicting physical unit roots')
            roots[sid] = root
    for sid, root in directory_roots.items():
        if sid in roots and roots[sid] != root:
            raise ValueError('Conflicting physical unit directory')
        roots[sid] = root
    if set(roots) != set(range(248)) or len(set(roots.values())) != 245:
        raise ValueError('Complete physical unit directory dependencies are unavailable')
    starts = sorted(set(roots.values()))
    if starts[0] != INDEX_START or any((b - a) % 24 for a, b in zip(starts, starts[1:] + [DIRECTORY])):
        raise ValueError('Invalid physical unit root boundaries')
    aliases = {root: [] for root in starts}
    for sid, root in roots.items():
        aliases[root].append(sid)
    by_cell = {binding['index_source']: binding for binding in projected['bindings'] if binding['index_source'] is not None}
    bindings = []
    for cell in cells:
        source, target = cell['source_offset'], cell['target']; ordinal = bisect.bisect_right(starts, source) - 1
        if ordinal < 0:
            raise ValueError('Physical unit cell before first root')
        root = starts[ordinal]; end = starts[ordinal + 1] if ordinal + 1 < len(starts) else DIRECTORY
        if not root <= source < end or (source - root) % 4:
            raise ValueError('Physical unit cell outside its root')
        relative = source - root; animation = (relative // 24) * 4 + (1 if relative % 24 >= 12 else 0)
        part = relative % 12 // 4; found = by_cell.get(source)
        if target['kind'] == 'sequence' and found is None:
            raise ValueError('Physical unit sequence link was lost')
        for sid in aliases[root]:
            binding = {'sprite_id': sid, 'animation': animation, 'part': part, 'index_source': source,
                       'sequence': found['sequence'] if found else None}
            if found and 'target' in found:
                binding['target'] = found['target']
            elif not found:
                binding['target'] = {'kind': 'null'}
            bindings.append(binding)
    bindings.sort(key=lambda item: (item['sprite_id'], item['animation'], item['part']))
    return {**projected, 'physical_sprite_ids': True, 'bindings': bindings}
