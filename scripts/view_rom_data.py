#!/usr/bin/env python3
"""Generate a local HTML index and palette viewer for an extracted dataset."""
import argparse
import hashlib
import json
from pathlib import Path
import tomllib

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

    def decode(tokens, expected, base_runs=False):
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

    maps = []
    source, width, height = geometry(catalog)
    if catalog['mode'] != 4:
        raise ValueError('Unsupported tilemap mode')
    maps.append({'kind': 'Tilemap', 'source_offset': source, 'width': width, 'height': height,
                 'cells': decode(catalog['runs'], width * height, True)})
    streams = catalog.get('additional_streams') or []
    if not isinstance(streams, list):
        raise ValueError('Invalid additional tilemaps')
    for stream in streams:
        source, width, height = geometry(stream)
        if stream['mode'] != 4:
            raise ValueError('Unsupported tilemap mode')
        maps.append({'kind': 'Tilemap', 'source_offset': source, 'width': width, 'height': height,
                     'cells': decode(stream['tokens'], width * height)})
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
    return {'keys': [{'base': key['base'], 'shift': key['shift']} for key in keys], 'bases': bases}


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


def generate(dataset, output, filename='index.html'):
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
    world_nodes = None
    audio = None
    short_texts = None
    sprite_pixels = None
    equipment_sprites = None
    battle_sprites = {}
    script_sprites = {}
    unit_palettes = None
    script_texts = None
    pointer_texts = None
    relative_texts = None
    font = None
    font_path = None
    name_keyboard = None
    sequence_directory = None
    sequences = None
    pcm_pitch = None
    bg_graphics = None
    ui_graphics = None
    ui_palettes = None
    obj_graphics = None
    world_roads = None
    observed_tilemaps = None
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
            if hashlib.sha256(content).hexdigest() != row['sha256']:
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
            elif (row['id'] == 'graphics.observed-tilemap'
                  and row['format'] in {f'observed-tilemap-v{revision}-json' for revision in range(1, 5)}):
                try:
                    observed_tilemaps = {'maps': project_observed_tilemaps(json.loads(content)), 'path': row['path']}
                    row['status'] = 'Available for tilemap preview'
                except (ValueError, KeyError, TypeError) as error:
                    observed_tilemaps = None
                    row['status'] = f'Invalid tilemap data: {error}'
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
                    spans = json.loads(content)['spans']
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
                    row['status'] = 'Available'
                except (ValueError, KeyError, TypeError) as error:
                    unit_palettes = None
                    row['status'] = f'Invalid color mode data: {error}'
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
                    row['status'] = 'Available: PCM pitch tables'
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
    if script_texts:
        used_glyphs = {unit for language in script_texts['languages'] for bank in language['banks']
                       for program in bank['programs'] for unit in program['units'] if isinstance(unit, int)}
        script_texts['glyphs'] = [font[index] for index in sorted(used_glyphs) if font and index in font]
    data = {'profile': manifest['profile'], 'extractor': manifest['extractor_revision'],
            'decoder': manifest['decoder_revision'], 'resources': rows, 'palettes': palettes,
            'dimensions': dimensions, 'items': items, 'item_links': item_links, 'world_nodes': world_nodes, 'world_roads': world_roads,
            'audio': audio, 'short_texts': short_texts, 'sprite_pixels': sprite_pixels,
            'equipment_sprites': equipment_sprites, 'battle_sprites': battle_sprites, 'script_sprites': script_sprites,
            'unit_palettes': unit_palettes, 'script_texts': script_texts, 'pointer_texts': pointer_texts,
            'relative_texts': relative_texts, 'sequence_directory': sequence_directory, 'sequences': sequences,
            'pcm_pitch': pcm_pitch,
            'bg_graphics': bg_graphics, 'ui_graphics': ui_graphics, 'ui_palettes': ui_palettes, 'obj_graphics': obj_graphics,
            'observed_tilemaps': observed_tilemaps,
            'font': {'glyphs': [font[index] for index in sorted(font)], 'path': font_path} if font else None,
            'name_keyboard': name_keyboard}
    encoded = json.dumps(data, ensure_ascii=True).replace('<', '\\u003c')
    template = (Path(__file__).resolve().parent.parent / 'view_rom_data.html').read_text(encoding='utf-8')
    output.mkdir(parents=True, exist_ok=True)
    if Path(filename).name != filename or filename in ('', '.', '..'):
        raise ValueError('Output filename must be a simple filename')
    page = output / filename
    page.write_text(template.replace('/* DATA */', encoded), encoding='utf-8')
    return page


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('dataset', type=Path)
    parser.add_argument('output', nargs='?', type=Path, help='output directory outside the dataset')
    parser.add_argument('--filename', help='output filename inside the chosen directory')
    args = parser.parse_args()
    try:
        output = args.output or Path(__file__).resolve().parent.parent
        filename = args.filename or ('index.html' if args.output else 'rom_data_viewer_generated.html')
        print(generate(args.dataset, output, filename))
    except (OSError, ValueError, KeyError, tomllib.TOMLDecodeError) as error:
        parser.exit(1, f'Cannot generate viewer: {error}\n')


if __name__ == '__main__':
    main()
