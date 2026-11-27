"""Offline projection of authenticated EUR localized OBJ animation sources."""
import hashlib
import struct


class SourceBytes:
    """Sparse, overlap-checked ROM addresses reconstructed from extracted owners."""
    def __init__(self):
        self.data = {}

    def add(self, offset, content):
        if type(offset) is not int or offset < 0 or offset + len(content) > 0x2000000:
            raise ValueError('Invalid localized OBJ source address')
        for index, byte in enumerate(content):
            at = offset + index
            if at in self.data and self.data[at] != byte:
                raise ValueError('Conflicting localized OBJ source owners')
            self.data[at] = byte

    def read(self, offset, size):
        if type(offset) is not int or type(size) is not int or size < 0 or size > 65536:
            raise ValueError('Invalid localized OBJ read')
        try:
            return bytes(self.data[offset + i] for i in range(size))
        except KeyError as error:
            raise ValueError('Missing localized OBJ source at ' + hex(error.args[0])) from error

    def half(self, offset):
        return int.from_bytes(self.read(offset, 2), 'little')

    def word(self, offset):
        return int.from_bytes(self.read(offset, 4), 'little')


def add_verified_source(memory, sources, offset, payload):
    matches = [source for source in sources if source['offset'] == offset and source['length'] == len(payload)]
    if len(matches) != 1 or hashlib.sha256(payload).hexdigest() != matches[0]['sha256']:
        raise ValueError('Prior localized OBJ source SHA-256 mismatch at ' + hex(offset))
    memory.add(offset, payload)


def encode_obj_list(item):
    objects = item['objects']
    if not isinstance(objects, list) or len(objects) >= 32768:
        raise ValueError('Invalid prior OBJ list')
    out = bytearray(struct.pack('<H', len(objects)))
    for obj in objects:
        x, y = obj['x'], obj['y']
        if type(x) is not int or type(y) is not int or not -256 <= x <= 255 or not -128 <= y <= 127:
            raise ValueError('Invalid prior OBJ coordinates')
        for key, maximum in (('affine_mode', 3), ('object_mode', 3), ('shape', 3), ('transform', 31),
                             ('size', 3), ('tile', 1023), ('priority', 3), ('palette', 15)):
            if type(obj[key]) is not int or not 0 <= obj[key] <= maximum:
                raise ValueError('Invalid prior OBJ attribute')
        if type(obj['mosaic']) is not bool or type(obj['color_256']) is not bool:
            raise ValueError('Invalid prior OBJ flags')
        a0 = (y & 255) | obj['affine_mode'] << 8 | obj['object_mode'] << 10 | obj['mosaic'] << 12 | obj['color_256'] << 13 | obj['shape'] << 14
        a1 = (x & 511) | obj['transform'] << 9 | obj['size'] << 14
        a2 = obj['tile'] | obj['priority'] << 10 | obj['palette'] << 12
        out.extend(struct.pack('<HHH', a0, a1, a2))
    return bytes(out)


def decode_custom_lz(source, output_size):
    """Decode the original BE-size LZ/RLE format with strict source bounds."""
    if len(source) < 4 or int.from_bytes(source[:4], 'big') != output_size or not 0 < output_size <= 65536:
        raise ValueError('Invalid localized custom LZ size')
    out, position = bytearray(), 4
    def take():
        nonlocal position
        if position >= len(source):
            raise ValueError('Truncated localized custom LZ stream')
        value = source[position]; position += 1
        return value
    while len(out) < output_size:
        token = take()
        if token & 0x80:
            length, distance, fill = ((token >> 3) & 15) + 3, ((token & 7) << 8 | take()) + 1, 0
        elif token & 0x40:
            length = (token & 63) + 1
            if position + length > len(source) or len(out) + length > output_size:
                raise ValueError('Invalid localized custom LZ literal')
            out.extend(source[position:position + length]); position += length
            continue
        elif token & 0x20:
            length, distance, fill = (token & 31) + 2, None, 0
        elif token & 0x10:
            nxt = take(); length, distance, fill = ((token & 15) | ((nxt >> 2) & 48)) + 4, ((nxt & 63) << 8 | take()) + 1, 0
        elif token == 0:
            length, high = take() + 5, take(); distance, fill = (high << 8 | take()) + 1, 0
        elif token in (1, 2):
            length, distance, fill = take() + 3, None, 255 if token == 1 else 0
        else:
            continue
        if len(out) + length > output_size or distance is not None and distance > len(out):
            raise ValueError('Invalid localized custom LZ token')
        if distance is None:
            out.extend(bytes([fill]) * length)
        else:
            for _ in range(length):
                out.append(out[-distance])
    if position != len(source):
        raise ValueError('Trailing localized custom LZ bytes')
    return bytes(out)


