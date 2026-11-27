#!/usr/bin/env python3
"""Generate an offline resource wiki for an extracted dataset."""
import argparse
import base64
import bisect
from datetime import datetime
import hashlib
import json
import os
from pathlib import Path
import re
import struct
import tempfile
import tomllib
from urllib.parse import quote

from extract_rom_data import DESCRIPTIONS
from viewer_conditional_lookups import CONDITIONAL_LOOKUPS
from viewer_localized_obj import project_localized_obj, project_localized_tile_archives
from viewer_physical_audio import merge_physical_audio
from viewer_physical_compact import (PRIOR_IDS as PHYSICAL_COMPACT_PRIOR_IDS, annotate_authenticated_contexts,
                                     project_physical_compact, project_physical_descriptors, project_witnessed_links)


def indexed_glyphs(glyphs):
    if not isinstance(glyphs, list) or not glyphs:
        raise ValueError('Empty font bank')
    indexed = {}
    for glyph in glyphs:
        index, width, words = (glyph[key] for key in ('index', 'width', 'tile_words'))
        if (type(index) is not int or not 0 <= index <= 65535 or index in indexed
                or type(width) is not int or not 0 <= width <= 16
                or not isinstance(words, list) or len(words) != 32
                or any(type(word) is not int or not 0 <= word <= 0xffffffff for word in words)):
            raise ValueError('Invalid font glyph')
        indexed[index] = glyph
    return indexed


def decode_gba_lz77(source, output_limit=16 * 1024 * 1024):
    """Decode one complete, bounded GBA type-0x10 stream."""
    if len(source) < 4 or source[0] != 0x10:
        raise ValueError('Invalid LZ77 header')
    size = int.from_bytes(source[1:4], 'little')
    if not 0 < size <= output_limit:
        raise ValueError('Invalid LZ77 output length')
    output, position = bytearray(), 4
    while len(output) < size:
        if position >= len(source):
            raise ValueError('Truncated LZ77 flags')
        flags = source[position]
        position += 1
        for bit in range(7, -1, -1):
            if len(output) == size:
                break
            if flags & (1 << bit):
                if position + 2 > len(source):
                    raise ValueError('Truncated LZ77 reference')
                first, second = source[position:position + 2]
                position += 2
                length, distance = (first >> 4) + 3, ((first & 15) << 8 | second) + 1
                if distance > len(output):
                    raise ValueError('Invalid LZ77 history reference')
                for _ in range(min(length, size - len(output))):
                    output.append(output[-distance])
            else:
                if position >= len(source):
                    raise ValueError('Truncated LZ77 literal')
                output.append(source[position])
                position += 1
    if position != len(source):
        raise ValueError('Trailing LZ77 source bytes')
    return bytes(output)


def project_observed_lz(catalog):
    """Validate extracted streams and retain their outputs for manual tile previews."""
    def byte_array(value):
        if (not isinstance(value, list) or len(value) > 16 * 1024 * 1024
                or any(type(item) is not int or not 0 <= item <= 255 for item in value)):
            raise ValueError('Invalid LZ77 byte array')
        return bytes(value)

    contexts = {
        0xb0e2b8: 'Red Spring observation', 0xb02ea4: 'Red Spring observation',
        0xb030c0: 'Red Spring observation', 0xb0dd5c: 'Red Spring observation',
        0xb0df2c: 'Red Spring observation', 0xb0ecd0: 'Red Spring observation',
        0xb151d8: 'Red Spring observation', 0xaff6e0: 'Soil Evidence observation',
        0xb0b4e4: 'Soil Evidence observation', 0xb0dd10: 'Soil Evidence observation',
        0xaf4998: 'Firaga observation', 0xaf5454: 'Firaga observation', 0xaf5660: 'Firaga observation',
        0xaffc70: 'Star Cross observation', 0xb00d9c: 'Star Cross observation',
        0xb4fa4c: 'VRAM initialization', 0xb52764: 'VRAM initialization', 0xb53010: 'VRAM initialization',
    }
    additional = catalog.get('additional_blocks') or []
    vram = catalog.get('vram_bank')
    if not isinstance(additional, list) or len(additional) > 256:
        raise ValueError('Invalid additional LZ77 streams')
    blocks = [catalog, *additional]
    if vram is not None:
        if (not isinstance(vram, dict) or not isinstance(vram['blocks'], list) or len(vram['blocks']) != 3
                or not isinstance(vram['source_literals'], list) or len(vram['source_literals']) != 3):
            raise ValueError('Invalid VRAM LZ77 bank')
        for literal, block in zip(vram['source_literals'], vram['blocks']):
            raw = byte_array(literal)
            if len(raw) != 4 or int.from_bytes(raw, 'little') != 0x08000000 + block['source_offset']:
                raise ValueError('Invalid VRAM LZ77 source binding')
        blocks.extend(vram['blocks'])
    streams, seen, total = [], set(), 0
    for block in blocks:
        offset = block['source_offset']
        if type(offset) is not int or not 0 <= offset <= 0xffffffff or offset in seen:
            raise ValueError('Invalid LZ77 source identity')
        encoded, decoded = byte_array(block['encoded_source']), byte_array(block['decoded'])
        total += len(decoded)
        if total > 32 * 1024 * 1024 or decode_gba_lz77(encoded) != decoded:
            raise ValueError('LZ77 decoded output differs from source')
        seen.add(offset)
        words = [int.from_bytes(decoded[index:index + 4], 'little') for index in range(0, len(decoded) - 3, 4)]
        streams.append({'source_offset': offset, 'encoded_length': len(encoded), 'decoded_length': len(decoded),
                        'context': contexts.get(offset), 'words': words, 'tail': list(decoded[len(words) * 4:])})
    return {'streams': streams}


def project_transfer_lz(content, sources):
    """Decode individually hashed transfer inputs in manifest file order."""
    if (not isinstance(content, bytes) or len(content) > 32 * 1024 * 1024
            or not isinstance(sources, list) or not 0 < len(sources) <= 4096):
        raise ValueError('Invalid transfer LZ77 input or source list')
    streams, ranges, cursor, total = [], [], 0, 0
    for source in sources:
        offset, length = source['offset'], source['length']
        if (type(offset) is not int or not 0 <= offset <= 0xffffffff
                or type(length) is not int or length <= 0 or offset + length > 0x100000000
                or source['role'] != 'lz77-input' or cursor + length > len(content)):
            raise ValueError('Invalid transfer LZ77 source extent or role')
        encoded = content[cursor:cursor + length]
        if hashlib.sha256(encoded).hexdigest() != source['sha256']:
            raise ValueError('Transfer LZ77 source SHA-256 mismatch')
        decoded = decode_gba_lz77(encoded)
        total += len(decoded)
        if total > 32 * 1024 * 1024:
            raise ValueError('Transfer LZ77 output limit exceeded')
        words = [int.from_bytes(decoded[index:index + 4], 'little') for index in range(0, len(decoded) - 3, 4)]
        streams.append({'source_offset': offset, 'encoded_length': length, 'decoded_length': len(decoded),
                        'context': 'Task 345 transfer input', 'words': words, 'tail': list(decoded[len(words) * 4:])})
        ranges.append((offset, offset + length)); cursor += length
    ranges.sort()
    if cursor != len(content) or any(left[1] > right[0] for left, right in zip(ranges, ranges[1:])):
        raise ValueError('Overlapping transfer LZ77 sources or trailing bytes')
    return {'streams': streams}


def decode_tilemap_tokens(tokens, expected, base_runs=False):
    """Decode the stored literal, repeating and incrementing halfword runs."""
    def word(value):
        return type(value) is int and 0 <= value <= 0xffff

    if not isinstance(tokens, list) or not tokens:
        raise ValueError('Empty tilemap stream')
    cells = []
    for token in tokens:
        control = token['control']
        if not word(control):
            raise ValueError('Invalid tilemap control')
        count, mode = control & 0x3fff, control & 0xc000
        values = [token['value']] if base_runs else token['values']
        if (not count or len(cells) + count > expected or base_runs and not mode
                or not isinstance(values, list) or len(values) != (1 if mode else count)
                or any(not word(value) for value in values)):
            raise ValueError('Invalid tilemap token')
        if mode & 0x8000:
            cells.extend((values[0] + index) & 0xffff for index in range(count))
        elif mode & 0x4000:
            cells.extend([values[0]] * count)
        else:
            cells.extend(values)
    if len(cells) != expected:
        raise ValueError('Incomplete tilemap stream')
    return cells


def project_binary_segments(content, sources):
    """Map individually hashed source ranges to their concatenated file bytes."""
    if (not isinstance(content, bytes) or len(content) > 16 * 1024 * 1024
            or not isinstance(sources, list) or not 0 < len(sources) <= 8192):
        raise ValueError('Invalid binary source list')
    segments, intervals, cursor = [], [], 0
    for source in sources:
        offset, length, role = source['offset'], source['length'], source['role']
        if (type(offset) is not int or not 0 <= offset <= 0xffffffff or type(length) is not int or length <= 0
                or offset + length > 0x100000000 or not isinstance(role, str) or cursor + length > len(content)):
            raise ValueError('Invalid binary source extent')
        chunk = content[cursor:cursor + length]
        if hashlib.sha256(chunk).hexdigest() != source['sha256']:
            raise ValueError('Binary source SHA-256 mismatch')
        segments.append((offset, role, chunk)); intervals.append((offset, offset + length)); cursor += length
    intervals.sort()
    if cursor != len(content) or any(a[1] > b[0] for a, b in zip(intervals, intervals[1:])):
        raise ValueError('Overlapping binary sources or trailing bytes')
    return segments


def project_screen_update_curves(content, sources):
    """Expose both source-backed curve banks, including their index-150 aliases."""
    segments = project_binary_segments(content, sources)
    expected = [(0x11a17c, 'reader_literals', 4), (0x11a1cc, 'reader_literals', 4),
                (0x11a204, 'reader_literals', 4), (0x11a39c, 'reader_literals', 8),
                (0x11b874, 'reader_literals', 4), (0x11b8c4, 'reader_literals', 4),
                (0x11b8fc, 'reader_literals', 4), (0x11bab0, 'reader_literals', 4),
                (0x3a8e3c, 'curves_and_directories', 3199),
                (0x3aa1c4, 'curves_and_directories', 3199)]
    if [(offset, role, len(chunk)) for offset, role, chunk in segments] != expected:
        raise ValueError('Unexpected screen-update source ranges')
    banks = []
    for root, _, chunk in segments[-2:]:
        slots = (1, 2, 7, 8)
        directory = struct.unpack_from('<12I', chunk, 1200)
        for index, pointer in enumerate(directory):
            expected_pointer = 0x08000000 + root + 300 * slots.index(index) if index in slots else 0
            if pointer != expected_pointer:
                raise ValueError('Invalid screen-update scale directory')
        scales = [{'kind': 'scale', 'slot': slot, 'source_offset': root + 300 * index,
                   'values': list(struct.unpack_from('<151H', chunk, 300 * index))}
                  for index, slot in enumerate(slots)]
        rows = [{'kind': 'row', 'index': index, 'source_offset': root + 1248 + 150 * index,
                 'values': list(chunk[1248 + 150 * index:1248 + 150 * index + 151])}
                for index in range(13)]
        if any(len(item['values']) != 151 for item in rows) or chunk[-1] != 0:
            raise ValueError('Invalid screen-update curve extent')
        banks.append({'source_offset': root, 'directory_offset': root + 1200, 'curves': scales + rows})
    if any(a['values'] != b['values'] for a, b in zip(banks[0]['curves'], banks[1]['curves'])):
        raise ValueError('Screen-update bank copies disagree')
    return {'banks': banks}


def project_screen_update_grids(curves, catalog):
    """Join documented grid-row pointers to the corresponding byte curves."""
    if not curves or not catalog or not catalog.get('auxiliary'):
        return None
    groups = catalog['auxiliary']['groups']
    if len(curves['banks']) != 2:
        raise ValueError('Invalid screen-update bank count')
    grids = []
    for bank, (name, root) in zip(curves['banks'], (('grid_first', 0x3a9abc), ('grid_second', 0x3aae44))):
        matches = [(index, item) for index, item in enumerate(groups) if item['id'] == name]
        if len(matches) != 1:
            raise ValueError('Missing or ambiguous screen-update grid')
        group_index, group = matches[0]
        if group['source_offset'] != root or len(group['rows']) != 24:
            raise ValueError('Invalid screen-update grid extent')
        by_pointer = {0x08000000 + item['source_offset']: index for index, item in enumerate(bank['curves'])
                      if item['kind'] == 'row'}
        if len(by_pointer) != 13:
            raise ValueError('Invalid screen-update row curve directory')
        rows = []
        for index, row in enumerate(group['rows']):
            x, y, pointer = row['x'], row['y'], row['value']
            if (type(x) is not int or type(y) is not int or not -32768 <= x <= 32767 or not -32768 <= y <= 32767
                    or type(pointer) is not int or pointer not in by_pointer):
                raise ValueError('Invalid screen-update grid row')
            rows.append({'source_offset': root + 16 * index, 'x': x, 'y': y, 'curve': by_pointer[pointer]})
        grids.append({'source_offset': root, 'group': group_index, 'rows': rows})
    return {'path': catalog['path'], 'banks': grids}


def project_background_effect_graphics(content, sources):
    """Expose the documented EUR effect descriptors, tile blocks and maps."""
    segments = project_binary_segments(content, sources)
    expected = [('descriptor-table-literal', 0x12fa38, 4), ('tile-bank-literal', 0x12fad4, 4),
                ('graphics-descriptors', 0x3ad318, 300), ('tile-blocks', 0xd6b4c8, 147008),
                ('tilemaps', 0xd8f308, 30000)]
    if [(role, offset, len(chunk)) for offset, role, chunk in segments] != expected:
        raise ValueError('Unexpected background effect source ranges')
    roots, tile_root = (int.from_bytes(segments[index][2], 'little') for index in (0, 1))
    if roots != 0x083ad318 or tile_root != 0x08d6b4c8:
        raise ValueError('Unexpected background effect source roots')
    descriptors, tiles, maps = (segments[index][2] for index in (2, 3, 4))
    result, cursor = [], 0
    for index in range(25):
        offset, length, map_pointer = struct.unpack_from('<III', descriptors, index * 12)
        if (offset != cursor or not length or length % 64 or offset + length > len(tiles)
                or map_pointer != 0x08d8f308 + index * 1200):
            raise ValueError('Invalid background effect descriptor')
        words = list(struct.unpack_from('<' + str(length // 4) + 'I', tiles, offset))
        cells = list(struct.unpack_from('<600H', maps, index * 1200))
        result.append({'id': index, 'descriptor_offset': 0x3ad318 + index * 12,
                       'tile_offset': 0xd6b4c8 + offset, 'tile_bytes': length,
                       'map_offset': map_pointer - 0x08000000, 'words': words, 'cells': cells})
        cursor += length
    if cursor != len(tiles):
        raise ValueError('Incomplete background effect tile bank')
    return {'effects': result}


def project_background_screen_banks(content, sources):
    """Expose the five bounded screen tile blocks and their declared maps."""
    segments = project_binary_segments(content, sources)
    expected = [('tile-directory-literals', 0x123618, 8), ('map-construction-literals', 0x1236bc, 16),
                ('map-upload-literals', 0x123724, 8), ('tile-pointers', 0x3ac770, 20),
                ('tile-byte-counts', 0x3ac784, 10), ('map-pointers', 0x3ac790, 20),
                ('map-dimensions', 0x3ac7a4, 20), ('tile-blocks', 0x6cd968, 65664),
                ('tilemaps-before-extra-row', 0x6dd9e8, 3994),
                ('tilemaps-after-extra-row', 0x6de9cc, 2320)]
    if [(role, offset, len(chunk)) for offset, role, chunk in segments] != expected:
        raise ValueError('Unexpected background screen source ranges')
    if (struct.unpack('<2I', segments[0][2]) != (0x083ac770, 0x083ac784)
            or struct.unpack('<4I', segments[1][2]) != (0x01000800, 0x083ac7a4, 0x083ac790, 0x03ff)
            or struct.unpack('<2I', segments[2][2]) != (0x083ac7a4, 0x06006000)):
        raise ValueError('Unexpected background screen reader literals')
    pointers = struct.unpack('<5I', segments[3][2])
    sizes = struct.unpack('<5H', segments[4][2])
    maps = struct.unpack('<5I', segments[5][2])
    dimensions = struct.unpack('<5I', segments[6][2])
    tiles = segments[7][2]; tile_offset = 0; map_address = 0x6dd9e8; screens = []
    for index, (pointer, size, map_pointer, dimension) in enumerate(zip(pointers, sizes, maps, dimensions)):
        width, height = dimension & 255, dimension >> 8 & 255
        if (pointer != 0x086cd968 + tile_offset or not size or size % 32
                or tile_offset + size > len(tiles) or map_pointer != 0x08000000 + map_address
                or dimension >> 16 or not 1 <= width <= 64 or not 1 <= height <= 64
                or 32 * (height - 1) + (width - 1) % 32 + 1024 * ((width - 1) // 32) >= 2048):
            raise ValueError('Invalid background screen descriptor')
        source_offset, _, source = segments[8 if index <= 2 else 9]
        start, length = map_address - source_offset, width * height * 2
        if start < 0 or start + length > len(source):
            raise ValueError('Invalid background screen map extent')
        screens.append({'id': index, 'tile_offset': 0x6cd968 + tile_offset, 'map_offset': map_address,
                        'width': width, 'height': height,
                        'words': list(struct.unpack_from('<' + str(size // 4) + 'I', tiles, tile_offset)),
                        'cells': list(struct.unpack_from('<' + str(width * height) + 'H', source, start))})
        tile_offset += size; map_address += length
        if index == 2:
            if map_address != 0x6de982:
                raise ValueError('Unexpected background screen extra-row boundary')
            map_address = 0x6de9cc
    if tile_offset != len(tiles) or map_address != 0x6df2dc:
        raise ValueError('Incomplete background screen banks')
    return {'screens': screens}


def project_type3_payloads(content, sources):
    """Validate type-3 ranges and expose complete LZ77 and raw payloads."""
    compressed, raw, errors, total = [], [], [], 0
    for offset, role, encoded in project_binary_segments(content, sources):
        if role not in ('lz77-input', 'raw-input'):
            continue
        try:
            decoded = decode_gba_lz77(encoded) if role == 'lz77-input' else encoded
        except ValueError as error:
            errors.append({'source_offset': offset, 'reason': str(error)})
            continue
        total += len(decoded)
        if total > 32 * 1024 * 1024:
            raise ValueError('Type-3 payload output limit exceeded')
        words = [int.from_bytes(decoded[index:index + 4], 'little') for index in range(0, len(decoded) - 3, 4)]
        item = {'source_offset': offset, 'encoded_length': len(encoded), 'decoded_length': len(decoded),
                'words': words, 'tail': list(decoded[len(words) * 4:])}
        (compressed if role == 'lz77-input' else raw).append(item)
    return {'streams': compressed, 'raw_streams': raw, 'errors': errors}


def project_transfer_directory(catalog):
    """Keep bounded transfer slots, rows and descriptors without inferring usage."""
    def number(value, maximum, label):
        if type(value) is not int or not 0 <= value <= maximum:
            raise ValueError('Invalid transfer directory ' + label)
        return value

    def entries(name, limit):
        value = catalog[name]
        if not isinstance(value, list) or len(value) > limit:
            raise ValueError('Invalid transfer directory ' + name)
        return value

    slots, rows, descriptors = [], [], []
    row_offsets, descriptor_offsets = set(), set()
    for slot in entries('slots', 512):
        slots.append({'row_pointer': number(slot['row_pointer'], 0xffffffff, 'row pointer'),
                      'row_count': number(slot['row_count'], 255, 'row count')})
    if sum(slot['row_count'] for slot in slots) > 4096:
        raise ValueError('Transfer directory row count exceeds limit')
    for row in entries('rows', 4096):
        offset = number(row['source_offset'], 0x01ffffff, 'row offset')
        if offset in row_offsets:
            raise ValueError('Duplicate transfer row offset')
        row_offsets.add(offset)
        rows.append({'source_offset': offset, 'field0': number(row['field0'], 255, 'field0'),
                     'field1': number(row['field1'], 255, 'field1'),
                     'field2': number(row['field2'], 65535, 'field2'),
                     'descriptor_pointer': number(row['descriptor_pointer'], 0xffffffff, 'descriptor pointer'),
                     'field8': number(row['field8'], 0xffffffff, 'field8')})
    for item in entries('descriptors', 4096):
        offset = number(item['source_offset'], 0x01ffffff, 'descriptor offset')
        if offset in descriptor_offsets:
            raise ValueError('Duplicate transfer descriptor offset')
        descriptor_offsets.add(offset)
        pointer = number(item['asset_pointer'], 0xffffffff, 'asset pointer')
        wrapper = item['wrapper']
        if wrapper is not None:
            wrapper = {'source_offset': number(wrapper['source_offset'], 0x01ffffff, 'wrapper offset'),
                       'kind': number(wrapper['kind'], 255, 'wrapper kind'),
                       'flags': number(wrapper['flags'], 255, 'wrapper flags'),
                       'opaque_halfword': number(wrapper['opaque_halfword'], 65535, 'wrapper halfword')}
            if not 0x08000000 <= pointer < 0x0a000000 or pointer - 0x08000000 != wrapper['source_offset']:
                raise ValueError('Transfer wrapper does not match its stored pointer')
        descriptors.append({'source_offset': offset, 'asset_pointer': pointer,
                            'parameter16': number(item['parameter16'], 65535, 'parameter16'),
                            'parameter8': number(item['parameter8'], 255, 'parameter8'),
                            'opaque_byte': number(item['opaque_byte'], 255, 'opaque byte'), 'wrapper': wrapper})
    return {'slots': slots, 'rows': rows, 'descriptors': descriptors}


def project_indexed_tilemap_controls(catalog):
    """Keep bounded control records and their stored pointers without inventing scene names."""
    def number(value, maximum, label):
        if type(value) is not int or not 0 <= value <= maximum:
            raise ValueError('Invalid indexed tilemap ' + label)
        return value

    slots = catalog['slots']
    if not isinstance(slots, list) or len(slots) > 512:
        raise ValueError('Invalid indexed tilemap slots')
    projected = []
    for slot in slots:
        row, descriptor = slot['row'], slot['descriptor']
        projected.append({
            'pointer_offset': number(slot['pointer_offset'], 0x01fffffc, 'pointer offset'),
            'pointer': number(slot['pointer'], 0xffffffff, 'pointer'),
            'row': {'source_offset': number(row['source_offset'], 0x01fffff4, 'row offset'),
                    'field0': number(row['field0'], 255, 'row byte 0'),
                    'field1': number(row['field1'], 255, 'row byte 1'),
                    'field2': number(row['field2'], 65535, 'row halfword'),
                    'descriptor_pointer': number(row['descriptor_pointer'], 0xffffffff, 'descriptor pointer'),
                    'field8': number(row['field8'], 0xffffffff, 'row word')},
            'descriptor': {'source_offset': number(descriptor['source_offset'], 0x01fffff8, 'descriptor offset'),
                           'tilemap_pointer': number(descriptor['tilemap_pointer'], 0xffffffff, 'tilemap pointer'),
                           'parameter16': number(descriptor['parameter16'], 65535, 'descriptor halfword'),
                           'parameter8': number(descriptor['parameter8'], 255, 'descriptor byte'),
                           'opaque_byte': number(descriptor['opaque_byte'], 255, 'opaque byte')},
        })
    return {'slots': projected}


def decode_compact_cell_record(source_offset, record):
    """Decode bounded compact-cell frames and retain their stored sequence bytes."""
    def half(offset):
        if offset < 0 or offset + 2 > len(record):
            raise ValueError('Truncated compact-cell halfword')
        return int.from_bytes(record[offset:offset + 2], 'little')

    flags, split = half(0), half(2)
    if flags not in (12, 14) or not 6 <= split <= len(record) - 2:
        raise ValueError('Invalid compact-cell record header')
    directory = record[4:split]
    prefix = int.from_bytes(directory[:2], 'little')
    if prefix < 2 or prefix > 128 or prefix % 2 or prefix >= len(directory):
        raise ValueError('Invalid compact-cell frame directory')
    offsets = [int.from_bytes(directory[index:index + 2], 'little') for index in range(0, prefix, 2)]
    value_bytes = 2 if flags == 12 else 1
    frames = []
    for index, start in enumerate(offsets):
        end = offsets[index + 1] if index + 1 < len(offsets) else len(directory)
        if start % 2 or start < prefix or start >= end or end > len(directory):
            raise ValueError('Invalid compact-cell frame offset')
        encoded = directory[start:end]
        if len(encoded) < 4:
            raise ValueError('Truncated compact-cell frame header')
        width, height, x, y = encoded[:4]
        count, cursor, cells, tokens = width * height, 4, [], []
        if not count:
            raise ValueError('Empty compact-cell frame')
        while len(cells) < count:
            if cursor + 2 > len(encoded):
                raise ValueError('Truncated compact-cell run control')
            control = int.from_bytes(encoded[cursor:cursor + 2], 'little'); cursor += 2
            length, mode = control & 0x3fff, control & 0xc000
            if not length or length > count - len(cells):
                raise ValueError('Invalid compact-cell run length')
            needed = value_bytes * (length if mode == 0 else 1)
            stored = (needed + 1) & ~1
            if cursor + stored > len(encoded):
                raise ValueError('Truncated compact-cell run payload')
            values = [int.from_bytes(encoded[pos:pos + value_bytes], 'little')
                      for pos in range(cursor, cursor + needed, value_bytes)]
            cursor += stored
            if mode == 0:
                cells.extend(values)
            else:
                mask, step = (0xff if value_bytes == 1 else 0xffff), bool(mode & 0x8000)
                cells.extend((values[0] + (n if step else 0)) & mask for n in range(length))
            tokens.append({'control': control, 'values': values})
        padding = len(encoded) - cursor
        if padding and not (index + 1 == len(offsets) and padding == 2):
            raise ValueError('Unexpected compact-cell frame padding')
        frames.append({'source_offset': source_offset + 4 + start, 'width': width, 'height': height,
                       'placement': [x, y], 'encoded_length': cursor, 'cells': cells, 'tokens': tokens})
    count = half(split)
    if not count or split + 2 + count * 2 != len(record):
        raise ValueError('Invalid compact-cell sequence extent')
    sequence = [{'frame_index': record[split + 2 + index * 2] & 63,
                 'high_flags': record[split + 2 + index * 2] >> 6,
                 'second_byte': record[split + 3 + index * 2]} for index in range(count)]
    return {'source_offset': source_offset, 'flags': flags, 'split_offset': split,
            'frames': frames, 'sequence': sequence}


def project_compact_cell_bank(content, sources):
    """Keep independently valid records from the authenticated packed source."""
    records, errors, total = [], [], 0
    for offset, role, record in project_binary_segments(content, sources):
        if role != 'compact-record':
            continue
        try:
            decoded = decode_compact_cell_record(offset, record)
        except ValueError as error:
            errors.append({'source_offset': offset, 'reason': str(error)})
            continue
        total += sum(len(frame['cells']) for frame in decoded['frames'])
        if total > 2_000_000:
            raise ValueError('Compact-cell output limit exceeded')
        records.append(decoded)
    return {'records': records, 'errors': errors}


def project_type2_rectangles(content, sources, observed=None, palette_rows=None):
    """Join available type-2 rectangles, copy windows and prior typed sources."""
    segments = project_binary_segments(content, sources)
    grouped = {}
    for offset, role, chunk in segments:
        grouped.setdefault(role, {})[offset] = chunk
    cell_bytes = {offset + index: value for offset, chunk in grouped.get('rectangle-cells', {}).items()
                  for index, value in enumerate(chunk)}
    maps, by_root, errors = [], {}, []
    for root, header in grouped.get('rectangle-header', {}).items():
        if len(header) != 2 or not 1 <= header[0] <= 16 or not 1 <= header[1] <= 16:
            errors.append({'source_offset': root, 'reason': 'Invalid rectangle header'})
            continue
        height, width = header
        try:
            raw = bytes(cell_bytes[root + 2 + index] for index in range(height * width * 2))
        except KeyError:
            errors.append({'source_offset': root, 'reason': 'Rectangle cells are incomplete'})
            continue
        cells = [int.from_bytes(raw[index:index + 2], 'little') for index in range(0, len(raw), 2)]
        item = {'kind': 'Type-2 source rectangle', 'source_offset': root, 'width': width, 'height': height,
                'cells': cells, 'metadata': {'source': 'type-2 packed header and cells'}}
        maps.append(item); by_root[root] = item

    entry_chunks = grouped.get('rectangle-list-entry', {})
    roots = {int.from_bytes(chunk[:4], 'little') - 0x08000000 for chunk in entry_chunks.values()
             if len(chunk) == 8 and 0x08000000 <= int.from_bytes(chunk[:4], 'little') < 0x09000000}
    if palette_rows:
        for row in palette_rows['rows']:
            root, width, height = row['source_offset'], row['width'], row['declared_rows']
            if root not in roots or root in by_root:
                continue
            start = root + 2 + width * 2
            try:
                tail = bytes(cell_bytes[start + index] for index in range((height - 1) * width * 2))
            except KeyError:
                errors.append({'source_offset': root, 'reason': 'Palette-row rectangle cells are incomplete'})
                continue
            cells = row['colors'] + [int.from_bytes(tail[index:index + 2], 'little') for index in range(0, len(tail), 2)]
            item = {'kind': 'Type-2 palette-row rectangle', 'source_offset': root, 'width': width, 'height': height,
                    'cells': cells, 'metadata': {'source': 'Firaga palette row and type-2 cells; values can be colors'}}
            maps.append(item); by_root[root] = item

    prior = {item['source_offset']: item for item in (observed or {}).get('maps', [])
             if item.get('source_offset') in roots and item.get('source_offset') not in by_root}
    entry_headers, header_rows = {}, {}
    def pointer(chunk):
        value = int.from_bytes(chunk, 'little')
        return value - 0x08000000 if 0x08000000 <= value < 0x09000000 else None
    for offset, chunk in grouped.get('rectangle-list-header', {}).items():
        if len(chunk) != 8 or not 1 <= chunk[0] <= 7:
            continue
        start = pointer(chunk[4:8])
        if start is not None:
            for index in range(chunk[0]):
                entry_headers.setdefault(start + index * 8, []).append(offset)
    for offset, chunk in grouped.get('transfer-row', {}).items():
        if len(chunk) != 12 or chunk[0] != 2:
            continue
        header = pointer(chunk[4:8])
        if header is not None:
            header_rows.setdefault(header, []).append(offset)

    prior_references, unresolved = {}, []
    for offset, chunk in entry_chunks.items():
        if len(chunk) != 8:
            unresolved.append({'entry_offset': offset, 'reason': 'Incomplete entry'})
            continue
        root = pointer(chunk[:4]); selected = by_root.get(root) or prior.get(root)
        reference = {'entry_offset': offset, 'source_offset': root, 'raw_byte4': chunk[4], 'raw_byte5': chunk[5],
                     'first_row': chunk[6], 'row_count': chunk[7],
                     'list_headers': sorted(set(entry_headers.get(offset, [])))}
        reference['transfer_rows'] = sorted({row for header in reference['list_headers'] for row in header_rows.get(header, [])})
        if not selected or not chunk[7] or chunk[6] + chunk[7] > selected['height']:
            unresolved.append({**reference, 'reason': 'Rectangle unavailable or invalid copy window'})
        elif root in by_root:
            selected.setdefault('type2_references', []).append(reference)
        else:
            prior_references.setdefault(root, []).append(reference)
    maps.sort(key=lambda item: item['source_offset'])
    return {'maps': maps, 'prior_references': prior_references, 'unresolved': unresolved, 'errors': errors}


def decode_tilemap_stream(start, read, byte_values):
    """Decode the bounded first traversal, leaving alignment and mode transforms unmodeled."""
    header = read(start, 6)
    width, height, origin_x, origin_y, mode, flags = header
    if (not 1 <= width <= 32 or not 1 <= height <= 32 or not flags & 2
            or mode not in ((1, 4) if byte_values else (1, 2, 4, 5, 6, 7))):
        raise ValueError('Invalid tilemap stream header')
    unit, cursor, cells, token_count = (1 if byte_values else 2), start + 6, [], 0
    while len(cells) < width * height:
        control = int.from_bytes(read(cursor, 2), 'little'); cursor += 2
        count, operation = control & 0x3fff, control & 0xc000
        if not count or len(cells) + count > width * height:
            raise ValueError('Invalid tilemap stream run length')
        size = unit * (count if operation == 0 else 1)
        raw = read(cursor, size); cursor += size
        values = [int.from_bytes(raw[index:index + unit], 'little') for index in range(0, size, unit)]
        if operation & 0x8000:
            mask = (1 << (8 * unit)) - 1
            cells.extend((values[0] + index) & mask for index in range(count))
        elif operation & 0x4000:
            cells.extend([values[0]] * count)
        else:
            cells.extend(values)
        if byte_values and size & 1:
            cursor += 1  # The source reader skips this unowned alignment byte.
        token_count += 1
    return {'kind': 'Decoded byte stream' if byte_values else 'Decoded halfword stream',
            'source_offset': start, 'width': width, 'height': height, 'cells': cells,
            'metadata': {'mode': mode, 'flags': flags, 'origin_x': origin_x, 'origin_y': origin_y,
                         'encoded_extent': cursor - start, 'tokens': token_count}}, cursor


def project_halfword_streams(segments):
    maps = []
    for offset, role, chunk in segments:
        if role not in ('halfword-stream', 'byte-stream'):
            raise ValueError('Unknown tilemap bank source role')
        if role != 'halfword-stream':
            continue
        def read(at, length):
            relative = at - offset
            if relative < 0 or relative + length > len(chunk):
                raise ValueError('Incomplete halfword tilemap source')
            return chunk[relative:relative + length]
        result, end = decode_tilemap_stream(offset, read, False)
        if end != offset + len(chunk):
            raise ValueError('Trailing halfword tilemap source')
        maps.append(result)
    return maps


def project_byte_streams(bank_segments, control_segments):
    """Join type-5 rows to their descriptors and decode available byte fragments."""
    data, starts = {}, set()
    for offset, role, chunk in bank_segments:
        if role == 'byte-stream':
            starts.add(offset)
            data.update((offset + index, byte) for index, byte in enumerate(chunk))
    descriptors = {}
    for offset, role, chunk in control_segments:
        if role == 'type5-transfer-descriptor':
            if len(chunk) != 6:
                raise ValueError('Invalid type-5 descriptor')
            descriptors[offset] = chunk
    roots = {}
    for offset, role, chunk in control_segments:
        if role != 'type5-transfer-row':
            continue
        if len(chunk) != 12 or chunk[0] != 5:
            raise ValueError('Invalid type-5 row')
        pointer = int.from_bytes(chunk[4:8], 'little')
        if not 0x08000000 <= pointer < 0x09000000:
            raise ValueError('Invalid type-5 descriptor pointer')
        descriptor = pointer - 0x08000000
        if descriptor not in descriptors:
            continue
        root = int.from_bytes(descriptors[descriptor][:4], 'little')
        if not 0x08000000 <= root < 0x09000000:
            raise ValueError('Invalid type-5 stream pointer')
        root -= 0x08000000
        if root in starts:
            roots.setdefault(root, []).append({'row_offset': offset, 'descriptor_offset': descriptor})
    used, maps = set(), []
    for root, references in sorted(roots.items()):
        consumed = set()
        def read(at, length):
            try:
                result = bytes(data[at + index] for index in range(length))
            except KeyError as error:
                raise ValueError('Incomplete byte tilemap source') from error
            consumed.update(range(at, at + length)); return result
        result, _ = decode_tilemap_stream(root, read, True)
        if used & consumed:
            raise ValueError('Overlapping byte tilemap streams')
        used.update(consumed); result['references'] = references; maps.append(result)
    return maps


def project_observed_tilemaps(catalog):
    """Project documented mode-4 streams and row-major rectangles into cells."""
    def word(value):
        return type(value) is int and 0 <= value <= 0xffff

    def geometry(item):
        source, width, height = (item[key] for key in ('source_offset', 'width', 'height'))
        if (type(source) is not int or source < 0 or type(width) is not int or not 1 <= width <= 64
                or type(height) is not int or not 1 <= height <= 64):
            raise ValueError('Invalid tilemap geometry')
        return source, width, height

    maps = []
    source, width, height = geometry(catalog)
    if catalog['mode'] != 4:
        raise ValueError('Unsupported tilemap mode')
    maps.append({'kind': 'Tilemap', 'source_offset': source, 'width': width, 'height': height,
                 'cells': decode_tilemap_tokens(catalog['runs'], width * height, True)})
    streams = catalog.get('additional_streams') or []
    if not isinstance(streams, list):
        raise ValueError('Invalid additional tilemaps')
    for stream in streams:
        source, width, height = geometry(stream)
        if stream['mode'] != 4:
            raise ValueError('Unsupported tilemap mode')
        maps.append({'kind': 'Tilemap', 'source_offset': source, 'width': width, 'height': height,
                     'cells': decode_tilemap_tokens(stream['tokens'], width * height)})
    groups = [('rectangles', 'Rectangle'), ('alternate_rectangles', 'Alternate rectangle'),
              ('additional_rectangles', 'Additional rectangle')]
    soil = catalog.get('soil_transfer_sources') or {}
    if not isinstance(soil, dict):
        raise ValueError('Invalid transfer rectangles')
    for key, kind in groups + [('soil_rectangles', 'Transfer rectangle')]:
        rectangles = soil.get('rectangles') if key == 'soil_rectangles' else catalog.get(key)
        if rectangles is None:
            continue
        if not isinstance(rectangles, list):
            raise ValueError('Invalid tilemap rectangles')
        for rectangle in rectangles:
            source, width, height = geometry(rectangle)
            cells = rectangle['cells']
            if not isinstance(cells, list) or len(cells) != width * height or any(not word(value) for value in cells):
                raise ValueError('Invalid rectangle cells')
            maps.append({'kind': kind, 'source_offset': source, 'width': width, 'height': height, 'cells': cells})
    return maps


def project_firaga_compact_cells(catalog):
    """Decode the witnessed compact cell frames without interpreting opaque timing fields."""
    def uint(value, maximum):
        return type(value) is int and 0 <= value <= maximum

    expected = {'caller_literal': 0x0839cc98, 'table_pointer': 0x08ae9c30, 'flags': 12, 'split_offset': 628}
    if any(type(catalog[key]) is not int or catalog[key] != value for key, value in expected.items()):
        raise ValueError('Invalid Firaga compact cell source')
    offsets = [20, 28, 44, 96, 148, 192, 280, 362, 434, 522]
    shapes = [(3, 2, 8), (3, 4, 16), (6, 6, 52), (7, 8, 52), (7, 8, 44),
              (10, 9, 88), (9, 9, 82), (7, 8, 72), (7, 8, 88), (6, 8, 100)]
    frames, sequence = catalog['frames'], catalog['sequence']
    if (catalog['directory_offsets'] != offsets or any(type(value) is not int for value in catalog['directory_offsets'])
            or not isinstance(frames, list) or len(frames) != 10
            or not isinstance(sequence, list) or len(sequence) != 10):
        raise ValueError('Invalid Firaga compact cell directory')
    maps = []
    for index, (frame, (width, height, length)) in enumerate(zip(frames, shapes)):
        header, tokens = frame['opaque_header'], frame['tokens']
        if (type(frame['source_offset']) is not int or frame['source_offset'] != 0xae9c34 + offsets[index]
                or type(frame['width']) is not int or frame['width'] != width
                or type(frame['height']) is not int or frame['height'] != height
                or not isinstance(header, list) or len(header) != 2 or any(not uint(value, 255) for value in header)):
            raise ValueError('Invalid Firaga compact frame geometry')
        cells = decode_tilemap_tokens(tokens, width * height)
        if 4 + sum(2 + 2 * len(token['values']) for token in tokens) != length:
            raise ValueError('Invalid Firaga compact frame extent')
        maps.append({'kind': 'Firaga compact frame ' + str(index), 'source_offset': frame['source_offset'],
                     'width': width, 'height': height, 'cells': cells, 'opaque_header': header, 'tokens': tokens})
    for entry in sequence:
        if (not uint(entry['frame_index'], 63) or not uint(entry['high_flags'], 3)
                or not uint(entry['second_byte'], 255)):
            raise ValueError('Invalid Firaga compact sequence entry')
    indexed = catalog.get('indexed_fields')
    if indexed is not None:
        if (not isinstance(indexed, dict) or any(not uint(indexed[key], maximum) for key, maximum in
                (('byte_cc96', 255), ('halfword_cca8', 65535), ('byte_ccba', 255),
                 ('byte_ccd2', 255), ('word_ccfc', 0xffffffff)))):
            raise ValueError('Invalid Firaga indexed fields')
    return {'maps': maps, 'sequence': sequence, 'metadata': {**expected, 'directory_offsets': offsets,
                                                               'indexed_fields': indexed}}


def project_source_grids(catalog, kind):
    """Keep documented matrix and rectangle values in their original row order."""
    def uint(value, maximum):
        return type(value) is int and 0 <= value <= maximum

    if kind == 'firaga_matrices':
        literals = catalog['literals']
        if (literals != [0x08370eb4, 0x083b9f7c, 0x08370eb4, 0x083b9e80]
                or any(type(value) is not int for value in literals)):
            raise ValueError('Invalid Firaga matrix literals')
        records, shapes = catalog['matrices'], [(0x3b9e80, 18, 7), (0x3b9f7c, 17, 6)]
        metadata, label = {'literals': literals}, 'Firaga copied matrix'
    elif kind == 'soil_rectangles':
        if type(catalog['alternate_literal']) is not int or catalog['alternate_literal'] != 0x08ae7874:
            raise ValueError('Invalid Soil Evidence rectangle pointer')
        records, shapes = catalog['rectangles'], [(0xae7810, 7, 7), (0xae7874, 3, 1)]
        metadata, label = {'alternate_literal': catalog['alternate_literal']}, 'Soil Evidence rectangle'
    else:
        raise ValueError('Unknown source grid catalog')
    if not isinstance(records, list) or len(records) != len(shapes):
        raise ValueError('Invalid source grid count')
    maps = []
    for index, (record, (offset, width, height)) in enumerate(zip(records, shapes)):
        cells = record['cells']
        if (type(record['source_offset']) is not int or record['source_offset'] != offset
                or type(record['width']) is not int or record['width'] != width
                or type(record['height']) is not int or record['height'] != height
                or not isinstance(cells, list) or len(cells) != width * height
                or any(not uint(value, 65535) for value in cells)):
            raise ValueError('Invalid source grid geometry or cells')
        maps.append({'kind': label + ' ' + str(index), 'source_offset': offset,
                     'width': width, 'height': height, 'cells': cells})
    return {'maps': maps, 'metadata': metadata}


def project_ui_tilemap_rectangles(catalog):
    """Project the three bounded source grids without interpreting their cells."""
    copies = catalog['halfword_copies']['tilemap_rectangles']
    literals = copies['caller_literals']
    expected = [0x083b0e5c, 0x083b0efc, 0x083b0e5c, 0x083b0efc, 0x0871e66c, 0x083ba95c]
    if literals != expected or any(type(value) is not int for value in literals):
        raise ValueError('Invalid UI rectangle source pointers')
    records = copies['rectangles']
    shapes = [(0x3b0e5c, 10, 8), (0x3b0efc, 12, 3), (0x3ba95c, 22, 17)]
    if not isinstance(records, list) or len(records) != len(shapes):
        raise ValueError('Invalid UI rectangle count')
    maps = []
    for index, (record, (offset, width, height)) in enumerate(zip(records, shapes)):
        bank = record['bank']
        cells = bank['halfwords']
        if (type(record['width']) is not int or record['width'] != width
                or type(record['height']) is not int or record['height'] != height
                or type(bank['source_offset']) is not int or bank['source_offset'] != offset
                or not isinstance(cells, list) or len(cells) != width * height
                or any(type(value) is not int or not 0 <= value <= 65535 for value in cells)):
            raise ValueError('Invalid UI rectangle geometry or cells')
        label = 'UI staged template' if index < 2 else 'UI localized-loader rectangle'
        maps.append({'kind': label + ' ' + str(index), 'source_offset': offset,
                     'width': width, 'height': height, 'cells': list(cells),
                     'ui_template': index < 2})
    return {'maps': maps, 'metadata': {'caller_literals': list(literals)}}


def project_signed_map_deltas(catalog):
    """Expose the bounded signed bytes and pointer from the European map reader."""
    table = catalog['signed_map_deltas']
    if not isinstance(table, dict):
        raise ValueError('Invalid signed map delta table')
    pointer, entries = table['caller_literal'], table['entries']
    if (type(pointer) is not int or pointer != 0x083954a4
            or not isinstance(entries, list) or len(entries) != 48
            or any(type(entry) is not int or not -128 <= entry <= 127 for entry in entries)):
        raise ValueError('Invalid signed map delta pointer or entries')
    return {'source_offset': pointer - 0x08000000, 'caller_literal': pointer, 'entries': list(entries)}


def project_localized_queued_grids(catalog):
    """Retain the documented 30 localized, queued halfword rectangles."""
    queued = catalog['localized_queued']
    pointers, lengths, literals, banks = (queued[key] for key in ('pointers', 'lengths', 'literals', 'banks'))
    roots = (0x438d00, 0x449f00, 0x45baf0, 0x46cd8c, 0x47e434)
    def uint(value):
        return type(value) is int and 0 <= value <= 0xffffffff
    if (not isinstance(pointers, list) or len(pointers) != 30 or any(not uint(value) for value in pointers)
            or not isinstance(lengths, list) or len(lengths) != 30 or any(type(value) is not int or value != 192 for value in lengths)
            or not isinstance(literals, list) or len(literals) != 2
            or any(type(value) is not int for value in literals) or literals != [0x08397fbc, 0x08398034]
            or not isinstance(banks, list) or len(banks) != len(roots)):
        raise ValueError('Invalid localized queued transfer tables')
    maps = []
    for language, (bank, root) in enumerate(zip(banks, roots)):
        words = bank['words']
        if (type(bank['bank_offset']) is not int or bank['bank_offset'] != root
                or type(bank['relative_offset']) is not int or bank['relative_offset'] != 0
                or not isinstance(words, list) or len(words) != 288 or any(not uint(value) for value in words)):
            raise ValueError('Invalid localized queued transfer bank')
        for entry in range(6):
            source = root + entry * 192
            if pointers[language * 6 + entry] != 0x08000000 + source:
                raise ValueError('Invalid localized queued transfer pointer')
            cells = [half for word in words[entry * 48:(entry + 1) * 48] for half in (word & 0xffff, word >> 16)]
            maps.append({'kind': f'Localized transfer · language {language} · entry {entry}',
                         'source_offset': source, 'width': 24, 'height': 4, 'cells': cells,
                         'metadata': {'language_id': language, 'entry_id': entry, 'pointer': pointers[language * 6 + entry],
                                      'source_bytes': 192, 'source_row_bytes': 48, 'destination_row_bytes': 64,
                                      'pointer_table': literals[0], 'length_table': literals[1]}})
    return {'maps': maps}


def project_firaga_transfer(catalog):
    """Preserve the source-authenticated transfer records and their pointer relationships."""
    expected = {
        'caller_literals': [0x0839cbd8, 0x0839cbcc], 'program_pointer': 0x0839cb9c, 'program_count': 4,
        'records': [
            {'kind': 2, 'flags': 0, 'parameter': 0, 'payload': 0x0839cafc, 'destination': 0},
            {'kind': 3, 'flags': 0, 'parameter': 0x60, 'payload': 0x0839cb04, 'destination': 0x06017f80},
            {'kind': 3, 'flags': 0, 'parameter': 0x2820, 'payload': 0x0839cb2c, 'destination': 0x0600d7c0},
            {'kind': 3, 'flags': 0, 'parameter': 0x0c40, 'payload': 0x0839cb34, 'destination': 0x06017340}],
        'palette_count': 2, 'palette_reserved': [0, 0, 0], 'palette_entries_pointer': 0x0839caec,
        'palette_entries': [{'source': 0x08ae6db8, 'parameters': [7, 1, 0, 1]},
                            {'source': 0x08ae6db0, 'parameters': [6, 1, 0, 1]}],
        'lz_wrappers': [{'header': 0x08af565c, 'argument': 0}, {'header': 0x08af4994, 'argument': 0},
                        {'header': 0x08af5450, 'argument': 3}],
        'format_headers': [{'kind': 2, 'flags': 2, 'reserved': 0x7ffd},
                           {'kind': 2, 'flags': 2, 'reserved': 0x7ffd},
                           {'kind': 1, 'flags': 2, 'reserved': 0x7ffd}],
    }

    def matches(value, pattern):
        if type(value) is not type(pattern):
            return False
        if isinstance(pattern, dict):
            return value.keys() == pattern.keys() and all(matches(value[key], item) for key, item in pattern.items())
        if isinstance(pattern, list):
            return len(value) == len(pattern) and all(matches(a, b) for a, b in zip(value, pattern))
        return value == pattern

    if not matches(catalog, expected):
        raise ValueError('Invalid Firaga transfer source binding')
    return {**catalog, 'lz_targets': [item['header'] - 0x08000000 + 4 for item in catalog['lz_wrappers']]}


def project_firaga_palette_rows(catalog):
    """Keep the two documented row prefixes without inventing additional colors."""
    rows = catalog['rows']
    if not isinstance(rows, list) or len(rows) != 2:
        raise ValueError('Invalid Firaga palette row count')
    projected = []
    for index, (row, declared, offset) in enumerate(zip(rows, (1, 3), (0xae6db0, 0xae6db8))):
        colors = row['colors']
        if (type(row['declared_rows']) is not int or row['declared_rows'] != declared
                or type(row['width']) is not int or row['width'] != 3
                or not isinstance(colors, list) or len(colors) != 3
                or any(type(value) is not int or not 0 <= value <= 65535 for value in colors)):
            raise ValueError('Invalid Firaga palette row')
        projected.append({'source_offset': offset, 'declared_rows': declared, 'width': 3, 'colors': colors})
    return {'rows': projected}


def project_text_program(program):
    source, header, units = (program[key] for key in ('source_offset', 'header', 'units'))
    if (type(source) is not int or type(header) is not int or not 0 <= header <= 65535
            or not isinstance(units, list)):
        raise ValueError('Invalid text program')
    projected = []
    for unit in units:
        if unit['kind'] == 'glyph' and type(unit['index']) is int and 0 <= unit['index'] <= 65535:
            projected.append(unit['index'])
        elif (unit['kind'] == 'command' and type(unit['opcode']) is int
              and 0 <= unit['opcode'] <= 255 and isinstance(unit['operands'], list)
              and all(type(value) is int and 0 <= value <= 255 for value in unit['operands'])):
            projected.append([unit['opcode'], *unit['operands']])
        else:
            raise ValueError('Invalid text program unit')
    return {'source_offset': source, 'header': header, 'units': projected}


def project_audio_sequences(catalog):
    clocks = catalog['driver']['clocks']
    tracks, instructions = catalog['tracks'], catalog['instructions']
    if (not isinstance(clocks, list) or len(clocks) != 49
            or any(type(value) is not int or not 0 <= value <= 255 for value in clocks)
            or not isinstance(tracks, list) or not tracks or not isinstance(instructions, list) or not instructions):
        raise ValueError('Invalid sequence tables')
    def byte(value):
        return type(value) is int and 0 <= value <= 255
    def optional_note_byte(value):
        return value is None or type(value) is int and 0 <= value < 128
    projected, lengths, offsets = [], [], {}
    previous_end = 0
    for index, instruction in enumerate(instructions):
        source, explicit, code = (instruction[key] for key in ('source_offset', 'explicit', 'code'))
        if (instruction['id'] != index or type(source) is not int or source < previous_end
                or type(explicit) is not bool or not isinstance(code, dict)):
            raise ValueError('Invalid sequence instruction identity')
        kind = code['kind']
        if kind == 'wait' and type(code['clock']) is int and 0 <= code['clock'] < len(clocks):
            values, length = [kind, code['clock']], 0
        elif kind in ('end', 'return'):
            values, length = [kind], 0
        elif kind in ('jump', 'pattern') and type(code['target']) is int and 0 <= code['target'] < len(instructions):
            values, length = [kind, code['target']], 4
        elif kind == 'parameter' and byte(code['command']) and byte(code['value']):
            values, length = [kind, code['command'], code['value']], 1
        elif kind == 'extended' and byte(code['selector']) and byte(code['value']):
            values, length = [kind, code['selector'], code['value']], 2
        elif kind == 'end_tie' and optional_note_byte(code['key']):
            values, length = [kind, code['key']], int(code['key'] is not None)
        elif (kind == 'note' and type(code['clock']) is int and 0 <= code['clock'] < len(clocks)
              and all(optional_note_byte(code[key]) for key in ('key', 'velocity', 'extension'))
              and (code['key'] is not None or code['velocity'] is None)
              and (code['velocity'] is not None or code['extension'] is None)):
            values = [kind, code['clock'], code['key'], code['velocity'], code['extension']]
            length = sum(value is not None for value in values[2:])
        else:
            raise ValueError('Invalid sequence instruction')
        length += int(explicit)
        if length == 0:
            raise ValueError('Empty sequence instruction')
        offsets[source] = index
        previous_end = source + length
        lengths.append(length)
        projected.append({'source_offset': source, 'next': None, 'code': values})
    for index, instruction in enumerate(projected):
        successor = offsets.get(instruction['source_offset'] + lengths[index])
        if successor is None and instruction['code'][0] not in ('end', 'jump', 'return'):
            raise ValueError('Missing sequence successor')
        instruction['next'] = successor
    projected_tracks, track_keys = [], set()
    for index, track in enumerate(tracks):
        if (track['id'] != index or type(track['directory_header']) is not int or not 0 <= track['directory_header'] <= 65535
                or type(track['track_index']) is not int or not 0 <= track['track_index'] < 16
                or type(track['entry']) is not int or not 0 <= track['entry'] < len(projected)
                or (track['directory_header'], track['track_index']) in track_keys):
            raise ValueError('Invalid sequence track')
        track_keys.add((track['directory_header'], track['track_index']))
        projected_tracks.append({key: track[key] for key in ('id', 'directory_header', 'track_index', 'entry')})
    return {'clocks': clocks, 'tracks': projected_tracks, 'instructions': projected}


def project_audio_instruments(catalog):
    fields = ('banks', 'tones', 'map_links')
    if not any(field in catalog for field in fields):
        return None
    banks, tones, links = (catalog[field] for field in fields)
    if not isinstance(banks, list) or not isinstance(tones, list) or not isinstance(links, list):
        raise ValueError('Invalid instrument catalog')
    def word(value):
        return type(value) is int and 0 <= value <= 0xffffffff
    projected_tones, tone_ids = [], set()
    for tone in tones:
        if (type(tone['id']) is not int or tone['id'] < 0 or tone['id'] in tone_ids
                or type(tone['ctype']) is not int or not 0 <= tone['ctype'] <= 255
                or not word(tone['word04'])):
            raise ValueError('Invalid instrument tone')
        tone_ids.add(tone['id'])
        if 'word08' in tone and not word(tone['word08']):
            raise ValueError('Invalid instrument envelope word')
        projected_tone = {key: tone[key] for key in ('id', 'ctype', 'word04')}
        if 'word08' in tone:
            projected_tone['word08'] = tone['word08']
        projected_tones.append(projected_tone)
    projected_banks, addresses = [], set()
    for bank in banks:
        if not word(bank['source_address']) or bank['source_address'] in addresses or not isinstance(bank['programs'], list):
            raise ValueError('Invalid instrument bank')
        addresses.add(bank['source_address'])
        programs, selectors = [], set()
        for program in bank['programs']:
            selector, tone = program['selector'], program['tone']
            if (type(selector) is not int or not 0 <= selector <= 255 or selector in selectors
                    or type(tone) is not int or tone not in tone_ids):
                raise ValueError('Invalid instrument program')
            selectors.add(selector); programs.append({'selector': selector, 'tone': tone})
        projected_banks.append({'source_address': bank['source_address'], 'programs': programs})
    projected_links, keys = [], set()
    for link in links:
        parent, key, tone = (link[field] for field in ('parent_tone', 'key', 'tone'))
        if (type(parent) is not int or parent not in tone_ids or type(tone) is not int or tone not in tone_ids
                or type(key) is not int or not 0 <= key < 128 or (parent, key) in keys):
            raise ValueError('Invalid instrument key map')
        keys.add((parent, key)); projected_links.append({'parent_tone': parent, 'key': key, 'tone': tone})
    return {'banks': projected_banks, 'tones': projected_tones, 'map_links': projected_links}


def project_timed_sound_sources(directory):
    """Preserve observed schedules and source fragments without running the scheduler."""
    def uint(value, maximum):
        return type(value) is int and 0 <= value <= maximum
    def rows(values):
        if not isinstance(values, list) or not 0 < len(values) <= 65536:
            raise ValueError('Invalid timed sound rows')
        for row in values:
            words = row['sound_words']
            if (not uint(row['tick'], 65535) or not uint(row['channel'], 255) or not uint(row['opaque03'], 255)
                    or not isinstance(words, list) or len(words) != 2 or any(not uint(word, 65535) for word in words)):
                raise ValueError('Invalid timed sound row')
        return [{'tick': row['tick'], 'channel': row['channel'], 'opaque03': row['opaque03'],
                 'sound_words': list(row['sound_words'])} for row in values]

    groups, errors = [], []
    schedules = directory.get('red_spring_sounds')
    if schedules is not None:
        try:
            if not isinstance(schedules, list) or not 0 < len(schedules) <= 256:
                raise ValueError('Invalid observed sound schedules')
            observed = []
            for item in schedules:
                source, root, pointer = item['source_offset'], item['root_offset'], item['root_pointer']
                values = rows(item['rows'])
                if (not uint(source, 0xffffff) or source + 8 * len(values) > 0x1000000
                        or not uint(root, 0xffffff) or not uint(pointer, 0xffffffff) or pointer != 0x08000000 + source):
                    raise ValueError('Invalid observed sound source')
                observed.append({'kind': 'rows', 'origin': 'observed Red Spring', 'source_offset': source,
                                 'root_offset': root, 'root_pointer': pointer, 'values': values})
            groups.extend(observed)
        except (ValueError, KeyError, TypeError, AttributeError) as error:
            errors.append('observed schedules: ' + str(error))
    bank = directory.get('timed_sounds')
    if bank is not None:
        try:
            sources = bank['sources']
            if not isinstance(sources, list) or not 0 < len(sources) <= 8192:
                raise ValueError('Invalid timed sound source list')
            projected, seen = [], set()
            sizes = {'literal': (4, 0xffffffff), 'directory': (4, 0xffffffff),
                     'counts': (1, 255), 'ram-words': (2, 65535)}
            for item in sources:
                source, packed = item['offset'], item['values']
                kind, values = packed['kind'], packed['values']
                if kind == 'rows':
                    values, size = rows(values), 8
                elif kind in sizes:
                    size, maximum = sizes[kind]
                    values = [values] if kind == 'literal' else values
                    if (not isinstance(values, list) or not 0 < len(values) <= 65536
                            or any(not uint(value, maximum) for value in values)):
                        raise ValueError('Invalid timed sound source values')
                    values = list(values)
                else:
                    raise ValueError('Unknown timed sound source kind')
                if not uint(source, 0xffffff) or source in seen or source + size * len(values) > 0x1000000:
                    raise ValueError('Invalid timed sound source extent')
                seen.add(source)
                projected.append({'kind': kind, 'origin': 'source fragment', 'source_offset': source, 'values': values})
            groups.extend(projected)
        except (ValueError, KeyError, TypeError, AttributeError) as error:
            errors.append('timed sources: ' + str(error))
    return groups, errors


def project_audio_pcm_pitch(catalog):
    keys, bases = catalog['pcm_keys']['entries'], catalog['pcm_bases']['entries']
    if (not isinstance(keys, list) or len(keys) != 180 or not isinstance(bases, list) or len(bases) != 12
            or any(type(value) is not int or not 0 <= value <= 0xffffffff for value in bases)):
        raise ValueError('Invalid PCM pitch tables')
    for key in keys:
        if (type(key['base']) is not int or not 0 <= key['base'] < len(bases)
                or type(key['shift']) is not int or not 0 <= key['shift'] < 16):
            raise ValueError('Invalid PCM pitch key')
    result = {'keys': [{'base': key['base'], 'shift': key['shift']} for key in keys], 'bases': bases}
    if 'cgb_keys' in catalog or 'cgb_bases' in catalog:
        if 'cgb_keys' not in catalog or 'cgb_bases' not in catalog:
            raise ValueError('Incomplete CGB pitch tables')
        cgb_keys, cgb_bases = catalog['cgb_keys']['entries'], catalog['cgb_bases']['entries']
        if (not isinstance(cgb_keys, list) or len(cgb_keys) != 132
                or not isinstance(cgb_bases, list) or len(cgb_bases) != 12
                or any(type(value) is not int or not -32768 <= value <= 32767 for value in cgb_bases)):
            raise ValueError('Invalid CGB pitch tables')
        for key in cgb_keys:
            if (type(key['base']) is not int or not 0 <= key['base'] < len(cgb_bases)
                    or type(key['shift']) is not int or not 0 <= key['shift'] < 16):
                raise ValueError('Invalid CGB pitch key')
        result['cgb_keys'] = [{'base': key['base'], 'shift': key['shift']} for key in cgb_keys]
        result['cgb_bases'] = cgb_bases
    return result


def project_audio_noise(catalog):
    """Expose the original driver's 60 noise-register codes independently of pitch."""
    if not isinstance(catalog, dict):
        raise ValueError('Invalid CGB noise catalog')
    if 'noise_codes' not in catalog:
        return None
    table = catalog['noise_codes']
    if not isinstance(table, dict):
        raise ValueError('Invalid CGB noise table')
    entries, offset = table['entries'], table['source_offset']
    if (not isinstance(entries, list) or len(entries) != 60
            or any(type(value) is not int or not 0 <= value <= 255 for value in entries)
            or type(offset) is not int or not 0 <= offset <= 0x1000000 - len(entries)):
        raise ValueError('Invalid CGB noise table')
    return {'entries': list(entries), 'source_offset': offset}


def project_text_reader_registry(catalog):
    """Preserve recorded callback slots without inferring execution."""
    offset, callbacks = catalog['table_offset'], catalog['callbacks']
    if (type(offset) is not int or offset < 0 or offset % 4
            or not isinstance(callbacks, list) or not 0 < len(callbacks) <= 1024
            or offset + 4 * len(callbacks) > 0x2000000
            or any(type(value) is not int or not 0 <= value <= 0xffffffff for value in callbacks)):
        raise ValueError('Invalid text reader registry')
    return {'table_offset': offset, 'callbacks': list(callbacks)}


def link_text_reader_registry(registry, relative_texts):
    """Join unique extracted readers to equal Thumb callback addresses."""
    readers = relative_texts['catalogs'] if relative_texts else []
    rows = []
    for slot, callback in enumerate(registry['callbacks']):
        reader = callback & ~1 if callback & 1 and 0x08000000 <= callback < 0x0a000000 else None
        matches = [index for index, item in enumerate(readers) if item['reader'] == reader] if reader is not None else []
        rows.append({'slot': slot, 'callback': callback, 'reader': reader,
                     'reader_index': matches[0] if len(matches) == 1 else None})
    return {'path': registry['path'], 'table_offset': registry['table_offset'], 'rows': rows}


def project_equipment_sprites(catalog, *, alternate=False):
    """Retain sprite bindings, commands and links to their indexed graphics."""
    def source_map(items, name):
        if not isinstance(items, list):
            raise ValueError('Invalid equipment ' + name)
        result = {}
        for index, item in enumerate(items):
            source = item['source_offset']
            if type(source) is not int or source < 0 or source in result:
                raise ValueError('Invalid equipment ' + name + ' source')
            result[source] = index
        return result

    blocks, objects, sequences = (catalog[key] for key in ('pixels', 'objects', 'sequences'))
    block_sources = source_map(blocks, 'pixel')
    object_sources = source_map(objects, 'object')
    sequence_sources = source_map(sequences, 'sequence')
    for block in blocks:
        tiles = block['tiles']
        if not isinstance(tiles, list) or not tiles:
            raise ValueError('Empty equipment pixel block')
        for tile in tiles:
            indices = tile['indices']
            if (not isinstance(indices, list) or len(indices) != 8 or any(
                not isinstance(line, list) or len(line) != 8 or any(
                    type(index) is not int or not 0 <= index <= 15 for index in line
                ) for line in indices
            )):
                raise ValueError('Invalid equipment indexed tile')
    for item in objects:
        if not isinstance(item['objects'], list):
            raise ValueError('Invalid equipment object list')
    projected_sequences = []
    for sequence in sequences:
        frames = sequence['frames']
        if not isinstance(frames, list) or not frames:
            raise ValueError('Empty equipment sequence')
        projected_frames = []
        for frame in frames:
            word00, word04, duration, operation, tail = (frame[key] for key in
                ('word00', 'word04', 'duration', 'operation', 'bytes_0a_13'))
            if (any(type(word) is not int or not 0 <= word <= 0xffffffff for word in (word00, word04))
                    or type(duration) is not int or not 0 <= duration <= 255
                    or type(operation) is not int or not 0 <= operation <= 255
                    or not isinstance(tail, list) or len(tail) != 10
                    or any(type(byte) is not int or not 0 <= byte <= 255 for byte in tail)):
                raise ValueError('Invalid equipment frame')
            result = {'word00': word00, 'word04': word04, 'duration': duration,
                      'operation': operation, 'bytes_0a_13': tail}
            if operation == 1:
                pixel_source, object_source = 0x898a40 + word00, 0xa50020 + word04
                if pixel_source not in block_sources or object_source not in object_sources:
                    raise ValueError('Missing equipment graphics link')
                result['block'], result['objects'] = block_sources[pixel_source], object_sources[object_source]
            projected_frames.append(result)
        projected_sequences.append({'source_offset': sequence['source_offset'],
                                    'opaque_header': sequence['opaque_header'], 'frames': projected_frames})
    bindings, keys = [], set()
    if not isinstance(catalog['bindings'], list) or not catalog['bindings']:
        raise ValueError('Empty equipment bindings')
    for binding in catalog['bindings']:
        sprite, animation, part, source = (binding[key] for key in
            ('sprite_id', 'animation', 'part', 'index_source'))
        target = binding['target'] if alternate else None
        if alternate:
            if not isinstance(target, dict) or target.get('kind') not in ('null', 'sequence', 'shared-index-root'):
                raise ValueError('Invalid sprite target')
            if target['kind'] == 'sequence':
                sequence = target['source_offset']
                if type(sequence) is not int or sequence < 0:
                    raise ValueError('Invalid sprite sequence target')
            elif target['kind'] == 'shared-index-root':
                if (type(target.get('source_offset')) is not int or target['source_offset'] < 0
                        or type(target.get('sprite_id')) is not int or target['sprite_id'] < 0):
                    raise ValueError('Invalid shared sprite index root')
                sequence = None
            else:
                sequence = None
        else:
            sequence = binding['sequence_source']
        key = (sprite, animation, part)
        if (any(type(value) is not int or value < 0 for value in (sprite, animation, part, source))
                or key in keys or sequence is not None and sequence not in sequence_sources):
            raise ValueError('Invalid equipment binding')
        keys.add(key)
        projected = {'sprite_id': sprite, 'animation': animation, 'part': part, 'index_source': source,
                     'sequence': None if sequence is None else sequence_sources[sequence]}
        if alternate:
            projected['target'] = target
        bindings.append(projected)
    return {'blocks': blocks, 'objects': objects, 'sequences': projected_sequences, 'bindings': bindings}


def project_script_sprites(catalog, *, alternate=False):
    """Retain conditional producer witnesses alongside the extracted sprite graph."""
    projected = project_equipment_sprites(catalog, alternate=alternate)
    producers = catalog['producers']
    if not isinstance(producers, list) or not producers:
        raise ValueError('Empty script sprite producers')
    integer_fields = ('selector', 'unit_kind', 'unit_logical_id', 'job_id',
                      'metadata_source', 'assignment_source')
    retained = []
    for producer in producers:
        if not isinstance(producer, dict):
            raise ValueError('Invalid script sprite producer')
        sprite, base, sources = (producer[key] for key in ('sprite_id', 'base', 'script_sources'))
        if (any(type(value) is not int or not 0 <= value <= 0xffff for value in (sprite, base))
                or not isinstance(sources, list) or not sources
                or any(type(value) is not int or not 0 <= value <= 0xffffffff for value in sources)
                or any(type(producer[key]) is not int or not 0 <= producer[key] <= 0xffffffff
                       for key in integer_fields if key in producer)
                or 'mode' in producer and producer['mode'] not in ('normal', 'alternate')):
            raise ValueError('Invalid script sprite producer')
        fields = ('sprite_id', 'base', 'script_sources', 'mode') + integer_fields
        retained.append({key: producer[key] for key in fields if key in producer})
    projected['producers'] = retained
    return projected


def validate_script39_poses(catalog):
    """Validate newly owned pose sequences without assuming actor assignments."""
    def uint(value, maximum=0xffffffff):
        return type(value) is int and 0 <= value <= maximum
    sequences, cells, pixels = (catalog[key] for key in ('sequences', 'cells', 'pixels'))
    objects = catalog.get('objects', [])
    if (not isinstance(sequences, list) or not sequences or len(sequences) > 65536
            or not isinstance(cells, list) or len(cells) > 65536
            or not isinstance(objects, list) or len(objects) > 65536
            or not isinstance(pixels, list) or len(pixels) > 65536):
        raise ValueError('Invalid opcode-39 pose catalog')
    sequence_sources, cell_sources, block_sources = set(), set(), set()
    for sequence in sequences:
        source, header, frames = (sequence[key] for key in ('source_offset', 'opaque_header', 'frames'))
        if (not uint(source, 0xffffff) or source % 4 or source in sequence_sources
                or not uint(header, 65535) or not isinstance(frames, list) or not 0 < len(frames) < 32768
                or source + 4 + 20 * len(frames) > 0x1000000):
            raise ValueError('Invalid opcode-39 pose sequence')
        sequence_sources.add(source)
        for frame in frames:
            if (not uint(frame['word00']) or not uint(frame['word04']) or not uint(frame['duration'], 255)
                    or not uint(frame['operation'], 255) or not isinstance(frame['bytes_0a_13'], list)
                    or len(frame['bytes_0a_13']) != 10 or any(not uint(value, 255) for value in frame['bytes_0a_13'])):
                raise ValueError('Invalid opcode-39 pose frame')
    for cell in cells:
        source, target = cell['source_offset'], cell['target']
        if (not uint(source, 0xffffff) or source % 4 or source in cell_sources
                or not isinstance(target, dict) or target.get('kind') not in ('null', 'sequence')):
            raise ValueError('Invalid opcode-39 pose cell')
        if target['kind'] == 'sequence' and (not uint(target['source_offset'], 0xffffff) or target['source_offset'] % 4):
            raise ValueError('Invalid opcode-39 pose target')
        cell_sources.add(source)
    if objects:
        project_obj_static_lists({'lists': objects})
    for block in pixels:
        source, tiles = block['source_offset'], block['tiles']
        if (not uint(source, 0xffffff) or source % 4 or source in block_sources
                or not isinstance(tiles, list) or not tiles or source + len(tiles) * 32 > 0x1000000):
            raise ValueError('Invalid opcode-39 pixel block')
        block_sources.add(source)
        for tile in tiles:
            lines = tile['indices']
            if (not isinstance(lines, list) or len(lines) != 8 or any(not isinstance(line, list) or len(line) != 8
                    or any(not uint(value, 15) for value in line) for line in lines)):
                raise ValueError('Invalid opcode-39 pixel tile')
    return catalog


def project_script39_poses(catalog, path, object_pool, pixel_pool, sequence_pool=()):
    """Resolve each frame only where compatible extracted source ranges agree."""
    sizes = (((8,8),(16,16),(32,32),(64,64)), ((16,8),(32,8),(32,16),(64,32)),
             ((8,16),(8,32),(16,32),(32,64)))
    objects = {}
    for item, origin in object_pool:
        try:
            project_obj_static_lists({'lists': [item]})
            source = item['source_offset']
            if source in objects and objects[source] is not None and objects[source][0] != item:
                objects[source] = None
            elif source not in objects:
                objects[source] = (item, origin)
        except (ValueError, KeyError, TypeError):
            continue
    blocks = sorted(pixel_pool, key=lambda entry: entry[0]['source_offset'])
    starts = [entry[0]['source_offset'] for entry in blocks]
    pixel_cache, used_blocks, used_objects, block_indices, object_indices = {}, [], [], {}, {}
    references = {}
    for cell in catalog['cells']:
        target = cell['target']
        if target['kind'] == 'sequence':
            references.setdefault(target['source_offset'], []).append(cell['source_offset'])
    # Earlier owners may contain sequences referenced by these new index cells.
    # Compare the stored fields, ignoring renderer-specific block indices.
    fields = ('word00', 'word04', 'duration', 'operation', 'bytes_0a_13')
    earlier = {}
    for candidate, origin in sequence_pool:
        normalized = {'source_offset': candidate['source_offset'], 'opaque_header': candidate['opaque_header'],
                      'frames': [{key: frame[key] for key in fields} for frame in candidate['frames']]}
        source = normalized['source_offset']
        if source in earlier and earlier[source] is not None and earlier[source][0] != normalized:
            earlier[source] = None
        elif source not in earlier:
            earlier[source] = (normalized, origin)
    owned = {sequence['source_offset'] for sequence in catalog['sequences']}
    candidates = [(sequence, path) for sequence in catalog['sequences']]
    candidates.extend(earlier[source] for source in sorted(references) if source not in owned and earlier.get(source))
    sequences, bindings, sequence_indices = [], [], {}
    for sequence, sequence_path in candidates:
        frames = []
        for frame in sequence['frames']:
            projected = {key: frame[key] for key in ('word00', 'word04', 'duration', 'operation', 'bytes_0a_13')}
            if frame['operation'] == 1:
                pixel_source, object_source = 0x898a40 + frame['word00'], 0xa50020 + frame['word04']
                projected.update(block=None, objects=None, pixel_source=pixel_source, object_source=object_source)
                match = objects.get(object_source)
                if match:
                    item, origin = match
                    needed = 0
                    for obj in item['objects']:
                        if (obj['shape'] >= 3 or obj['affine_mode'] or obj['color_256']
                                or obj['object_mode'] or obj['mosaic']):
                            needed = 0; break
                        width, height = sizes[obj['shape']][obj['size']]
                        needed = max(needed, obj['tile'] + width * height // 64)
                    if needed:
                        key = (pixel_source, needed)
                        if key not in pixel_cache:
                            chosen, chosen_origin, overlap = None, None, []
                            for block, block_origin in blocks[:bisect.bisect_left(starts, pixel_source + 32 * needed)]:
                                delta = pixel_source - block['source_offset']
                                if delta % 32 or delta >= len(block['tiles']) * 32:
                                    continue
                                overlap.append(block)
                                if delta >= 0 and delta // 32 + needed <= len(block['tiles']):
                                    chosen = block['tiles'][delta // 32:delta // 32 + needed]
                                    chosen_origin = block_origin
                            conflict = False
                            if chosen is not None:
                                for block in overlap:
                                    begin = max(pixel_source, block['source_offset'])
                                    end = min(pixel_source + 32 * needed, block['source_offset'] + len(block['tiles']) * 32)
                                    left, right = (begin - pixel_source) // 32, (begin - block['source_offset']) // 32
                                    if chosen[left:left + (end - begin) // 32] != block['tiles'][right:right + (end - begin) // 32]:
                                        conflict = True; break
                            pixel_cache[key] = None if conflict or chosen is None else (chosen, chosen_origin)
                        resolved = pixel_cache[key]
                        if resolved is not None:
                            if key not in block_indices:
                                block_indices[key] = len(used_blocks)
                                used_blocks.append({'source_offset': pixel_source, 'tiles': resolved[0], 'path': resolved[1]})
                            if object_source not in object_indices:
                                object_indices[object_source] = len(used_objects)
                                used_objects.append({**item, 'path': origin})
                            projected['block'], projected['objects'] = block_indices[key], object_indices[object_source]
            frames.append(projected)
        sequence_indices[sequence['source_offset']] = len(sequences)
        sequences.append({'source_offset': sequence['source_offset'], 'opaque_header': sequence['opaque_header'],
                          'path': sequence_path, 'frames': frames})
    for target, cells in references.items():
        for source in cells:
            index = sequence_indices.get(target)
            binding = {'sprite_id': target, 'animation': 0, 'part': 0, 'index_source': source, 'sequence': index}
            if index is None:
                binding['target'] = {'kind': 'unavailable-sequence', 'source_offset': target}
            bindings.append(binding)
    # A sequence without an owned cell is still inspectable; it has no claimed source binding.
    bound = {item['sequence'] for item in bindings}
    for index, sequence in enumerate(sequences):
        if index not in bound:
            bindings.append({'sprite_id': sequence['source_offset'], 'animation': 0, 'part': 0,
                             'index_source': None, 'sequence': index})
    cells = [{'source_offset': item['source_offset'],
              'target': None if item['target']['kind'] == 'null' else item['target']['source_offset']}
             for item in catalog['cells']]
    return {'path': path, 'source_sequences': True, 'blocks': used_blocks, 'objects': used_objects,
            'sequences': sequences, 'bindings': bindings, 'cells': cells}


def project_item_links(catalog):
    """Retain all item-linked pair slots, including storage beyond each count."""
    source, records = catalog['source_offset'], catalog['records']
    if (type(source) is not int or not 0 <= source <= 0xffffffff
            or not isinstance(records, list) or not records or len(records) > 65536
            or source + len(records) * 20 > 0x100000000):
        raise ValueError('Invalid item link source or records')
    for record in records:
        count, opaque, slots = (record[key] for key in ('count', 'opaque', 'slots'))
        if (type(count) is not int or not 0 <= count <= 9
                or type(opaque) is not int or not 0 <= opaque <= 255
                or not isinstance(slots, list) or len(slots) != 9):
            raise ValueError('Invalid item link record')
        for slot in slots:
            if any(type(slot[key]) is not int or not 0 <= slot[key] <= 255 for key in ('selector', 'value')):
                raise ValueError('Invalid item link slot')
    return {'source_offset': source, 'records': records}


def project_item_effect_parameters(catalog):
    """Validate the documented EUR item-effect bank and its direct item IDs."""
    source, records = catalog['source_offset'], catalog['records']
    if type(source) is not int or source != 0x6e879c or not isinstance(records, list) or len(records) != 14:
        raise ValueError('Invalid item effect source or record count')
    for index, record in enumerate(records):
        coefficients = record['coefficients']
        if (type(record['id']) is not int or record['id'] != index
                or type(record['item_id']) is not int or record['item_id'] != 362 + index
                or type(record['source_offset']) is not int or record['source_offset'] != source + index * 7
                or not isinstance(coefficients, list) or len(coefficients) != 9
                or any(type(value) is not int or not 0 <= value <= 31 for value in coefficients)
                or type(record['cap45_52']) is not int or not 0 <= record['cap45_52'] <= 255
                or type(record['opaque53_55']) is not int or not 0 <= record['opaque53_55'] <= 7):
            raise ValueError('Invalid item effect ID, source, or packed field')
    return {'source_offset': source, 'records': records}


def project_item_parameters(catalog):
    """Retain the bounded European four-byte item-derived records."""
    source, records = catalog['source_offset'], catalog['records']
    if type(source) is not int or source != 0x6e84e0 or not isinstance(records, list) or len(records) != 31:
        raise ValueError('Invalid item parameter source or record count')
    for record in records:
        if any(type(record[key]) is not int or not 0 <= record[key] <= maximum
               for key, maximum in (('byte00', 255), ('byte01', 255), ('value02', 65535))):
            raise ValueError('Invalid item parameter field')
    return {'source_offset': source, 'records': records}


def project_evaluation_parameters(catalog):
    """Validate the fixed European coefficient and threshold source records."""
    score, bias = catalog['score_offset'], catalog['threshold_address_bias']
    terms, bonuses, thresholds = (catalog[key] for key in ('terms', 'bonuses', 'thresholds'))
    if (type(score) is not int or score != 0x6e79a2 or type(bias) is not int or bias != 0x6e79c2
            or not isinstance(terms, list) or len(terms) != 7
            or not isinstance(bonuses, list) or len(bonuses) != 2
            or not isinstance(thresholds, list) or len(thresholds) != 5):
        raise ValueError('Invalid evaluation parameter roots or counts')
    for records, first, base, stride, fields in (
            (terms, 0, score, 2, ('divisor', 'weight')),
            (bonuses, 7, score + 14, 2, ('byte00', 'weight')),
            (thresholds, 1, bias + 4, 4, ('threshold',))):
        for index, record in enumerate(records):
            if (type(record['id']) is not int or record['id'] != first + index
                    or type(record['source_offset']) is not int or record['source_offset'] != base + index * stride
                    or any(type(record[key]) is not int or not 0 <= record[key] <= 255 for key in fields)):
                raise ValueError('Invalid evaluation parameter ID, source or byte')
    for record in thresholds:
        tail = record['bytes01_03']
        if not isinstance(tail, list) or len(tail) != 3 or any(type(value) is not int or not 0 <= value <= 255 for value in tail):
            raise ValueError('Invalid evaluation threshold bytes')
    return {'score_offset': score, 'threshold_address_bias': bias, 'terms': terms,
            'bonuses': bonuses, 'thresholds': thresholds}


def project_progression_catalog(catalog, kind):
    """Retain documented EUR progression and pricing records as stored."""
    specs = {
        'rewards': (0x6e79da, 100, 10, 0, {'discount08': 255, 'byte09': 255}, {'deltas00_07': 8}),
        'gates': (0x6e7dce, 64, 12, 0, {'byte00': 255, 'halfword01': 65535, 'byte03': 255},
                  {'requirements04_0b': 8}),
        'selection': (0x6e80de, 64, 16, 1, {'selector00': 255, 'minimum01': 255, 'minimum02': 65535,
                                           'minimum04': 65535, 'flag0e': 65535}, {'any_minima06_0d': 8}),
        'trade': (0x6e85bc, 5, 96, 2, {'markup40_41': 65535},
                  {'bytes00_1f': 32, 'bytes20_3f': 32, 'discounts42_5f': 30}),
    }
    source, count, stride, first_id, scalar_fields, array_fields = specs[kind]
    records = catalog['records']
    if (type(catalog['source_offset']) is not int or catalog['source_offset'] != source
            or not isinstance(records, list) or len(records) != count):
        raise ValueError('Invalid progression/pricing source or record count')
    for index, record in enumerate(records):
        if (type(record['id']) is not int or record['id'] != first_id + index
                or type(record['source_offset']) is not int or record['source_offset'] != source + index * stride
                or any(type(record[key]) is not int or not 0 <= record[key] <= maximum
                       for key, maximum in scalar_fields.items())):
            raise ValueError('Invalid progression/pricing identity or scalar field')
        for key, length in array_fields.items():
            values = record[key]
            if (not isinstance(values, list) or len(values) != length
                    or any(type(value) is not int or not 0 <= value <= 255 for value in values)):
                raise ValueError('Invalid progression/pricing byte array')
    return {'source_offset': source, 'records': records}


def project_raster_palettes(catalog):
    """Keep physical color programs and their descriptor selector pairs."""
    source, programs, descriptors = (catalog[key] for key in ('source_offset', 'programs', 'descriptor_programs'))
    if (type(source) is not int or not 0 <= source <= 0xffffffff
            or not isinstance(programs, list) or not programs or len(programs) > 65536
            or not isinstance(descriptors, list) or not descriptors or len(descriptors) > 65536):
        raise ValueError('Invalid raster palette catalog')
    for index, program in enumerate(programs):
        offset, header, colors = (program[key] for key in ('source_offset', 'header', 'colors'))
        if (type(program['id']) is not int or program['id'] != index
                or type(offset) is not int or not 0 <= offset <= 0xffffffff
                or not isinstance(colors, list) or not colors or len(colors) > 4096
                or offset + 6 + 2 * len(colors) > 0x100000000
                or any(type(value) is not int or not 0 <= value <= 65535 for value in colors)
                or any(type(header[key]) is not int or not 0 <= header[key] <= 65535
                       for key in ('color_count', 'range_length', 'range_start'))
                or header['color_count'] != len(colors)):
            raise ValueError('Invalid raster palette program')
    if any(not isinstance(pair, list) or len(pair) != 2 or any(
            type(value) is not int or not 0 <= value < len(programs) for value in pair)
           for pair in descriptors):
        raise ValueError('Invalid raster palette descriptor links')
    return {'source_offset': source, 'programs': programs, 'descriptor_programs': descriptors}


def project_encounter_catalog(catalog, kind):
    """Preserve encounter records while validating their physical field shapes."""
    def uint(value, maximum):
        return type(value) is int and 0 <= value <= maximum

    def fields(record, names, maximum):
        return all(uint(record[name], maximum) for name in names)

    def array(record, name, length, maximum):
        values = record[name]
        return (isinstance(values, list) and len(values) == length
                and all(uint(value, maximum) for value in values))

    source = catalog['source_offset']
    key = {'headers': 'headers', 'units': 'records', 'coordinates': 'lists'}[kind]
    records = catalog[key]
    if (not uint(source, 0xffffffff) or not isinstance(records, list) or not records
            or len(records) > 65536):
        raise ValueError('Invalid encounter catalog source or records')
    cursor = source
    for index, record in enumerate(records):
        if kind == 'headers':
            if (not uint(record['index'], 0xffff) or record['index'] != index + 1
                    or not fields(record, ('unit_count',), 0xffff)
                    or not fields(record, ('unit_templates_pointer',), 0xffffffff)
                    or not fields(record, ('map_argument', 'placement_parameter0', 'placement_parameter2',
                        'placement_count0', 'random_position_range', 'placement_count2',
                        'unit_count_comparison', 'nonzero_gate', 'switch_selector', 'zero_gate', 'script_default'), 255)
                    or not array(record, 'position_pointers', 4, 0xffffffff)
                    or not array(record, 'opaque', 7, 255)):
                raise ValueError('Invalid encounter header')
            cursor += 40
        elif kind == 'units':
            if (not fields(record, ('index', 'header_index', 'unit_index'), 0xffff)
                    or record['index'] != index or record['source_offset'] != cursor
                    or type(record['source_offset']) is not int
                    or not fields(record, ('byte00', 'byte01', 'byte02', 'byte03', 'byte04',
                                           'byte28', 'byte29', 'byte2c', 'byte2d'), 255)
                    or not fields(record, ('halfword12', 'flags2a'), 0xffff)
                    or not array(record, 'arguments05_07', 3, 255)
                    or not array(record, 'equipment08_10', 5, 0xffff)
                    or not array(record, 'ability_bits14_27', 20, 255)
                    or not array(record, 'opaque2e_2f', 2, 255)):
                raise ValueError('Invalid encounter unit template')
            cursor += 48
        else:
            points = record['points']
            if (not fields(record, ('index', 'header_index'), 0xffff) or record['index'] != index
                    or not uint(record['slot'], 3) or record['source_offset'] != cursor
                    or type(record['source_offset']) is not int
                    or not isinstance(points, list) or len(points) > 65535
                    or any(not fields(point, ('x', 'y'), 255) for point in points)):
                raise ValueError('Invalid encounter coordinate list')
            cursor += 2 * len(points)
        if cursor > 0x100000000:
            raise ValueError('Encounter catalog source overflow')
    return {'source_offset': source, key: records}


def project_mission_event_catalog(catalog, kind):
    """Retain mission, event and sparse parameter records without inferred semantics."""
    def uint(value, maximum):
        return type(value) is int and 0 <= value <= maximum

    def fields(record, names, minimum, maximum):
        return all(type(record[name]) is int and minimum <= record[name] <= maximum for name in names)

    def array(record, name, length):
        values = record[name]
        return isinstance(values, list) and len(values) == length and all(uint(value, 255) for value in values)

    source, records = catalog['source_offset'], catalog['records']
    stride = {'missions': 70, 'events': 12, 'parameters': 27}[kind]
    if (not uint(source, 0xffffffff) or not isinstance(records, list) or not records
            or len(records) > 65536):
        raise ValueError('Invalid mission/event catalog source or records')
    seen = set()
    for ordinal, record in enumerate(records):
        index = record['index']
        if (not uint(index, 255 if kind == 'parameters' else 0xffff) or index in seen
                or kind != 'parameters' and index != ordinal or source + (index + 1) * stride > 0x100000000):
            raise ValueError('Invalid mission/event record identity')
        seen.add(index)
        if kind == 'missions':
            valid = (fields(record, ('word00', 'key05', 'key08', 'key0b', 'flags41_raw', 'word43'), 0, 65535)
                     and fields(record, ('operand07', 'operand0a', 'operand0d', 'scaled33_raw', 'byte34', 'byte35',
                         'mapped36_raw', 'mapped37_raw', 'mapped3d_raw', 'scaled3e_raw', 'byte3f', 'byte40',
                         'event_row45'), 0, 255)
                     and all(array(record, name, length) for name, length in
                             (('packed02', 3), ('packed0e', 4), ('inline12', 16), ('inline22', 8),
                              ('inline2a', 9), ('packed38', 5))))
        elif kind == 'events':
            valid = (fields(record, ('header00', 'event_key01', 'parameter06', 'byte07'), 0, 255)
                     and fields(record, ('primary02', 'alternate04', 'flags08_raw'), 0, 65535)
                     and array(record, 'opaque0a', 2))
        else:
            valid = (fields(record, ('header00', 'packed01', 'packed02', 'packed03', 'packed04', 'packed05',
                         'packed06', 'byte09', 'flags0d', 'byte0e', 'byte0f', 'byte10', 'byte11', 'opaque14'), 0, 255)
                     and fields(record, ('value07', 'value0a', 'value12'), 0, 65535)
                     and fields(record, ('signed0c', 'signed15'), -128, 127) and array(record, 'opaque16', 5))
        if not valid:
            raise ValueError('Invalid mission/event record fields')
    return {'source_offset': source, 'records': records}


def project_world_records(catalog):
    """Keep physical world rows and their stored threshold bytes."""
    def uint(value, maximum):
        return type(value) is int and 0 <= value <= maximum

    source, records = catalog['source_offset'], catalog['records']
    if (not uint(source, 0x1ffffff) or not isinstance(records, list) or not 1 <= len(records) <= 64
            or source + len(records) * 38 > 0x2000000):
        raise ValueError('Invalid world record source or extent')
    keys = ('opaque00', 'byte01', 'byte02', 'byte03', 'opaque06')
    projected = []
    for index, record in enumerate(records):
        thresholds = record['thresholds07']
        if (type(record['index']) is not int or record['index'] != index
                or not all(uint(record[key], 255) for key in keys)
                or not uint(record['value04'], 65535)
                or not isinstance(thresholds, list) or len(thresholds) != 31
                or any(not uint(value, 255) for value in thresholds)):
            raise ValueError('Invalid world record fields')
        projected.append({'index': index, **{key: record[key] for key in keys},
                          'value04': record['value04'], 'thresholds07': list(thresholds)})
    return {'source_offset': source, 'records': projected}


def project_action_records(catalog):
    """Retain all fields of the physical 28-byte metadata records."""
    def uint(value, maximum):
        return type(value) is int and 0 <= value <= maximum

    source, records = catalog['source_offset'], catalog['records']
    if (not uint(source, 0x1ffffff) or not isinstance(records, list) or not 1 <= len(records) <= 65536
            or source + 28 * len(records) > 0x2000000):
        raise ValueError('Invalid action record source or extent')
    scalar = {'field00': 65535, 'field02': 255, 'opaque03': 255, 'flags10': 0xffffffff,
              'field14': 65535, 'field16': 65535, 'field18': 255, 'field19': 255,
              'field1a': 255, 'opaque1b': 255}
    arrays = {'fields04_0b': 8, 'inline0c_0f': 4}
    projected = []
    for index, record in enumerate(records):
        if (type(record['index']) is not int or record['index'] != index
                or any(not uint(record[key], maximum) for key, maximum in scalar.items())
                or any(not isinstance(record[key], list) or len(record[key]) != length
                       or any(not uint(value, 255) for value in record[key]) for key, length in arrays.items())):
            raise ValueError('Invalid action record fields')
        projected.append({'index': index, **{key: record[key] for key in scalar},
                          **{key: list(record[key]) for key in arrays}})
    return {'source_offset': source, 'records': projected}


def project_linked_records4(catalog):
    """Keep the fixed EUR four-byte records and optional callback partitions."""
    source, records = catalog['source_offset'], catalog['records']
    if (type(source) is not int or source != 0x7133f8 or not isinstance(records, list) or len(records) != 211):
        raise ValueError('Invalid four-byte record source or count')
    for index, record in enumerate(records):
        if (type(record['index']) is not int or record['index'] != index
                or any(type(record[key]) is not int or not 0 <= record[key] <= 255
                       for key in ('field00', 'field01', 'field02', 'field03'))):
            raise ValueError('Invalid four-byte record index or field')
    def callback(value):
        return type(value) is int and 0x08000000 <= value < 0x09000000 and value & 1 == 1
    tables = catalog.get('callback_tables')
    if tables is not None and (not isinstance(tables, list) or len(tables) != 3 or any(
            not isinstance(table, list) or len(table) != count or any(not callback(value) for value in table)
            for table, count in zip(tables, (29, 32, 46)))):
        raise ValueError('Invalid four-byte callback tables')
    actions = catalog.get('action_callbacks')
    if actions is not None and (not isinstance(actions, list) or len(actions) != 93 or any(
            not callback(item['callback']) or any(type(item[key]) is not int or not 0 <= item[key] <= 0xffffffff
                                                   for key in ('opaque_first', 'opaque_second'))
            for item in actions)):
        raise ValueError('Invalid action callback records')
    return {'source_offset': source, 'records': records, 'callback_tables': tables, 'action_callbacks': actions}


def project_conditional_records8(catalog):
    """Keep the fixed European conditional slots without interpreting live state."""
    source, records = catalog['source_offset'], catalog['records']
    if (type(source) is not int or source != 0x713744 or not isinstance(records, list) or len(records) != 127):
        raise ValueError('Invalid conditional record source or count')
    fields = {'field00': 255, 'field01': 255, 'key02': 65535, 'operand04': 255, 'key05': 65535, 'operand07': 255}
    for index, record in enumerate(records):
        if (type(record['index']) is not int or record['index'] != index
                or any(type(record[key]) is not int or not 0 <= record[key] <= maximum for key, maximum in fields.items())):
            raise ValueError('Invalid conditional record index or field')
    return {'source_offset': source, 'records': records}


def project_packed_groups(groups, count):
    """Validate the four least-significant-first pairs in each source byte."""
    if (not isinstance(groups, list) or len(groups) != count or any(
            not isinstance(group, list) or len(group) != 4 or any(
                type(value) is not int or not 0 <= value <= 3 for value in group)
            for group in groups)):
        raise ValueError('Invalid packed pair groups')
    return [list(group) for group in groups]


def project_metadata12(catalog):
    """Keep the two independent EUR indexed banks without inferred owners."""
    banks = catalog['banks']
    if not isinstance(banks, list) or len(banks) != 2:
        raise ValueError('Invalid indexed packed pair banks')
    projected = []
    for bank, (identity, source, count) in zip(banks, (
            ('primary', 0x6e7484, 92), ('secondary', 0x6e78d4, 16))):
        if (bank['id'] != identity or type(bank['source_offset']) is not int
                or bank['source_offset'] != source or not isinstance(bank['records'], list)
                or len(bank['records']) != count):
            raise ValueError('Invalid indexed packed pair bank identity or extent')
        records = []
        for index, record in enumerate(bank['records']):
            if (type(record['id']) is not int or record['id'] != index
                    or type(record['source_offset']) is not int or record['source_offset'] != source + index * 12):
                raise ValueError('Invalid indexed packed pair record')
            records.append({'id': index, 'source_offset': record['source_offset'],
                            'groups': project_packed_groups(record['groups'], 12)})
        projected.append({'id': identity, 'source_offset': source, 'records': records})
    return {'banks': projected}


def project_global_pair_flags(catalog):
    """Keep the separately rooted eleven-byte EUR global pair source."""
    source = catalog['source_offset']
    if type(source) is not int or source != 0x6e7994:
        raise ValueError('Invalid global packed pair source')
    return {'source_offset': source, 'groups': project_packed_groups(catalog['groups'], 11)}


def project_bg_plane_programs(catalog):
    """Keep bounded indexed-pair writes and descriptor inheritance for previews."""
    family = catalog['paired']
    links, programs = family['descriptor_programs'], family['programs']
    if (not isinstance(links, list) or not 1 <= len(links) <= 512
            or not isinstance(programs, list) or not 1 <= len(programs) <= len(links)
            or any(type(index) is not int or not 0 <= index < len(programs) for index in links)):
        raise ValueError('Invalid paired plane program links')
    seen = set()
    for index in links:
        if index not in seen:
            if index != len(seen):
                raise ValueError('Invalid paired plane program order')
            seen.add(index)
    if len(seen) != len(programs):
        raise ValueError('Unreferenced paired plane program')
    result = []
    for program in programs:
        source, parent, writes = (program[key] for key in ('relative_source', 'parent_descriptor', 'writes'))
        if (type(source) is not int or not 0 <= source <= 0xffffffff
                or parent is not None and (type(parent) is not int or not 0 <= parent < len(links))
                or not isinstance(writes, dict) or writes.get('kind') != 'paired'):
            raise ValueError('Invalid paired plane program')
        header, dictionary, payload = writes['header'], writes['dictionary_offset'], writes['payload']
        if (type(header) is not int or not 0 <= header <= 0xffffffff or not header & 0x80000000
                or type(dictionary) is not int or dictionary != header & 0x0fffffff
                or not isinstance(payload, dict) or not isinstance(payload.get('packets'), list)
                or len(payload['packets']) > 4096):
            raise ValueError('Unverified paired plane packet format')
        total, packets = 0, []
        for packet in payload['packets']:
            offset, values = packet['byte_offset'], packet['values']
            if (type(offset) is not int or offset < 0 or offset % 4 or not isinstance(values, list)
                    or offset + len(values) * 4 > 0x4000
                    or any(type(value) is not int or not 0 <= value <= 0xffffffff for value in values)):
                raise ValueError('Invalid paired plane packet')
            total += len(values) * 4
            if total > 65536:
                raise ValueError('Paired plane packet writes exceed limit')
            packets.append({'byte_offset': offset, 'values': values})
        result.append({'relative_source': source, 'parent_descriptor': parent, 'packets': packets})
    for descriptor in range(len(links)):
        seen, current = set(), descriptor
        while current is not None:
            if current in seen:
                raise ValueError('Cyclic paired plane inheritance')
            seen.add(current)
            current = result[links[current]]['parent_descriptor']
    return {'descriptor_programs': links, 'programs': result}


def project_terrain_seeds(catalog):
    """Apply ordered auxiliary writes to the normal 16-by-16 zero-filled grid."""
    family = catalog['auxiliary']
    links, programs = family['descriptor_programs'], family['programs']
    if (not isinstance(links, list) or not links or len(links) > 512
            or not isinstance(programs, list) or not programs or len(programs) > len(links)
            or any(type(index) is not int or not 0 <= index < len(programs) for index in links)):
        raise ValueError('Invalid terrain program links')
    for program in programs:
        source, parent, writes = (program[key] for key in ('relative_source', 'parent_descriptor', 'writes'))
        if (type(source) is not int or not 0 <= source <= 0xffffffff
                or parent is not None and (type(parent) is not int or not 0 <= parent < len(links))
                or not isinstance(writes, dict) or writes.get('kind') != 'halfwords'
                or not isinstance(writes.get('packets'), list) or len(writes['packets']) > 4096):
            raise ValueError('Invalid terrain program')
        total = 0
        for packet in writes['packets']:
            offset, values = packet['byte_offset'], packet['values']
            if (type(offset) is not int or offset < 0 or offset % 2 or not isinstance(values, list)
                    or offset + len(values) * 2 > 512
                    or any(type(value) is not int or not 0 <= value <= 65535 for value in values)):
                raise ValueError('Invalid terrain packet')
            total += len(values) * 2
            if total > 65536:
                raise ValueError('Terrain packet writes exceed limit')
    maps = []
    for descriptor, program_id in enumerate(links):
        chain, seen, current = [], set(), descriptor
        while current is not None:
            if current in seen:
                raise ValueError('Cyclic terrain inheritance')
            seen.add(current); chain.append(current)
            current = programs[links[current]]['parent_descriptor']
        chain.reverse()
        cells = [0] * 256
        for ancestor in chain:
            for packet in programs[links[ancestor]]['writes']['packets']:
                start = packet['byte_offset'] // 2
                cells[start:start + len(packet['values'])] = packet['values']
        maps.append({'descriptor': descriptor, 'program': program_id, 'chain': chain, 'cells': cells})
    return {'maps': maps}


def project_obj_animation_banks(catalog):
    """Retain physical object lists, animation frames and selector roots."""
    banks, selectors = catalog['banks'], catalog.get('selector_tables', [])
    if not isinstance(banks, list) or not banks or len(banks) > 65536 or not isinstance(selectors, list):
        raise ValueError('Invalid OBJ animation catalog')
    def integer(value, minimum, maximum):
        return type(value) is int and minimum <= value <= maximum
    pairs = set()
    for bank in banks:
        object_root, sequence_root = bank['object_root'], bank['sequence_root']
        if not integer(object_root, 0, 0xffffffff) or not integer(sequence_root, 0, 0xffffffff):
            raise ValueError('Invalid OBJ animation roots')
        pair = (object_root, sequence_root)
        if pair in pairs:
            raise ValueError('Duplicate OBJ animation roots')
        pairs.add(pair)
        for root, offsets, records in ((object_root, bank['object_offsets'], bank['lists']),
                                       (sequence_root, bank['sequence_offsets'], bank['sequences'])):
            if (root % 4 or not isinstance(offsets, list) or not offsets or len(offsets) > 65536
                    or not isinstance(records, list) or len(records) != len(offsets)
                    or offsets[0] != len(offsets) * 4
                    or any(not integer(value, 0, 0xffffffff) or value % 2 for value in offsets)
                    or any(a >= b for a, b in zip(offsets, offsets[1:]))
                    or any(root + offset > 0xffffffff or item['source_offset'] != root + offset
                           for offset, item in zip(offsets, records))):
                raise ValueError('Invalid OBJ animation index')
        for item in bank['lists']:
            objects = item['objects']
            if not isinstance(objects, list) or len(objects) >= 32768:
                raise ValueError('Invalid OBJ object list')
            for obj in objects:
                if (not integer(obj['x'], -256, 255) or not integer(obj['y'], -128, 127)
                        or any(not integer(obj[key], 0, 3) for key in ('affine_mode', 'object_mode', 'shape', 'size', 'priority'))
                        or not integer(obj['transform'], 0, 31) or not integer(obj['tile'], 0, 1023)
                        or not integer(obj['palette'], 0, 15) or type(obj['mosaic']) is not bool or type(obj['color_256']) is not bool):
                    raise ValueError('Invalid OBJ object attributes')
        for sequence in bank['sequences']:
            frames = sequence['frames']
            if not isinstance(frames, list) or len(frames) > 255 or any(
                    not integer(frame['object_list'], 0, len(bank['lists']) - 1)
                    or not integer(frame['duration'], 0, 65535) for frame in frames):
                raise ValueError('Invalid OBJ animation frames')
            hold = sequence.get('zero_count_hold')
            if hold is not None and (frames or not integer(hold['object_list'], 0, len(bank['lists']) - 1)
                                     or hold['duration_source_offset'] != sequence['source_offset'] + 4):
                raise ValueError('Invalid OBJ zero-count hold')
    for table in selectors:
        roots, sequences = table['object_roots'], table['sequence_roots']
        if (not all(integer(table[key], 0, 0xffffffff) for key in ('selector_address', 'object_table_offset', 'sequence_table_offset'))
                or not isinstance(roots, list) or not roots or len(roots) > 256
                or not isinstance(sequences, list) or len(sequences) != len(roots)
                or any(not integer(a, 0, 0xffffffff) or not integer(b, 0, 0xffffffff) or (a, b) not in pairs
                       for a, b in zip(roots, sequences))):
            raise ValueError('Invalid OBJ animation selector')
    return {'banks': banks, 'selector_tables': selectors}


def project_obj_static_lists(catalog):
    """Retain the ordered source identities and attributes of static OBJ lists."""
    lists = catalog['lists']
    if not isinstance(lists, list) or not 0 < len(lists) <= 65536:
        raise ValueError('Invalid static OBJ catalog')

    def integer(value, minimum, maximum):
        return type(value) is int and minimum <= value <= maximum

    end = 0
    for item in lists:
        offset, objects = item['source_offset'], item['objects']
        if (not integer(offset, 0, 0xffffffff) or offset % 2 or offset < end
                or not isinstance(objects, list) or len(objects) >= 32768
                or offset + 2 + len(objects) * 6 > 0xffffffff):
            raise ValueError('Invalid static OBJ list extent')
        end = offset + 2 + len(objects) * 6
        for obj in objects:
            if (not integer(obj['x'], -256, 255) or not integer(obj['y'], -128, 127)
                    or any(not integer(obj[key], 0, 3) for key in ('affine_mode', 'object_mode', 'shape', 'size', 'priority'))
                    or not integer(obj['transform'], 0, 31) or not integer(obj['tile'], 0, 1023)
                    or not integer(obj['palette'], 0, 15) or type(obj['mosaic']) is not bool or type(obj['color_256']) is not bool):
                raise ValueError('Invalid static OBJ attributes')
    return {'lists': lists}


def project_layered_obj_animations(catalog):
    """Retain layered and separately rooted title animation banks."""
    projected = project_obj_animation_banks({'banks': catalog['banks']})
    descriptors = catalog['descriptors']
    if not isinstance(descriptors, list) or len(descriptors) != len(projected['banks']):
        raise ValueError('Invalid layered OBJ descriptors')
    for descriptor, bank in zip(descriptors, projected['banks']):
        opaque = descriptor['opaque']
        if (descriptor['object_root'] != bank['object_root'] or descriptor['sequence_root'] != bank['sequence_root']
                or not isinstance(opaque, list) or len(opaque) != 20
                or any(type(value) is not int or not 0 <= value <= 255 for value in opaque)):
            raise ValueError('Invalid layered OBJ descriptor')
    projected['descriptors'] = descriptors
    for key in ('title_language_bank', 'title_main_bank'):
        if key in catalog and catalog[key] is not None:
            projected[key] = project_obj_animation_banks({'banks': [catalog[key]]})['banks'][0]
    return projected


COMPACT_OBJ_SUFFIXES = ('', '-secondary', '-tertiary', '-quaternary', '-quinary', '-senary', '-septenary', '-octonary')


def project_compact_obj_animations(catalog, *, primary=False):
    """Normalize sparse compact frame references while retaining their source IDs."""
    def integer(value, minimum, maximum):
        return type(value) is int and minimum <= value <= maximum
    if primary:
        offsets, source_lists = catalog['list_offsets'], catalog['lists']
        seq_offsets, source_sequences = catalog['sequence_offsets'], catalog['sequences']
        if (not isinstance(offsets, list) or not isinstance(source_lists, list) or len(offsets) != len(source_lists)
                or not isinstance(seq_offsets, list) or not isinstance(source_sequences, list)
                or len(seq_offsets) != len(source_sequences)):
            raise ValueError('Invalid compact OBJ directories')
        lists = [{'index': index, 'relative_offset': offset, 'list': item}
                 for index, (offset, item) in enumerate(zip(offsets, source_lists))]
        sequences = [{'relative_offset': offset, 'sequence': item}
                     for offset, item in zip(seq_offsets, source_sequences)]
    else:
        lists, sequences = catalog['lists'], catalog['sequences']
    if (not isinstance(lists, list) or not 0 < len(lists) <= 256
            or not isinstance(sequences, list) or not 0 < len(sequences) <= 256):
        raise ValueError('Empty compact OBJ projection')
    list_roots, list_ids, projected_lists = set(), {}, []
    for item in lists:
        index, relative, source = item['index'], item['relative_offset'], item['list']
        offset, objects = source['source_offset'], source['objects']
        if (not integer(index, 0, 255) or index in list_ids or not integer(relative, 0, 65535)
                or not integer(offset, relative, 0xffffffff) or not isinstance(objects, list) or len(objects) >= 32768):
            raise ValueError('Invalid compact OBJ list')
        for obj in objects:
            if (not integer(obj['x'], -256, 255) or not integer(obj['y'], -128, 127)
                    or any(not integer(obj[key], 0, 3) for key in ('affine_mode', 'object_mode', 'shape', 'size', 'priority'))
                    or not integer(obj['transform'], 0, 31) or not integer(obj['tile'], 0, 1023)
                    or not integer(obj['palette'], 0, 15) or type(obj['mosaic']) is not bool or type(obj['color_256']) is not bool):
                raise ValueError('Invalid compact OBJ attributes')
        list_roots.add(offset - relative); list_ids[index] = len(projected_lists)
        projected_lists.append({'source_offset': offset, 'source_index': index, 'relative_offset': relative, 'objects': objects})
    if len(list_roots) != 1:
        raise ValueError('Inconsistent compact OBJ list root')
    sequence_roots, selectors, projected_sequences = set(), set(), []
    for item in sequences:
        relative, sequence = item['relative_offset'], item['sequence']
        selector, offset, frames = (sequence[key] for key in ('selector', 'source_offset', 'frames'))
        if (not integer(relative, 0, 65535) or not integer(selector, 0, 65535) or selector in selectors
                or not integer(offset, relative, 0xffffffff) or not isinstance(frames, list) or len(frames) > 65535):
            raise ValueError('Invalid compact OBJ sequence')
        projected_frames = []
        for frame in frames:
            index, duration = frame['list'], frame['duration']
            if not integer(index, 0, 255) or index not in list_ids or not integer(duration, 0, 255):
                raise ValueError('Invalid compact OBJ frame')
            projected_frames.append({'object_list': list_ids[index], 'list_id': index, 'duration': duration})
        sequence_roots.add(offset - relative); selectors.add(selector)
        projected_sequences.append({'selector': selector, 'source_offset': offset, 'relative_offset': relative, 'frames': projected_frames})
    if len(sequence_roots) != 1:
        raise ValueError('Inconsistent compact OBJ sequence root')
    return {'banks': [{'object_root': list_roots.pop(), 'sequence_root': sequence_roots.pop(),
                       'lists': projected_lists, 'sequences': projected_sequences}], 'selector_tables': []}


def project_compact_obj_catalog(catalog):
    """Expose each witnessed compact bank while preserving roots and selectors."""
    banks = catalog['banks']
    if not isinstance(banks, list) or not 0 < len(banks) <= 65536:
        raise ValueError('Invalid compact OBJ catalog')
    projected, roots = [], set()
    for bank in banks:
        root, flags, relative = (bank[key] for key in ('root', 'flags', 'sequence_relative_offset'))
        if (type(root) is not int or not 0 <= root <= 0xffffffff or root in roots
                or type(flags) is not int or not 0 <= flags <= 65535
                or type(relative) is not int or not 0 <= relative <= 65535):
            raise ValueError('Invalid compact OBJ catalog header')
        item = project_compact_obj_animations(bank)['banks'][0]
        if item['object_root'] != root + 4 or item['sequence_root'] != root + relative:
            raise ValueError('Invalid compact OBJ catalog root binding')
        item['flags'] = flags; projected.append(item); roots.add(root)
    return {'banks': projected, 'selector_tables': []}


def project_compact_catalog_auxiliary(catalog, banks):
    """Keep bounded source tables and only documented links to available sequences."""
    roots = {bank['object_root'] - 4: (index, bank) for index, bank in enumerate(banks)}
    table_targets = {
        'motion': [(0x6c58b8, 'selector', 0)], 'positions': [(0x6c6e6c, 'selector', 0)],
        'horizontal': [(0x6cbaf8, 'selector', 0)], 'timed_positions': [(0x6c992c, 'selector', 0)],
        'motion_positions': [(0x6c95d4, 'selector', 0)], 'timer_positions': [(0x6c0614, 'selector', 0)],
        'schedule': [(0x6bac28, 'selector', 0)], 'flight': [(0x6babc4, 'selector', 0)],
        'burst': [(0x6babc4, 'selector', 0)], 'effects': [(0x6bb574, 'selector', 0)],
        'script_lookup': [(0x6cb278, 'value', 0)],
        'scene': [(0x6ccae4, 'first_selector', 0), (0x6ccdec, 'second_selector', 1)],
    }
    def uint(value, maximum):
        return type(value) is int and 0 <= value <= maximum
    def target(root, selector, field):
        if not uint(selector, 65535) or root not in roots:
            return None
        index, bank = roots[root]
        for sequence, item in enumerate(bank['sequences']):
            if item['selector'] == selector:
                return {'field': field, 'bank': index, 'sequence': sequence, 'selector': selector}
        return None
    def checked_row(row):
        if type(row) is int:
            row = {'value': row}
        if not isinstance(row, dict) or len(row) > 32:
            raise ValueError('Invalid compact catalog table row')
        for key, value in row.items():
            if (not isinstance(key, str) or not key.isidentifier() or len(key) > 64
                    or type(value) is not int or not -0x80000000 <= value <= 0xffffffff):
                raise ValueError('Invalid compact catalog table field')
        return row
    def row_links(name, row):
        result = []
        for root, field, add in table_targets.get(name, []):
            value = row.get(field)
            if type(value) is int:
                link = target(root, value + add, field + (' + 1' if add else ''))
                if link is not None:
                    result.append(link)
        if name in ('grid_first', 'grid_second'):
            base, x, y = row.get('base_selector'), row.get('x_mode'), row.get('y_mode')
            if all(uint(value, 255) for value in (base, x, y)):
                adjusted = any(3 <= value <= 5 or value > 8 for value in (x, y))
                link = target(0x6cac40, (base + (4 if adjusted else 0)) & 255, 'computed selector')
                if link is not None:
                    result.append(link)
        return result
    groups = []
    tables = catalog.get('selector_tables')
    if tables is not None:
        if not isinstance(tables, dict) or len(tables) > 64:
            raise ValueError('Invalid compact catalog tables')
        for name, table in tables.items():
            if table is None:
                continue
            if (not isinstance(name, str) or not name.isidentifier() or len(name) > 64
                    or not isinstance(table, dict) or not uint(table['source_offset'], 0xffffffff)
                    or not isinstance(table['rows'], list) or len(table['rows']) > 65536):
                raise ValueError('Invalid compact catalog table')
            rows = [checked_row(row) for row in table['rows']]
            groups.append({'id': name, 'source_offset': table['source_offset'], 'rows': rows,
                           'links': [row_links(name, row) for row in rows]})
    for name in ('red_spring_records', 'star_cross_records'):
        records = catalog.get(name)
        if records is None:
            continue
        if not isinstance(records, list) or len(records) > 65536:
            raise ValueError('Invalid compact catalog records')
        links = []
        for record in records:
            values = record['halfwords']
            if (not uint(record['source_offset'], 0xffffffff) or not uint(record['bank_pointer'], 0xffffffff)
                    or not isinstance(values, list) or len(values) != 10 or any(not uint(value, 65535) for value in values)):
                raise ValueError('Invalid compact catalog record')
            link = target(record['bank_pointer'] - 0x08000000, values[0], 'halfwords[0]')
            links.append([link] if link is not None else [])
        groups.append({'id': name, 'source_offset': None, 'rows': records, 'links': links})
    soil = catalog.get('soil_evidence_metadata')
    if soil is not None:
        if (not isinstance(soil, dict) or any(not uint(soil[key], 65535) for key in
                ('flags', 'sequence_relative_offset', 'selector0_relative_offset'))
                or not isinstance(soil['list_offsets'], list) or len(soil['list_offsets']) > 65536
                or any(not uint(value, 65535) for value in soil['list_offsets'])
                or not isinstance(soil['frames'], list) or len(soil['frames']) > 65536):
            raise ValueError('Invalid compact catalog metadata')
        for frame in soil['frames']:
            if not uint(frame['list'], 255) or not uint(frame['duration'], 255):
                raise ValueError('Invalid compact catalog metadata frame')
        header = {key: soil[key] for key in ('flags', 'sequence_relative_offset', 'selector0_relative_offset')}
        lists = [{'index': index, 'relative_offset': value} for index, value in enumerate(soil['list_offsets'])]
        frames = [{'index': index, **frame} for index, frame in enumerate(soil['frames'])]
        for name, rows in [('soil_evidence_header', [header]), ('soil_evidence_lists', lists), ('soil_evidence_frames', frames)]:
            groups.append({'id': name, 'source_offset': None, 'rows': rows, 'links': [[] for _ in rows]})
    return {'groups': groups} if groups else None


def project_soil_evidence_animation(static, compact):
    """Join the documented selector-zero sequence to its separately owned OBJ lists."""
    if not static or not compact or not compact.get('auxiliary'):
        return None
    groups = compact['auxiliary']['groups']
    selected = {}
    for name in ('soil_evidence_header', 'soil_evidence_lists', 'soil_evidence_frames'):
        matches = [group for group in groups if group['id'] == name]
        if len(matches) != 1:
            return None
        selected[name] = matches[0]['rows']
    header = selected['soil_evidence_header']
    offsets = selected['soil_evidence_lists']
    frames = selected['soil_evidence_frames']
    if (len(header) != 1 or header[0] != {'flags': 12, 'sequence_relative_offset': 0x278,
                                           'selector0_relative_offset': 2}
            or len(offsets) != 20 or len(frames) != 20):
        return None
    by_offset = {item['source_offset']: item for item in static['lists']}
    lists, sequence = [], []
    for index, (offset, frame) in enumerate(zip(offsets, frames)):
        relative, duration = offset.get('relative_offset'), frame.get('duration')
        if (offset.get('index') != index or type(relative) is not int or not 0 <= relative <= 65535
                or frame.get('index') != index or frame.get('list') != index
                or type(duration) is not int or not 1 <= duration <= 255):
            return None
        item = by_offset.get(0x6c1740 + relative)
        if item is None or item in lists:
            return None
        lists.append(item)
        sequence.append({'object_list': index, 'list_id': index, 'duration': duration})
    return {'path': compact['path'] + ' · lists: ' + static['path'], 'selector_tables': [],
            'banks': [{'object_root': 0x6c1740, 'sequence_root': 0x6c19b4, 'flags': 12,
                       'lists': lists, 'sequences': [{'selector': 0, 'source_offset': 0x6c19b6,
                                                      'frames': sequence}]}]}


def project_constructor_obj_animation(source, static):
    """Join four constructor phases to their independently owned static lists."""
    if not static:
        raise ValueError('Static object lists are unavailable')
    lists = project_obj_static_lists({'lists': static['lists']})['lists']
    if (type(source['source_offset']) is not int or source['source_offset'] != 0x394fbc
            or not isinstance(source['frames'], list) or len(source['frames']) != 4):
        raise ValueError('Invalid constructor animation table')
    by_offset = {item['source_offset']: item for item in lists}
    selected, frames = [], []
    for index, (item, expected) in enumerate(zip(source['frames'], (0x394f82, 0x394f90, 0x394f9e, 0x394fac))):
        threshold, offset = item['threshold'], item['list_offset']
        if (type(threshold) is not int or not 0 <= threshold <= 0xffffffff
                or type(offset) is not int or offset != expected):
            raise ValueError('Invalid constructor animation threshold or list pointer')
        layout = by_offset.get(offset)
        if layout is None:
            raise ValueError('Referenced constructor object list is unavailable')
        if len(layout['objects']) != 2 or any(obj['shape'] >= 3 for obj in layout['objects']):
            raise ValueError('Invalid constructor object list')
        selected.append(layout)
        # The guest reloads an 8-bit counter before comparing it to the word.
        stalled = threshold >= 255
        frames.append({'object_list': index, 'list_id': index, 'threshold': threshold,
                       'duration': 1 if stalled else threshold + 1, 'counter_stalls': stalled})
    return {'selector_tables': [], 'preview_mode': 'constructor_thresholds',
            'metadata': {'table_offset': source['source_offset'], 'counter_bits': 8,
                         'thresholds': [frame['threshold'] for frame in frames]},
            'banks': [{'object_root': selected[0]['source_offset'], 'sequence_root': source['source_offset'],
                       'lists': selected, 'sequences': [{'selector': 0, 'source_offset': source['source_offset'],
                                                        'frames': frames}]}]}


FIRAGA_OBJ_SPECS = {
    'graphics.firaga-indexed-obj-bank': {
        'key': 'firaga_indexed', 'root': 0x418e9c, 'sequence_root': 0x412050, 'sequence_offset': 4,
        'offsets': [0x50, 0x54, 0x5c, 0x64, 0x74, 0x84, 0x94, 0xa8, 0xbc, 0xd8, 0xf4, 0x110,
                    0x12c, 0x148, 0x164, 0x184, 0x1a4, 0x1c4, 0x1e4, 0x204], 'frame_count': 19},
    'graphics.firaga-compact-obj-bank': {
        'key': 'firaga_compact', 'root': 0x6b3578, 'sequence_root': 0x6b3694, 'sequence_offset': 2,
        'offsets': [20, 28, 42, 68, 106, 162, 218, 274, 276, 278], 'frame_count': 10},
    'graphics.firaga-adjacent-compact-obj-bank': {
        'key': 'firaga_adjacent', 'root': 0x6b36ac, 'sequence_root': 0x6b4224, 'sequence_offset': 8,
        'offsets': [0x44, 0xb2, 0x120, 0x194, 0x208, 0x27c, 0x2f6, 0x37c, 0x402, 0x488, 0x48a],
        'frame_count': 17},
    'graphics.firaga-6c12c8-obj-bank': {
        'key': 'firaga_6c12c8', 'root': 0x6c12c8, 'sequence_root': 0x6c13e0, 'sequence_offset': 2,
        'offsets': [20, 40, 60, 98, 136, 174, 212, 232, 252, 272], 'frame_count': 13},
}


def project_firaga_obj_animation(source, static, resource_id):
    """Join one witnessed Firaga sequence to the static lists it actually references."""
    spec = FIRAGA_OBJ_SPECS[resource_id]
    if not static:
        raise ValueError('Static object lists are unavailable')

    def fixed(key, expected):
        if type(source[key]) is not int or source[key] != expected:
            raise ValueError('Invalid Firaga OBJ source binding')

    def fields(key, length):
        value = source[key]
        if (not isinstance(value, list) or len(value) != length
                or any(type(item) is not int or not 0 <= item <= 65535 for item in value)):
            raise ValueError('Invalid Firaga OBJ descriptor fields')
        return value

    indexed = spec['key'] == 'firaga_indexed'
    if indexed:
        fixed('sequence_pointer', 0x08412050)
        fixed('object_pointer', 0x08418e9c)
        fixed('sequence_relative_offset', 4)
        if fields('descriptor_fields', 2) != [127, 128]:
            raise ValueError('Invalid Firaga indexed descriptor')
        sequence = source['sequence']
        if (type(sequence['source_offset']) is not int or sequence['source_offset'] != 0x412054
                or sequence.get('zero_count_hold') is not None):
            raise ValueError('Invalid Firaga indexed sequence')
        offsets, frames = source['object_offsets'], sequence['frames']
        object_root = spec['root']
        metadata_keys = ('sequence_pointer', 'object_pointer', 'descriptor_fields',
                         'sequence_relative_offset', 'object_offsets')
    else:
        fixed('flags', 12)
        fixed('split_offset', spec['sequence_root'] - spec['root'])
        fixed('sequence_relative_offset', spec['sequence_offset'])
        offsets, frames = source['list_offsets'], source['frames']
        object_root = spec['root'] + 4
        if spec['key'] == 'firaga_compact':
            for key, value in (('caller_literal', 0x0839cc6c), ('descriptor_root', 0x086b3578),
                               ('selected_pointer', 0x0839cc3c)):
                fixed(key, value)
            metadata_keys = ('caller_literal', 'descriptor_root', 'selected_pointer', 'flags',
                             'split_offset', 'list_offsets', 'sequence_relative_offset')
        elif spec['key'] == 'firaga_adjacent':
            for key, value in (('count_literal', 0x0839cc78), ('count', 2), ('second_bank_pointer', 0x086b36ac)):
                fixed(key, value)
            if fields('first_descriptor_fields', 10)[0] or fields('second_descriptor_fields', 10)[0]:
                raise ValueError('Invalid Firaga sequence selector')
            metadata_keys = ('count_literal', 'count', 'first_descriptor_fields', 'second_bank_pointer',
                             'second_descriptor_fields', 'flags', 'split_offset', 'list_offsets', 'sequence_relative_offset')
        else:
            for key, value in (('count', 1), ('descriptor_pointer', 0x0839a8bc), ('bank_pointer', 0x086c12c8)):
                fixed(key, value)
            if fields('descriptor_fields', 10)[0]:
                raise ValueError('Invalid Firaga sequence selector')
            metadata_keys = ('count', 'descriptor_pointer', 'bank_pointer', 'descriptor_fields', 'flags',
                             'split_offset', 'list_offsets', 'sequence_relative_offset')
    if (not isinstance(offsets, list) or offsets != spec['offsets']
            or any(type(value) is not int for value in offsets)
            or not isinstance(frames, list) or len(frames) != spec['frame_count']):
        raise ValueError('Invalid Firaga OBJ directory or frame count')
    by_offset = {item['source_offset']: item for item in static['lists']}
    lists, list_indices, projected_frames = [], {}, []
    for frame in frames:
        index, duration = frame['object_list' if indexed else 'list'], frame['duration']
        if (type(index) is not int or not 0 <= index < len(offsets) or type(duration) is not int
                or not 0 <= duration <= (65535 if indexed else 255)):
            raise ValueError('Invalid Firaga OBJ frame')
        if index not in list_indices:
            item = by_offset.get(object_root + offsets[index])
            if item is None:
                raise ValueError('Referenced Firaga object list is unavailable')
            list_indices[index] = len(lists)
            lists.append({**item, 'source_index': index, 'relative_offset': offsets[index]})
        projected_frames.append({'object_list': list_indices[index], 'list_id': index, 'duration': duration})
    return {'selector_tables': [], 'metadata': {key: source[key] for key in metadata_keys},
            'banks': [{'object_root': object_root, 'sequence_root': spec['sequence_root'],
                       'lists': lists, 'sequences': [{'selector': 0,
                         'source_offset': spec['sequence_root'] + spec['sequence_offset'],
                         'frames': projected_frames}]}]}


def project_script_commands(catalog):
    source, commands = catalog['source_offset'], catalog['commands']
    def uint(value, maximum):
        return type(value) is int and 0 <= value <= maximum
    if (not uint(source, 0xffffffff) or not isinstance(commands, list) or not 0 < len(commands) <= 256):
        raise ValueError('Invalid script command directory')
    for index, command in enumerate(commands):
        offset = command['callback']['rom_offset']
        if (command['opcode'] != index or type(command['opcode']) is not int
                or not uint(command['advance'], 65535) or not uint(offset, 0xffffff) or offset % 2):
            raise ValueError('Invalid script command record')
    return {'source_offset': source, 'commands': commands}


def project_auxiliary_script_archive(catalog):
    """Retain physical programs and each slot's exact instruction entry."""
    def uint(value, maximum=0xffffffff):
        return type(value) is int and 0 <= value <= maximum

    banks = catalog['banks']
    if (not uint(catalog['source_offset']) or not isinstance(banks, list)
            or not 0 < len(banks) <= 256):
        raise ValueError('Invalid auxiliary script archive')
    program_ids, ranges = set(), []
    for bank_index, bank in enumerate(banks):
        entries, programs, alignment = bank['entries'], bank['programs'], bank['alignment_after']
        if (type(bank['id']) is not int or bank['id'] != bank_index
                or not uint(bank['source_offset']) or not isinstance(entries, list)
                or not 0 < len(entries) <= 256 or not isinstance(programs, list)
                or not 0 < len(programs) <= 65536 or not isinstance(alignment, list)
                or len(alignment) > 256 or any(not uint(value, 255) for value in alignment)):
            raise ValueError('Invalid auxiliary script bank')
        in_bank = {}
        for program in programs:
            identity, source, instructions = program['id'], program['source_offset'], program['instructions']
            if (not uint(identity, 65535) or identity in program_ids or not uint(source)
                    or not isinstance(instructions, list) or not 0 < len(instructions) <= 65536):
                raise ValueError('Invalid auxiliary script program')
            program_ids.add(identity)
            in_bank[identity] = program
            next_offset = source
            for instruction in instructions:
                params = instruction['parameters']
                if (not uint(instruction['source_offset']) or instruction['source_offset'] != next_offset
                        or not uint(instruction['command'], 255) or not isinstance(params, list)
                        or len(params) > 255 or any(not uint(value, 255) for value in params)):
                    raise ValueError('Invalid auxiliary script instruction')
                next_offset += 1 + len(params)
                if next_offset > 0x100000000:
                    raise ValueError('Auxiliary script source overflows')
            ranges.append((source, next_offset))
        for entry in entries:
            identity, instruction = entry['program_id'], entry['instruction']
            if (not uint(identity, 65535) or identity not in in_bank
                    or not uint(instruction, len(in_bank[identity]['instructions']) - 1)):
                raise ValueError('Invalid auxiliary script entry')
    ranges.sort()
    if any(left[1] > right[0] for left, right in zip(ranges, ranges[1:])):
        raise ValueError('Overlapping auxiliary script programs')
    return {'source_offset': catalog['source_offset'], 'banks': banks}


def project_task_descriptors(catalog):
    """Keep the three physical catalogs and their nullable Thumb callbacks."""
    def uint(value, maximum):
        return type(value) is int and 0 <= value <= maximum

    result, spans = {}, []
    for name in ('primary', 'auxiliary', 'script'):
        table = catalog[name]
        source, entries = table['source_offset'], table['entries']
        if (not uint(source, 0xffffff) or not isinstance(entries, list)
                or not 0 < len(entries) <= 65535 or source + len(entries) * 8 > 0x1000000):
            raise ValueError('Invalid task descriptor table')
        for index, item in enumerate(entries):
            callback = item['callback']
            if (type(item['ordinal']) is not int or item['ordinal'] != index
                    or not uint(item['context_size'], 65535) or not uint(item['metadata_raw'], 65535)):
                raise ValueError('Invalid task descriptor')
            if callback is not None:
                offset = callback['rom_offset']
                if not uint(offset, 0xffffff) or offset % 2:
                    raise ValueError('Invalid task Thumb callback')
        spans.append((source, source + len(entries) * 8))
        result[name] = {'source_offset': source, 'entries': entries}
    spans.sort()
    if any(left[1] > right[0] for left, right in zip(spans, spans[1:])):
        raise ValueError('Overlapping task descriptor tables')
    return result


def project_effect_event_directories(catalog):
    """Retain stored fields and documented, bounded task references."""
    def uint(value, maximum):
        return type(value) is int and 0 <= value <= maximum

    events, selectors = catalog['events'], catalog['selectors']
    if (not uint(catalog['event_source_offset'], 0xffffff)
            or not uint(catalog['selector_source_offset'], 0xffffff)
            or not isinstance(events, list) or not 0 < len(events) <= 65535
            or not isinstance(selectors, list) or not 0 < len(selectors) <= 65535
            or catalog['event_source_offset'] + len(events) * 16 > 0x1000000
            or catalog['selector_source_offset'] + len(selectors) * 4 > 0x1000000):
        raise ValueError('Invalid event or selector table')
    for index, item in enumerate(events):
        if (type(item['index']) is not int or item['index'] != index
                or any(not uint(item[key], 255) for key in ('opcode', 'byte1_raw', 'post_delay', 'byte5_raw'))
                or any(not uint(item[key], 65535) for key in ('id_raw', 'half6_raw'))
                or any(not uint(item[key], 0xffffffff) for key in ('arg8_raw', 'arg12_raw'))):
            raise ValueError('Invalid event record')
        descriptor = item['script_descriptor']
        if item['opcode'] == 0:
            if not uint(descriptor, 65535 - 363) or item['id_raw'] != descriptor + 363:
                raise ValueError('Invalid event descriptor reference')
        elif descriptor is not None:
            raise ValueError('Unexpected event descriptor reference')
    for index, item in enumerate(selectors):
        params = item['parameters']
        if (type(item['index']) is not int or item['index'] != index
                or not uint(item['kind'], 255) or not isinstance(params, list) or len(params) != 3
                or any(not uint(value, 255) for value in params)
                or type(item['script_descriptor']) is not int or item['script_descriptor'] != index):
            raise ValueError('Invalid effect selector')
    for name in ('event_dispatch', 'selector_dispatch'):
        table = catalog[name]
        if (not uint(table['source_offset'], 0xffffff) or not isinstance(table['slots'], list)
                or not 0 < len(table['slots']) <= 256
                or table['source_offset'] + len(table['slots']) * 4 > 0x1000000):
            raise ValueError('Invalid dispatch table')
        for index, item in enumerate(table['slots']):
            offset = item['target']['rom_offset']
            if (type(item['selector']) is not int or item['selector'] != index
                    or not uint(offset, 0xffffff) or offset % 2):
                raise ValueError('Invalid same-state dispatch target')
    return {key: catalog[key] for key in ('event_source_offset', 'events', 'selector_source_offset',
                                           'selectors', 'event_dispatch', 'selector_dispatch')}


def project_script_archive(catalog):
    keys, banks, programs, padding = (catalog[key] for key in ('keys', 'banks', 'programs', 'padding'))
    def uint(value, maximum):
        return type(value) is int and 0 <= value <= maximum
    if (any(not uint(catalog[key], 0xffffffff) for key in ('archive_offset', 'key_offset'))
            or not isinstance(keys, list) or not 0 < len(keys) <= 65536
            or not isinstance(banks, list) or not 0 < len(banks) <= 255
            or not isinstance(programs, list) or not 0 < len(programs) <= 65536
            or not isinstance(padding, list) or len(padding) > 256):
        raise ValueError('Invalid script archive catalog')
    for index, program in enumerate(programs):
        instructions = program['instructions']
        if (type(program['ordinal']) is not int or program['ordinal'] != index
                or not uint(program['source_offset'], 0xffffffff) or not isinstance(instructions, list)
                or not 0 < len(instructions) <= 65536):
            raise ValueError('Invalid script program')
        for instruction in instructions:
            params, selector = instruction['parameters'], instruction['effect_selector']
            if (not uint(instruction['source_offset'], 0xffffffff) or not uint(instruction['command'], 255)
                    or not isinstance(params, list) or len(params) > 255 or any(not uint(value, 255) for value in params)
                    or selector is not None and not uint(selector, 255)):
                raise ValueError('Invalid stored script instruction')
    for item in padding:
        if (not uint(item['source_offset'], 0xffffffff) or not uint(item['length'], 255) or not item['length']
                or not uint(item['value'], 255)):
            raise ValueError('Invalid script padding')
    for index, bank in enumerate(banks):
        slots = bank['slots']
        if (type(bank['ordinal']) is not int or bank['ordinal'] != index + 1
                or not uint(bank['source_offset'], 0xffffffff) or not isinstance(slots, list) or not 0 < len(slots) <= 256):
            raise ValueError('Invalid script bank')
        for slot_index, slot in enumerate(slots):
            target = slot['target']; kind = target['kind']
            if type(slot['index']) is not int or slot['index'] != slot_index:
                raise ValueError('Invalid script slot')
            if kind == 'program':
                valid = uint(target['ordinal'], len(programs) - 1)
            elif kind == 'header':
                valid = uint(target['bank'], len(banks)) and target['bank'] > 0
            elif kind == 'fill':
                valid = uint(target['ordinal'], len(padding) - 1)
            else:
                valid = False
            if not valid:
                raise ValueError('Invalid script target')
    for index, key in enumerate(keys):
        if (type(key['id']) is not int or key['id'] != index
                or any(not uint(key[field], 255) for field in ('script_bank', 'script_index', 'text_bank', 'byte3_raw'))
                or key['script_bank'] > len(banks)
                or key['script_bank'] and key['script_index'] >= len(banks[key['script_bank'] - 1]['slots'])):
            raise ValueError('Invalid script event key')
    return {key: catalog[key] for key in ('archive_offset', 'key_offset', 'keys', 'banks', 'programs', 'padding')}


def project_script_text_refs(languages):
    """Retain the small top-level/variant mapping without embedding text."""
    if not all('top' in language and 'variants' in language for language in languages):
        return None
    result = []
    for language in languages:
        count, top, variants = len(language['banks']), language['top'], language['variants']['bank_indices']
        if (not isinstance(top, list) or not top or len(top) > 256
                or not isinstance(variants, list) or len(variants) > 256):
            raise ValueError('Invalid script text links')
        def bank_index(value):
            return type(value) is int and 0 <= value < count
        mapped = []
        for target in top:
            if target['kind'] == 'bank' and bank_index(target['index']):
                mapped.append(target['index'])
            elif target['kind'] == 'variants':
                mapped.append(None)
            else:
                raise ValueError('Invalid script text target')
        if any(not bank_index(value) for value in variants):
            raise ValueError('Invalid script text variants')
        result.append({'id': language['id'], 'top': mapped, 'variants': variants})
    return result


def project_bg_transfers(catalog):
    """Keep descriptor-linked tile frames, shared banks and raw timing."""
    links, variants, controls, tiles = (catalog[key] for key in
                                       ('descriptor_links', 'variant_sources', 'controls', 'tiles'))
    def uint(value, maximum):
        return type(value) is int and 0 <= value <= maximum
    if (not isinstance(links, list) or not 0 < len(links) <= 512
            or not isinstance(variants, list) or len(variants) != len(links) or any(value is not None for value in variants)
            or not isinstance(controls, list) or not 0 < len(controls) <= len(links) * 5
            or not isinstance(tiles, list) or not 0 < len(tiles) <= len(links) * 5):
        raise ValueError('Invalid background transfer catalog')
    for control in controls:
        size, frames = control['transfer_bytes'], control['frames']
        if (not uint(control['relative_source'], 0xffffffff) or not uint(size, 65535) or not size or size % 32
                or not uint(control['header_byte3'], 255) or not isinstance(frames, list) or not 0 < len(frames) <= 255
                or any(not uint(frame['duration_ticks'], 65535) or not uint(frame['source_tile'], 65535) for frame in frames)):
            raise ValueError('Invalid background transfer control')
    for bank in tiles:
        words = bank['words']
        if (not uint(bank['relative_source'], 0xffffffff) or not isinstance(words, list) or not words
                or len(words) > 4 * 1024 * 1024 // 4 or len(words) % 8
                or any(not uint(word, 0xffffffff) for word in words)):
            raise ValueError('Invalid background transfer tile bank')
    seen_controls, seen_tiles = set(), set()
    for descriptor in links:
        if not isinstance(descriptor, list) or len(descriptor) != 5:
            raise ValueError('Invalid background transfer slots')
        for link in descriptor:
            if link is None:
                continue
            control, bank, destination = (link[key] for key in ('control', 'tiles', 'destination'))
            if (not uint(control, len(controls) - 1) or not uint(bank, len(tiles) - 1)
                    or not uint(destination, 0xffffffff) or destination % 32 or destination < 0x06000000
                    or destination + controls[control]['transfer_bytes'] > 0x06018000
                    or any(frame['source_tile'] * 32 + controls[control]['transfer_bytes'] > len(tiles[bank]['words']) * 4
                           for frame in controls[control]['frames'])):
                raise ValueError('Invalid background transfer link or extent')
            seen_controls.add(control); seen_tiles.add(bank)
    if len(seen_controls) != len(controls) or len(seen_tiles) != len(tiles):
        raise ValueError('Unreferenced background transfer data')
    return {'descriptor_links': links, 'controls': controls, 'tiles': tiles}


def project_blend_programs(catalog):
    """Validate stored blend commands and producer references for browsing."""
    programs, producers = catalog['programs'], catalog['producers']
    if (not isinstance(programs, list) or not 0 < len(programs) <= 65536
            or not isinstance(producers, list) or len(producers) > 65536):
        raise ValueError('Invalid blend program catalog')
    def uint(value, maximum):
        return type(value) is int and 0 <= value <= maximum
    seen = set()
    for program in programs:
        source, header, commands = (program[key] for key in ('source_offset', 'header', 'commands'))
        control, initial = header['control'], header['initial']
        if (not uint(source, 0xffffffff) or source in seen
                or any(not uint(control[key], maximum) for key, maximum in
                       (('first_targets', 63), ('mode', 3), ('second_targets', 63), ('reserved', 3)))
                or any(not uint(initial[key], maximum) for key, maximum in
                       (('first', 31), ('first_reserved', 7), ('second', 31), ('second_reserved', 7)))
                or not uint(header['command_count'], 65535) or not isinstance(commands, list)
                or len(commands) != header['command_count']):
            raise ValueError('Invalid blend program header')
        seen.add(source)
        for command in commands:
            if any(not uint(command[key], maximum) for key, maximum in
                   (('duration', 65535), ('first', 255), ('second', 255))):
                raise ValueError('Invalid blend command')
    for producer in producers:
        if (not uint(producer['source_offset'], 0xffffffff) or not isinstance(producer['programs'], list)
                or len(producer['programs']) > 65536
                or any(not uint(value, len(programs) - 1) for value in producer['programs'])):
            raise ValueError('Invalid blend producer')
    return {'programs': programs, 'producers': producers}


def project_palette_programs(catalog):
    """Keep documented palette program commands and their producer links."""
    programs, producers = catalog['programs'], catalog['producers']
    if (not isinstance(programs, list) or not 0 < len(programs) <= 65536
            or not isinstance(producers, list) or len(producers) > 65536):
        raise ValueError('Invalid palette program catalog')
    def uint(value, maximum):
        return type(value) is int and 0 <= value <= maximum
    seen = set()
    for program in programs:
        source, header, commands = (program[key] for key in ('source_offset', 'header', 'commands'))
        if (not uint(source, 0xffffffff) or source in seen
                or any(not uint(header[key], 255) for key in ('first', 'flags', 'width', 'command_count'))
                or not uint(header['initial_delay'], 65535) or not isinstance(commands, list)
                or len(commands) != header['command_count']):
            raise ValueError('Invalid palette program header')
        seen.add(source)
        for command in commands:
            opcode, mode, duration, relative, parameter = (command[key] for key in
                ('opcode', 'mode', 'duration', 'relative_parameter', 'parameter'))
            if (not uint(opcode, 255) or not uint(mode, 255) or not uint(duration, 65535)
                    or not uint(relative, 65535) or not isinstance(parameter, dict)):
                raise ValueError('Invalid palette command')
            kind = parameter.get('kind')
            if kind == 'unused':
                valid = opcode in (1, 2, 13) or (opcode == 12 and mode != 0)
            elif kind == 'color-words':
                first = header['first'] + (256 if header['flags'] & 2 else 0)
                end = min(511, (first + header['width'] - 1) & 0xffff)
                expected = (max(0, end - first + 1) if mode else header['width'])
                colors = parameter['colors']
                valid = opcode == 0 and isinstance(colors, list) and len(colors) == expected and all(uint(v, 65535) for v in colors)
            elif kind == 'components':
                values = parameter['values']
                valid = (opcode in (3, 6) or (opcode == 12 and mode == 0)) and isinstance(values, list) and len(values) == 3 \
                    and all(type(v) is int and -32768 <= v <= 32767 for v in values)
            elif kind == 'halfword':
                valid = opcode in (4, 5) and uint(parameter['value'], 65535)
            elif kind == 'halfword-two-bytes':
                valid = opcode == 7 and uint(parameter['value'], 65535) and all(uint(parameter[k], 255) for k in ('first', 'second'))
            elif kind == 'byte':
                valid = opcode in (8, 9) and uint(parameter['value'], 255)
            elif kind == 'two-bytes':
                valid = opcode == 10 and all(uint(parameter[k], 255) for k in ('first', 'second'))
            elif kind == 'color-cycle':
                count, direction, colors = (parameter[key] for key in ('count', 'direction', 'colors'))
                valid = opcode == 11 and uint(count, 255) and uint(direction, 255) and isinstance(colors, list) \
                    and len(colors) == max(1, count) and all(uint(v, 65535) for v in colors)
            else:
                valid = False
            if not valid:
                raise ValueError('Invalid palette command parameter')
    for producer in producers:
        if not uint(producer['source_offset'], 0xffffffff) or not isinstance(producer['data'], dict):
            raise ValueError('Invalid palette producer')
        data = producer['data']
        if data.get('kind') == 'pointers':
            links = data['programs']
            if not isinstance(links, list) or len(links) > 65536 or any(
                    value is not None and (not uint(value, len(programs) - 1)) for value in links):
                raise ValueError('Invalid palette producer pointers')
        elif data.get('kind') == 'list':
            entries = data['entries']
            if not isinstance(entries, list) or len(entries) > 65536 or any(
                    not uint(item['program'], len(programs) - 1) or not uint(item['delay_override'], 65535)
                    or not uint(item['reserved'], 65535) for item in entries):
                raise ValueError('Invalid palette producer entries')
        else:
            raise ValueError('Invalid palette producer kind')
    return {'programs': programs, 'producers': producers}


def project_ui_choice_lists(catalog):
    """Retain physical lists and all eight original bytes per entry."""
    lists = catalog['lists']
    if not isinstance(lists, list) or not lists or len(lists) > 65536:
        raise ValueError('Invalid choice lists')
    seen, end = set(), 0
    for item in lists:
        name, source, kind, entries = (item[key] for key in ('id', 'source_offset', 'kind', 'entries'))
        if (not isinstance(name, str) or not 0 < len(name) <= 128 or name in seen
                or type(source) is not int or not end <= source <= 0xffffffff
                or kind not in ('direct-list', 'filtered-template')
                or not isinstance(entries, list) or not 0 < len(entries) <= 256):
            raise ValueError('Invalid choice list metadata')
        seen.add(name)
        end = source + 8 * len(entries)
        if end > 0x100000000:
            raise ValueError('Invalid choice list extent')
        for entry in entries:
            navigation = entry['navigation']
            values = [entry[key] for key in ('identifier', 'cursor_x', 'cursor_y', 'opaque')]
            values.extend(navigation[key] for key in ('left', 'right', 'up', 'down'))
            if any(type(value) is not int or not 0 <= value <= 255 for value in values):
                raise ValueError('Invalid choice entry bytes')
    return {'lists': lists}


EUROPE_ROM_SHA256 = '8fab1e390cb9d5e695bfdb86e81246892e1aa8c0cb4014db8235ff76538ea632'
EUROPE_UI_CONTEXT_SHA256 = 'f3f20b6c259a2353b0db6189b8596c37dabdb312850b7d92a8b0b9d23898c16f'
EUROPE_UI_CONTEXT_CATEGORIES = (0, 6, 7, 1, 2, 3, 3, 3, 0, 1, 4, 5, 8, 9, 10)


def project_ui_context_templates(catalog, *, source=None, profile=None, rom_sha256=None):
    """Validate and retain all bytes of eleven physical UI templates."""
    records = catalog['records']
    if not isinstance(records, list) or len(records) != 11:
        raise ValueError('Expected eleven UI context templates')
    encoded, projected = bytearray(), []
    for record in records:
        if not isinstance(record, dict):
            raise ValueError('Invalid UI context record')
        x, y, count, flags = (record[key] for key in ('x_units8', 'y_units8', 'entry_count', 'flags'))
        blocks = [record[key] for key in ('opaque_02_03', 'opaque_06_08', 'opaque_0a_13')]
        if (any(type(value) is not int or not 0 <= value <= 255 for value in (x, y, flags))
                or type(count) is not int or not 0 <= count <= 65535
                or any(not isinstance(block, list) or len(block) != length
                       or any(type(value) is not int or not 0 <= value <= 255 for value in block)
                       for block, length in zip(blocks, (2, 3, 10)))):
            raise ValueError('Invalid UI context fields')
        encoded.extend(bytes((x, y, *blocks[0])))
        encoded.extend(count.to_bytes(2, 'little'))
        encoded.extend(bytes((*blocks[1], flags, *blocks[2])))
        projected.append({'x_units8': x, 'y_units8': y, 'entry_count': count, 'flags': flags,
                          'opaque_02_03': list(blocks[0]), 'opaque_06_08': list(blocks[1]),
                          'opaque_0a_13': list(blocks[2])})
    source_offset = None
    digest = hashlib.sha256(encoded).hexdigest()
    if source is not None:
        if (not isinstance(source, dict) or type(source.get('offset')) is not int
                or not 0 <= source['offset'] <= 0xffffffff - len(encoded)
                or source.get('length') != len(encoded) or source.get('sha256') != digest):
            raise ValueError('UI context source does not match reconstructed bytes')
        source_offset = source['offset']
    known_selector = (profile == 'europe' and rom_sha256 == EUROPE_ROM_SHA256 and
                      source_offset == 0x394e44 and digest == EUROPE_UI_CONTEXT_SHA256)
    return {'source_offset': source_offset, 'records': projected,
            'category_templates': list(EUROPE_UI_CONTEXT_CATEGORIES) if known_selector else []}


def project_alternate_palettes(catalog):
    """Keep decoded banks and their documented descriptor/catalog links."""
    keys, sources, catalogs = (catalog[name] for name in ('descriptor_keys', 'relative_sources', 'catalogs'))
    if not isinstance(keys, list) or not keys or not isinstance(sources, list) or len(sources) != 2 \
            or not isinstance(catalogs, list) or len(catalogs) != 2:
        raise ValueError('Invalid alternate palette catalog')
    count = len(catalogs[0]) if isinstance(catalogs[0], list) else 0
    if not 0 < count < 255 or any(not isinstance(group, list) or len(group) != count for group in catalogs + sources):
        raise ValueError('Invalid alternate palette group count')
    if any(type(key) is not int or not (0 <= key < count or key == 255) for key in keys):
        raise ValueError('Invalid alternate palette descriptor key')
    valid_keys = [key for key in keys if key != 255]
    if len(valid_keys) != count or set(valid_keys) != set(range(count)):
        raise ValueError('Invalid alternate palette key mapping')
    projected = []
    for links, group in zip(sources, catalogs):
        entries, ranges = [], []
        for offset, palette in zip(links, group):
            banks, encoded = palette['banks'], palette['encoded_source']
            if (type(offset) is not int or not count * 2 <= offset <= 65535
                    or not isinstance(encoded, list) or not 0 < len(encoded) <= 4096
                    or any(type(value) is not int or not 0 <= value <= 255 for value in encoded)
                    or not isinstance(banks, list) or len(banks) != 8
                    or any(not isinstance(bank, list) or len(bank) != 16
                           or any(type(word) is not int or not 0 <= word <= 65535 for word in bank)
                           for bank in banks)):
                raise ValueError('Invalid alternate palette entry')
            ranges.append((offset, offset + len(encoded)))
            entries.append({'banks': banks, 'encoded_length': len(encoded)})
        ranges.sort()
        if any(right[0] < left[1] for left, right in zip(ranges, ranges[1:])):
            raise ValueError('Overlapping alternate palette entries')
        projected.append(entries)
    return {'descriptor_keys': keys, 'relative_sources': sources, 'catalogs': projected}


def project_script_paths(catalog):
    """Retain ordered XY paths, source alignment and direct script references."""
    def uint(value, maximum):
        return type(value) is int and 0 <= value <= maximum

    source, paths, links = (catalog[key] for key in ('source_offset', 'paths', 'script_links'))
    if (not uint(source, 0xffffffff) or not isinstance(paths, list) or not paths or len(paths) > 65536
            or not isinstance(links, list)):
        raise ValueError('Invalid script path catalog')
    cursor = source + len(paths) * 4
    for index, path in enumerate(paths):
        points, padding = path['points'], path['alignment_after']
        if (not uint(path['id'], 0xffff) or path['id'] != index
                or not uint(path['source_offset'], 0xffffffff) or path['source_offset'] != cursor
                or not isinstance(points, list) or not points or len(points) > 65535
                or any(not uint(point[key], 255) for point in points for key in ('x', 'y'))
                or not isinstance(padding, list)):
            raise ValueError('Invalid script path')
        cursor += 2 + len(points) * 2
        alignment = (4 - cursor % 4) % 4
        if len(padding) != alignment or any(type(value) is not int or value != 255 for value in padding):
            raise ValueError('Invalid script path alignment')
        cursor += alignment
        if cursor > 0x100000000:
            raise ValueError('Script path source overflow')
    previous = -1
    for link in links:
        offset, opcode, path_id = (link[key] for key in ('instruction_offset', 'script_opcode', 'path_id'))
        if (not uint(offset, 0xffffffff) or offset <= previous or not uint(opcode, 255)
                or opcode not in (0x44, 0x45) or not uint(path_id, 0xffff) or path_id >= len(paths)):
            raise ValueError('Invalid script path reference')
        previous = offset
    return {'source_offset': source, 'paths': paths, 'script_links': links}


def project_slot_job_lookup(catalog):
    entries = catalog['entries']
    if (catalog['source_offset'] != 0x399750 or not isinstance(entries, list) or len(entries) != 20
            or any(type(entry['slot']) is not int or entry['slot'] != index
                   or type(entry['job_id']) is not int or not 0 <= entry['job_id'] < 116
                   for index, entry in enumerate(entries))):
        raise ValueError('Invalid European slot-job lookup')
    return {'source_offset': catalog['source_offset'], 'entries': entries}


def project_job_metadata(catalog):
    records = catalog['records']
    fields = (('bytes_00_04', 5), ('bytes_06_1f', 26), ('bytes_20_33', 20))
    if (catalog['source_offset'] != 0x6e5464 or not isinstance(records, list) or len(records) != 116):
        raise ValueError('Invalid European job metadata extent')
    for record in records:
        if type(record['routing']) is not int or not 0 <= record['routing'] <= 255:
            raise ValueError('Invalid job metadata routing')
        for key, count in fields:
            values = record[key]
            if (not isinstance(values, list) or len(values) != count
                    or any(type(value) is not int or not 0 <= value <= 255 for value in values)):
                raise ValueError('Invalid job metadata bytes')
    return {'source_offset': catalog['source_offset'], 'records': records}


def project_metadata2(catalog):
    """Keep the fixed EUR byte-pair map, including its final forward-only row."""
    records = catalog['records']
    if (type(catalog['source_offset']) is not int or catalog['source_offset'] != 0x6e6c44
            or not isinstance(records, list) or len(records) != 44):
        raise ValueError('Invalid European byte-pair metadata extent')
    result = []
    for record in records:
        key, value = record['key'], record['value']
        if any(type(item) is not int or not 0 <= item <= 255 for item in (key, value)):
            raise ValueError('Invalid byte-pair metadata value')
        result.append({'key': key, 'value': value})
    return {'source_offset': catalog['source_offset'], 'records': result}


def project_metadata4(catalog):
    """Retain both selector/value slots without collapsing zero or hole entries."""
    records = catalog['records']
    if (catalog['source_offset'] != 0x6e6bf4 or not isinstance(records, list) or len(records) != 20):
        raise ValueError('Invalid European two-pair metadata extent')
    projected = []
    for record in records:
        slots = record['slots']
        if not isinstance(slots, list) or len(slots) != 2:
            raise ValueError('Invalid two-pair metadata slots')
        pair = []
        for slot in slots:
            selector, value = slot['selector'], slot['value']
            if any(type(item) is not int or not 0 <= item <= 255 for item in (selector, value)):
                raise ValueError('Invalid two-pair metadata bytes')
            pair.append({'selector': selector, 'value': value})
        projected.append({'slots': pair})
    return {'source_offset': catalog['source_offset'], 'records': projected}


def project_metadata14(catalog):
    """Retain the fixed EUR special-unit records and their original field widths."""
    records = catalog['records']
    if (catalog['address_bias'] != 0x6e6f50 or catalog['source_offset'] != 0x6e6f6c
            or catalog['first_id'] != 2 or not isinstance(records, list) or len(records) != 93):
        raise ValueError('Invalid European special-unit extent')
    fields = (('selectors01_02', 2, 255), ('halfwords04_06', 2, 65535),
              ('packed08_0a', 3, 255), ('bytes0c_0d', 2, 255))
    projected = []
    for index, item in enumerate(records):
        if (type(item['id']) is not int or item['id'] != index + 2
                or type(item['source_offset']) is not int or item['source_offset'] != 0x6e6f6c + index * 14
                or any(type(item[key]) is not int or not 0 <= item[key] <= 255 for key in ('byte00', 'byte03', 'byte0b'))):
            raise ValueError('Invalid special-unit ID, source or byte')
        record = {key: item[key] for key in ('id', 'source_offset', 'byte00', 'byte03', 'byte0b')}
        for key, count, maximum in fields:
            values = item[key]
            if (not isinstance(values, list) or len(values) != count
                    or any(type(value) is not int or not 0 <= value <= maximum for value in values)):
                raise ValueError('Invalid special-unit ' + key)
            record[key] = list(values)
        projected.append(record)
    return {'address_bias': catalog['address_bias'], 'source_offset': catalog['source_offset'],
            'first_id': catalog['first_id'], 'records': projected}


def project_slot_job_names(slots, metadata, pointer_texts):
    """Join direct selected and separately documented rows to the EUR name reader."""
    catalogs = [] if not pointer_texts else [(index, catalog) for index, catalog in enumerate(pointer_texts['catalogs'])
                                             if catalog['reader'] == 0x80165e8]
    selected = catalogs[0] if len(catalogs) == 1 else None
    languages = [] if selected is None or metadata is None else selected[1]['languages']
    used = set()

    def name_for(job_id):
        name_index, names = None, []
        if metadata is not None:
            record = metadata['records'][job_id]
            if record['routing'] == 0:
                first = record['bytes_00_04']
                name_index = first[0] | first[1] << 8
        for language in languages:
            if name_index is None or name_index >= len(language['entries']):
                names.append(None)
                continue
            text = pointer_texts['strings'][language['entries'][name_index]]
            names.append(text['units'])
            used.update(unit for unit in text['units'] if type(unit) is int)
        return name_index, names

    rows = []
    for entry in slots['entries']:
        name_index, names = name_for(entry['job_id'])
        rows.append({'slot': entry['slot'], 'job_id': entry['job_id'], 'name_index': name_index, 'names': names})
    references = []
    if metadata is not None:
        for job_id in (104, 105, 106):
            name_index, names = name_for(job_id)
            references.append({'job_id': job_id, 'name_index': name_index, 'names': names})
    return {'source_offset': slots['source_offset'], 'path': slots['path'],
            'metadata_path': metadata['path'] if metadata else None,
            'text_path': pointer_texts['path'] if languages else None,
            'reader_index': selected[0] if languages else None,
            'languages': list(range(len(languages))), 'rows': rows, 'reference_rows': references,
            'glyphs': [glyph for glyph in pointer_texts['glyphs'] if glyph['index'] in used] if languages else []}


def project_metadata_name_references(metadata, pointer_texts):
    """Expose direct property-0 text references without resolving routed rows."""
    catalogs = [] if not pointer_texts else [(index, catalog) for index, catalog in enumerate(pointer_texts['catalogs'])
                                             if catalog['reader'] == 0x80165e8]
    selected = catalogs[0] if len(catalogs) == 1 else None
    languages = [] if selected is None else selected[1]['languages']
    used, rows = set(), []
    for index, record in enumerate(metadata['records']):
        first = record['bytes_00_04']
        name_index = first[0] | first[1] << 8 if record['routing'] == 0 else None
        names = []
        for language in languages:
            if name_index is None or name_index >= len(language['entries']):
                names.append(None)
                continue
            units = pointer_texts['strings'][language['entries'][name_index]]['units']
            names.append(units)
            used.update(unit for unit in units if type(unit) is int)
        rows.append({'id': index, 'routing': record['routing'], 'name_index': name_index, 'names': names})
    return {'path': metadata['path'], 'source_offset': metadata['source_offset'],
            'text_path': pointer_texts['path'] if languages else None,
            'reader_index': selected[0] if languages else None,
            'languages': list(range(len(languages))), 'rows': rows,
            'glyphs': [glyph for glyph in pointer_texts['glyphs'] if glyph['index'] in used] if languages else []}


EUROPE_WORLD_NODES_SHA256 = '4336cfcc8b4bfb898530729f6ff5f8fd33e475dda17db8de643ee0e6ee3e1fdc'
EUROPE_TEXT_CATALOG_SHA256 = '514af42870d44e5f772e26d9cd246db4cde2d8ab8ecc9f3d065f67510db05c33'
EUROPE_ITEMS_SHA256 = 'c151e82d6fac0c171414451623142c77aaf29f6a66ccb25495d652a37324ffc7'
EUROPE_ITEM_PARAMETERS_SHA256 = '24d762eb015b11982d5c3d9e749a120ae3807ef4cafbf369e0953c27420d0dc9'
EUROPE_EVALUATION_PARAMETERS_SHA256 = 'a327d9f5d294c1906ef0ecf27b71e0a11487538b815f8bda528a8edf47ae772e'
EUROPE_ACTION_RECORDS_SHA256 = '590ba29559b1f5a44212f3ac9e09eab723c14dcc32c5a8e4642097ec64b60966'
EUROPE_LINKED_RECORDS4_SHA256 = 'dc06db432d88980a7d5733b8dd3f22c722c0a09b58faae52de577388d9919aa3'
EUROPE_CONDITIONAL_RECORDS8_SHA256 = 'ed324120046a3e854da483d311c23822cdb68aa860449ca986887fa99836c254'
EUROPE_METADATA12_SHA256 = '732081be5a06ad089e16358eef40e9ab126ab239e28c0c07830179b47bc969ba'
EUROPE_GLOBAL_PAIR_FLAGS_SHA256 = '618dd9c1e358597d76aa90feaec7d9e86a6006340f278096e9504114855685ee'
EUROPE_ITEM_LINKS_SHA256 = 'c6817d93fe8e808c13487037c9f780448011091537c9b95e06baf2053af6ec6f'
EUROPE_METADATA52_SHA256 = '6ac18b39995e6677bf5d80e830ab17d7b442d36171cfb90a45b6ea8a231d2fbd'
EUROPE_METADATA2_SHA256 = 'ff95d762868b257f84da4d37b6c7ed36515608c1d999af814c6f66693bf412ba'
EUROPE_METADATA4_SHA256 = '172f858a68236d78adca393004e431230f19a88f9a20d4bce8cb1172e82bb594'
EUROPE_METADATA14_SHA256 = 'b9f66ba25c3fb1308b92372c251ace8180ad5594cef59e9d6f295b987f72fc0e'
EUROPE_ENCOUNTER_UNITS_SHA256 = '037cd9f9783fc98b95fb9b69d05fb1e09bf2f3888072c7bd17f9ef6e42153788'
EUROPE_COLOR_MODE_SHA256 = '98068136afc44d24640c2c27027ccc34954aa0d92a01a603db767b07cfaab382'
EUROPE_OBJ_PROPERTY_WORDS_SHA256 = 'be429aaf9073f5dc811a0272d8d76da35badbc8d6b40ff08f727c2f276ca7bb7'
EUROPE_EQUIPMENT_SPRITES_SHA256 = '23391d28a69a1bb728353d267a6080e47f6a741846c0c72ad7b631d887fb701c'
EUROPE_BATTLE_SPRITE_SHA256 = {
    'normal': '33d0c76b2a254cb565733c53317950531bf9b8fc0604eb6f1905d98e5c019d53',
    'alternate': 'cad7771dbddf3d92b93c6a9df6a62fa430b775b9e7e2932253207d71dff35580',
}


def project_special_unit_palettes(profile, digest, palettes):
    """Expose only the three verified, complete special-unit color fragments."""
    if profile != 'europe' or digest != EUROPE_COLOR_MODE_SHA256 or not palettes:
        return None
    roots = {0: 0x40fa90, 1: 0x4110f0, 2: 0x410610}
    spans = [span for span in palettes['spans'] if span['book'] == 'special-unit']
    if (len(spans) != 3 or {span['variant'] for span in spans} != set(roots)
            or any(span['source_offset'] != roots[span['variant']] or len(span['colors']) != 144 for span in spans)):
        return None
    banks = []
    for span in sorted(spans, key=lambda item: item['variant']):
        for index in range(9):
            banks.append({'variant': span['variant'], 'index': index, 'source_offset': span['source_offset'] + index * 32,
                          'colors': span['colors'][index * 16:index * 16 + 16]})
    return {'path': palettes['path'], 'banks': banks}


def battle_preview_ids(battle_sha, battle):
    """Find sprite IDs with an extracted graphics frame in an authenticated catalog."""
    sprite_ids = {}
    for mode, expected in EUROPE_BATTLE_SPRITE_SHA256.items():
        catalog = battle.get(mode)
        if battle_sha.get(mode) == expected and catalog:
            def previewable(binding):
                sequence = binding['sequence']
                return sequence is not None and any(frame['operation'] == 1 and frame['block'] is not None and frame['objects'] is not None
                                                    for frame in catalog['sequences'][sequence]['frames'])
            sprite_ids[mode] = sorted({item['sprite_id'] for item in catalog['bindings'] if previewable(item)})
    return sprite_ids


def link_metadata14(profile, digest, metadata_sha, text_sha, battle_sha, special, metadata, names, battle):
    """Join only verified source owners; live unit overrides remain separate."""
    if profile != 'europe' or digest != EUROPE_METADATA14_SHA256 or not special:
        return None
    metadata_verified = bool(metadata_sha == EUROPE_METADATA52_SHA256 and metadata and names
                             and all(item['selectors01_02'][1] < len(metadata['records']) for item in special['records']))
    return {'metadata_verified': metadata_verified,
            'names_verified': bool(metadata_verified and text_sha == EUROPE_TEXT_CATALOG_SHA256
                                   and names['reader_index'] is not None),
            'sprite_ids': battle_preview_ids(battle_sha, battle)}


def project_job_sprite_producers(catalog):
    """Retain bounded job producers, rejecting duplicate or malformed references."""
    producers = catalog['producers']
    if not isinstance(producers, list) or len(producers) > 4096:
        raise ValueError('Invalid actor sprite producers')
    rows, keys = [], set()
    for item in producers:
        if not isinstance(item, dict):
            raise ValueError('Invalid actor sprite producer')
        if item.get('kind') != 'job':
            continue
        mode, identity, source, sprite = (item[key] for key in ('mode', 'logical_id', 'metadata_source', 'sprite_id'))
        if (mode not in ('normal', 'alternate') or type(identity) is not int or not 0 <= identity < 116
                or type(source) is not int or not 0 <= source <= 0xffffffff
                or type(sprite) is not int or not 0 <= sprite < 65535 or (mode, identity) in keys):
            raise ValueError('Invalid job sprite producer')
        keys.add((mode, identity))
        rows.append({'mode': mode, 'logical_id': identity, 'metadata_source': source, 'sprite_id': sprite})
    return rows


def link_job_sprites(profile, metadata_sha, producer_sha, text_sha, palette_sha, battle_sha,
                     metadata, producers, names, battle, unit_palettes):
    """Join direct rows to their authenticated normal and alternate sprite sources."""
    if (profile != 'europe' or metadata_sha != EUROPE_METADATA52_SHA256
            or producer_sha != EUROPE_OBJ_PROPERTY_WORDS_SHA256 or not metadata or producers is None):
        return None
    expected = {}
    for identity, record in enumerate(metadata['records']):
        if record['routing']:
            continue
        raw = record['bytes_06_1f']
        for mode, offset in (('normal', 1), ('alternate', 3)):
            sprite = raw[offset] | raw[offset + 1] << 8
            if sprite != 65535:
                expected[(mode, identity)] = (metadata['source_offset'] + identity * 52 + 6 + offset, sprite)
    actual = {(row['mode'], row['logical_id']): (row['metadata_source'], row['sprite_id']) for row in producers}
    if len(actual) != len(producers) or actual != expected:
        return None
    available = {mode: set(ids) for mode, ids in battle_preview_ids(battle_sha, battle).items()}
    rows = []
    for identity, record in enumerate(metadata['records']):
        raw = record['bytes_06_1f']; entry = {'id': identity, 'routing': record['routing']}
        entry['palette_nibbles'] = None if record['routing'] else [raw[5] & 15, raw[5] >> 4]
        for mode in ('normal', 'alternate'):
            value = expected.get((mode, identity))
            entry[mode] = value[1] if value and value[1] in available.get(mode, set()) else None
        rows.append(entry)
    palettes = bool(palette_sha == EUROPE_COLOR_MODE_SHA256 and unit_palettes and len(unit_palettes) == 48
                    and {(item['variant'], item['bank']) for item in unit_palettes} ==
                    {(variant, bank) for variant in range(3) for bank in range(16)})
    return {'rows': rows, 'names_verified': bool(text_sha == EUROPE_TEXT_CATALOG_SHA256 and names
                                                and names['reader_index'] is not None),
            'palette_preview': palettes}


def link_item_parameters(profile, item_sha, parameters_sha, items, parameters, names):
    """Link the documented stored selector only for authenticated owners."""
    if (profile != 'europe' or item_sha != EUROPE_ITEMS_SHA256
            or parameters_sha != EUROPE_ITEM_PARAMETERS_SHA256 or not items or not parameters):
        return None
    selectors = [record['bytes_18_1f'][0] for record in items['records']]
    if any(index >= len(parameters['records']) for index in selectors):
        return None
    return {'selectors': selectors, 'names_verified': bool(names)}


def link_item_sprites(profile, item_sha, equipment_sha, items, equipment, names):
    """Link documented normal equipment producers only to verified graphics frames."""
    if (profile != 'europe' or item_sha != EUROPE_ITEMS_SHA256 or equipment_sha != EUROPE_EQUIPMENT_SPRITES_SHA256
            or not items or not equipment):
        return None
    available = set()
    for binding in equipment['bindings']:
        sequence = binding['sequence']
        if (binding['animation'] in (0, 1, 4, 5, 8, 9) and binding['part'] in (0, 1)
                and sequence is not None and any(frame['operation'] == 1 and frame['block'] is not None and frame['objects'] is not None
                                                 for frame in equipment['sequences'][sequence]['frames'])):
            available.add(binding['sprite_id'])
    ids = []
    for item_id, item in enumerate(items['records']):
        raw = item['bytes_08_0f']; sprite = raw[6] | raw[7] << 8
        ids.append(sprite if item_id and 1 <= raw[0] <= 19 and sprite in available else None)
    return {'sprite_ids': ids, 'names_verified': bool(names)}


def link_encounter_jobs(profile, encounter_sha, metadata_sha, special_sha, text_sha, units, metadata, special, names):
    """Resolve documented template job selectors only from authenticated owners."""
    if (profile != 'europe' or encounter_sha != EUROPE_ENCOUNTER_UNITS_SHA256
            or metadata_sha != EUROPE_METADATA52_SHA256 or not units or not metadata):
        return None
    descriptors = ({record['id']: record for record in special['records']}
                   if special and special_sha == EUROPE_METADATA14_SHA256 else {})
    jobs = []
    for record in units['records']:
        descriptor, explicit = record['byte00'], record['byte01']
        if explicit:
            job = explicit
        elif descriptor == 1:
            job = 0
        else:
            source = descriptors.get(descriptor)
            job = source['selectors01_02'][1] if source else None
        jobs.append(job if job is not None and job < len(metadata['records']) else None)
    return {'jobs': jobs, 'names_verified': bool(text_sha == EUROPE_TEXT_CATALOG_SHA256
                                                  and names and names['reader_index'] is not None)}


def link_metadata2(profile, pair_sha, metadata_sha, text_sha, pairs, metadata, names):
    """Authenticate stored keys against their independently extracted metadata owner."""
    if (profile != 'europe' or pair_sha != EUROPE_METADATA2_SHA256 or metadata_sha != EUROPE_METADATA52_SHA256
            or not pairs or not metadata or any(row['key'] >= len(metadata['records']) for row in pairs['records'])):
        return None
    return {'names_verified': bool(text_sha == EUROPE_TEXT_CATALOG_SHA256 and names and names['reader_index'] is not None)}


def link_metadata4(profile, pair_sha, metadata_sha, text_sha, pairs, metadata, names):
    """Link raw indices only when both independently extracted owners match."""
    if (profile != 'europe' or pair_sha != EUROPE_METADATA4_SHA256 or metadata_sha != EUROPE_METADATA52_SHA256
            or not pairs or not metadata):
        return None
    records, owners = pairs['records'], [record['bytes_20_33'][16] for record in metadata['records']]
    if any(index >= len(records) for index in owners) or any(
            slot['selector'] >= len(owners) for record in records for slot in record['slots']):
        return None
    return {'metadata_path': metadata['path'], 'owners': owners,
            'names_verified': bool(text_sha == EUROPE_TEXT_CATALOG_SHA256 and names and names['reader_index'] is not None)}


def project_item_names(profile, item_sha, text_sha, items, pointer_texts):
    """Follow the EUR item's property-0 index into its original name reader."""
    if (profile != 'europe' or item_sha != EUROPE_ITEMS_SHA256 or text_sha != EUROPE_TEXT_CATALOG_SHA256
            or not items or not pointer_texts):
        return None
    catalogs = [(index, catalog) for index, catalog in enumerate(pointer_texts['catalogs'])
                if catalog['reader'] == 0x080165e8]
    if len(catalogs) != 1:
        return None
    reader_index, catalog = catalogs[0]
    rows, used = [], set()
    for item_id, record in enumerate(items['records']):
        name_index, names = record['halfwords_00_06'][0], []
        for language in catalog['languages']:
            if name_index >= len(language['entries']):
                names.append(None)
                continue
            units = pointer_texts['strings'][language['entries'][name_index]]['units']
            names.append(units)
            used.update(unit for unit in units if type(unit) is int)
        rows.append({'id': item_id, 'name_index': name_index, 'names': names})
    return {'path': items['path'], 'text_path': pointer_texts['path'], 'reader_index': reader_index,
            'languages': list(range(len(catalog['languages']))), 'rows': rows,
            'glyphs': [glyph for glyph in pointer_texts['glyphs'] if glyph['index'] in used]}


def project_action_names(profile, record_sha, text_sha, records, pointer_texts):
    """Follow the EUR metadata property-0 index into its original text reader."""
    if (profile != 'europe' or record_sha != EUROPE_ACTION_RECORDS_SHA256 or text_sha != EUROPE_TEXT_CATALOG_SHA256
            or not records or not pointer_texts):
        return None
    catalogs = [(index, catalog) for index, catalog in enumerate(pointer_texts['catalogs'])
                if catalog['reader'] == 0x08016588]
    if len(catalogs) != 1:
        return None
    reader_index, catalog = catalogs[0]
    rows, used = [], set()
    for record in records['records']:
        name_index, names = record['field00'], []
        for language in catalog['languages']:
            if name_index >= len(language['entries']):
                names.append(None)
                continue
            units = pointer_texts['strings'][language['entries'][name_index]]['units']
            names.append(units)
            used.update(unit for unit in units if type(unit) is int)
        rows.append({'id': record['index'], 'name_index': name_index, 'names': names})
    return {'path': records['path'], 'text_path': pointer_texts['path'], 'reader_index': reader_index,
            'languages': list(range(len(catalog['languages']))), 'rows': rows,
            'glyphs': [glyph for glyph in pointer_texts['glyphs'] if glyph['index'] in used]}


def link_action_records4(profile, action_sha, records_sha, actions, records, names):
    """Join only the three source-proven inline selectors to physical records."""
    if (profile != 'europe' or action_sha != EUROPE_ACTION_RECORDS_SHA256
            or records_sha != EUROPE_LINKED_RECORDS4_SHA256 or not actions or not records):
        return None
    references = [list(row['inline0c_0f'][:3]) for row in actions['records']]
    if any(value >= len(records['records']) for row in references for value in row):
        return None
    return {'references': references, 'names_verified': bool(names)}


def project_item_metadata_links(profile, links_sha, metadata_sha, text_sha, links, names):
    """Enable the documented EUR selector-to-metadata relation only for known inputs."""
    if (profile != 'europe' or links_sha != EUROPE_ITEM_LINKS_SHA256 or metadata_sha != EUROPE_METADATA52_SHA256
            or not links or not names or text_sha not in (None, EUROPE_TEXT_CATALOG_SHA256)):
        return None
    return {'names_verified': bool(text_sha == EUROPE_TEXT_CATALOG_SHA256 and names['reader_index'] is not None)}


def project_observed_world_names(profile, node_sha, text_sha, world_nodes, pointer_texts):
    """Connect source-backed EUR placement observations to their text entries."""
    if (profile != 'europe' or node_sha != EUROPE_WORLD_NODES_SHA256 or text_sha != EUROPE_TEXT_CATALOG_SHA256
            or not world_nodes or not pointer_texts):
        return None
    catalogs = [(index, catalog) for index, catalog in enumerate(pointer_texts['catalogs'])
                if catalog['reader'] == 0x080165e8]
    if len(catalogs) != 1:
        return None
    reader_index, catalog = catalogs[0]
    # The node positions come from one recorded placement, not a fixed region map.
    observations = ((19, 7, 725, 13, 36), (17, 1, 719, 14, 26), (25, 2, 720, 13, 20),
                    (26, 6, 724, 3, 20), (14, 11, 729, 19, 24), (2, 16, 734, 8, 11))
    locations, used = [], set()
    for node_index, region_index, name_index, x, y in observations:
        if node_index >= len(world_nodes['nodes']):
            continue
        node = world_nodes['nodes'][node_index]
        if (node['index'], node['x_tile'], node['y_tile']) != (node_index, x, y):
            continue
        names = []
        for language in catalog['languages']:
            if len(language['entries']) <= name_index:
                names.append(None)
                continue
            units = pointer_texts['strings'][language['entries'][name_index]]['units']
            names.append(units)
            used.update(unit for unit in units if type(unit) is int)
        if any(name is not None for name in names):
            locations.append({'node_index': node_index, 'region_index': region_index,
                              'name_index': name_index, 'names': names})
    if not locations:
        return None
    return {'path': pointer_texts['path'], 'reader_index': reader_index,
            'languages': list(range(len(catalog['languages']))),
            'glyphs': [glyph for glyph in pointer_texts['glyphs'] if glyph['index'] in used], 'locations': locations}


def project_world_region_names(profile, text_sha, pointer_texts):
    """Project the EUR region list's conditional 718 + region text references."""
    if profile != 'europe' or text_sha != EUROPE_TEXT_CATALOG_SHA256 or not pointer_texts:
        return None
    catalogs = [(index, catalog) for index, catalog in enumerate(pointer_texts['catalogs'])
                if catalog['reader'] == 0x080165e8]
    if len(catalogs) != 1:
        return None
    reader_index, catalog = catalogs[0]
    regions, used = [], set()
    for region_index in range(31):
        name_index, names = 718 + region_index, []
        for language in catalog['languages']:
            if len(language['entries']) <= name_index:
                names.append(None)
                continue
            units = pointer_texts['strings'][language['entries'][name_index]]['units']
            names.append(units)
            used.update(unit for unit in units if type(unit) is int)
        regions.append({'region_index': region_index, 'name_index': name_index, 'names': names})
    return {'path': pointer_texts['path'], 'reader_index': reader_index,
            'languages': list(range(len(catalog['languages']))), 'regions': regions,
            'glyphs': [glyph for glyph in pointer_texts['glyphs'] if glyph['index'] in used]}


def project_bg_descriptors(catalog):
    """Validate the stored descriptor fields without inferring scene semantics."""
    entries = catalog['entries']
    if not isinstance(entries, list) or not 1 <= len(entries) <= 512:
        raise ValueError('Invalid background descriptor count')
    minimum = len(entries) * 88
    source_fields = ('graphics_source', 'paired_planes_source', 'halfword_planes_source',
                     'palette_source', 'auxiliary_patches_source')
    def word(value):
        return type(value) is int and 0 <= value <= 0xffffffff
    def byte(value):
        return type(value) is int and 0 <= value <= 255
    def transfer(value):
        if not isinstance(value, dict):
            raise ValueError('Invalid background transfer')
        control, source, destination = (value[key] for key in ('control_source', 'data_source', 'destination'))
        if not all(word(v) for v in (control, source, destination)):
            raise ValueError('Invalid background transfer word')
        if (control, source, destination) != (0, 0, 0) and not (
                control >= minimum and source >= minimum and 0x06000000 <= destination < 0x06018000
                and destination % 2 == 0):
            raise ValueError('Invalid background transfer link')
        return {key: value[key] for key in ('control_source', 'data_source', 'destination')}
    result = []
    for entry in entries:
        if not isinstance(entry, dict) or any(not word(entry[key]) or entry[key] < minimum for key in source_fields):
            raise ValueError('Invalid background source')
        variant = entry['variant_source']
        if not word(variant) or 0 < variant < minimum:
            raise ValueError('Invalid background variant source')
        selectors, extra = entry['selectors'], entry['extra_transfers']
        if (not isinstance(selectors, list) or len(selectors) != 2 or not all(byte(v) for v in selectors)
                or not byte(entry['alternate_palette_key']) or not byte(entry['trailing_byte'])
                or not isinstance(extra, list) or len(extra) != 4):
            raise ValueError('Invalid background descriptor fields')
        result.append({**{key: entry[key] for key in source_fields}, 'transfer': transfer(entry['transfer']),
                       'extra_transfers': [transfer(value) for value in extra], 'variant_source': variant,
                       'selectors': list(selectors), 'alternate_palette_key': entry['alternate_palette_key'],
                       'trailing_byte': entry['trailing_byte']})
    return {'entries': result}


def project_indexed_graphics(catalog):
    """Expose decoded programs without embedding their packet streams on every page."""
    def uint(value, maximum):
        return type(value) is int and 0 <= value <= maximum

    if not isinstance(catalog, dict):
        raise ValueError('Invalid indexed graphics catalog')
    groups = [('base', 'Localized groups', catalog['archives']),
              ('color', 'Color-mode groups', catalog.get('color_archives'))]
    for key, label in (('white_flowers', 'White Flowers language group'),
                       ('additional', 'Additional language groups'),
                       ('reader_banks', 'Indexed reader banks')):
        value = catalog.get(key)
        if value is not None and not isinstance(value, dict):
            raise ValueError('Invalid indexed graphics family')
        groups.append((key, label, value.get('archives') if value is not None else None))
    direct = catalog.get('white_flowers_direct')
    if direct is not None and not isinstance(direct, dict):
        raise ValueError('Invalid direct indexed graphics family')
    if direct is not None:
        groups.append(('white_flowers_direct', 'White Flowers direct archive', [direct['archive']]))
    families, total_words, total_programs = [], 0, 0
    for key, label, archives in groups:
        if archives is None:
            if key == 'base':
                raise ValueError('Missing indexed graphics archives')
            continue
        if not isinstance(archives, list) or len(archives) > 256:
            raise ValueError('Invalid indexed graphics archives')
        projected = []
        for archive in archives:
            source, header = archive['source_offset'], archive['opaque_header']
            fixed, offsets, programs = archive['fixed_decoded_bytes'], archive['entry_offsets'], archive['programs']
            dictionary = archive['dictionary_bytes']
            if (not uint(source, 0xffffff) or not isinstance(header, list) or len(header) != 2
                    or any(not uint(value, 255) for value in header)
                    or not uint(archive['dictionary_relative_offset'], 0xffffffff)
                    or archive['dictionary_mode'] != 0 or type(archive['dictionary_mode']) is not int
                    or archive['dictionary_length_bits'] != 5 or type(archive['dictionary_length_bits']) is not int
                    or not uint(fixed, 65535) or fixed == 0 or fixed % 4
                    or not isinstance(dictionary, list) or len(dictionary) > 2048
                    or any(not uint(value, 255) for value in dictionary)
                    or not isinstance(programs, list) or not 1 <= len(programs) <= 65535
                    or not isinstance(offsets, list) or len(offsets) != len(programs)):
                raise ValueError('Invalid indexed graphics archive')
            rows = []
            for index, program in enumerate(programs):
                relative, words, packets = program['relative_offset'], program['decoded_words'], program['packets']
                if (type(program['index']) is not int or program['index'] != index
                        or not uint(relative, 0xffffffff) or type(offsets[index]) is not int or offsets[index] != relative
                        or not isinstance(words, list) or len(words) * 4 != fixed
                        or any(not uint(word, 0xffffffff) for word in words)
                        or not isinstance(packets, list) or len(packets) > 65535
                        or any(not isinstance(packet, dict) or packet.get('kind') not in ('literal', 'dictionary') for packet in packets)):
                    raise ValueError('Invalid indexed graphics program')
                rows.append({'index': index, 'relative_offset': relative, 'words': list(words),
                             'packet_count': len(packets),
                             'dictionary_packets': sum(packet['kind'] == 'dictionary' for packet in packets)})
                total_words += len(words); total_programs += 1
                if total_words > 4000000 or total_programs > 100000:
                    raise ValueError('Indexed graphics catalog is too large')
            projected.append({'source_offset': source, 'opaque_header': list(header),
                              'dictionary_relative_offset': archive['dictionary_relative_offset'],
                              'dictionary_bytes': len(dictionary), 'fixed_decoded_bytes': fixed, 'programs': rows})
        if projected:
            families.append({'id': key, 'label': label, 'archives': projected})
    if not families:
        raise ValueError('No indexed graphics programs')
    return {'families': families}


def project_direct_indexed_graphics(catalog, resource_id):
    """Project each documented direct root as its own independently validated family."""
    specs = {
        'graphics.direct-indexed-archive': (0x023d08, 0x3ae484, 4, 0, 'direct_literal', 'Direct archive · 0x3AE484'),
        'graphics.direct-indexed-dictionary-archive': (0x023d10, 0x3ae5a4, 41, 96, 'direct_dictionary', 'Direct archive · 0x3AE5A4'),
    }
    literal, offset, count, dictionary_length, family_id, label = specs[resource_id]
    archive = catalog['archive']
    if (type(catalog['caller_literal_offset']) is not int or catalog['caller_literal_offset'] != literal
            or type(archive['source_offset']) is not int or archive['source_offset'] != offset
            or type(archive['fixed_decoded_bytes']) is not int or archive['fixed_decoded_bytes'] != 64
            or not isinstance(archive['programs'], list) or len(archive['programs']) != count
            or not isinstance(archive['dictionary_bytes'], list) or len(archive['dictionary_bytes']) != dictionary_length):
        raise ValueError('Invalid direct indexed graphics source')
    family = project_indexed_graphics({'archives': [archive]})['families'][0]
    family.update(id=family_id, label=label, caller_literal_offset=literal, resource_id=resource_id)
    return family


def project_color_mode_palettes(catalog):
    """Keep owned fragments separate; gaps are not full palette rows."""
    if not isinstance(catalog, dict):
        raise ValueError('Invalid color mode catalog')
    spans, offsets = catalog['spans'], catalog['slice_offsets']
    if (not isinstance(spans, list) or not spans or len(spans) > 1024
            or not isinstance(offsets, list) or len(offsets) != 4
            or any(type(value) is not int or not 0 <= value <= 255 for value in offsets)):
        raise ValueError('Invalid color mode fragments or slice offsets')
    books = {'unit-base', 'companion-base', 'supplemental', 'shared-colors', 'special-unit'}
    projected, ids, ranges = [], set(), []
    for span in spans:
        identity, book, variant, offset, colors = (span[key] for key in ('id', 'book', 'variant', 'source_offset', 'colors'))
        if (not isinstance(identity, str) or not identity or len(identity) > 128 or identity in ids
                or not isinstance(book, str) or book not in books or type(variant) is not int or not 0 <= variant <= 2
                or type(offset) is not int or offset < 0 or offset % 2
                or not isinstance(colors, list) or not colors or len(colors) > 65536
                or offset + len(colors) * 2 > 0x02000000
                or any(type(color) is not int or not 0 <= color <= 65535 for color in colors)):
            raise ValueError('Invalid color mode fragment')
        ids.add(identity)
        ranges.append((offset, offset + len(colors) * 2))
        projected.append({'id': identity, 'book': book, 'variant': variant,
                          'source_offset': offset, 'colors': list(colors)})
    ranges.sort()
    if any(left[1] > right[0] for left, right in zip(ranges, ranges[1:])):
        raise ValueError('Overlapping color mode fragments')
    return {'spans': projected, 'slice_offsets': list(offsets)}


WIKI_SCOPES = (
    ('world', 'World and gameplay', ('items', 'item_names', 'item_sprite_links', 'item_parameters', 'item_parameter_links', 'evaluation_parameters', 'conditional_records8', 'conditional_lookups', 'metadata12', 'global_pair_flags', 'item_links', 'item_metadata_links', 'metadata2', 'metadata2_links', 'metadata4', 'metadata4_links', 'metadata14', 'metadata14_links', 'special_unit_palettes', 'item_effects', 'progression_catalogs', 'job_names', 'metadata_names', 'job_sprite_links', 'encounter_job_links', 'action_records', 'action_names', 'linked_records4', 'action_records4_links', 'encounters', 'mission_events', 'world_records',
      )),
    ('world-map', 'World maps and terrain', ('terrain_seeds', 'script_paths', 'world_nodes', 'world_roads',
      'world_location_names', 'world_region_names')),
    ('interface', 'Backgrounds and palettes', ('palettes', 'bg_plane_programs', 'bg_descriptors', 'bg_graphics', 'bg_transfers',
      'alternate_palettes', 'palette_programs', 'blend_programs', 'raster_palettes')),
    ('interface-ui', 'Interface controls and tiles', ('ui_choice_lists', 'ui_context_templates', 'ui_graphics', 'ui_palettes')),
    ('interface-curves', 'Screen update curves', ('screen_update_curves',)),
    ('interface-maps', 'Tilemaps and transfer graphics', ('palettes', 'bg_graphics', 'ui_graphics', 'ui_palettes', 'background_effect_graphics', 'background_screen_banks',
      'obj_graphics', 'unit_palettes', 'observed_tilemaps', 'firaga_compact_cells', 'extra_tilemaps', 'byte_tilemaps',
      'lz_graphics', 'transfer_directory', 'indexed_tilemap_controls', 'firaga_transfer', 'firaga_palette_rows',
      'compact_cell_bank', 'signed_map_deltas')),
    ('objects', 'Object animations', ('obj_animation_banks', 'obj_animation_extra', 'obj_animation_compact', 'obj_graphics',
      'ui_palettes', 'unit_palettes', 'lz_graphics')),
    ('objects-physical', 'Physical compact object animations', ('physical_obj_animations', 'physical_obj_descriptors', 'obj_graphics',
      'ui_palettes', 'unit_palettes', 'lz_graphics')),
    ('actors', 'Actors and equipment', ('dimensions', 'sprite_pixels', 'equipment_sprites', 'item_sprite_links', 'item_names', 'unit_palettes', 'color_mode_palettes')),
    ('battle', 'Battle actors: normal', ('battle_sprites', 'unit_palettes', 'special_unit_palettes')),
    ('battle-alternate', 'Battle actors: alternate', ('battle_sprites', 'unit_palettes', 'special_unit_palettes')),
    ('script-actors', 'Script actors', ('script_sprites', 'script39_poses', 'unit_palettes')),
    ('audio', 'Song sequences and timed sounds', ('audio', 'sequence_directory', 'sequences', 'pcm_pitch', 'timed_sound_sources')),
    ('audio-samples', 'Waveforms and instruments', ('audio', 'sequence_directory', 'pcm_pitch')),
    ('audio-noise', 'CGB noise', ('audio_noise',)),
    ('text', 'Search all text archives', ('short_texts', 'pointer_texts', 'relative_texts', 'text_reader_registry', 'script_texts', 'font', 'name_keyboard')),
    ('text-short', 'Short texts and fonts', ('short_texts', 'font', 'name_keyboard')),
    ('text-pointers', 'Pointer texts', ('pointer_texts',)),
    ('text-relative', 'Relative text archives', ('relative_texts', 'text_reader_registry')),
    ('text-scripts', 'Script text archives', ('script_texts',)),
    ('scripts', 'Event scripts', ('script_archive', 'script_commands', 'script_text_refs',
                                  'auxiliary_script_archive', 'auxiliary_script_commands',
                                  'task_descriptors', 'effect_events')),
    ('other', 'Other resources', ('text_reader_registry',)),
    ('indexed-graphics', 'Indexed graphics', ('indexed_archives', 'palettes', 'ui_palettes', 'unit_palettes')),
    ('transfer-graphics', 'Transfer graphics', ('type3_payloads', 'ui_palettes', 'unit_palettes')),
)


def viewer_section_updates():
    """Load stable section change times; never replace them with generation time."""
    path = Path(__file__).resolve().parent / 'viewer_section_updates.json'
    catalog = json.loads(path.read_text(encoding='utf-8'))
    if catalog.get('schema') != 1 or not isinstance(catalog.get('sections'), dict):
        raise ValueError('Invalid viewer section update catalog')
    scopes = {'index'} | {key for key, _, _ in WIKI_SCOPES}
    result = {}
    for scope, entries in catalog['sections'].items():
        if scope not in scopes or not isinstance(entries, dict):
            raise ValueError('Invalid viewer section update scope')
        result[scope] = {}
        for identity, change in entries.items():
            if (not re.fullmatch(r'[a-z0-9]+(?:-[a-z0-9]+)*', identity)
                    or not isinstance(change, dict) or set(change) != {'kind', 'at'}
                    or change['kind'] not in ('added', 'updated') or not isinstance(change['at'], str)
                    or not re.fullmatch(r'\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z', change['at'])):
                raise ValueError('Invalid viewer section update')
            try:
                datetime.strptime(change['at'], '%Y-%m-%dT%H:%M:%SZ')
            except ValueError as error:
                raise ValueError('Invalid viewer section update timestamp') from error
            result[scope][identity] = dict(change)
    return result


def viewer_resource_scope(resource_id):
    if resource_id == 'graphics.screen-update-tables':
        return 'interface-curves'
    if resource_id == 'graphics.physical-compact-obj-banks':
        return 'objects-physical'
    if resource_id in ('gameplay.world-nodes', 'gameplay.world-roads', 'engine.script-coordinate-paths'):
        return 'world-map'
    if resource_id in ('engine.ui-choice-lists', 'ui.context-templates', 'graphics.ui-custom-banks',
                       'graphics.ui-raw-palettes'):
        return 'interface-ui'
    if resource_id in ('graphics.ui-raw-transfers', 'graphics.dynamic-raw-fragments', 'graphics.tilemap-geometry',
                       'graphics.display-control-tables', 'graphics.compact-primary-alias', 'graphics.observed-lz-block',
                       'graphics.observed-tilemap', 'graphics.observed-tilemap-descriptor', 'graphics.observed-tilemap-row',
                       'graphics.soil-compact-record', 'graphics.soil-remaining-rectangles', 'graphics.firaga-copied-matrices',
                       'graphics.firaga-compact-cells', 'graphics.firaga-transfer-program', 'graphics.firaga-palette-rows',
                       'graphics.tilemap-transfer-directory', 'graphics.indexed-tilemap-controls', 'graphics.tilemap-stream-bank',
                       'graphics.tilemap-transfer-lz77', 'graphics.tilemap-type2-transfers', 'graphics.tilemap-direct-transfers',
                       'graphics.compact-cell-bank', 'graphics.background-effect-graphics', 'graphics.background-screen-banks'):
        return 'interface-maps'
    if resource_id == 'graphics.tilemap-type3-transfers':
        return 'transfer-graphics'
    if resource_id in ('graphics.indexed-archives', 'graphics.direct-indexed-archive',
                       'graphics.direct-indexed-dictionary-archive'):
        return 'indexed-graphics'
    if resource_id in ('engine.script-archive', 'engine.script-command-directory',
                       'engine.auxiliary-script-archive', 'engine.auxiliary-script-command-directory',
                       'engine.task-descriptors', 'engine.effect-event-directories'):
        return 'scripts'
    if resource_id in ('graphics.script39-registry-poses', 'graphics.script39-special-poses'):
        return 'script-actors'
    if resource_id in ('audio.m4a-tones', 'audio.physical-tone-banks'):
        return 'audio-samples'
    if resource_id.startswith('audio.'):
        return 'audio'
    if resource_id in ('text.short-strings', 'font.complete-bank', 'text.name-keyboard'):
        return 'text-short'
    if resource_id == 'text.catalogs':
        return 'text-pointers'
    if resource_id == 'text.script-archives':
        return 'text-scripts'
    if resource_id.startswith(('text.', 'font.')):
        return 'text'
    if resource_id.startswith('graphics.unit-obj-script-'):
        return 'script-actors'
    if resource_id == 'graphics.unit-obj-alternate-battle':
        return 'battle-alternate'
    if resource_id == 'graphics.unit-obj-normal-battle':
        return 'battle'
    if (resource_id.startswith('graphics.unit-obj-')
            or resource_id in ('graphics.obj-dimensions', 'graphics.color-mode-palettes')):
        return 'actors'
    if (resource_id in FIRAGA_OBJ_SPECS or resource_id in ('graphics.constructor-obj-animation', 'graphics.localized-obj-animation-banks')
            or resource_id.startswith(('graphics.obj-', 'graphics.conditional-compact-obj-animations'))
            or resource_id in ('graphics.word-task-obj-animations', 'graphics.layered-obj-animations',
                               'graphics.conditional-compact-obj-catalog')):
        return 'objects'
    if resource_id.startswith('graphics.') or resource_id in ('engine.ui-choice-lists', 'ui.context-templates'):
        return 'interface'
    if resource_id.startswith('gameplay.') or resource_id == 'engine.script-coordinate-paths':
        return 'world'
    return 'other'


# The menu is rendered on every offline page, so it needs the available view
# names without loading every topic's projected data. Keep these conditions in
# step with the corresponding section initializers in view_rom_data.html.
VIEWER_SECTION_KEYS = {
    'dimensions': 'dimensions', 'indexed-graphics': 'indexed_archives', 'color-books': 'color_mode_palettes',
    'bg-descriptors': 'bg_descriptors', 'palette': 'palettes', 'alternate-palettes': 'alternate_palettes',
    'bg-tiles': 'bg_graphics', 'bg-planes': 'bg_plane_programs', 'bg-transfers': 'bg_transfers',
    'palette-programs': 'palette_programs', 'blend-programs': 'blend_programs', 'raster-palettes': 'raster_palettes',
    'terrain-seeds': 'terrain_seeds', 'script-paths': 'script_paths', 'ui-choices': 'ui_choice_lists',
    'ui-context-templates': 'ui_context_templates', 'transfer-directory': 'transfer_directory',
    'indexed-tilemap-controls': 'indexed_tilemap_controls',
    'background-effects': 'background_effect_graphics', 'background-screens': 'background_screen_banks',
    'screen-update-curves': 'screen_update_curves',
    'ui-tiles': 'ui_graphics', 'firaga-transfers': 'firaga_transfer', 'firaga-palette-rows': 'firaga_palette_rows',
    'signed-map-deltas': 'signed_map_deltas', 'items': 'items', 'item-effects': 'item_effects',
    'item-parameters': 'item_parameters', 'evaluation-parameters': 'evaluation_parameters',
    'conditional-records8': 'conditional_records8', 'conditional-lookups': 'conditional_lookups', 'metadata12': 'metadata12',
    'physical-compact-descriptors': 'physical_obj_descriptors',
    'global-pair-flags': 'global_pair_flags',
    'job-names': 'job_names', 'metadata-names': 'metadata_names', 'action-records': 'action_records',
    'linked-records4': 'linked_records4',
    'metadata2': 'metadata2', 'metadata4': 'metadata4', 'metadata14': 'metadata14',
    'item-links': 'item_links', 'world-records': 'world_records', 'world': 'world_nodes',
    'world-regions': 'world_region_names', 'audio': 'audio', 'cgb-noise': 'audio_noise',
    'timed-sounds': 'timed_sound_sources', 'texts': 'short_texts', 'pointer-texts': 'pointer_texts',
    'text-reader-registry': 'text_reader_registry', 'relative-texts': 'relative_texts',
    'event-scripts': 'script_archive', 'script-commands': 'script_commands',
    'auxiliary-scripts': 'auxiliary_script_archive', 'auxiliary-commands': 'auxiliary_script_commands',
    'effect-events': 'effect_events', 'task-descriptors': 'task_descriptors', 'script-texts': 'script_texts',
    'font-atlas': 'font', 'name-keyboard': 'name_keyboard', 'sprites': 'sprite_pixels',
}


def viewer_available_sections(payload, titles):
    """Describe visible sections without putting their content in the navbar."""
    present = lambda value: value is not None and value is not False
    available = {section for section, key in VIEWER_SECTION_KEYS.items() if present(payload.get(key))}
    for section, key in (('progression-records', 'progression_catalogs'), ('mission-records', 'mission_events'),
                         ('encounters', 'encounters')):
        if payload.get(key):
            available.add(section)
    if (payload.get('text_search_sources') or
            any(present(payload.get(key)) for key in ('short_texts', 'pointer_texts', 'relative_texts', 'script_texts'))):
        available.add('text-search')
    if present(payload.get('equipment_sprites')) or any(payload.get(key) for key in ('battle_sprites', 'script_sprites', 'script39_poses')):
        available.add('equipment-sprites')
    extra = payload.get('obj_animation_extra') or {}
    compact = payload.get('obj_animation_compact') or {}
    extra_keys = ('word_task', 'layered', 'static_lists', 'soil_evidence', 'constructor_phases', 'localized',
                  'firaga_indexed', 'firaga_compact', 'firaga_adjacent', 'firaga_6c12c8')
    if any(present(value) for value in [payload.get('obj_animation_banks'), payload.get('physical_obj_animations'),
                                      *(extra.get(key) for key in extra_keys), *compact.values()]):
        available.add('obj-animations')
    if (compact.get('compact_catalog') or {}).get('auxiliary') is not None:
        available.add('compact-catalog-tables')
    audio = payload.get('audio') or {}
    if (audio.get('instruments') or {}).get('banks'):
        available.add('instruments')
    if present(payload.get('sequences')) and present(payload.get('sequence_directory')):
        available.add('sequence-audio')
    transfer = payload.get('type3_payloads') or {}
    if present(payload.get('obj_graphics')) or present(payload.get('lz_graphics')) or transfer.get('streams') or transfer.get('raw_streams'):
        available.add('obj-tiles')
    if present(payload.get('observed_tilemaps')) or present(payload.get('firaga_compact_cells')) or payload.get('extra_tilemaps'):
        available.add('tilemaps')
    if (payload.get('compact_cell_bank') or {}).get('records'):
        available.add('compact-cell-bank')
    if (payload.get('byte_tilemaps') or {}).get('maps'):
        available.add('byte-tilemaps')
    missing = available - titles.keys()
    if missing:
        raise ValueError('Missing viewer section titles: ' + ', '.join(sorted(missing)))
    sections = [{'id': section, 'label': titles[section]} for section in sorted(available)]
    for section in sections:
        if section['id'] == 'bg-tiles':
            section['count'] = len(payload['bg_graphics']['descriptor_graphics'])
        elif section['id'] == 'terrain-seeds':
            section['count'] = len(payload['terrain_seeds']['maps'])
    return sections


def split_viewer_template(template):
    """Keep the accepted file inspector while its large-JSON prototype is paused."""
    scripts = Path(__file__).resolve().parent
    def replace_region(text, start, end, replacement):
        if text.count(start) != 1:
            raise ValueError('Expected one file inspector template region')
        left = text.index(start)
        right = text.index(end, left)
        return text[:left] + replacement + text[right:]
    template = replace_region(template, '<section id="file-inspector">', '<section id="dimensions"',
                              (scripts / 'viewer_stable_inspector.html').read_text(encoding='utf-8'))
    template = replace_region(template, "{\n  const fileInput = document.getElementById('inspect-file');",
                              '\nif (data.dimensions) {',
                              (scripts / 'viewer_stable_inspector.js').read_text(encoding='utf-8'))
    for marker, name in (('/* WIKI NAVIGATION */', 'viewer_navigation.js'),
                         ('/* WIKI NAVIGATION CSS */', 'viewer_navigation.css')):
        if template.count(marker) != 1:
            raise ValueError('Expected one wiki navigation template marker: ' + marker)
        template = template.replace(marker, (scripts / name).read_text(encoding='utf-8'))
    return template


def index_viewer_template():
    """Assemble the small overview without embedding unrelated topic code."""
    scripts = Path(__file__).resolve().parent
    template = (scripts / 'viewer_index.html').read_text(encoding='utf-8')
    for marker, name in (('<!-- FILE INSPECTOR -->', 'viewer_stable_inspector.html'),
                         ('/* FILE INSPECTOR */', 'viewer_stable_inspector.js'),
                         ('/* WIKI NAVIGATION */', 'viewer_navigation.js'),
                         ('/* WIKI NAVIGATION CSS */', 'viewer_navigation.css'),
                         ('/* INDEX SCRIPT */', 'viewer_index.js'),
                         ('/* NAME SEARCH */', 'viewer_name_search.js')):
        if template.count(marker) != 1:
            raise ValueError('Expected one index template marker: ' + marker)
        template = template.replace(marker, (scripts / name).read_text(encoding='utf-8'))
    return template


RESOURCE_PAGE_LIMIT = 8 * 1024 * 1024


def readable_wiki_name(units, glyphs):
    """Make searchable text from recognized European glyphs without executing commands."""
    def character(index):
        if 166 <= index <= 175:
            return '0123456789'[index - 166]
        if 176 <= index <= 201:
            return 'ABCDEFGHIJKLMNOPQRSTUVWXYZ'[index - 176]
        if 202 <= index <= 227:
            return 'abcdefghijklmnopqrstuvwxyz'[index - 202]
        if 2 <= index <= 22:
            return 'ÀÁÂÄÈÉÊËÌÍÎÏÑÒÓÔÖÙÚÛÜ'[index - 2]
        if 24 <= index <= 44:
            return 'àáâäèéêëìíîïñòóôöùúûü'[index - 24]
        return {0: 'Ç', 1: '¿', 23: 'ß', 228: '.', 234: '?', 235: '!', 236: ',', 238: ':',
                239: '_', 241: '/', 242: '~', 243: "'", 244: "'", 245: '"', 246: '"',
                247: '(', 248: ')', 249: '{', 250: '}', 251: '[', 252: ']', 253: '+',
                254: '-', 257: '=', 258: '<', 259: '>'}.get(index)
    text, pending_space, recognized = '', False, False
    for unit in units:
        if type(unit) is not int:
            pending_space = bool(text)
            continue
        if pending_space and text:
            text += ' '
        value = character(unit) if unit in glyphs else None
        recognized = recognized or value is not None
        text += value if value is not None else '�'
        pending_space = False
    return text if recognized else None


def project_wiki_name_index(data):
    """Index only localized references already guarded by their source projections."""
    if data.get('profile') != 'europe':
        return []
    verified_jobs = any((data.get(key) or {}).get('names_verified') for key in
                        ('job_sprite_links', 'item_metadata_links', 'metadata2_links',
                         'metadata4_links', 'metadata14_links', 'encounter_job_links'))
    sources = [
        ('item', data.get('item_names') if data.get('items') else None, 'rows', 'id'),
        ('action', data.get('action_names') if data.get('action_records') else None, 'rows', 'id'),
        ('job', data.get('metadata_names') if verified_jobs else None, 'rows', 'id'),
        ('region', data.get('world_region_names'), 'regions', 'region_index'),
    ]
    entries = []
    for kind, source, rows_key, id_key in sources:
        if not source:
            continue
        glyphs = {glyph['index'] for glyph in source['glyphs']}
        for row in source[rows_key]:
            if kind == 'job' and row.get('routing'):
                continue
            names = {}
            for option, (language, units) in enumerate(zip(source['languages'], row['names'])):
                text = readable_wiki_name(units, glyphs) if units else None
                if text:
                    names[str(language)] = {'text': text, 'option': option}
            if names:
                entries.append({'kind': kind, 'id': row[id_key], 'names': names})
    return entries


def write_split_viewer(data, output, filename, template, resource_files=()):
    """Emit independent offline pages, embedding only each scope's data."""
    base_keys = ('profile', 'extractor', 'decoder')
    known = set(base_keys) | {'resources', 'section_updates', 'compact_witnessed_links'} | {key for _, _, keys in WIKI_SCOPES for key in keys}
    if set(data) - known:
        raise ValueError('Unassigned viewer data: ' + ', '.join(sorted(set(data) - known)))
    compact_links = project_witnessed_links((data.get('obj_animation_compact') or {}).get('compact_catalog'),
                                             data.get('physical_obj_animations'))
    titles = dict(re.findall(r'<section id="([^"]+)"[^>]*>\s*<details(?:\s[^>]*)?><summary><h2>([^<]+)</h2>', template))
    # Large source inventories can occur on several pages. Keep their manifest
    # details available offline without duplicating them in each page payload.
    manifest_assets, manifest_views = {}, {}
    for row in data['resources']:
        sources = row.get('sources')
        if not isinstance(sources, list) or 'source_count' in row or 'sources_asset' in row:
            continue
        encoded = json.dumps(sources, ensure_ascii=True, separators=(',', ':')).replace('<', '\\u003c')
        if len(encoded.encode('utf-8')) <= 65536:
            continue
        fingerprint = hashlib.sha256(encoded.encode('utf-8')).hexdigest()
        href = 'manifest-sources-' + fingerprint + '.js'
        manifest_assets[href] = ('globalThis.__FFTA_MANIFEST_SOURCES__ = globalThis.__FFTA_MANIFEST_SOURCES__ || {};\n'
                                 'globalThis.__FFTA_MANIFEST_SOURCES__[' + json.dumps(fingerprint) + '] = '
                                 '{"key":"' + fingerprint + '","sources":' + encoded + '};\n')
        manifest_views[id(row)] = {**{key: value for key, value in row.items() if key != 'sources'},
                                   'source_count': len(sources),
                                   'sources_asset': {'href': href, 'key': fingerprint}}
    # A song or instrument needs only the PCM waves it actually uses. Keep
    # sample arrays out of the page and give each a shared content identity.
    audio_assets, audio_view = {}, data.get('audio')
    if audio_view:
        waves = []
        for wave in audio_view['waves']:
            samples = wave['samples']
            encoded = json.dumps(samples, separators=(',', ':'))
            fingerprint = hashlib.sha256(encoded.encode('utf-8')).hexdigest()
            href = 'audio-pcm-' + fingerprint + '.js'
            content = ('globalThis.__FFTA_PCM_WAVES__ = globalThis.__FFTA_PCM_WAVES__ || {};\n'
                       'globalThis.__FFTA_PCM_WAVES__[' + json.dumps(fingerprint) + '] = '
                       '{"key":"' + fingerprint + '","samples":' + encoded + '};\n')
            audio_assets[href] = content
            waves.append({**{key: value for key, value in wave.items() if key != 'samples'},
                          'sample_count': len(samples), 'samples_asset': {'href': href, 'key': fingerprint}})
        audio_view = {**audio_view, 'waves': waves}
    # Sprite graphs remain browsable immediately; their large tile arrays are
    # grouped into bounded, shared assets for the frames being inspected.
    sprite_assets, sprite_views = {}, {}
    for family in ('battle_sprites', 'script_sprites'):
        catalogs = {}
        for name, catalog in (data.get(family) or {}).items():
            blocks = []
            for start in range(0, len(catalog['blocks']), 64):
                chunk = catalog['blocks'][start:start + 64]
                encoded = json.dumps([block['tiles'] for block in chunk], separators=(',', ':'))
                fingerprint = hashlib.sha256(encoded.encode('utf-8')).hexdigest()
                href = 'sprite-tiles-' + fingerprint + '.js'
                content = ('globalThis.__FFTA_SPRITE_TILES__ = globalThis.__FFTA_SPRITE_TILES__ || {};\n'
                           'globalThis.__FFTA_SPRITE_TILES__[' + json.dumps(fingerprint) + '] = '
                           '{"key":"' + fingerprint + '","blocks":' + encoded + '};\n')
                sprite_assets[href] = content
                for index, block in enumerate(chunk):
                    blocks.append({**{key: value for key, value in block.items() if key != 'tiles'},
                                   'tile_count': len(block['tiles']),
                                   'tiles_asset': {'href': href, 'key': fingerprint, 'index': index}})
            catalogs[name] = {**catalog, 'blocks': blocks}
        sprite_views[family] = catalogs
    text_assets, text_sources = [], {}
    for name, key in (('short_texts', 'short'), ('pointer_texts', 'pointer'),
                      ('relative_texts', 'relative'), ('script_texts', 'script')):
        value = data.get(name)
        if value is None:
            continue
        if key == 'short':
            languages = {str(item['language_id']) for item in value['languages']}
        elif key == 'script':
            languages = {str(item['id']) for item in value['languages']}
        else:
            languages = {str(index) for catalog in value['catalogs'] for index, _ in enumerate(catalog['languages'])}
        encoded = json.dumps(value, ensure_ascii=True, separators=(',', ':')).replace('<', '\\u003c')
        fingerprint = hashlib.sha256(encoded.encode('utf-8')).hexdigest()
        href = 'text-search-' + key + '-' + fingerprint + '.js'
        content = ('globalThis.__FFTA_TEXT_SEARCH__ = globalThis.__FFTA_TEXT_SEARCH__ || {};\n'
                   'globalThis.__FFTA_TEXT_SEARCH__[' + json.dumps(key) + '] = {"key":"' + fingerprint
                   + '","payload":' + encoded + '};\n')
        text_assets.append({'href': href, 'content': content})
        text_sources[key] = {'href': href, 'key': fingerprint, 'languages': sorted(languages)}

    def scope_payload(key, keys):
        if key == 'text':
            return {'text_search_sources': text_sources}
        payload = {name: data[name] for name in keys}
        if key in ('objects', 'objects-physical') and compact_links:
            payload['compact_witnessed_links'] = compact_links
        if 'audio' in payload:
            payload['audio'] = audio_view
        if key in ('battle', 'battle-alternate'):
            mode = 'normal' if key == 'battle' else 'alternate'
            payload['battle_sprites'] = {mode: sprite_views['battle_sprites'][mode]} if mode in sprite_views['battle_sprites'] else {}
        if 'script_sprites' in payload:
            payload['script_sprites'] = sprite_views['script_sprites']
        if key in ('battle', 'battle-alternate') and data.get('job_sprite_links'):
            # Keep the name index inside this reference panel: exposing it as a
            # top-level section would duplicate the world page on this scope.
            payload['battle_job_references'] = {'links': data['job_sprite_links'],
                                                'names': data.get('metadata_names')}
        if key == 'world-map':
            encounters = data.get('encounters') or {}
            payload['terrain_encounters'] = {name: encounters[name] for name in ('headers', 'coordinates') if name in encounters}
        return payload

    optional_scopes = {'objects-physical': ('physical_obj_animations', 'graphics.physical-compact-obj-banks'),
                       'interface-curves': ('screen_update_curves', 'graphics.screen-update-tables')}
    active_scopes = [scope for scope in WIKI_SCOPES if scope[0] not in optional_scopes
                     or data.get(optional_scopes[scope[0]][0])
                     or any(row['id'] == optional_scopes[scope[0]][1] for row in data['resources'])]
    updates = data.get('section_updates') or {}
    scope_sections = {key: viewer_available_sections(scope_payload(key, keys), titles)
                      for key, _, keys in active_scopes}
    sample_sections = {section['id'] for section in scope_sections['audio-samples']}
    scope_sections['audio'] = [section for section in scope_sections['audio'] if section['id'] not in sample_sections]
    for key, sections in scope_sections.items():
        for section in sections:
            change = updates.get(key, {}).get(section['id'])
            if change:
                section['change'] = change
    template = split_viewer_template(template)
    index_template = index_viewer_template()
    parts_name = filename + '.parts'
    parts = output / parts_name
    marker = parts / '.ffta-viewer-pages.json'
    if parts.is_symlink() or (parts.exists() and (not parts.is_dir() or not marker.is_file())):
        raise ValueError('Existing viewer pages directory is not owned by this generator')
    if marker.exists():
        try:
            owned = json.loads(marker.read_text(encoding='utf-8'))
        except (OSError, ValueError) as error:
            raise ValueError('Invalid viewer pages ownership marker') from error
        if owned != {'kind': 'ffta-viewer-pages', 'schema': 1, 'index': filename}:
            raise ValueError('Viewer pages ownership marker does not match output')
    for href, content in manifest_assets.items():
        existing = parts / href
        if existing.is_symlink() or (existing.exists() and
                (not existing.is_file() or existing.read_text(encoding='utf-8') != content)):
            raise ValueError('Existing manifest source asset does not match its content identity')
    for href, content in audio_assets.items():
        existing = parts / href
        if existing.is_symlink() or (existing.exists() and
                (not existing.is_file() or existing.read_text(encoding='utf-8') != content)):
            raise ValueError('Existing PCM wave asset does not match its content identity')
    for href, content in sprite_assets.items():
        existing = parts / href
        if existing.is_symlink() or (existing.exists() and
                (not existing.is_file() or existing.read_text(encoding='utf-8') != content)):
            raise ValueError('Existing sprite tile asset does not match its content identity')
    for asset in text_assets:
        existing = parts / asset['href']
        if existing.is_symlink() or (existing.exists() and
                (not existing.is_file() or existing.read_text(encoding='utf-8') != asset['content'])):
            raise ValueError('Existing text search asset does not match its content identity')

    groups = {key: [] for key, _, _ in active_scopes}
    overview = []
    for row in data['resources']:
        scope = viewer_resource_scope(row['id'])
        groups[scope].append(row)
        if row['id'] in ('text.short-strings', 'font.complete-bank', 'text.name-keyboard',
                         'text.catalogs', 'text.script-archives', 'engine.text-reader-registry'):
            groups['text'].append(row)
        if row['id'] in ('text.catalogs', 'engine.text-reader-registry'):
            groups['text-relative'].append(row)
        if row['id'] == 'graphics.bg-packet-programs':
            groups['world-map'].append(row)
        if row['id'] in ('graphics.bg-main-graphics', 'graphics.bg-base-palettes', 'graphics.ui-custom-banks', 'graphics.ui-raw-palettes'):
            groups['interface-maps'].append(row)
        if row['id'] == 'audio.m4a-tones':
            groups['audio'].append(row)
        if row['id'] in ('audio.m4a-directory', 'audio.m4a-driver-tables'):
            groups['audio-samples'].append(row)
        if row['id'] == 'audio.m4a-driver-tables':
            groups['audio-noise'].append(row)
        overview.append({**{key: row[key] for key in ('id', 'path', 'format', 'description', 'status')},
                         **({'embedded_view': row['embedded_view']} if 'embedded_view' in row else {}),
                         'viewer_scope': scope})

    transfer = data.get('type3_payloads')
    transfer_asset = None
    if transfer and (transfer['streams'] or transfer['raw_streams']):
        encoded = json.dumps(transfer, ensure_ascii=True, separators=(',', ':')).replace('<', '\\u003c')
        fingerprint = hashlib.sha256(encoded.encode('utf-8')).hexdigest()
        transfer_asset = {'href': 'type3-' + fingerprint + '.js', 'key': fingerprint,
                          'content': 'globalThis.__FFTA_TYPE3_PAYLOADS__ = {"key":"' + fingerprint + '","payload":' + encoded + '};\n'}
        existing = parts / transfer_asset['href']
        if existing.is_symlink() or (existing.exists() and
                (not existing.is_file() or existing.read_text(encoding='utf-8') != transfer_asset['content'])):
            raise ValueError('Existing transfer graphics asset does not match its content identity')

    name_entries = project_wiki_name_index(data)
    name_asset = None
    if name_entries:
        encoded = json.dumps(name_entries, ensure_ascii=True, separators=(',', ':')).replace('<', '\\u003c')
        fingerprint = hashlib.sha256(encoded.encode('utf-8')).hexdigest()
        name_asset = {'href': 'wiki-names-' + fingerprint + '.js', 'key': fingerprint,
                      'content': 'globalThis.__FFTA_WIKI_NAMES__ = {"key":"' + fingerprint + '","entries":' + encoded + '};\n'}
        existing = parts / name_asset['href']
        if existing.is_symlink() or (existing.exists() and
                (not existing.is_file() or existing.read_text(encoding='utf-8') != name_asset['content'])):
            raise ValueError('Existing wiki name index does not match its content identity')

    def pages(current):
        page_list = [{'id': 'index', 'label': 'All resources',
                      'href': quote(filename, safe='') if current == 'index' else '../' + quote(filename, safe='')}]
        for key, label, _ in active_scopes:
            href = quote(parts_name, safe='') + '/' + key + '.html' if current == 'index' else key + '.html'
            page_list.append({'id': key, 'label': label, 'href': href, 'sections': scope_sections[key]})
        directory = (quote(parts_name, safe='') + '/files/' if current == 'index' else 'files/')
        result = {'current': current, 'pages': page_list, 'resource_directory': directory}
        if current in ('interface-maps', 'objects') and transfer_asset:
            result['transfer_graphics'] = {'href': transfer_asset['href'], 'key': transfer_asset['key']}
        if name_asset:
            href = (quote(parts_name, safe='') + '/' if current == 'index' else '') + name_asset['href']
            result['name_search'] = {'href': href, 'key': name_asset['key']}
        return result

    def render(payload, page_template=template):
        encoded = json.dumps(payload, ensure_ascii=True).replace('<', '\\u003c')
        return page_template.replace('/* DATA */', encoded)

    output.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix='.viewer-pages-', dir=output) as temporary:
        stage = Path(temporary)
        staged_parts = stage / parts_name
        staged_parts.mkdir()
        for href, content in manifest_assets.items():
            (staged_parts / href).write_text(content, encoding='utf-8')
        for href, content in audio_assets.items():
            (staged_parts / href).write_text(content, encoding='utf-8')
        for href, content in sprite_assets.items():
            (staged_parts / href).write_text(content, encoding='utf-8')
        if transfer_asset:
            (staged_parts / transfer_asset['href']).write_text(transfer_asset['content'], encoding='utf-8')
        if name_asset:
            (staged_parts / name_asset['href']).write_text(name_asset['content'], encoding='utf-8')
        for asset in text_assets:
            (staged_parts / asset['href']).write_text(asset['content'], encoding='utf-8')
        if resource_files:
            resource_dir = parts / 'files'
            resource_marker = resource_dir / '.ffta-viewer-resources.json'
            if resource_dir.is_symlink() or (resource_dir.exists() and
                    (not resource_dir.is_dir() or resource_marker.is_symlink() or not resource_marker.is_file())):
                raise ValueError('Existing resource pages directory is not owned by this generator')
            owned_files = {}
            if resource_marker.exists():
                try:
                    ownership = json.loads(resource_marker.read_text(encoding='utf-8'))
                    if ownership.get('kind') != 'ffta-viewer-resources' or ownership.get('schema') != 1:
                        raise ValueError('Invalid resource pages ownership marker')
                    owned_files = ownership['files']
                    if not isinstance(owned_files, dict) or any(
                        not re.fullmatch(r'[0-9a-f]{64}\.html', name)
                        or not isinstance(digest, str) or not re.fullmatch(r'[0-9a-f]{64}', digest)
                        for name, digest in owned_files.items()
                    ):
                        raise ValueError('Invalid resource pages ownership marker')
                except (OSError, ValueError, KeyError, TypeError, AttributeError) as error:
                    raise ValueError('Invalid resource pages ownership marker') from error
            resource_template = (Path(__file__).resolve().parent / 'viewer_resource.html').read_text(encoding='utf-8')
            resource_template = resource_template.replace('<!-- FILE INSPECTOR -->',
                (Path(__file__).resolve().parent / 'viewer_stable_inspector.html').read_text(encoding='utf-8'))
            resource_template = resource_template.replace('/* FILE INSPECTOR */',
                (Path(__file__).resolve().parent / 'viewer_stable_inspector.js').read_text(encoding='utf-8'))
            staged_files = staged_parts / 'files'
            staged_files.mkdir()
            new_files = {}
            for row, path in resource_files:
                content = path.read_bytes()
                if len(content) > RESOURCE_PAGE_LIMIT or hashlib.sha256(content).hexdigest() != row['sha256']:
                    raise ValueError('Resource changed while generating its page: ' + row['path'])
                name = row['embedded_view']
                existing = resource_dir / name
                if existing.is_symlink() or (existing.exists() and
                        (name not in owned_files or not existing.is_file()
                         or hashlib.sha256(existing.read_bytes()).hexdigest() != owned_files[name])):
                    raise ValueError('Existing resource page is not an unchanged generated file')
                resource = {'id': row['id'], 'path': row['path'], 'sha256': row['sha256'],
                            'name': Path(row['path']).name, 'content': base64.b64encode(content).decode('ascii'),
                            'scope': '../' + viewer_resource_scope(row['id']) + '.html',
                            'index': '../../' + quote(filename, safe='')}
                encoded = json.dumps(resource, ensure_ascii=True).replace('<', '\\u003c')
                page = resource_template.replace('/* RESOURCE */', encoded).encode('utf-8')
                (staged_files / name).write_bytes(page)
                new_files[name] = hashlib.sha256(page).hexdigest()
            owned_files.update(new_files)
            (staged_files / '.ffta-viewer-resources.json').write_text(
                json.dumps({'kind': 'ffta-viewer-resources', 'schema': 1, 'files': owned_files}, sort_keys=True) + '\n',
                encoding='utf-8')
        base = {key: data[key] for key in base_keys}
        overview_by_id = {row['id']: row for row in overview}
        for key, _, keys in active_scopes:
            # Full source-range details remain on the focused resource pages;
            # the combined search only needs the small index entries.
            resources = ([overview_by_id[row['id']] for row in groups[key]] if key == 'text'
                         else [manifest_views.get(id(row), row) for row in groups[key]])
            payload = {**base, 'resources': resources, **scope_payload(key, keys), 'wiki': pages(key)}
            if updates.get(key):
                payload['section_updates'] = updates[key]
            (staged_parts / (key + '.html')).write_text(render(payload), encoding='utf-8')
        (staged_parts / '.ffta-viewer-pages.json').write_text(
            json.dumps({'kind': 'ffta-viewer-pages', 'schema': 1, 'index': filename}) + '\n', encoding='utf-8')
        (staged_parts / '.gitignore').write_text('*\n', encoding='utf-8')
        index_payload = {**base, 'resources': overview, 'wiki': pages('index')}
        if updates.get('index'):
            index_payload['section_updates'] = updates['index']
        (stage / filename).write_text(render(index_payload, index_template), encoding='utf-8')
        parts.mkdir(exist_ok=True)
        if resource_files:
            resource_dir.mkdir(exist_ok=True)
            for path in staged_files.iterdir():
                if path.name != '.ffta-viewer-resources.json':
                    os.replace(path, resource_dir / path.name)
            os.replace(staged_files / '.ffta-viewer-resources.json', resource_marker)
            staged_files.rmdir()
        if transfer_asset:
            path = staged_parts / transfer_asset['href']
            existing = parts / path.name
            if existing.is_symlink() or (existing.exists() and
                    (not existing.is_file() or existing.read_text(encoding='utf-8') != transfer_asset['content'])):
                raise ValueError('Existing transfer graphics asset changed during generation')
            os.replace(path, existing)
        if name_asset:
            path = staged_parts / name_asset['href']
            existing = parts / path.name
            if existing.is_symlink() or (existing.exists() and
                    (not existing.is_file() or existing.read_text(encoding='utf-8') != name_asset['content'])):
                raise ValueError('Existing wiki name index changed during generation')
            os.replace(path, existing)
        for href, content in manifest_assets.items():
            path, existing = staged_parts / href, parts / href
            if existing.is_symlink() or (existing.exists() and
                    (not existing.is_file() or existing.read_text(encoding='utf-8') != content)):
                raise ValueError('Existing manifest source asset changed during generation')
            os.replace(path, existing)
        for asset in text_assets:
            path, existing = staged_parts / asset['href'], parts / asset['href']
            if existing.is_symlink() or (existing.exists() and
                    (not existing.is_file() or existing.read_text(encoding='utf-8') != asset['content'])):
                raise ValueError('Existing text search asset changed during generation')
            os.replace(path, existing)
        for href, content in audio_assets.items():
            path, existing = staged_parts / href, parts / href
            if existing.is_symlink() or (existing.exists() and
                    (not existing.is_file() or existing.read_text(encoding='utf-8') != content)):
                raise ValueError('Existing PCM wave asset changed during generation')
            os.replace(path, existing)
        for href, content in sprite_assets.items():
            path, existing = staged_parts / href, parts / href
            if existing.is_symlink() or (existing.exists() and
                    (not existing.is_file() or existing.read_text(encoding='utf-8') != content)):
                raise ValueError('Existing sprite tile asset changed during generation')
            os.replace(path, existing)
        for path in staged_parts.iterdir():
            os.replace(path, parts / path.name)
        os.replace(stage / filename, output / filename)
    return output / filename


def generate(dataset, output, filename='index.html', *, split=False):
    dataset = dataset.resolve()
    output = output.resolve()
    if output == dataset or dataset in output.parents:
        raise ValueError('Output must be outside the dataset')
    manifest = tomllib.loads((dataset / 'manifest.toml').read_text(encoding='utf-8'))
    rows = []
    palettes = None
    dimensions = None
    items = None
    item_sha = None
    item_links = None
    item_links_sha = None
    item_effects = None
    item_parameters = None
    item_parameters_sha = None
    evaluation_parameters = None
    conditional_records8 = None
    conditional_lookups = {}
    metadata12 = None
    global_pair_flags = None
    progression_catalogs = {}
    slot_jobs = None
    job_metadata = None
    job_metadata_sha = None
    job_sprite_producers = None
    job_sprite_producer_sha = None
    job_sprite_producer_row = None
    metadata2 = None
    metadata2_sha = None
    metadata4 = None
    metadata4_sha = None
    metadata14 = None
    metadata14_sha = None
    action_records = None
    action_record_sha = None
    linked_records4 = None
    linked_records4_sha = None
    slot_job_row = None
    job_metadata_row = None
    world_nodes = None
    world_node_sha = None
    audio = None
    audio_source_catalog = None
    physical_audio_pending = None
    short_texts = None
    sprite_pixels = None
    equipment_sprites = None
    equipment_sprites_sha = None
    battle_sprites = {}
    battle_sprite_sha = {}
    script_sprites = {}
    script39_pending = {}
    script39_poses = {}
    unit_palettes = None
    color_mode_palettes = None
    color_mode_sha = None
    script_texts = None
    script_text_refs = None
    script_archive = None
    script_commands = None
    auxiliary_script_archive = None
    auxiliary_script_commands = None
    task_descriptors = None
    effect_events = None
    pointer_texts = None
    pointer_text_sha = None
    relative_texts = None
    text_reader_registry = None
    font = None
    font_path = None
    name_keyboard = None
    sequence_directory = None
    timed_sound_sources = None
    sequences = None
    pcm_pitch = None
    audio_noise = None
    bg_descriptors = None
    bg_graphics = None
    bg_transfers = None
    indexed_archives = None
    indexed_direct = []
    alternate_palettes = None
    palette_programs = None
    blend_programs = None
    raster_palettes = None
    encounters = {}
    encounter_units_sha = None
    mission_events = {}
    world_records = None
    terrain_seeds = None
    bg_plane_programs = None
    script_paths = None
    ui_choice_lists = None
    ui_context_templates = None
    ui_graphics = None
    ui_palettes = None
    obj_graphics = None
    lz_graphics = None
    transfer_directory = None
    indexed_tilemap_controls = None
    transfer_lz = None
    transfer_lz_row = None
    type3_payloads = None
    compact_cell_bank = None
    background_effect_graphics = None
    background_screen_banks = None
    screen_update_curves = None
    screen_update_row = None
    type2_pending = None
    firaga_transfer = None
    firaga_palette_rows = None
    obj_animation_banks = None
    obj_animation_extra = {}
    obj_animation_compact = {}
    physical_obj_animations = None
    physical_obj_descriptors = None
    physical_compact_priors = {}
    physical_compact_pending = None
    firaga_obj_pending = {}
    localized_obj_pending = None
    localized_ui_pending = None
    localized_static_row = None
    localized_indexed_pending = None
    constructor_obj_pending = None
    world_roads = None
    observed_tilemaps = None
    tilemap_bank = None
    tilemap_bank_row = None
    tilemap_direct = None
    tilemap_direct_row = None
    byte_tilemaps = None
    firaga_compact_cells = None
    extra_tilemaps = {}
    signed_map_deltas = None
    resource_files = []
    for resource in manifest['resources']:
        row = dict(resource)
        row['description'] = DESCRIPTIONS.get(row['path'], 'Decoded resource')
        path = (dataset / row['path']).resolve()
        if dataset not in path.parents:
            row['status'] = 'Path outside dataset'
        elif not path.is_file():
            row['status'] = 'File missing'
        else:
            content = path.read_bytes()
            content_digest = hashlib.sha256(content).hexdigest()
            if (manifest['profile'] == 'europe' and content_digest == row['sha256']
                    and row['id'] in PHYSICAL_COMPACT_PRIOR_IDS):
                physical_compact_priors[row['id']] = (content, row.get('sources', []))
            if split and content_digest == row['sha256'] and len(content) <= RESOURCE_PAGE_LIMIT:
                identity = (row['id'] + '\0' + row['path'] + '\0' + content_digest).encode('utf-8')
                row['embedded_view'] = hashlib.sha256(identity).hexdigest() + '.html'
                resource_files.append((row, path))
            if content_digest != row['sha256']:
                row['status'] = 'SHA-256 mismatch'
            elif row['id'] == 'graphics.obj-dimensions' and row['format'] == 'obj-dimensions-v1':
                try:
                    entries = tomllib.loads(content.decode('utf-8'))['entries']
                    if not isinstance(entries, list) or len(entries) != 12:
                        raise ValueError('Expected 12 dimensions')
                    if any(type(entry['width']) is not int or type(entry['height']) is not int
                           or entry['width'] not in (8, 16, 32, 64)
                           or entry['height'] not in (8, 16, 32, 64) for entry in entries):
                        raise ValueError('Invalid dimensions')
                    dimensions = {'entries': entries, 'path': row['path']}
                    row['status'] = 'Available'
                except (ValueError, KeyError, TypeError) as error:
                    row['status'] = f'Invalid dimensions data: {error}'
            elif row['id'] == 'graphics.firaga-transfer-program' and row['format'] == 'firaga-transfer-program-v1-json':
                try:
                    firaga_transfer = project_firaga_transfer(json.loads(content))
                    firaga_transfer['path'] = row['path']
                    row['status'] = 'Available: transfer sources'
                except (ValueError, KeyError, TypeError) as error:
                    row['status'] = f'Invalid Firaga transfer data: {error}'
            elif row['id'] == 'graphics.firaga-palette-rows' and row['format'] == 'firaga-palette-rows-v1-json':
                try:
                    firaga_palette_rows = project_firaga_palette_rows(json.loads(content))
                    firaga_palette_rows['path'] = row['path']
                    row['status'] = 'Available: palette row prefixes'
                except (ValueError, KeyError, TypeError) as error:
                    row['status'] = f'Invalid Firaga palette data: {error}'
            elif row['id'] == 'graphics.observed-lz-block' and row['format'] in ('observed-lz-block-v1-json', 'observed-lz-block-v2-json'):
                try:
                    lz_graphics = project_observed_lz(json.loads(content))
                    lz_graphics['path'] = row['path']
                    row['status'] = 'Available: decoded LZ77 outputs'
                except (ValueError, KeyError, TypeError) as error:
                    row['status'] = f'Invalid observed LZ77 data: {error}'
            elif row['id'] == 'graphics.indexed-tilemap-controls' and row['format'] == 'indexed-tilemap-controls-v1-json':
                try:
                    indexed_tilemap_controls = project_indexed_tilemap_controls(json.loads(content))
                    indexed_tilemap_controls['path'] = row['path']
                    row['status'] = f"Available: {len(indexed_tilemap_controls['slots'])} indexed tilemap controls"
                except (ValueError, KeyError, TypeError) as error:
                    indexed_tilemap_controls = None
                    row['status'] = f'Invalid indexed tilemap controls: {error}'
            elif row['id'] == 'graphics.tilemap-transfer-directory' and row['format'] == 'tilemap-transfer-directory-v1-json':
                try:
                    transfer_directory = project_transfer_directory(json.loads(content))
                    transfer_directory['path'] = row['path']
                    row['status'] = f"Available: {len(transfer_directory['slots'])} transfer selectors"
                except (ValueError, KeyError, TypeError) as error:
                    transfer_directory = None
                    row['status'] = f'Invalid transfer directory: {error}'
            elif row['id'] == 'graphics.tilemap-transfer-lz77' and row['format'] == 'tilemap-transfer-lz77-v1-bin':
                try:
                    transfer_lz = project_transfer_lz(content, row['sources'])
                    transfer_lz_row = row
                    for stream in transfer_lz['streams']:
                        stream['path'] = row['path']; stream['resource_id'] = row['id']
                    row['status'] = 'Available: decoded transfer LZ77 outputs'
                except (ValueError, KeyError, TypeError) as error:
                    transfer_lz = None
                    row['status'] = f'Invalid transfer LZ77 data: {error}'
            elif row['id'] == 'graphics.tilemap-type3-transfers' and row['format'] == 'tilemap-type3-transfers-v1-bin':
                try:
                    type3_payloads = project_type3_payloads(content, row['sources'])
                    type3_payloads['path'] = row['path']
                    for stream in type3_payloads['streams'] + type3_payloads['raw_streams']:
                        stream['path'] = row['path']; stream['resource_id'] = row['id']
                    count = len(type3_payloads['streams']) + len(type3_payloads['raw_streams'])
                    row['status'] = (f"Available: {len(type3_payloads['streams'])} LZ77 and {len(type3_payloads['raw_streams'])} raw inputs"
                                     if count else 'Preview unavailable: no valid type-3 inputs')
                    if type3_payloads['errors']:
                        row['status'] += f" ({len(type3_payloads['errors'])} invalid LZ77 inputs)"
                except (ValueError, KeyError, TypeError) as error:
                    type3_payloads = None
                    row['status'] = f'Invalid type-3 transfer data: {error}'
            elif row['id'] == 'graphics.compact-cell-bank' and row['format'] == 'compact-cell-bank-v1-bin':
                try:
                    compact_cell_bank = project_compact_cell_bank(content, row['sources'])
                    compact_cell_bank['path'] = row['path']
                    count = len(compact_cell_bank['records'])
                    row['status'] = f'Available: {count} compact-cell records' if count else 'Preview unavailable: no valid compact-cell records'
                    if compact_cell_bank['errors']:
                        row['status'] += f" ({len(compact_cell_bank['errors'])} invalid records)"
                except (ValueError, KeyError, TypeError) as error:
                    compact_cell_bank = None
                    row['status'] = f'Invalid compact-cell bank: {error}'
            elif row['id'] == 'graphics.tilemap-type2-transfers' and row['format'] == 'tilemap-type2-transfers-v1-bin':
                type2_pending = (content, row)
                row['status'] = 'Preview unavailable'
            elif row['id'] == 'graphics.tilemap-stream-bank' and row['format'] == 'tilemap-stream-bank-v1-bin':
                try:
                    segments = project_binary_segments(content, row['sources'])
                    tilemap_bank = {'segments': segments, 'maps': project_halfword_streams(segments)}
                    tilemap_bank_row = row
                    row['status'] = 'Available: decoded halfword streams' if tilemap_bank['maps'] else 'Preview unavailable'
                except (ValueError, KeyError, TypeError) as error:
                    tilemap_bank = None
                    row['status'] = f'Invalid tilemap stream bank: {error}'
            elif row['id'] == 'graphics.tilemap-direct-transfers' and row['format'] == 'tilemap-direct-transfers-v1-bin':
                try:
                    tilemap_direct = project_binary_segments(content, row['sources'])
                    tilemap_direct_row = row
                    row['status'] = 'Preview unavailable'
                except (ValueError, KeyError, TypeError) as error:
                    tilemap_direct = None
                    row['status'] = f'Invalid tilemap direct transfers: {error}'
            elif row['id'] == 'graphics.background-effect-graphics' and row['format'] == 'background-effect-graphics-v1-bin':
                try:
                    background_effect_graphics = project_background_effect_graphics(content, row['sources'])
                    background_effect_graphics['path'] = row['path']
                    row['status'] = 'Available: background effect tiles and maps'
                except (ValueError, KeyError, TypeError, struct.error) as error:
                    background_effect_graphics = None
                    row['status'] = f'Invalid background effect graphics: {error}'
            elif row['id'] == 'graphics.background-screen-banks' and row['format'] == 'background-screen-banks-v1-bin':
                try:
                    background_screen_banks = project_background_screen_banks(content, row['sources'])
                    background_screen_banks['path'] = row['path']
                    row['status'] = 'Available: five background screen maps'
                except (ValueError, KeyError, TypeError, struct.error) as error:
                    background_screen_banks = None
                    row['status'] = f'Invalid background screen banks: {error}'
            elif row['id'] == 'graphics.screen-update-tables' and row['format'] == 'screen-update-tables-v1-bin':
                screen_update_row = row
                try:
                    screen_update_curves = project_screen_update_curves(content, row['sources'])
                    screen_update_curves['path'] = row['path']
                    row['status'] = 'Available: screen update curves'
                except (ValueError, KeyError, TypeError, struct.error) as error:
                    screen_update_curves = None
                    row['status'] = f'Invalid screen update curves: {error}'
            elif (manifest['profile'] == 'europe' and row['id'] == 'graphics.localized-obj-animation-banks'
                  and row['format'] == 'localized-obj-animation-banks-v1-bin'):
                localized_obj_pending = (content, row)
                row['status'] = 'Preview unavailable: required source owners not loaded'
            elif row['id'] == 'graphics.indexed-archives' and row['format'] in ('indexed-graphics-v1-json', 'indexed-graphics-v2-json'):
                try:
                    indexed_catalog = json.loads(content)
                    indexed_archives = project_indexed_graphics(indexed_catalog)
                    localized_indexed_pending = (indexed_catalog.get('groups', []), row)
                    indexed_archives['path'] = row['path']
                    for family in indexed_archives['families']:
                        family.update(path=row['path'], resource_id=row['id'])
                    row['status'] = 'Available: indexed graphics programs'
                except (ValueError, KeyError, TypeError, AttributeError) as error:
                    indexed_archives = None
                    row['status'] = f'Invalid indexed graphics data: {error}'
            elif (row['id'], row['format']) in (
                    ('graphics.direct-indexed-archive', 'direct-indexed-archive-v1-json'),
                    ('graphics.direct-indexed-dictionary-archive', 'direct-indexed-dictionary-archive-v1-json')):
                try:
                    family = project_direct_indexed_graphics(json.loads(content), row['id'])
                    family['path'] = row['path']
                    indexed_direct.append(family)
                    row['status'] = 'Available: indexed graphics programs'
                except (ValueError, KeyError, TypeError, AttributeError) as error:
                    row['status'] = f'Invalid direct indexed graphics data: {error}'
            elif (row['id'], row['format']) in (('graphics.firaga-copied-matrices', 'firaga-copied-matrices-v1-json'),
                                                 ('graphics.soil-remaining-rectangles', 'soil-remaining-rectangles-v1-json')):
                key = 'firaga_matrices' if row['id'] == 'graphics.firaga-copied-matrices' else 'soil_rectangles'
                try:
                    projected = project_source_grids(json.loads(content), key)
                    projected['path'] = row['path']
                    extra_tilemaps[key] = projected
                    row['status'] = 'Available: source grids'
                except (ValueError, KeyError, TypeError) as error:
                    row['status'] = f'Invalid source grid data: {error}'
            elif row['id'] == 'graphics.ui-raw-transfers' and row['format'] == 'ui-raw-transfers-v1-json':
                raw = None
                try:
                    raw = json.loads(content)
                    copies = raw.get('halfword_copies')
                    if copies is not None and not isinstance(copies, dict):
                        raise ValueError('Invalid UI halfword copies')
                    if copies is not None and copies.get('tilemap_rectangles') is not None:
                        projected = project_ui_tilemap_rectangles(raw)
                        projected['path'] = row['path']
                        extra_tilemaps['ui_rectangles'] = projected
                        row['status'] = 'Available: UI tilemap rectangles'
                    else:
                        row['status'] = 'Preview unavailable'
                except (ValueError, KeyError, TypeError, AttributeError) as error:
                    row['status'] = f'Invalid UI tilemap rectangle data: {error}'
                if (manifest['profile'] == 'europe' and isinstance(raw, dict)
                        and raw.get('signed_map_deltas') is not None):
                    try:
                        signed_map_deltas = project_signed_map_deltas(raw)
                        signed_map_deltas['path'] = row['path']
                        if row['status'] == 'Preview unavailable':
                            row['status'] = 'Available: signed map deltas'
                        elif row['status'].startswith('Available:'):
                            row['status'] += ' and signed map deltas'
                        else:
                            row['status'] = 'Available: signed map deltas; ' + row['status']
                    except (ValueError, KeyError, TypeError, AttributeError) as error:
                        row['status'] += f'; invalid signed map delta data: {error}'
            elif (manifest['profile'] == 'europe' and row['id'] == 'graphics.dynamic-raw-fragments'
                  and row['format'] in ('dynamic-raw-fragments-v1-json', 'dynamic-raw-fragments-v2-json', 'dynamic-raw-fragments-v3-json')):
                try:
                    raw = json.loads(content)
                    if raw.get('localized_queued') is None:
                        row['status'] = 'Preview unavailable: no localized queued transfers'
                    else:
                        projected = project_localized_queued_grids(raw)
                        projected['path'] = row['path']
                        extra_tilemaps['localized_queued'] = projected
                        row['status'] = 'Available: localized transfer rectangles'
                except (ValueError, KeyError, TypeError, AttributeError) as error:
                    row['status'] = f'Invalid localized queued transfer data: {error}'
            elif row['id'] == 'graphics.firaga-compact-cells' and row['format'] == 'firaga-compact-cells-v1-json':
                try:
                    firaga_compact_cells = project_firaga_compact_cells(json.loads(content))
                    firaga_compact_cells['path'] = row['path']
                    row['status'] = 'Available: compact cell frames'
                except (ValueError, KeyError, TypeError) as error:
                    row['status'] = f'Invalid Firaga compact cells: {error}'
            elif (row['id'] == 'graphics.observed-tilemap'
                  and row['format'] in {f'observed-tilemap-v{revision}-json' for revision in range(1, 5)}):
                try:
                    observed_tilemaps = {'maps': project_observed_tilemaps(json.loads(content)), 'path': row['path']}
                    row['status'] = 'Available for tilemap preview'
                except (ValueError, KeyError, TypeError) as error:
                    observed_tilemaps = None
                    row['status'] = f'Invalid tilemap data: {error}'
            elif row['id'] == 'graphics.bg-packet-programs' and row['format'] == 'bg-packet-programs-v1-json':
                try:
                    packet_catalog = json.loads(content)
                except (ValueError, KeyError, TypeError) as error:
                    row['status'] = f'Invalid terrain packet data: {error}'
                else:
                    errors = []
                    try:
                        terrain_seeds = project_terrain_seeds(packet_catalog)
                        terrain_seeds['path'] = row['path']
                    except (ValueError, KeyError, TypeError) as error:
                        errors.append(f'terrain: {error}')
                    if isinstance(packet_catalog, dict) and 'paired' in packet_catalog:
                        try:
                            bg_plane_programs = project_bg_plane_programs(packet_catalog)
                            bg_plane_programs['path'] = row['path']
                        except (ValueError, KeyError, TypeError) as error:
                            errors.append(f'paired planes: {error}')
                    row['status'] = ('Available: terrain seeds and paired planes' if terrain_seeds and bg_plane_programs else
                                     'Available: auxiliary terrain seeds' if terrain_seeds else
                                     'Available: paired background planes' if bg_plane_programs else 'Invalid terrain packet data')
                    if errors:
                        row['status'] += '; ' + '; '.join(errors)
            elif row['id'] == 'graphics.map-raster-palettes' and row['format'] == 'map-raster-palettes-v1-json':
                try:
                    raster_palettes = project_raster_palettes(json.loads(content))
                    raster_palettes['path'] = row['path']
                    row['status'] = 'Available'
                except (ValueError, KeyError, TypeError) as error:
                    row['status'] = f'Invalid raster palette data: {error}'
            elif row['id'] == 'graphics.bg-alternate-palettes' and row['format'] == 'bg-alternate-palettes-v1-json':
                try:
                    alternate_palettes = project_alternate_palettes(json.loads(content))
                    alternate_palettes['path'] = row['path']
                    row['status'] = 'Available'
                except (ValueError, KeyError, TypeError) as error:
                    row['status'] = f'Invalid alternate palette data: {error}'
            elif row['id'] == 'graphics.palette-programs' and row['format'] == 'palette-programs-v1-json':
                try:
                    palette_programs = project_palette_programs(json.loads(content))
                    palette_programs['path'] = row['path']
                    row['status'] = 'Available'
                except (ValueError, KeyError, TypeError) as error:
                    row['status'] = f'Invalid palette program data: {error}'
            elif row['id'] == 'graphics.blend-programs' and row['format'] == 'blend-programs-v1-json':
                try:
                    blend_programs = project_blend_programs(json.loads(content))
                    blend_programs['path'] = row['path']
                    row['status'] = 'Available'
                except (ValueError, KeyError, TypeError) as error:
                    row['status'] = f'Invalid blend program data: {error}'
            elif row['id'] == 'graphics.bg-transfer-programs' and row['format'] == 'bg-transfer-programs-v1-json':
                try:
                    bg_transfers = project_bg_transfers(json.loads(content))
                    bg_transfers['path'] = row['path']
                    row['status'] = 'Available'
                except (ValueError, KeyError, TypeError) as error:
                    row['status'] = f'Invalid background transfer data: {error}'
            elif row['id'] == 'engine.script-command-directory' and row['format'] == 'script-command-directory-v1-json':
                try:
                    script_commands = project_script_commands(json.loads(content))
                    script_commands['path'] = row['path']
                    row['status'] = 'Available'
                except (ValueError, KeyError, TypeError) as error:
                    row['status'] = f'Invalid script command data: {error}'
            elif row['id'] == 'engine.script-archive' and row['format'] == 'script-archive-v1-json':
                try:
                    script_archive = project_script_archive(json.loads(content))
                    script_archive['path'] = row['path']
                    row['status'] = 'Available'
                except (ValueError, KeyError, TypeError) as error:
                    row['status'] = f'Invalid script archive data: {error}'
            elif (row['id'] == 'engine.auxiliary-script-command-directory'
                  and row['format'] == 'auxiliary-script-command-directory-v1-json'):
                try:
                    auxiliary_script_commands = project_script_commands(json.loads(content))
                    auxiliary_script_commands['path'] = row['path']
                    row['status'] = 'Available: auxiliary command directory'
                except (ValueError, KeyError, TypeError) as error:
                    row['status'] = f'Invalid auxiliary command data: {error}'
            elif (row['id'] == 'engine.auxiliary-script-archive'
                  and row['format'] == 'auxiliary-script-archive-v1-json'):
                try:
                    auxiliary_script_archive = project_auxiliary_script_archive(json.loads(content))
                    auxiliary_script_archive['path'] = row['path']
                    row['status'] = 'Available: auxiliary script archive'
                except (ValueError, KeyError, TypeError) as error:
                    row['status'] = f'Invalid auxiliary script data: {error}'
            elif (manifest['profile'] == 'europe' and row['id'] == 'engine.task-descriptors'
                  and row['format'] == 'task-descriptors-v1-json'):
                try:
                    task_descriptors = project_task_descriptors(json.loads(content))
                    task_descriptors['path'] = row['path']
                    row['status'] = 'Available: task descriptor tables'
                except (ValueError, KeyError, TypeError) as error:
                    row['status'] = f'Invalid task descriptor data: {error}'
            elif (manifest['profile'] == 'europe' and row['id'] == 'engine.effect-event-directories'
                  and row['format'] == 'effect-event-directories-v1-json'):
                try:
                    effect_events = project_effect_event_directories(json.loads(content))
                    effect_events['path'] = row['path']
                    row['status'] = 'Available: event and effect directories'
                except (ValueError, KeyError, TypeError) as error:
                    row['status'] = f'Invalid event directory data: {error}'
            elif row['id'] == 'engine.ui-choice-lists' and row['format'] == 'ui-choice-lists-v1-json':
                try:
                    ui_choice_lists = project_ui_choice_lists(json.loads(content))
                    ui_choice_lists['path'] = row['path']
                    row['status'] = 'Available'
                except (ValueError, KeyError, TypeError) as error:
                    row['status'] = f'Invalid choice list data: {error}'
            elif row['id'] == 'ui.context-templates' and row['format'] == 'ui-context-templates-v1-json':
                try:
                    sources = row.get('sources', [])
                    if not isinstance(sources, list):
                        raise ValueError('Invalid UI context source list')
                    matching = [source for source in sources if isinstance(source, dict)
                                and source.get('role') == 'ui-context-templates']
                    if len(matching) > 1:
                        raise ValueError('Ambiguous UI context sources')
                    ui_context_templates = project_ui_context_templates(
                        json.loads(content), source=matching[0] if matching else None,
                        profile=manifest['profile'], rom_sha256=manifest.get('source_rom_sha256'))
                    ui_context_templates['path'] = row['path']
                    row['status'] = 'Available: UI context templates'
                except (ValueError, KeyError, TypeError) as error:
                    ui_context_templates = None
                    row['status'] = f'Invalid UI context data: {error}'
            elif row['id'] == 'graphics.bg-descriptors' and row['format'] == 'bg-descriptors-json-v2':
                try:
                    bg_descriptors = project_bg_descriptors(json.loads(content))
                    sources = row.get('sources')
                    source = sources[0] if isinstance(sources, list) and len(sources) == 1 else None
                    base = source.get('offset') if isinstance(source, dict) and source.get('length') == len(bg_descriptors['entries']) * 88 else None
                    bg_descriptors['source_offset'] = base if type(base) is int and 0 <= base <= 0xffffffff else None
                    bg_descriptors['path'] = row['path']
                    row['status'] = 'Available: background descriptors'
                except (ValueError, KeyError, TypeError) as error:
                    bg_descriptors = None
                    row['status'] = f'Invalid background descriptor data: {error}'
            elif row['id'] == 'graphics.bg-main-graphics' and row['format'] == 'bg-main-graphics-v1-json':
                try:
                    catalog = json.loads(content)
                    links, graphics = catalog['descriptor_graphics'], catalog['graphics']
                    if not isinstance(links, list) or not links or not isinstance(graphics, list) or not graphics:
                        raise ValueError('Empty background graphics')
                    if any(type(index) is not int or not 0 <= index < len(graphics) for index in links):
                        raise ValueError('Invalid background graphics link')
                    projected = []
                    for graphic in graphics:
                        source, offset, words = (graphic[key] for key in ('relative_source', 'destination_offset', 'words'))
                        if (type(source) is not int or source < 0 or type(offset) is not int or offset < 0 or offset % 32
                                or not isinstance(words, list) or not words or len(words) % 8
                                or any(type(word) is not int or not 0 <= word <= 0xffffffff for word in words)):
                            raise ValueError('Invalid background tile words')
                        projected.append({'relative_source': source, 'destination_offset': offset, 'words': words})
                    bg_graphics = {'descriptor_graphics': links, 'graphics': projected, 'path': row['path']}
                    row['status'] = 'Available'
                except (ValueError, KeyError, TypeError) as error:
                    bg_graphics = None
                    row['status'] = f'Invalid background graphics: {error}'
            elif row['id'] == 'graphics.ui-custom-banks' and row['format'] == 'ui-custom-banks-v1-json':
                try:
                    catalog = json.loads(content)
                    banks, selectors = catalog['banks'], catalog.get('selectors', [])
                    if not isinstance(banks, list) or not banks or not isinstance(selectors, list):
                        raise ValueError('Empty UI graphics catalog')
                    projected_banks = []
                    for bank in banks:
                        source, words = bank['source_offset'], bank['words']
                        if (type(source) is not int or source < 0 or not isinstance(words, list)
                                or any(type(word) is not int or not 0 <= word <= 0xffffffff for word in words)):
                            raise ValueError('Invalid UI graphics bank')
                        projected_banks.append({'source_offset': source, 'words': words})
                    projected_selectors = []
                    for selector in selectors:
                        pointers = selector['pointers']
                        if (not isinstance(selector['id'], str) or not isinstance(pointers, list) or not pointers
                                or any(type(pointer['bank_offset']) is not int or pointer['bank_offset'] < 0 for pointer in pointers)):
                            raise ValueError('Invalid UI graphics selector')
                        projected_selectors.append({'id': selector['id'], 'bank_offsets': [pointer['bank_offset'] for pointer in pointers]})
                    ui_graphics = {'banks': projected_banks, 'selectors': projected_selectors, 'path': row['path']}
                    localized_ui_pending = (catalog, row)
                    row['status'] = 'Available'
                except (ValueError, KeyError, TypeError) as error:
                    ui_graphics = None
                    row['status'] = f'Invalid UI graphics: {error}'
            elif row['id'] == 'graphics.obj-custom-banks' and row['format'] == 'obj-custom-banks-v1-json':
                try:
                    catalog = json.loads(content)
                    archive = catalog.get('relative_archive')
                    if archive is not None and not isinstance(archive, dict):
                        raise ValueError('Invalid OBJ relative archive')
                    def project_obj_banks(banks):
                        if not isinstance(banks, list):
                            raise ValueError('Invalid OBJ bank list')
                        projected, seen = [], set()
                        for bank in banks:
                            source, words = bank['source_offset'], bank['words']
                            if (type(source) is not int or source < 0 or source in seen
                                    or not isinstance(words, list)
                                    or any(type(word) is not int or not 0 <= word <= 0xffffffff for word in words)):
                                raise ValueError('Invalid OBJ graphics bank')
                            seen.add(source); projected.append({'source_offset': source, 'words': words})
                        return projected
                    banks = project_obj_banks(catalog['banks'])
                    archive_banks = project_obj_banks(archive.get('banks', []) if archive is not None else [])
                    if not banks and not archive_banks:
                        raise ValueError('Empty OBJ graphics catalog')
                    selectors = catalog.get('pointer_tables', [])
                    if not isinstance(selectors, list):
                        raise ValueError('Invalid OBJ selectors')
                    projected_selectors = []
                    for selector in selectors:
                        source, offsets = selector['source_offset'], selector['bank_offsets']
                        if (type(source) is not int or source < 0 or not isinstance(offsets, list) or not offsets
                                or any(type(offset) is not int or offset < 0 for offset in offsets)):
                            raise ValueError('Invalid OBJ selector')
                        projected_selectors.append({'source_offset': source, 'bank_offsets': offsets})
                    obj_graphics = {'banks': banks, 'archive_banks': archive_banks,
                                    'selectors': projected_selectors, 'path': row['path']}
                    row['status'] = 'Available for OBJ tile preview'
                except (ValueError, KeyError, TypeError) as error:
                    obj_graphics = None
                    row['status'] = f'Invalid OBJ graphics: {error}'
            elif (manifest['profile'] == 'europe' and row['id'] == 'graphics.constructor-obj-animation'
                  and row['format'] == 'constructor-obj-animation-v1-json'):
                try:
                    constructor_obj_pending = (json.loads(content), row)
                    row['status'] = 'Preview unavailable: static object lists not loaded'
                except (ValueError, TypeError) as error:
                    row['status'] = f'Invalid constructor animation data: {error}'
            elif row['id'] in FIRAGA_OBJ_SPECS and row['format'] == row['id'].removeprefix('graphics.') + '-v1-json':
                try:
                    firaga_obj_pending[row['id']] = (json.loads(content), row)
                    row['status'] = 'Preview unavailable: static object lists not loaded'
                except (ValueError, TypeError) as error:
                    row['status'] = f'Invalid Firaga OBJ data: {error}'
            elif row['id'] == 'graphics.obj-static-lists' and row['format'] in ('obj-static-lists-v1-json', 'obj-static-lists-v2-json'):
                try:
                    projected = project_obj_static_lists(json.loads(content))
                    projected['path'] = row['path']
                    obj_animation_extra['static_lists'] = projected
                    localized_static_row = row
                    row['status'] = 'Available: static object layouts'
                except (ValueError, KeyError, TypeError) as error:
                    row['status'] = f'Invalid static OBJ data: {error}'
            elif row['id'] == 'graphics.obj-animation-banks' and row['format'] == 'obj-animation-banks-v1-json':
                try:
                    obj_animation_banks = project_obj_animation_banks(json.loads(content))
                    obj_animation_banks['path'] = row['path']
                    row['status'] = 'Available'
                except (ValueError, KeyError, TypeError) as error:
                    row['status'] = f'Invalid OBJ animation data: {error}'
            elif row['id'] == 'graphics.word-task-obj-animations' and row['format'] == 'word-task-obj-animations-v1-json':
                try:
                    projected = project_obj_animation_banks(json.loads(content))
                    projected['path'] = row['path']
                    obj_animation_extra['word_task'] = projected
                    row['status'] = 'Available'
                except (ValueError, KeyError, TypeError) as error:
                    row['status'] = f'Invalid word-task OBJ data: {error}'
            elif row['id'] == 'graphics.layered-obj-animations' and row['format'] == 'layered-obj-animations-v1-json':
                try:
                    projected = project_layered_obj_animations(json.loads(content))
                    projected['path'] = row['path']
                    obj_animation_extra['layered'] = projected
                    row['status'] = 'Available: indexed animation banks'
                except (ValueError, KeyError, TypeError) as error:
                    row['status'] = f'Invalid layered OBJ data: {error}'
            elif any(row['id'] == 'graphics.conditional-compact-obj-animations' + suffix
                     and row['format'] == 'compact-obj-animations' + suffix + '-v1-json' for suffix in COMPACT_OBJ_SUFFIXES):
                suffix = row['id'].removeprefix('graphics.conditional-compact-obj-animations')
                try:
                    projected = project_compact_obj_animations(json.loads(content), primary=not suffix)
                    projected['path'] = row['path']
                    obj_animation_compact['compact' + suffix.replace('-', '_')] = projected
                    row['status'] = 'Available'
                except (ValueError, KeyError, TypeError) as error:
                    row['status'] = f'Invalid compact OBJ data: {error}'
            elif (row['id'] == 'graphics.conditional-compact-obj-catalog'
                  and row['format'] in {f'compact-obj-catalog-v{version}-json' for version in range(1, 16)}):
                try:
                    catalog = json.loads(content)
                    projected = project_compact_obj_catalog(catalog)
                    projected['path'] = row['path']
                    obj_animation_compact['compact_catalog'] = projected
                    try:
                        auxiliary = project_compact_catalog_auxiliary(catalog, projected['banks'])
                        if auxiliary:
                            projected['auxiliary'] = auxiliary
                        row['status'] = ('Available: compact animation banks and source tables' if auxiliary
                                         else 'Available: compact animation banks')
                    except (ValueError, KeyError, TypeError) as error:
                        row['status'] = f'Available: compact animation banks; invalid source tables: {error}'
                except (ValueError, KeyError, TypeError) as error:
                    row['status'] = f'Invalid compact OBJ catalog: {error}'
            elif row['id'] == 'graphics.ui-raw-palettes' and row['format'] == 'ui-raw-palettes-v1-json':
                try:
                    banks = json.loads(content)['banks']
                    if not isinstance(banks, list) or not banks:
                        raise ValueError('Empty UI palettes')
                    projected = []
                    for bank in banks:
                        source, colors = bank['source_offset'], bank['colors']
                        if (type(source) is not int or source < 0 or not isinstance(colors, list) or not colors or len(colors) % 16
                                or any(type(color) is not int or not 0 <= color <= 65535 for color in colors)):
                            raise ValueError('Invalid UI palette bank')
                        projected.append({'source_offset': source, 'colors': colors})
                    ui_palettes = {'banks': projected, 'path': row['path']}
                    row['status'] = 'Available for UI tile preview'
                except (ValueError, KeyError, TypeError) as error:
                    ui_palettes = None
                    row['status'] = f'Invalid UI palettes: {error}'
            elif row['id'] == 'graphics.color-mode-palettes' and row['format'] == 'color-mode-palettes-v1-json':
                try:
                    palette_catalog = json.loads(content)
                    spans = palette_catalog['spans']
                    unit_palettes = []
                    for span in spans:
                        if span['book'] != 'unit-base':
                            continue
                        colors = span['colors']
                        if not isinstance(colors, list) or len(colors) != 256 or any(
                            type(color) is not int or not 0 <= color <= 65535 for color in colors
                        ):
                            raise ValueError('Invalid unit-base palette')
                        for index in range(16):
                            unit_palettes.append({'variant': span['variant'], 'bank': index,
                                                  'colors': colors[index * 16:index * 16 + 16]})
                    if not unit_palettes:
                        raise ValueError('No unit-base palettes')
                    row['status'] = 'Available for unit sprites'
                except (ValueError, KeyError, TypeError) as error:
                    unit_palettes = None
                    row['status'] = f'Invalid color mode data: {error}'
                try:
                    color_mode_palettes = project_color_mode_palettes(json.loads(content))
                    color_mode_palettes['path'] = row['path']
                    color_mode_sha = content_digest
                    row['status'] = 'Available: color fragments' + (' and unit palettes' if unit_palettes else '')
                except (ValueError, KeyError, TypeError) as error:
                    color_mode_palettes = None
                    if unit_palettes:
                        row['status'] = f'Available for unit sprites; invalid color fragments: {error}'
            elif row['id'] == 'graphics.unit-obj-property-words' and row['format'] == 'unit-obj-property-words-v1-json':
                try:
                    job_sprite_producers = project_job_sprite_producers(json.loads(content))
                    job_sprite_producer_sha = content_digest
                    job_sprite_producer_row = row
                    row['status'] = 'Preview unavailable'
                except (ValueError, KeyError, TypeError) as error:
                    job_sprite_producers = None
                    row['status'] = f'Invalid job sprite references: {error}'
            elif (row['id'] in {f'graphics.unit-obj-script-{mode}' for mode in
                                ('special-normal', 'special-alternate', 'override',
                                 'registry-special', 'registry-generic')}
                  and row['format'] == row['id'].removeprefix('graphics.') + '-v1-json'):
                mode = row['id'].removeprefix('graphics.unit-obj-script-')
                try:
                    script_sprites[mode] = project_script_sprites(json.loads(content), alternate=mode == 'registry-generic')
                    script_sprites[mode]['path'] = row['path']
                    row['status'] = 'Available'
                except (ValueError, KeyError, TypeError) as error:
                    row['status'] = f'Invalid script sprite data: {error}'
            elif (manifest['profile'] == 'europe'
                  and row['id'] in ('graphics.script39-registry-poses', 'graphics.script39-special-poses')
                  and row['format'] == row['id'].removeprefix('graphics.') + '-v1-json'):
                try:
                    script39_pending[row['id']] = (validate_script39_poses(json.loads(content)), row)
                    row['status'] = 'Preview unavailable: resolving pose sources'
                except (ValueError, KeyError, TypeError) as error:
                    row['status'] = f'Invalid opcode-39 pose data: {error}'
            elif (row['id'] in ('graphics.unit-obj-normal-battle', 'graphics.unit-obj-alternate-battle')
                  and row['format'] == row['id'].removeprefix('graphics.') + '-v1-json'):
                mode = 'normal' if row['id'] == 'graphics.unit-obj-normal-battle' else 'alternate'
                try:
                    battle_sprites[mode] = project_equipment_sprites(json.loads(content), alternate=mode == 'alternate')
                    battle_sprites[mode]['path'] = row['path']
                    battle_sprite_sha[mode] = content_digest
                    row['status'] = 'Available'
                except (ValueError, KeyError, TypeError) as error:
                    row['status'] = f'Invalid battle sprite data: {error}'
            elif row['id'] == 'graphics.unit-obj-equipment' and row['format'] == 'unit-obj-equipment-v1-json':
                try:
                    equipment_sprites = project_equipment_sprites(json.loads(content))
                    equipment_sprites['path'] = row['path']
                    equipment_sprites_sha = content_digest
                    row['status'] = 'Available'
                except (ValueError, KeyError, TypeError) as error:
                    equipment_sprites = None
                    row['status'] = f'Invalid equipment sprite data: {error}'
            elif row['id'] == 'graphics.unit-obj-initial-pose' and row['format'] == 'unit-obj-initial-pose-v1-json':
                try:
                    catalog = json.loads(content)
                    blocks = catalog['pixels']
                    if not isinstance(blocks, list) or not blocks:
                        raise ValueError('No sprite pixel blocks')
                    for block in blocks:
                        if type(block['source_offset']) is not int or not isinstance(block['tiles'], list) or not block['tiles']:
                            raise ValueError('Invalid pixel block')
                        for tile in block['tiles']:
                            indices = tile['indices']
                            if not isinstance(indices, list) or len(indices) != 8 or any(
                                not isinstance(line, list) or len(line) != 8 or
                                any(type(index) is not int or not 0 <= index <= 15 for index in line)
                                for line in indices
                            ):
                                raise ValueError('Invalid indexed sprite tile')
                    bindings = catalog['bindings']
                    sprite_sequences = catalog['sequences']
                    objects = catalog['objects']
                    sequences_by_source = {sequence['source_offset']: sequence for sequence in sprite_sequences}
                    blocks_by_source = {block['source_offset']: index for index, block in enumerate(blocks)}
                    objects_by_source = {item['source_offset']: item for item in objects}
                    sprite_frames = []
                    for binding in bindings:
                        sequence = sequences_by_source[binding['sequence_source']]
                        if not isinstance(sequence['frames'], list) or not sequence['frames']:
                            raise ValueError('Empty sprite sequence')
                        frames = []
                        for frame in sequence['frames']:
                            if (type(frame['duration']) is not int or not 0 <= frame['duration'] <= 255
                                    or ('operation' in frame and frame['operation'] != 1)):
                                raise ValueError('Invalid sprite frame duration or operation')
                            pixel_source = 0x898a40 + frame['pixel_offset']
                            object_source = 0xa50020 + frame['object_offset']
                            frames.append({'block': blocks_by_source[pixel_source],
                                           'object_source': object_source,
                                           'object_count': len(objects_by_source[object_source]['objects']),
                                           'objects': objects_by_source[object_source]['objects'],
                                           'duration': frame['duration']})
                        sprite_frames.append({'id': binding['sprite_id'], 'frames': frames})
                    sprite_pixels = {'blocks': blocks, 'sprites': sprite_frames, 'path': row['path']}
                    row['status'] = 'Available'
                except (ValueError, KeyError, TypeError) as error:
                    row['status'] = f'Invalid sprite pixel data: {error}'
            elif row['id'] == 'font.complete-bank' and row['format'] == 'font-bank-v1-json':
                try:
                    font = indexed_glyphs(json.loads(content)['glyphs'])
                    font_path = row['path']
                    row['status'] = 'Available for glyph atlas and script text'
                except (ValueError, KeyError, TypeError) as error:
                    font = None
                    font_path = None
                    row['status'] = f'Invalid font data: {error}'
            elif row['id'] == 'text.name-keyboard' and row['format'] == 'name-keyboard-v1-json':
                try:
                    catalog = json.loads(content)
                    widths, cells, programs = (catalog[key] for key in ('row_widths', 'cells', 'programs'))
                    if any(not isinstance(value, list) or len(value) != 3 for value in (widths, cells, programs)):
                        raise ValueError('Expected three keyboard pages')
                    pages = []
                    for page_widths, page_cells, program in zip(widths, cells, programs):
                        if (not isinstance(page_widths, list) or len(page_widths) != 6
                                or any(type(width) is not int or not 1 <= width <= 10 for width in page_widths)
                                or not isinstance(page_cells, list) or len(page_cells) != 60
                                or any(type(cell) is not int or not 0 <= cell <= 65535 for cell in page_cells)
                                or type(program['source_offset']) is not int or program['source_offset'] < 0):
                            raise ValueError('Invalid keyboard page')
                        for index, width in enumerate(page_widths):
                            row_cells = page_cells[index * 10:(index + 1) * 10]
                            if any(cell == 0 for cell in row_cells[:width]) or any(row_cells[width:]):
                                raise ValueError('Keyboard row and width mismatch')
                        pages.append({'row_widths': page_widths, 'cells': page_cells,
                                      'source_offset': program['source_offset']})
                    name_keyboard = {'pages': pages, 'path': row['path']}
                    row['status'] = 'Available'
                except (ValueError, KeyError, TypeError) as error:
                    name_keyboard = None
                    row['status'] = f'Invalid name keyboard data: {error}'
            elif row['id'] == 'text.catalogs' and row['format'] == 'text-catalogs-v1-json':
                try:
                    catalog = json.loads(content)
                    glyphs = indexed_glyphs(catalog['glyphs'])
                    source_strings = catalog['strings']
                    source_catalogs = catalog['pointer_catalogs']
                    if not isinstance(source_strings, list) or not source_strings or not isinstance(source_catalogs, list) or not source_catalogs:
                        raise ValueError('Empty pointer text catalog')
                    strings = []
                    used = set()
                    for source_text in source_strings:
                        offset, encoding, units = (source_text[key] for key in ('source_offset', 'encoding', 'units'))
                        if type(offset) is not int or encoding not in ('compact', 'extended') or not isinstance(units, list):
                            raise ValueError('Invalid pointer text')
                        projected = []
                        for unit in units:
                            if unit['kind'] == 'glyph' and type(unit['index']) is int and 0 <= unit['index'] <= 65535:
                                projected.append(unit['index'])
                                used.add(unit['index'])
                            elif unit['kind'] == 'control_byte' and type(unit['value']) is int and 0 <= unit['value'] <= 255:
                                projected.append([unit['value']])
                            else:
                                raise ValueError('Invalid pointer text unit')
                        strings.append({'source_offset': offset, 'encoding': encoding, 'units': projected})
                    catalogs = []
                    for source_catalog in source_catalogs:
                        reader, languages = source_catalog['reader'], source_catalog['languages']
                        if type(reader) is not int or not isinstance(languages, list) or not languages:
                            raise ValueError('Invalid pointer text reader')
                        for language in languages:
                            if (type(language['source_offset']) is not int or not isinstance(language['entries'], list)
                                    or not language['entries'] or any(type(index) is not int or not 0 <= index < len(strings)
                                                                      for index in language['entries'])):
                                raise ValueError('Invalid pointer text index')
                        catalogs.append({'reader': reader, 'languages': languages})
                    pointer_texts = {'catalogs': catalogs, 'strings': strings,
                                     'glyphs': [glyphs[index] for index in sorted(used) if index in glyphs],
                                     'path': row['path']}
                    pointer_text_sha = content_digest
                    relative_error = None
                    try:
                        source_archives = catalog.get('archive_catalogs', [])
                        if not isinstance(source_archives, list):
                            raise ValueError('Invalid relative archive catalog')
                        archives, archive_used = [], set()
                        for archive in source_archives:
                            reader, languages = archive['reader'], archive['languages']
                            if type(reader) is not int or not isinstance(languages, list) or not languages:
                                raise ValueError('Invalid relative archive reader')
                            projected_languages = []
                            for language in languages:
                                source, programs = language['source_offset'], language['programs']
                                offsets = language['offsets']
                                if (type(source) is not int or not isinstance(programs, list) or not programs
                                        or not isinstance(offsets, list) or len(offsets) != len(programs) + 1
                                        or offsets[-1] != 65535 or any(type(value) is not int or not 0 <= value <= 65535 for value in offsets)):
                                    raise ValueError('Invalid relative archive index')
                                projected_programs = [project_text_program(program) for program in programs]
                                archive_used.update(unit for program in projected_programs for unit in program['units'] if isinstance(unit, int))
                                projected_languages.append({'source_offset': source, 'programs': projected_programs})
                            archives.append({'reader': reader, 'languages': projected_languages})
                        if archives:
                            relative_texts = {'catalogs': archives,
                                              'glyphs': [glyphs[index] for index in sorted(archive_used) if index in glyphs],
                                              'path': row['path']}
                    except (ValueError, KeyError, TypeError) as error:
                        relative_texts = None
                        relative_error = str(error)
                    source_archives = archive = programs = None
                    del catalog, source_strings, source_catalogs, source_text, source_catalog, language, languages
                    row['status'] = ('Available: pointer texts; invalid relative archives: ' + relative_error if relative_error
                                     else 'Available: pointer and relative texts' if relative_texts else 'Available: pointer texts')
                except (ValueError, KeyError, TypeError) as error:
                    pointer_texts = None
                    pointer_text_sha = None
                    relative_texts = None
                    row['status'] = f'Invalid pointer text data: {error}'
            elif row['id'] == 'engine.text-reader-registry' and row['format'] == 'text-reader-registry-v1-json':
                try:
                    text_reader_registry = project_text_reader_registry(json.loads(content))
                    text_reader_registry['path'] = row['path']
                    row['status'] = 'Available: text reader callback slots'
                except (ValueError, KeyError, TypeError, AttributeError) as error:
                    text_reader_registry = None
                    row['status'] = f'Invalid text reader registry: {error}'
            elif row['id'] == 'text.script-archives' and row['format'] == 'script-text-v1-json':
                try:
                    languages = json.loads(content)['languages']
                    if not isinstance(languages, list) or not languages:
                        raise ValueError('No script text languages')
                    projected_languages = []
                    for language in languages:
                        language_id, banks = language['id'], language['banks']
                        if type(language_id) is not int or not isinstance(banks, list) or not banks:
                            raise ValueError('Invalid script text language')
                        projected_banks = []
                        for bank in banks:
                            source, programs = bank['source_offset'], bank['programs']
                            if type(source) is not int or not isinstance(programs, list) or not programs:
                                raise ValueError('Invalid script text bank')
                            projected_programs = []
                            for program in programs:
                                projected_programs.append(project_text_program(program))
                            projected_banks.append({'source_offset': bank['source_offset'], 'programs': projected_programs})
                        projected_languages.append({'id': language_id, 'banks': projected_banks})
                    script_texts = {'languages': projected_languages, 'path': row['path']}
                    try:
                        script_text_refs = project_script_text_refs(languages)
                    except (ValueError, KeyError, TypeError):
                        script_text_refs = None
                    del languages, language, bank, program
                    row['status'] = 'Available'
                except (ValueError, KeyError, TypeError) as error:
                    script_texts = None
                    row['status'] = f'Invalid script text data: {error}'
            elif row['id'] == 'text.short-strings' and row['format'] == 'short-texts-v1-json':
                try:
                    catalog = json.loads(content)
                    languages, strings, glyphs = (catalog[key] for key in ('languages', 'strings', 'glyphs'))
                    if not languages or not strings or not isinstance(glyphs, list):
                        raise ValueError('Empty short-text catalog')
                    for language in languages:
                        if any(type(index) is not int or not 0 <= index < len(strings) for index in language['entries']):
                            raise ValueError('Invalid text index')
                    for glyph in glyphs:
                        if (type(glyph['index']) is not int or type(glyph['width']) is not int
                                or not 0 <= glyph['width'] <= 16 or len(glyph['tile_words']) != 32
                                or any(type(word) is not int or not 0 <= word <= 0xffffffff for word in glyph['tile_words'])):
                            raise ValueError('Invalid glyph')
                    for string in strings:
                        for unit in string['units']:
                            if unit['kind'] == 'glyph' and type(unit['index']) is int:
                                continue
                            if unit['kind'] == 'control_byte' and type(unit['value']) is int and 0 <= unit['value'] <= 255:
                                continue
                            raise ValueError('Invalid text unit')
                    short_texts = {'languages': languages, 'strings': strings, 'glyphs': glyphs, 'path': row['path']}
                    row['status'] = 'Available'
                except (ValueError, KeyError, TypeError) as error:
                    row['status'] = f'Invalid short-text data: {error}'
            elif row['id'] == 'audio.m4a-directory' and row['format'] == 'audio-directory-v2-json':
                timed_sound_sources = None
                try:
                    directory = json.loads(content)
                    sources, sound_errors = project_timed_sound_sources(directory)
                    timed_sound_sources = {'path': row['path'], 'groups': sources} if sources else None
                    songs, headers = directory['songs'], directory['headers']
                    if not isinstance(songs, list) or not songs or not isinstance(headers, list) or not headers:
                        raise ValueError('Empty audio directory')
                    header_ids = set()
                    for header in headers:
                        if type(header['id']) is not int or header['id'] in header_ids:
                            raise ValueError('Invalid audio header')
                        header_ids.add(header['id'])
                    if any(type(song['id']) is not int or type(song['header_id']) is not int
                           or song['header_id'] not in header_ids for song in songs):
                        raise ValueError('Invalid audio song')
                    projected_headers = []
                    for header in headers:
                        bank_address = header.get('tone_bank_address')
                        if bank_address is not None and (type(bank_address) is not int or not 0 <= bank_address <= 0xffffffff):
                            raise ValueError('Invalid audio tone bank address')
                        projected_headers.append({'id': header['id'], 'tone_bank_address': bank_address})
                    sequence_directory = {'songs': [{'id': song['id'], 'header_id': song['header_id']} for song in songs],
                                          'headers': projected_headers, 'path': row['path']}
                    row['status'] = 'Available for sequence preview' + (' and timed sound sources' if sources else '')
                    if sound_errors:
                        row['status'] += '; invalid ' + '; '.join(sound_errors)
                except (ValueError, KeyError, TypeError, AttributeError) as error:
                    sequence_directory = None
                    row['status'] = (f'Available: timed sound sources; invalid song directory: {error}'
                                     if timed_sound_sources else f'Invalid audio directory: {error}')
            elif row['id'] == 'audio.m4a-sequences' and row['format'] == 'audio-sequences-v1-json':
                try:
                    sequences = project_audio_sequences(json.loads(content))
                    sequences['path'] = row['path']
                    row['status'] = 'Available for sequence preview'
                except (ValueError, KeyError, TypeError) as error:
                    sequences = None
                    row['status'] = f'Invalid sequence data: {error}'
            elif row['id'] == 'audio.m4a-driver-tables' and row['format'] == 'audio-driver-tables-v1-json':
                pcm_pitch = audio_noise = None
                try:
                    driver = json.loads(content)
                except (ValueError, TypeError) as error:
                    pcm_pitch = None
                    row['status'] = f'Invalid PCM pitch tables: {error}'
                else:
                    available, errors = [], []
                    try:
                        pcm_pitch = project_audio_pcm_pitch(driver)
                        pcm_pitch['path'] = row['path']
                        available.append('PCM and CGB pitch tables' if 'cgb_keys' in pcm_pitch else 'PCM pitch tables')
                    except (ValueError, KeyError, TypeError, AttributeError) as error:
                        pcm_pitch = None
                        errors.append('PCM pitch tables: ' + str(error))
                    try:
                        audio_noise = project_audio_noise(driver)
                        if audio_noise:
                            audio_noise['path'] = row['path']
                            available.append('CGB noise codes')
                    except (ValueError, KeyError, TypeError, AttributeError) as error:
                        audio_noise = None
                        errors.append('CGB noise table: ' + str(error))
                    row['status'] = ('Available: ' + ', '.join(available) if available else 'Invalid audio driver tables')
                    if errors:
                        row['status'] += '; invalid ' + '; '.join(errors)
            elif row['id'] == 'audio.m4a-tones' and row['format'] == 'audio-tones-v1-json':
                try:
                    audio_catalog = json.loads(content)
                    waves = audio_catalog['pcm_waves']
                    cgb_waves = audio_catalog.get('cgb_waves', [])
                    if not isinstance(waves, list) or not waves:
                        raise ValueError('No PCM waves')
                    for wave in waves:
                        if (type(wave['id']) is not int or type(wave['loop_start']) is not int
                                or type(wave['declared_length']) is not int or
                                not isinstance(wave['samples'], list) or
                                wave['declared_length'] != len(wave['samples']) or
                                any(type(value) is not int or not -128 <= value <= 127 for value in wave['samples'])):
                            raise ValueError('Invalid PCM wave')
                    if not isinstance(cgb_waves, list):
                        raise ValueError('Invalid CGB wave catalog')
                    for wave in cgb_waves:
                        amplitudes = wave['amplitudes']
                        if (type(wave['id']) is not int or not isinstance(amplitudes, list)
                                or len(amplitudes) != 32
                                or any(type(value) is not int or not 0 <= value <= 15 for value in amplitudes)):
                            raise ValueError('Invalid CGB wave')
                    instrument_error = None
                    try:
                        instruments = project_audio_instruments(audio_catalog)
                    except (ValueError, KeyError, TypeError) as error:
                        instruments = None; instrument_error = str(error)
                    audio = {'waves': waves, 'cgb_waves': cgb_waves, 'instruments': instruments, 'path': row['path']}
                    audio_source_catalog = audio_catalog
                    row['status'] = 'Available' if instrument_error is None else 'Available: invalid instrument metadata: ' + instrument_error
                except (ValueError, KeyError, TypeError) as error:
                    row['status'] = f'Invalid audio data: {error}'
            elif (manifest['profile'] == 'europe' and row['id'] == 'graphics.physical-compact-obj-banks'
                  and row['format'] == 'physical-compact-obj-banks-v1-bin'):
                physical_compact_pending = (content, row)
                row['status'] = 'Preview unavailable: resolving prior compact sources'
            elif (manifest['profile'] == 'europe' and row['id'] == 'audio.physical-tone-banks'
                  and row['format'] == 'physical-tone-banks-v1-bin'):
                physical_audio_pending = (content, row)
                row['status'] = 'Preview unavailable: original tone catalog not loaded'
            elif row['id'] == 'gameplay.world-roads' and row['format'] == 'world-roads-v1-json':
                try:
                    catalog = json.loads(content)
                    paths = catalog['paths']
                    if not isinstance(paths, list) or not paths:
                        raise ValueError('No world roads')
                    projected = []
                    for index, road in enumerate(paths):
                        points = road['points']
                        if (road['index'] != index or type(road['point_count']) is not int
                                or not isinstance(points, list) or len(points) != road['point_count']
                                or type(road['encoded_pointer']) is not int
                                or any(type(point[key]) is not int or not -128 <= point[key] <= 127
                                       for point in points for key in ('x', 'y'))):
                            raise ValueError('Invalid world road')
                        projected.append({'index': index, 'points': points, 'encoded_pointer': road['encoded_pointer']})
                    world_roads = {'paths': projected, 'path': row['path']}
                    row['status'] = 'Available'
                except (ValueError, KeyError, TypeError) as error:
                    world_roads = None
                    row['status'] = f'Invalid world road data: {error}'
            elif row['id'] == 'gameplay.world-nodes' and row['format'] == 'world-nodes-v1-json':
                try:
                    nodes = json.loads(content)['nodes']
                    if not isinstance(nodes, list) or not nodes:
                        raise ValueError('No world nodes')
                    for index, node in enumerate(nodes):
                        if node['index'] != index or any(type(node[key]) is not int or not 0 <= node[key] <= 255 for key in ('x_tile', 'y_tile')):
                            raise ValueError('Invalid node coordinates')
                        if not isinstance(node['links'], list) or len(node['links']) != 4:
                            raise ValueError('Invalid node links')
                        for link in node['links']:
                            if any(type(link[key]) is not int or not 0 <= link[key] <= 255 for key in ('path_id', 'neighbor')):
                                raise ValueError('Invalid link')
                    world_nodes = {'nodes': nodes, 'path': row['path']}
                    world_node_sha = content_digest
                    row['status'] = 'Available'
                except (ValueError, KeyError, TypeError) as error:
                    row['status'] = f'Invalid world node data: {error}'
            elif row['id'] == 'engine.script-coordinate-paths' and row['format'] == 'script-coordinate-paths-v1-json':
                try:
                    script_paths = project_script_paths(json.loads(content))
                    script_paths['path'] = row['path']
                    row['status'] = 'Available'
                except (ValueError, KeyError, TypeError) as error:
                    row['status'] = f'Invalid script path data: {error}'
            elif row['id'] == 'gameplay.linked-metadata28' and row['format'] == 'linked-metadata28-v1-json':
                try:
                    action_records = project_action_records(json.loads(content))
                    action_records['path'] = row['path']
                    action_record_sha = content_digest
                    row['status'] = 'Available: action source records'
                except (ValueError, KeyError, TypeError) as error:
                    action_records = None
                    row['status'] = f'Invalid action record data: {error}'
            elif (manifest['profile'] == 'europe' and row['id'] == 'gameplay.linked-records4'
                  and row['format'] == 'linked-records4-v1-json'):
                try:
                    linked_records4 = project_linked_records4(json.loads(content))
                    linked_records4['path'] = row['path']
                    linked_records4_sha = content_digest
                    row['status'] = 'Available: four-byte linked records' + (
                        ' and callback sources' if linked_records4['callback_tables'] or linked_records4['action_callbacks'] else '')
                except (ValueError, KeyError, TypeError) as error:
                    linked_records4 = None
                    row['status'] = f'Invalid four-byte linked records: {error}'
            elif (manifest['profile'] == 'europe' and row['id'] in CONDITIONAL_LOOKUPS
                  and row['format'] == CONDITIONAL_LOOKUPS[row['id']][0]):
                _, key, expected_digest, projector = CONDITIONAL_LOOKUPS[row['id']]
                try:
                    projected = projector(json.loads(content))
                    projected['path'] = row['path']
                    projected['reader_verified'] = content_digest == expected_digest
                    conditional_lookups[key] = projected
                    row['status'] = 'Available: conditional lookup records'
                except (ValueError, KeyError, TypeError, AttributeError) as error:
                    row['status'] = f'Invalid conditional lookup data: {error}'
            elif (manifest['profile'] == 'europe' and row['id'] == 'gameplay.conditional-records8'
                  and row['format'] == 'conditional-records8-v1-json'):
                try:
                    conditional_records8 = project_conditional_records8(json.loads(content))
                    conditional_records8['path'] = row['path']
                    conditional_records8['reader_verified'] = content_digest == EUROPE_CONDITIONAL_RECORDS8_SHA256
                    row['status'] = 'Available: conditional source records'
                except (ValueError, KeyError, TypeError) as error:
                    conditional_records8 = None
                    row['status'] = f'Invalid conditional records: {error}'
            elif (manifest['profile'] == 'europe' and row['id'] == 'gameplay.linked-metadata12'
                  and row['format'] == 'linked-metadata12-v1-json'):
                try:
                    metadata12 = project_metadata12(json.loads(content))
                    metadata12['path'] = row['path']
                    metadata12['reader_verified'] = content_digest == EUROPE_METADATA12_SHA256
                    row['status'] = 'Available: indexed packed pair fields'
                except (ValueError, KeyError, TypeError) as error:
                    metadata12 = None
                    row['status'] = f'Invalid indexed packed pairs: {error}'
            elif (manifest['profile'] == 'europe' and row['id'] == 'gameplay.global-pair-flags'
                  and row['format'] == 'global-pair-flags-v1-json'):
                try:
                    global_pair_flags = project_global_pair_flags(json.loads(content))
                    global_pair_flags['path'] = row['path']
                    global_pair_flags['reader_verified'] = content_digest == EUROPE_GLOBAL_PAIR_FLAGS_SHA256
                    row['status'] = 'Available: global packed pair fields'
                except (ValueError, KeyError, TypeError) as error:
                    global_pair_flags = None
                    row['status'] = f'Invalid global packed pairs: {error}'
            elif row['id'] == 'gameplay.world-records38' and row['format'] == 'world-records38-v1-json':
                try:
                    world_records = project_world_records(json.loads(content))
                    world_records['path'] = row['path']
                    row['status'] = 'Available: world records and thresholds'
                except (ValueError, KeyError, TypeError) as error:
                    world_records = None
                    row['status'] = f'Invalid world record data: {error}'
            elif (row['id'] in ('gameplay.mission-records70', 'gameplay.event-records12', 'gameplay.parameter-records27')
                  and row['format'] == row['id'].removeprefix('gameplay.') + '-v1-json'):
                kind = {'gameplay.mission-records70': 'missions', 'gameplay.event-records12': 'events',
                        'gameplay.parameter-records27': 'parameters'}[row['id']]
                try:
                    mission_events[kind] = project_mission_event_catalog(json.loads(content), kind)
                    mission_events[kind]['path'] = row['path']
                    row['status'] = 'Available'
                except (ValueError, KeyError, TypeError) as error:
                    row['status'] = f'Invalid mission/event {kind} data: {error}'
            elif (row['id'] in ('gameplay.encounter-headers', 'gameplay.encounter-unit-templates',
                                'gameplay.encounter-coordinates')
                  and row['format'] == row['id'].removeprefix('gameplay.') + '-v1-json'):
                kind = {'gameplay.encounter-headers': 'headers', 'gameplay.encounter-unit-templates': 'units',
                        'gameplay.encounter-coordinates': 'coordinates'}[row['id']]
                try:
                    encounters[kind] = project_encounter_catalog(json.loads(content), kind)
                    encounters[kind]['path'] = row['path']
                    if kind == 'units':
                        encounter_units_sha = content_digest
                    row['status'] = 'Available'
                except (ValueError, KeyError, TypeError) as error:
                    row['status'] = f'Invalid encounter {kind} data: {error}'
            elif (manifest['profile'] == 'europe' and row['id'] == 'gameplay.slot-job-lookup'
                  and row['format'] == 'slot-job-lookup-v1-json'):
                try:
                    slot_jobs = project_slot_job_lookup(json.loads(content))
                    slot_jobs['path'] = row['path']
                    slot_job_row = row
                    row['status'] = 'Available: job slot references'
                except (ValueError, KeyError, TypeError) as error:
                    slot_jobs = None
                    row['status'] = f'Invalid job slot data: {error}'
            elif (manifest['profile'] == 'europe' and row['id'] == 'gameplay.linked-metadata52'
                  and row['format'] == 'linked-metadata52-v1-json'):
                try:
                    job_metadata = project_job_metadata(json.loads(content))
                    job_metadata['path'] = row['path']
                    job_metadata_sha = content_digest
                    job_metadata_row = row
                    row['status'] = 'Preview unavailable'
                except (ValueError, KeyError, TypeError) as error:
                    job_metadata = None
                    row['status'] = f'Invalid job metadata: {error}'
            elif (manifest['profile'] == 'europe' and row['id'] == 'gameplay.linked-metadata14'
                  and row['format'] == 'linked-metadata14-v1-json'):
                try:
                    metadata14 = project_metadata14(json.loads(content))
                    metadata14['path'] = row['path']
                    metadata14_sha = content_digest
                    row['status'] = 'Available: special-unit source records'
                except (ValueError, KeyError, TypeError) as error:
                    metadata14 = None
                    row['status'] = f'Invalid special-unit metadata: {error}'
            elif (manifest['profile'] == 'europe' and row['id'] == 'gameplay.linked-metadata2'
                  and row['format'] == 'linked-metadata2-v1-json'):
                try:
                    metadata2 = project_metadata2(json.loads(content))
                    metadata2['path'] = row['path']
                    metadata2['reader_verified'] = content_digest == EUROPE_METADATA2_SHA256
                    metadata2_sha = content_digest
                    row['status'] = 'Available: byte-pair lookup records'
                except (ValueError, KeyError, TypeError) as error:
                    metadata2 = None
                    row['status'] = f'Invalid byte-pair metadata: {error}'
            elif (manifest['profile'] == 'europe' and row['id'] == 'gameplay.linked-metadata4'
                  and row['format'] == 'linked-metadata4-v1-json'):
                try:
                    metadata4 = project_metadata4(json.loads(content))
                    metadata4['path'] = row['path']
                    metadata4_sha = content_digest
                    row['status'] = 'Available: two-pair metadata records'
                except (ValueError, KeyError, TypeError) as error:
                    metadata4 = None
                    row['status'] = f'Invalid two-pair metadata: {error}'
            elif row['id'] == 'gameplay.item-links' and row['format'] == 'item-links-v1-json':
                try:
                    item_links = project_item_links(json.loads(content))
                    item_links['path'] = row['path']
                    item_links_sha = content_digest
                    row['status'] = 'Available'
                except (ValueError, KeyError, TypeError) as error:
                    row['status'] = f'Invalid item link data: {error}'
            elif (manifest['profile'] == 'europe' and row['id'] == 'gameplay.item-effect-parameters'
                  and row['format'] == 'item-effect-parameters-v1-json'):
                try:
                    item_effects = project_item_effect_parameters(json.loads(content))
                    item_effects['path'] = row['path']
                    row['status'] = 'Available: item effect parameters'
                except (ValueError, KeyError, TypeError) as error:
                    item_effects = None
                    row['status'] = f'Invalid item effect data: {error}'
            elif (manifest['profile'] == 'europe' and row['id'] == 'gameplay.conditional-item-parameters'
                  and row['format'] == 'conditional-item-parameters-v1-json'):
                try:
                    item_parameters = project_item_parameters(json.loads(content))
                    item_parameters['path'] = row['path']
                    item_parameters_sha = content_digest
                    row['status'] = 'Available: item-derived parameter records'
                except (ValueError, KeyError, TypeError) as error:
                    item_parameters = None
                    row['status'] = f'Invalid item parameter data: {error}'
            elif (manifest['profile'] == 'europe' and row['id'] == 'gameplay.evaluation-parameters'
                  and row['format'] == 'evaluation-parameters-v1-json'):
                try:
                    evaluation_parameters = project_evaluation_parameters(json.loads(content))
                    evaluation_parameters['path'] = row['path']
                    evaluation_parameters['reader_verified'] = content_digest == EUROPE_EVALUATION_PARAMETERS_SHA256
                    row['status'] = 'Available: evaluation coefficients and thresholds'
                except (ValueError, KeyError, TypeError) as error:
                    evaluation_parameters = None
                    row['status'] = f'Invalid evaluation parameter data: {error}'
            elif (manifest['profile'] == 'europe'
                  and row['id'] in ('gameplay.progression-rewards', 'gameplay.progression-gates',
                                    'gameplay.progression-selection-gates', 'gameplay.trade-parameters')
                  and row['format'] == row['id'].removeprefix('gameplay.') + '-v1-json'):
                kind = {'gameplay.progression-rewards': 'rewards', 'gameplay.progression-gates': 'gates',
                        'gameplay.progression-selection-gates': 'selection', 'gameplay.trade-parameters': 'trade'}[row['id']]
                try:
                    progression_catalogs[kind] = project_progression_catalog(json.loads(content), kind)
                    progression_catalogs[kind]['path'] = row['path']
                    row['status'] = 'Available: progression and pricing records'
                except (ValueError, KeyError, TypeError) as error:
                    row['status'] = f'Invalid progression/pricing {kind} data: {error}'
            elif row['id'] == 'gameplay.item-records' and row['format'] == 'item-records-v1-json':
                try:
                    records = json.loads(content)['records']
                    fields = [('halfwords_00_06', 4, 0, 65535), ('bytes_08_0f', 8, 0, 255),
                              ('signed_10_17', 8, -128, 127), ('bytes_18_1f', 8, 0, 255)]
                    if not isinstance(records, list) or not records:
                        raise ValueError('No item records')
                    for record in records:
                        for name, length, minimum, maximum in fields:
                            values = record[name]
                            if not isinstance(values, list) or len(values) != length or any(
                                type(value) is not int or not minimum <= value <= maximum for value in values
                            ):
                                raise ValueError('Invalid item field ' + name)
                    items = {'records': records, 'path': row['path']}
                    item_sha = content_digest
                    row['status'] = 'Available'
                except (ValueError, KeyError, TypeError) as error:
                    row['status'] = f'Invalid item data: {error}'
            elif row['format'] == 'bg-base-palettes-v2-json':
                try:
                    entries = json.loads(content)['entries']
                    if not isinstance(entries, list) or not entries:
                        raise ValueError('No palette entries')
                    for entry in entries:
                        if not isinstance(entry['banks'], list) or not entry['banks']:
                            raise ValueError('No palette banks')
                        for bank in entry['banks']:
                            if not isinstance(bank, list) or len(bank) != 16 or any(type(v) is not int or not 0 <= v <= 65535 for v in bank):
                                raise ValueError('Invalid palette bank')
                    palettes = {'entries': entries, 'path': row['path']}
                    row['status'] = 'Available'
                except (ValueError, KeyError, TypeError) as error:
                    row['status'] = f'Invalid palette data: {error}'
            else:
                row['status'] = 'Preview unavailable'
        rows.append(row)
    if type2_pending:
        content, row = type2_pending
        try:
            projected = project_type2_rectangles(content, row['sources'], observed_tilemaps, firaga_palette_rows)
            prior_references = projected.pop('prior_references')
            if observed_tilemaps:
                for item in observed_tilemaps['maps']:
                    if item['source_offset'] in prior_references:
                        item['type2_references'] = prior_references[item['source_offset']]
            projected['path'] = row['path']
            if projected['maps']:
                extra_tilemaps['type2_rectangles'] = projected
                row['status'] = f"Available: {len(projected['maps'])} type-2 source rectangles"
                if projected['unresolved'] or projected['errors']:
                    row['status'] += f" ({len(projected['unresolved'])} unresolved entries, {len(projected['errors'])} incomplete sources)"
        except (ValueError, KeyError, TypeError) as error:
            row['status'] = f'Invalid type-2 transfer data: {error}'
    if tilemap_bank:
        if tilemap_bank['maps']:
            extra_tilemaps['stream_bank'] = {'path': tilemap_bank_row['path'], 'maps': tilemap_bank['maps']}
        if tilemap_direct:
            try:
                byte_maps = project_byte_streams(tilemap_bank['segments'], tilemap_direct)
                if byte_maps:
                    byte_tilemaps = {'path': tilemap_bank_row['path'], 'controls_path': tilemap_direct_row['path'], 'maps': byte_maps}
                    tilemap_bank_row['status'] = 'Available: decoded halfword and byte streams' if tilemap_bank['maps'] else 'Available: decoded byte streams'
                    tilemap_direct_row['status'] = 'Available: byte stream links'
            except (ValueError, KeyError, TypeError) as error:
                tilemap_direct_row['status'] = f'Invalid byte tilemap links: {error}'
    if physical_compact_pending:
        content, row = physical_compact_pending
        try:
            segments = project_binary_segments(content, row['sources'])
            physical_obj_animations = project_physical_compact(segments, physical_compact_priors)
            annotate_authenticated_contexts(physical_obj_animations, manifest.get('source_rom_sha256'), hashlib.sha256(content).hexdigest())
            physical_obj_animations['path'] = row['path'] + ' · prior source owners'
            row['status'] = 'Available: physical compact object animations'
            try:
                physical_obj_descriptors = project_physical_descriptors(segments, physical_compact_priors, physical_obj_animations)
                physical_obj_descriptors['path'] = row['path'] + ' · prior source owners'
                row['status'] += ' and descriptors'
            except (ValueError, KeyError, TypeError, AttributeError, struct.error) as error:
                row['status'] += f'; descriptor preview unavailable: {error}'
        except (ValueError, KeyError, TypeError, AttributeError, struct.error) as error:
            row['status'] = f'Preview unavailable: {error}'
    if physical_audio_pending:
        content, row = physical_audio_pending
        try:
            audio = merge_physical_audio(project_binary_segments(content, row['sources']), audio, audio_source_catalog)
            audio['physical_path'] = row['path']
            row['status'] = 'Available: additional physical tones and direct waves'
        except (ValueError, KeyError, TypeError, AttributeError, struct.error) as error:
            row['status'] = f'Preview unavailable: {error}'
    if transfer_lz:
        existing = lz_graphics['streams'] if lz_graphics else []
        if any(left['source_offset'] < right['source_offset'] + right['encoded_length']
               and right['source_offset'] < left['source_offset'] + left['encoded_length']
               for left in existing for right in transfer_lz['streams']):
            transfer_lz_row['status'] = 'Invalid transfer LZ77 data: overlaps an observed source'
        elif lz_graphics:
            lz_graphics['streams'].extend(transfer_lz['streams'])
        else:
            transfer_lz['path'] = transfer_lz_row['path']
            lz_graphics = transfer_lz
    if indexed_direct:
        if indexed_archives is None:
            indexed_archives = {'families': []}
        indexed_archives['families'].extend(indexed_direct)
    if constructor_obj_pending:
        source, row = constructor_obj_pending
        try:
            static = obj_animation_extra.get('static_lists')
            projected = project_constructor_obj_animation(source, static)
            projected['path'] = row['path'] + ' · lists: ' + static['path']
            obj_animation_extra['constructor_phases'] = projected
            row['status'] = 'Available: constructor animation phases'
        except (ValueError, KeyError, TypeError, AttributeError) as error:
            row['status'] = f'Preview unavailable: {error}'
    for resource_id, (source, row) in firaga_obj_pending.items():
        try:
            projected = project_firaga_obj_animation(source, obj_animation_extra.get('static_lists'), resource_id)
            projected['path'] = row['path'] + ' · lists: ' + obj_animation_extra['static_lists']['path']
            obj_animation_extra[FIRAGA_OBJ_SPECS[resource_id]['key']] = projected
            row['status'] = 'Available: selected Firaga animation'
        except (ValueError, KeyError, TypeError) as error:
            row['status'] = f'Preview unavailable: {error}'
    if localized_obj_pending:
        content, row = localized_obj_pending
        try:
            static = obj_animation_extra.get('static_lists')
            firaga, firaga_row = firaga_obj_pending['graphics.firaga-indexed-obj-bank']
            ui, ui_row = localized_ui_pending
            if not static or not localized_static_row:
                raise ValueError('static object lists not loaded')
            projected = project_localized_obj(project_binary_segments(content, row['sources']), static,
                localized_static_row['sources'], firaga, firaga_row['sources'], ui, ui_row['sources'])
            project_obj_animation_banks(projected)
            projected['path'] = row['path'] + ' · prior source owners'
            obj_animation_extra['localized'] = projected
            row['status'] = f"Available: {len(projected['banks'])} localized object animation banks"
            if localized_indexed_pending and indexed_archives:
                try:
                    groups, indexed_row = localized_indexed_pending
                    projected['tile_archives'] = project_localized_tile_archives(groups, indexed_archives,
                        indexed_row['sources'], projected['banks'])
                    projected['tile_archive_path'] = indexed_row['path']
                except (ValueError, KeyError, TypeError, AttributeError, struct.error) as error:
                    row['status'] += f'; linked graphics unavailable: {error}'
        except (ValueError, KeyError, TypeError, AttributeError, struct.error) as error:
            row['status'] = f'Preview unavailable: {error}'
    if screen_update_curves:
        try:
            grids = project_screen_update_grids(screen_update_curves, obj_animation_compact.get('compact_catalog'))
            if grids:
                screen_update_curves['grids'] = grids
                screen_update_row['status'] += ' and grid associations'
        except (ValueError, KeyError, TypeError, AttributeError) as error:
            screen_update_row['status'] += f'; grid preview unavailable: {error}'
    soil_animation = project_soil_evidence_animation(obj_animation_extra.get('static_lists'),
                                                       obj_animation_compact.get('compact_catalog'))
    if soil_animation:
        obj_animation_extra['soil_evidence'] = soil_animation
    if script_texts:
        used_glyphs = {unit for language in script_texts['languages'] for bank in language['banks']
                       for program in bank['programs'] for unit in program['units'] if isinstance(unit, int)}
        script_texts['glyphs'] = [font[index] for index in sorted(used_glyphs) if font and index in font]
    if script39_pending:
        object_pool, pixel_pool, sequence_pool = [], [], []
        static_lists = obj_animation_extra.get('static_lists')
        if static_lists:
            object_pool.extend((item, static_lists['path']) for item in static_lists['lists'])
        if sprite_pixels:
            pixel_pool.extend((block, sprite_pixels['path']) for block in sprite_pixels['blocks'])
            for sprite in sprite_pixels['sprites']:
                object_pool.extend(({'source_offset': frame['object_source'], 'objects': frame['objects']}, sprite_pixels['path'])
                                   for frame in sprite['frames'])
        for source in [equipment_sprites, *battle_sprites.values(), *script_sprites.values()]:
            if source:
                object_pool.extend((item, source['path']) for item in source['objects'])
                pixel_pool.extend((item, source['path']) for item in source['blocks'])
                sequence_pool.extend((item, source['path']) for item in source['sequences'])
        for source, row in script39_pending.values():
            object_pool.extend((item, row['path']) for item in source.get('objects', []))
            pixel_pool.extend((item, row['path']) for item in source['pixels'])
        for resource_id, (source, row) in script39_pending.items():
            try:
                projected = project_script39_poses(source, row['path'], object_pool, pixel_pool, sequence_pool)
                frames = [frame for sequence in projected['sequences'] for frame in sequence['frames'] if frame['operation'] == 1]
                resolved = sum(frame['block'] is not None for frame in frames)
                row['status'] = f'Available: pose sequences ({resolved}/{len(frames)} graphics frames resolved)'
                script39_poses[resource_id] = projected
            except (ValueError, KeyError, TypeError) as error:
                row['status'] = f'Invalid opcode-39 pose links: {error}'
    job_names = project_slot_job_names(slot_jobs, job_metadata, pointer_texts) if slot_jobs else None
    metadata_names = project_metadata_name_references(job_metadata, pointer_texts) if job_metadata else None
    metadata2_links = link_metadata2(manifest['profile'], metadata2_sha, job_metadata_sha, pointer_text_sha,
                                     metadata2, job_metadata, metadata_names)
    metadata4_links = link_metadata4(manifest['profile'], metadata4_sha, job_metadata_sha, pointer_text_sha,
                                     metadata4, job_metadata, metadata_names)
    metadata14_links = link_metadata14(manifest['profile'], metadata14_sha, job_metadata_sha, pointer_text_sha,
                                       battle_sprite_sha, metadata14, job_metadata, metadata_names, battle_sprites)
    special_unit_palettes = project_special_unit_palettes(manifest['profile'], color_mode_sha, color_mode_palettes)
    job_sprite_links = link_job_sprites(manifest['profile'], job_metadata_sha, job_sprite_producer_sha, pointer_text_sha,
                                        color_mode_sha, battle_sprite_sha, job_metadata, job_sprite_producers,
                                        metadata_names, battle_sprites, unit_palettes)
    encounter_job_links = link_encounter_jobs(manifest['profile'], encounter_units_sha, job_metadata_sha, metadata14_sha,
                                              pointer_text_sha, encounters.get('units'), job_metadata, metadata14, metadata_names)
    if job_sprite_links:
        job_sprite_producer_row['status'] = 'Available: job sprite references'
    item_metadata_links = project_item_metadata_links(manifest['profile'], item_links_sha, job_metadata_sha,
                                                       pointer_text_sha, item_links, metadata_names)
    item_names = project_item_names(manifest['profile'], item_sha, pointer_text_sha, items, pointer_texts)
    item_sprite_links = link_item_sprites(manifest['profile'], item_sha, equipment_sprites_sha,
                                         items, equipment_sprites, item_names)
    item_parameter_links = link_item_parameters(manifest['profile'], item_sha, item_parameters_sha,
                                                items, item_parameters, item_names)
    action_names = project_action_names(manifest['profile'], action_record_sha, pointer_text_sha, action_records, pointer_texts)
    action_records4_links = link_action_records4(manifest['profile'], action_record_sha, linked_records4_sha,
                                                  action_records, linked_records4, action_names)
    if text_reader_registry:
        text_reader_registry = link_text_reader_registry(text_reader_registry, relative_texts)
    world_location_names = project_observed_world_names(manifest['profile'], world_node_sha, pointer_text_sha,
                                                         world_nodes, pointer_texts)
    world_region_names = project_world_region_names(manifest['profile'], pointer_text_sha, pointer_texts)
    if metadata_names:
        job_metadata_row['status'] = 'Available: direct metadata name references'
    if job_names and any(any(name is not None for name in item['names']) for item in job_names['rows']):
        slot_job_row['status'] = 'Available: localized names for selected job slots'
        if job_metadata_row:
            job_metadata_row['status'] = 'Available: selected job name references'
    data = {'profile': manifest['profile'], 'extractor': manifest['extractor_revision'],
            'section_updates': viewer_section_updates(),
            'decoder': manifest['decoder_revision'], 'resources': rows, 'palettes': palettes,
            'dimensions': dimensions, 'items': items, 'item_names': item_names, 'item_sprite_links': item_sprite_links, 'item_links': item_links,
            'item_metadata_links': item_metadata_links, 'metadata2': metadata2, 'metadata2_links': metadata2_links,
            'metadata4': metadata4, 'metadata4_links': metadata4_links,
            'metadata14': metadata14, 'metadata14_links': metadata14_links, 'special_unit_palettes': special_unit_palettes,
            'item_effects': item_effects, 'item_parameters': item_parameters, 'item_parameter_links': item_parameter_links,
            'evaluation_parameters': evaluation_parameters, 'conditional_records8': conditional_records8,
            'conditional_lookups': conditional_lookups or None,
            'metadata12': metadata12, 'global_pair_flags': global_pair_flags,
            'progression_catalogs': progression_catalogs,
            'job_names': job_names, 'metadata_names': metadata_names, 'job_sprite_links': job_sprite_links,
            'encounter_job_links': encounter_job_links, 'action_records': action_records,
            'action_names': action_names, 'linked_records4': linked_records4, 'action_records4_links': action_records4_links,
            'encounters': encounters,
            'mission_events': mission_events, 'world_records': world_records, 'terrain_seeds': terrain_seeds, 'bg_plane_programs': bg_plane_programs, 'script_paths': script_paths,
            'ui_choice_lists': ui_choice_lists, 'ui_context_templates': ui_context_templates,
            'world_nodes': world_nodes, 'world_roads': world_roads, 'world_location_names': world_location_names,
            'world_region_names': world_region_names,
            'audio': audio, 'short_texts': short_texts, 'sprite_pixels': sprite_pixels,
            'equipment_sprites': equipment_sprites, 'battle_sprites': battle_sprites, 'script_sprites': script_sprites,
            'script39_poses': script39_poses,
            'unit_palettes': unit_palettes, 'color_mode_palettes': color_mode_palettes, 'script_texts': script_texts, 'pointer_texts': pointer_texts,
            'script_archive': script_archive, 'script_commands': script_commands, 'script_text_refs': script_text_refs,
            'auxiliary_script_archive': auxiliary_script_archive, 'auxiliary_script_commands': auxiliary_script_commands,
            'task_descriptors': task_descriptors, 'effect_events': effect_events,
            'relative_texts': relative_texts, 'text_reader_registry': text_reader_registry,
            'sequence_directory': sequence_directory, 'sequences': sequences,
            'timed_sound_sources': timed_sound_sources,
            'pcm_pitch': pcm_pitch, 'audio_noise': audio_noise,
            'bg_descriptors': bg_descriptors, 'bg_graphics': bg_graphics, 'bg_transfers': bg_transfers, 'alternate_palettes': alternate_palettes, 'palette_programs': palette_programs,
            'indexed_archives': indexed_archives,
            'blend_programs': blend_programs, 'raster_palettes': raster_palettes,
            'ui_graphics': ui_graphics, 'ui_palettes': ui_palettes, 'obj_graphics': obj_graphics, 'lz_graphics': lz_graphics,
            'transfer_directory': transfer_directory,
            'indexed_tilemap_controls': indexed_tilemap_controls,
            'firaga_transfer': firaga_transfer, 'firaga_palette_rows': firaga_palette_rows,
            'obj_animation_banks': obj_animation_banks,
            'obj_animation_extra': obj_animation_extra,
            'obj_animation_compact': obj_animation_compact,
            'physical_obj_animations': physical_obj_animations,
            'physical_obj_descriptors': physical_obj_descriptors,
            'observed_tilemaps': observed_tilemaps, 'firaga_compact_cells': firaga_compact_cells, 'extra_tilemaps': extra_tilemaps,
            'byte_tilemaps': byte_tilemaps,
            'signed_map_deltas': signed_map_deltas,
            'type3_payloads': type3_payloads,
            'compact_cell_bank': compact_cell_bank,
            'background_effect_graphics': background_effect_graphics,
            'background_screen_banks': background_screen_banks,
            'screen_update_curves': screen_update_curves,
            'font': {'glyphs': [font[index] for index in sorted(font)], 'path': font_path} if font else None,
            'name_keyboard': name_keyboard}
    if not physical_obj_animations:
        data['section_updates'].pop('objects-physical', None)
    template = (Path(__file__).resolve().parent.parent / 'view_rom_data.html').read_text(encoding='utf-8')
    if Path(filename).name != filename or filename in ('', '.', '..'):
        raise ValueError('Output filename must be a simple filename')
    if split:
        return write_split_viewer(data, output, filename, template, resource_files)
    data['compact_witnessed_links'] = project_witnessed_links(obj_animation_compact.get('compact_catalog'),
                                                               physical_obj_animations)
    encoded = json.dumps(data, ensure_ascii=True).replace('<', '\\u003c')
    output.mkdir(parents=True, exist_ok=True)
    page = output / filename
    page.write_text(template.replace('/* DATA */', encoded), encoding='utf-8')
    return page


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('dataset', type=Path)
    parser.add_argument('output', nargs='?', type=Path, help='output directory outside the dataset')
    parser.add_argument('--filename', help='output filename inside the chosen directory')
    parser.add_argument('--single-page', action='store_true', help='generate the legacy single-page viewer')
    args = parser.parse_args()
    try:
        output = args.output or Path(__file__).resolve().parent.parent
        filename = args.filename or ('index.html' if args.output else 'rom_data_viewer_generated.html')
        print(generate(args.dataset, output, filename, split=not args.single_page))
    except (OSError, ValueError, KeyError, tomllib.TOMLDecodeError) as error:
        parser.exit(1, f'Cannot generate viewer: {error}\n')


if __name__ == '__main__':
    main()
