#!/usr/bin/env python3
"""Generate an offline resource wiki for an extracted dataset."""
import argparse
import base64
import hashlib
import json
import os
from pathlib import Path
import re
import tempfile
import tomllib
from urllib.parse import quote

from extract_rom_data import DESCRIPTIONS


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
    ('world', 'World and interface', ('palettes', 'items', 'item_links', 'job_names', 'encounters', 'mission_events',
      'terrain_seeds', 'bg_plane_programs', 'script_paths', 'ui_choice_lists', 'world_nodes', 'world_roads', 'bg_descriptors', 'bg_graphics', 'bg_transfers',
      'alternate_palettes', 'palette_programs', 'blend_programs', 'raster_palettes', 'ui_graphics', 'ui_palettes',
      'obj_graphics', 'unit_palettes', 'observed_tilemaps', 'firaga_compact_cells', 'extra_tilemaps', 'lz_graphics',
      'firaga_transfer', 'firaga_palette_rows')),
    ('objects', 'Object animations', ('obj_animation_banks', 'obj_animation_extra', 'obj_animation_compact', 'obj_graphics',
      'ui_palettes', 'unit_palettes', 'lz_graphics')),
    ('actors', 'Actors and equipment', ('dimensions', 'sprite_pixels', 'equipment_sprites', 'unit_palettes', 'color_mode_palettes')),
    ('battle', 'Battle actors', ('battle_sprites', 'unit_palettes')),
    ('script-actors', 'Script actors', ('script_sprites', 'unit_palettes')),
    ('audio', 'Audio', ('audio', 'sequence_directory', 'sequences', 'pcm_pitch')),
    ('text', 'Text and fonts', ('short_texts', 'pointer_texts', 'relative_texts', 'script_texts', 'font', 'name_keyboard')),
    ('scripts', 'Event scripts', ('script_archive', 'script_commands', 'script_text_refs')),
    ('other', 'Other resources', ()),
)


def viewer_resource_scope(resource_id):
    if resource_id in ('engine.script-archive', 'engine.script-command-directory'):
        return 'scripts'
    if resource_id.startswith('audio.'):
        return 'audio'
    if resource_id.startswith(('text.', 'font.')):
        return 'text'
    if resource_id.startswith('graphics.unit-obj-script-'):
        return 'script-actors'
    if resource_id in ('graphics.unit-obj-normal-battle', 'graphics.unit-obj-alternate-battle'):
        return 'battle'
    if (resource_id.startswith('graphics.unit-obj-')
            or resource_id in ('graphics.obj-dimensions', 'graphics.color-mode-palettes')):
        return 'actors'
    if (resource_id in FIRAGA_OBJ_SPECS
            or resource_id.startswith(('graphics.obj-', 'graphics.conditional-compact-obj-animations'))
            or resource_id in ('graphics.word-task-obj-animations', 'graphics.layered-obj-animations',
                               'graphics.conditional-compact-obj-catalog')):
        return 'objects'
    if (resource_id.startswith(('graphics.', 'gameplay.'))
            or resource_id in ('engine.script-coordinate-paths', 'engine.ui-choice-lists')):
        return 'world'
    return 'other'


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
    return replace_region(template, "{\n  const fileInput = document.getElementById('inspect-file');",
                          '\nif (data.dimensions) {',
                          (scripts / 'viewer_stable_inspector.js').read_text(encoding='utf-8'))


RESOURCE_PAGE_LIMIT = 8 * 1024 * 1024