def decode_bank(memory, objects, sequence):
    def offsets(root):
        if root % 4:
            raise ValueError('Unaligned localized OBJ index')
        first = memory.word(root)
        if not first or first % 4 or first // 4 > 4096:
            raise ValueError('Invalid localized OBJ index boundary')
        values = list(struct.unpack('<' + str(first // 4) + 'I', memory.read(root, first)))
        if any(v % 2 for v in values) or any(a >= b for a, b in zip(values, values[1:])):
            raise ValueError('Invalid localized OBJ relative offsets')
        return values
    object_offsets, sequence_offsets = offsets(objects), offsets(sequence)
    lists, sequences = [], []
    for offset in object_offsets:
        at = objects + offset; count = memory.half(at)
        if count >= 32768:
            raise ValueError('Invalid localized OBJ list count')
        raw = memory.read(at + 2, count * 6); items = []
        for a0, a1, a2 in struct.iter_unpack('<HHH', raw):
            x, y = a1 & 511, a0 & 255
            items.append({'x': x - 512 if x >= 256 else x, 'y': y - 256 if y >= 128 else y,
                          'affine_mode': a0 >> 8 & 3, 'object_mode': a0 >> 10 & 3,
                          'mosaic': bool(a0 & 0x1000), 'color_256': bool(a0 & 0x2000), 'shape': a0 >> 14,
                          'transform': a1 >> 9 & 31, 'size': a1 >> 14, 'tile': a2 & 1023,
                          'priority': a2 >> 10 & 3, 'palette': a2 >> 12})
        lists.append({'source_offset': at, 'objects': items})
    for offset in sequence_offsets:
        at = sequence + offset; count = memory.half(at)
        if count > 255:
            raise ValueError('Invalid localized OBJ frame count')
        raw = memory.read(at + 2, count * 4); frames = []
        for index, duration in struct.iter_unpack('<HH', raw):
            if index >= len(lists):
                raise ValueError('Invalid localized OBJ list reference')
            frames.append({'object_list': index, 'duration': duration})
        sequences.append({'source_offset': at, 'frames': frames})
    return {'object_root': objects, 'object_offsets': object_offsets, 'lists': lists,
            'sequence_root': sequence, 'sequence_offsets': sequence_offsets, 'sequences': sequences}


def project_localized_obj(segments, static, static_sources, firaga, firaga_sources, ui, ui_sources):
    """Join e357 spans with authenticated existing owners, then decode all banks."""
    memory = SourceBytes()
    allowed = {'animation-descriptors', 'selector-literals', 'obj-animation-list-index', 'obj-animation-list',
               'obj-animation-sequence-index', 'obj-animation-sequence', 'tile-streams'}
    for offset, role, content in segments:
        if role not in allowed:
            raise ValueError('Unexpected localized OBJ source role')
        memory.add(offset, content)
    descriptor = struct.pack('<IIHH', firaga['sequence_pointer'], firaga['object_pointer'], *firaga['descriptor_fields'])
    add_verified_source(memory, firaga_sources, 0x398f90, descriptor)
    add_verified_source(memory, firaga_sources, 0x418e9c, struct.pack('<' + str(len(firaga['object_offsets'])) + 'I', *firaga['object_offsets']))
    add_verified_source(memory, firaga_sources, 0x412050, struct.pack('<I', firaga['sequence_relative_offset']))
    sequence = firaga['sequence']; frames = sequence['frames']
    sequence_bytes = struct.pack('<H', len(frames)) + b''.join(struct.pack('<HH', f['object_list'], f['duration']) for f in frames)
    add_verified_source(memory, firaga_sources, 0x412054, sequence_bytes)
    prior_lists = [item for item in static['lists'] if 0x418eec <= item['source_offset'] < 0x4190c0]
    if len(prior_lists) != 19:
        raise ValueError('Missing prior localized OBJ lists')
    for item in prior_lists:
        add_verified_source(memory, static_sources, item['source_offset'], encode_obj_list(item))
    pointers = [item for item in ui['direct_pointers'] if item['source_offset'] == 0x95548]
    if len(pointers) != 1 or pointers[0]['bank_offset'] != 0x476214:
        raise ValueError('Missing prior localized graphics pointer')
    add_verified_source(memory, ui_sources, 0x95548, struct.pack('<I', 0x08476214))
    prior_banks = [item for item in ui['banks'] if item['source_offset'] == 0x476214]
    if len(prior_banks) != 1:
        raise ValueError('Missing prior localized graphics bank')
    third_encoded = bytes(prior_banks[0]['encoded_source'])
    add_verified_source(memory, ui_sources, 0x476214, third_encoded)
    graphics = []
    for address, size in ((0x430c5c, 1984), (0x4537d0, 1856), (0x476214, 1920)):
        candidates = [content for offset, role, content in segments if offset == address and role == 'tile-streams']
        encoded = third_encoded if address == 0x476214 else candidates[0] if len(candidates) == 1 else None
        if encoded is None:
            raise ValueError('Missing localized tile stream')
        decoded = decode_custom_lz(encoded, size)
        words = list(struct.unpack('<' + str(size // 4) + 'I', decoded))
        if address == 0x476214 and words != prior_banks[0]['words']:
            raise ValueError('Prior localized tile output mismatch')
        graphics.append({'source_offset': address, 'words': words})
    banks, labels = [], []
    for language in range(5):
        for mode in range(28):
            at = 0x398e94 + (language * 28 + mode) * 12
            sequence, objects, tile_offset, tile_size = struct.unpack('<IIHH', memory.read(at, 12))
            if mode < 2:
                if sequence or objects or tile_offset or tile_size:
                    raise ValueError('Invalid reserved localized OBJ descriptor')
                continue
            if not tile_size or tile_size % 32:
                raise ValueError('Invalid localized OBJ tile size')
            if not 0x08000000 <= objects < 0x0a000000 or not 0x08000000 <= sequence < 0x0a000000:
                raise ValueError('Invalid localized OBJ root pointer')
            decoded = decode_bank(memory, objects - 0x08000000, sequence - 0x08000000)
            decoded['tile_source'] = {'kind': 'localized', 'language': language, 'mode': mode,
                                      'offset_tiles': tile_offset, 'length_bytes': tile_size}
            banks.append(decoded)
            labels.append('Language ' + str(language) + ' · mode ' + str(mode) + ' · tile offset ' + str(tile_offset) + ' · ' + str(tile_size) + ' bytes')
    for name, root in (('Default', 0x954ac), ('Language 2', 0x954c8), ('Language 4', 0x95548)):
        tiles, sequence, objects = struct.unpack('<III', memory.read(root, 12))
        if tiles not in (0x08430c5c, 0x084537d0, 0x08476214) or not 0x08000000 <= objects < 0x0a000000 or not 0x08000000 <= sequence < 0x0a000000:
            raise ValueError('Invalid direct localized OBJ roots')
        decoded = decode_bank(memory, objects - 0x08000000, sequence - 0x08000000)
        decoded['tile_source'] = {'kind': 'direct', 'source_offset': tiles - 0x08000000}
        banks.append(decoded)
        labels.append(name + ' · graphics ' + hex(tiles - 0x08000000))
    return {'banks': banks, 'labels': labels, 'selector_tables': [], 'graphics': graphics}


def project_localized_tile_archives(groups, catalog, sources, banks):
    """Link language-selected archive output to descriptor-proven tile slices."""
    matches = [item for item in groups if item['source_offset'] == 0x71e658]
    if len(matches) != 1 or not isinstance(matches[0]['pointers'], list) or len(matches[0]['pointers']) != 5:
        raise ValueError('Missing localized graphics selector')
    pointers = matches[0]['pointers']
    if any(type(value) is not int or not 0 <= value < 0x2000000 for value in pointers) or len(set(pointers)) != 5:
        raise ValueError('Invalid localized graphics selector pointers')
    directory = struct.pack('<5I', *(0x08000000 + value for value in pointers))
    add_verified_source(SourceBytes(), sources, 0x71e658, directory)
    base = [family for family in catalog['families'] if family['id'] == 'base']
    if len(base) != 1:
        raise ValueError('Missing localized indexed graphics family')
    output = []
    for source in pointers:
        matches = [item for item in base[0]['archives'] if item['source_offset'] == source]
        if len(matches) != 1 or matches[0]['fixed_decoded_bytes'] != 64:
            raise ValueError('Invalid localized graphics archive')
        programs = matches[0]['programs']
        if not programs or len(programs) > 4096 or any(len(item['words']) != 16 for item in programs):
            raise ValueError('Invalid localized graphics program size')
        output.append({'source_offset': source, 'words': [word for item in programs for word in item['words']]})
    for item in banks:
        source = item['tile_source']
        if source['kind'] != 'localized':
            continue
        language, start, size = source['language'], source['offset_tiles'] * 8, source['length_bytes'] // 4
        if not 0 <= language < len(output) or size <= 0 or source['length_bytes'] % 32 or start + size > len(output[language]['words']):
            raise ValueError('Localized graphics slice is outside its language archive')
    return output
