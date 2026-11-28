"""Source-bound European screen-effect motion and scanline tables."""
import struct


EFFECT_SHA256 = 'd5c84bfa65261cb7be9607621859d68beb3dcb8895c207010fbcdf1a5faa1402'
LITERALS = [
    (0x122904, (0x3ac51e, 0x3ac534, 0x3ac54a)), (0x122914, (0x3ac508, 0x3ac560)),
    (0x122a18, (0x3ac580, 0x3ac5c0, 0x3ac600)), (0x122a28, (0x3ac640,)),
    (0x122d04, (0x3ac4dc,)), (0x1231fc, (0x3ac680,)), (0x123264, (0x3ac680,)),
    (0x1233f8, (0x3ac6a8,)), (0x123858, (0x3ac7b8,)), (0x1239ac, (0x3ac860,)),
    (0x1239f4, (0x3ac860,)), (0x123c48, (0x3ac928, 0x3ac900)),
    (0x123c58, (0x3ac914, 0x3ac92d, 0x3ac938)), (0x123dec, (0x3ac932, 0x3ac92d, 0x3ac938)),
]
TABLES = [
    (0x3ac4dc, 'sprite_spawn_times', 44), (0x3ac508, 'first_sprite_selectors', 88),
    (0x3ac560, 'first_sprite_motion', 32), (0x3ac580, 'second_sprite_selectors', 192),
    (0x3ac640, 'second_sprite_motion', 64), (0x3ac680, 'offset_curve', 40),
    (0x3ac6a8, 'timed_motion_records', 140), (0x3ac7b8, 'scale_curve', 168),
    (0x3ac860, 'scanline_profile', 160), (0x3ac900, 'background_motion', 40),
    (0x3ac928, 'background_selectors', 15), (0x3ac938, 'background_delay', 10),
]


def project(segments):
    expected = [(offset, 'reader_literals', 4 * len(values)) for offset, values in LITERALS] + TABLES
    if [(offset, role, len(raw)) for offset, role, raw in segments] != expected:
        raise ValueError('Unexpected screen-effect source ranges')
    for index, (_, pointers) in enumerate(LITERALS):
        actual = struct.unpack('<' + 'I' * len(pointers), segments[index][2])
        if actual != tuple(0x08000000 + pointer for pointer in pointers):
            raise ValueError('Invalid screen-effect reader literals')
    tables = {role: raw for _, role, raw in segments[len(LITERALS):]}
    ticks = struct.unpack('<22H', tables['sprite_spawn_times'])
    banks = []
    for group, (row_count, count, selector_role, motion_role, selector_base, motion_base) in enumerate((
            (22, 4, 'first_sprite_selectors', 'first_sprite_motion', 0x3ac508, 0x3ac560),
            (64, 8, 'second_sprite_selectors', 'second_sprite_motion', 0x3ac580, 0x3ac640))):
        raw, motion_raw = tables[selector_role], tables[motion_role]
        columns = [raw[i * row_count:(i + 1) * row_count] for i in range(4 if group == 0 else 3)]
        motions = []
        for index in range(count):
            record = motion_raw[index * 8:index * 8 + 8]
            motions.append({'source_offset': motion_base + index * 8, 'duration': record[0],
                            'delta': list(struct.unpack_from('<hh', record, 2)), 'bytes': list(record)})
        rows = []
        for index in range(row_count):
            cells = [column[index] for column in columns]
            kind, selector = (cells[0], cells[1:]) if group == 0 else (None, cells)
            if selector[0] >= count:
                raise ValueError('Invalid sprite motion selector')
            rows.append({'index': index, 'tick': ticks[index] if group == 0 else None,
                         'kind': kind, 'motion': selector[0], 'position': selector[1:],
                         'source_offsets': [selector_base + i * row_count + index for i in range(len(columns))]})
        banks.append({'rows': rows, 'motions': motions})
    timed = []
    raw = tables['timed_motion_records']
    for index in range(7):
        record = raw[index * 20:index * 20 + 20]
        pairs = [list(struct.unpack_from('<hh', record, 4 + axis * 4)) for axis in range(3)]
        duration = struct.unpack_from('<H', record, 16)[0]
        timed.append({'source_offset': 0x3ac6a8 + 20 * index, 'tick': struct.unpack_from('<H', record)[0],
                      'parameter': record[2], 'pairs': pairs, 'stored_duration': duration,
                      'duration': (duration - 1) & 0xffff, 'bytes': list(record)})
    background = []
    motion, selectors, delays = tables['background_motion'], tables['background_selectors'], tables['background_delay']
    for index in range(5):
        start = list(struct.unpack_from('<hh', motion, 4 * index))
        end = list(struct.unpack_from('<hh', motion, 20 + 4 * index))
        background.append({'index': index, 'start_offset': 0x3ac900 + 4 * index,
                           'end_offset': 0x3ac914 + 4 * index, 'start': start, 'end': end,
                           'selector': selectors[index], 'width': selectors[5 + index],
                           'advance': selectors[10 + index], 'delay': struct.unpack_from('<h', delays, 2 * index)[0],
                           'trigger_tick': 260 - selectors[10 + index]})
    return {'sprite_banks': banks, 'timed': timed, 'background': background,
            'curves': [{'label': 'Offset samples', 'source_offset': 0x3ac680, 'stride': 2,
                        'values': list(struct.unpack('<20h', tables['offset_curve']))},
                       {'label': 'Scale samples', 'source_offset': 0x3ac7b8, 'stride': 2,
                        'values': list(struct.unpack('<84h', tables['scale_curve']))},
                       {'label': 'Scanline profile bytes', 'source_offset': 0x3ac860, 'stride': 1,
                        'values': list(tables['scanline_profile'])}]}