def write_split_viewer(data, output, filename, template, resource_files=()):
    """Emit independent offline pages, embedding only each scope's data."""
    base_keys = ('profile', 'extractor', 'decoder')
    known = set(base_keys) | {'resources'} | {key for _, _, keys in WIKI_SCOPES for key in keys}
    if set(data) - known:
        raise ValueError('Unassigned viewer data: ' + ', '.join(sorted(set(data) - known)))
    template = split_viewer_template(template)
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

    groups = {key: [] for key, _, _ in WIKI_SCOPES}
    overview = []
    for row in data['resources']:
        scope = viewer_resource_scope(row['id'])
        groups[scope].append(row)
        overview.append({**{key: row[key] for key in ('id', 'path', 'format', 'description', 'status')},
                         **({'embedded_view': row['embedded_view']} if 'embedded_view' in row else {}),
                         'viewer_scope': scope})

    def pages(current):
        page_list = [{'id': 'index', 'label': 'All resources',
                      'href': quote(filename, safe='') if current == 'index' else '../' + quote(filename, safe='')}]
        for key, label, _ in WIKI_SCOPES:
            href = quote(parts_name, safe='') + '/' + key + '.html' if current == 'index' else key + '.html'
            page_list.append({'id': key, 'label': label, 'href': href})
        directory = (quote(parts_name, safe='') + '/files/' if current == 'index' else 'files/')
        return {'current': current, 'pages': page_list, 'resource_directory': directory}

    def render(payload):
        encoded = json.dumps(payload, ensure_ascii=True).replace('<', '\\u003c')
        return template.replace('/* DATA */', encoded)

    output.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix='.viewer-pages-', dir=output) as temporary:
        stage = Path(temporary)
        staged_parts = stage / parts_name
        staged_parts.mkdir()
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
        for key, _, keys in WIKI_SCOPES:
            payload = {**base, 'resources': groups[key], **{name: data[name] for name in keys}, 'wiki': pages(key)}
            (staged_parts / (key + '.html')).write_text(render(payload), encoding='utf-8')
        (staged_parts / '.ffta-viewer-pages.json').write_text(
            json.dumps({'kind': 'ffta-viewer-pages', 'schema': 1, 'index': filename}) + '\n', encoding='utf-8')
        (staged_parts / '.gitignore').write_text('*\n', encoding='utf-8')
        (stage / filename).write_text(render({**base, 'resources': overview, 'wiki': pages('index')}), encoding='utf-8')
        parts.mkdir(exist_ok=True)
        if resource_files:
            resource_dir.mkdir(exist_ok=True)
            for path in staged_files.iterdir():
                if path.name != '.ffta-viewer-resources.json':
                    os.replace(path, resource_dir / path.name)
            os.replace(staged_files / '.ffta-viewer-resources.json', resource_marker)
            staged_files.rmdir()
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
    item_links = None
    slot_jobs = None
    job_metadata = None
    slot_job_row = None
    job_metadata_row = None
    world_nodes = None
    audio = None
    short_texts = None
    sprite_pixels = None
    equipment_sprites = None
    battle_sprites = {}
    script_sprites = {}
    unit_palettes = None
    color_mode_palettes = None
    script_texts = None
    script_text_refs = None
    script_archive = None
    script_commands = None
    pointer_texts = None
    relative_texts = None
    font = None
    font_path = None
    name_keyboard = None
    sequence_directory = None
    sequences = None
    pcm_pitch = None
    bg_descriptors = None
    bg_graphics = None
    bg_transfers = None
    alternate_palettes = None
    palette_programs = None
    blend_programs = None
    raster_palettes = None
    encounters = {}
    mission_events = {}
    terrain_seeds = None
    bg_plane_programs = None
    script_paths = None
    ui_choice_lists = None
    ui_graphics = None
    ui_palettes = None
    obj_graphics = None
    lz_graphics = None
    firaga_transfer = None
    firaga_palette_rows = None
    obj_animation_banks = None
    obj_animation_extra = {}
    obj_animation_compact = {}
    firaga_obj_pending = {}
    world_roads = None
    observed_tilemaps = None
    firaga_compact_cells = None
    extra_tilemaps = {}
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
            elif row['id'] == 'engine.ui-choice-lists' and row['format'] == 'ui-choice-lists-v1-json':
                try:
                    ui_choice_lists = project_ui_choice_lists(json.loads(content))
                    ui_choice_lists['path'] = row['path']
                    row['status'] = 'Available'
                except (ValueError, KeyError, TypeError) as error:
                    row['status'] = f'Invalid choice list data: {error}'
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
                    row['status'] = 'Available: color fragments' + (' and unit palettes' if unit_palettes else '')
                except (ValueError, KeyError, TypeError) as error:
                    color_mode_palettes = None
                    if unit_palettes:
                        row['status'] = f'Available for unit sprites; invalid color fragments: {error}'
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
            elif (row['id'] in ('graphics.unit-obj-normal-battle', 'graphics.unit-obj-alternate-battle')
                  and row['format'] == row['id'].removeprefix('graphics.') + '-v1-json'):
                mode = 'normal' if row['id'] == 'graphics.unit-obj-normal-battle' else 'alternate'
                try:
                    battle_sprites[mode] = project_equipment_sprites(json.loads(content), alternate=mode == 'alternate')
                    battle_sprites[mode]['path'] = row['path']
                    row['status'] = 'Available'
                except (ValueError, KeyError, TypeError) as error:
                    row['status'] = f'Invalid battle sprite data: {error}'
            elif row['id'] == 'graphics.unit-obj-equipment' and row['format'] == 'unit-obj-equipment-v1-json':
                try:
                    equipment_sprites = project_equipment_sprites(json.loads(content))
                    equipment_sprites['path'] = row['path']
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
                    relative_texts = None
                    row['status'] = f'Invalid pointer text data: {error}'
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
                try:
                    directory = json.loads(content)
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
                    row['status'] = 'Available for sequence preview'
                except (ValueError, KeyError, TypeError) as error:
                    sequence_directory = None
                    row['status'] = f'Invalid audio directory: {error}'
            elif row['id'] == 'audio.m4a-sequences' and row['format'] == 'audio-sequences-v1-json':
                try:
                    sequences = project_audio_sequences(json.loads(content))
                    sequences['path'] = row['path']
                    row['status'] = 'Available for sequence preview'
                except (ValueError, KeyError, TypeError) as error:
                    sequences = None
                    row['status'] = f'Invalid sequence data: {error}'
            elif row['id'] == 'audio.m4a-driver-tables' and row['format'] == 'audio-driver-tables-v1-json':
                try:
                    pcm_pitch = project_audio_pcm_pitch(json.loads(content))
                    pcm_pitch['path'] = row['path']
                    row['status'] = ('Available: PCM and CGB pitch tables' if 'cgb_keys' in pcm_pitch
                                     else 'Available: PCM pitch tables')
                except (ValueError, KeyError, TypeError) as error:
                    pcm_pitch = None
                    row['status'] = f'Invalid PCM pitch tables: {error}'
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
                    row['status'] = 'Available' if instrument_error is None else 'Available: invalid instrument metadata: ' + instrument_error
                except (ValueError, KeyError, TypeError) as error:
                    row['status'] = f'Invalid audio data: {error}'
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
                    job_metadata_row = row
                    row['status'] = 'Preview unavailable'
                except (ValueError, KeyError, TypeError) as error:
                    job_metadata = None
                    row['status'] = f'Invalid job metadata: {error}'
            elif row['id'] == 'gameplay.item-links' and row['format'] == 'item-links-v1-json':
                try:
                    item_links = project_item_links(json.loads(content))
                    item_links['path'] = row['path']
                    row['status'] = 'Available'
                except (ValueError, KeyError, TypeError) as error:
                    row['status'] = f'Invalid item link data: {error}'
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
    for resource_id, (source, row) in firaga_obj_pending.items():
        try:
            projected = project_firaga_obj_animation(source, obj_animation_extra.get('static_lists'), resource_id)
            projected['path'] = row['path'] + ' · lists: ' + obj_animation_extra['static_lists']['path']
            obj_animation_extra[FIRAGA_OBJ_SPECS[resource_id]['key']] = projected
            row['status'] = 'Available: selected Firaga animation'
        except (ValueError, KeyError, TypeError) as error:
            row['status'] = f'Preview unavailable: {error}'
    soil_animation = project_soil_evidence_animation(obj_animation_extra.get('static_lists'),
                                                       obj_animation_compact.get('compact_catalog'))
    if soil_animation:
        obj_animation_extra['soil_evidence'] = soil_animation
    if script_texts:
        used_glyphs = {unit for language in script_texts['languages'] for bank in language['banks']
                       for program in bank['programs'] for unit in program['units'] if isinstance(unit, int)}
        script_texts['glyphs'] = [font[index] for index in sorted(used_glyphs) if font and index in font]
    job_names = project_slot_job_names(slot_jobs, job_metadata, pointer_texts) if slot_jobs else None
    if job_names and any(any(name is not None for name in item['names']) for item in job_names['rows']):
        slot_job_row['status'] = 'Available: localized names for selected job slots'
        if job_metadata_row:
            job_metadata_row['status'] = 'Available: selected job name references'
    data = {'profile': manifest['profile'], 'extractor': manifest['extractor_revision'],
            'decoder': manifest['decoder_revision'], 'resources': rows, 'palettes': palettes,
            'dimensions': dimensions, 'items': items, 'item_links': item_links, 'job_names': job_names, 'encounters': encounters,
            'mission_events': mission_events, 'terrain_seeds': terrain_seeds, 'bg_plane_programs': bg_plane_programs, 'script_paths': script_paths,
            'ui_choice_lists': ui_choice_lists,
            'world_nodes': world_nodes, 'world_roads': world_roads,
            'audio': audio, 'short_texts': short_texts, 'sprite_pixels': sprite_pixels,
            'equipment_sprites': equipment_sprites, 'battle_sprites': battle_sprites, 'script_sprites': script_sprites,
            'unit_palettes': unit_palettes, 'color_mode_palettes': color_mode_palettes, 'script_texts': script_texts, 'pointer_texts': pointer_texts,
            'script_archive': script_archive, 'script_commands': script_commands, 'script_text_refs': script_text_refs,
            'relative_texts': relative_texts, 'sequence_directory': sequence_directory, 'sequences': sequences,
            'pcm_pitch': pcm_pitch,
            'bg_descriptors': bg_descriptors, 'bg_graphics': bg_graphics, 'bg_transfers': bg_transfers, 'alternate_palettes': alternate_palettes, 'palette_programs': palette_programs,
            'blend_programs': blend_programs, 'raster_palettes': raster_palettes,
            'ui_graphics': ui_graphics, 'ui_palettes': ui_palettes, 'obj_graphics': obj_graphics, 'lz_graphics': lz_graphics,
            'firaga_transfer': firaga_transfer, 'firaga_palette_rows': firaga_palette_rows,
            'obj_animation_banks': obj_animation_banks,
            'obj_animation_extra': obj_animation_extra,
            'obj_animation_compact': obj_animation_compact,
            'observed_tilemaps': observed_tilemaps, 'firaga_compact_cells': firaga_compact_cells, 'extra_tilemaps': extra_tilemaps,
            'font': {'glyphs': [font[index] for index in sorted(font)], 'path': font_path} if font else None,
            'name_keyboard': name_keyboard}
    template = (Path(__file__).resolve().parent.parent / 'view_rom_data.html').read_text(encoding='utf-8')
    if Path(filename).name != filename or filename in ('', '.', '..'):
        raise ValueError('Output filename must be a simple filename')
    if split:
        return write_split_viewer(data, output, filename, template, resource_files)
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
