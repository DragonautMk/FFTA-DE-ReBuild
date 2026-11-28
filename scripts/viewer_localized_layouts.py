"""Source-backed projections of the European localized layout tables."""
import struct


LITERALS = (
    (0x092850, 12), (0x093990, 4), (0x0939d0, 4), (0x0939d8, 4), (0x0939e0, 4), (0x0939e8, 4),
    (0x093a18, 4), (0x093a30, 4), (0x093a38, 4), (0x093a40, 4), (0x093a48, 4), (0x093a7c, 4),
    (0x093a94, 4), (0x093a9c, 4), (0x093aa4, 4), (0x093aac, 4), (0x093ad4, 4), (0x093b24, 4),
    (0x093b2c, 4), (0x093b34, 4), (0x093b3c, 4), (0x093c18, 4), (0x093cd4, 4), (0x093d14, 4),
    (0x093d1c, 4), (0x093d24, 4), (0x093d2c, 4), (0x093d5c, 4), (0x093d74, 4), (0x093d7c, 4),
    (0x093d84, 4), (0x093d8c, 4), (0x093dc0, 4), (0x093dd8, 4), (0x093de0, 4), (0x093de8, 4),
    (0x093df0, 4), (0x093e18, 4), (0x093e68, 4), (0x093e70, 4), (0x093e78, 4), (0x093e80, 4),
    (0x093f3c, 4), (0x093ffc, 4), (0x09403c, 4), (0x094044, 4), (0x09404c, 4), (0x094054, 4),
    (0x094084, 4), (0x09409c, 4), (0x0940a4, 4), (0x0940ac, 4), (0x0940b4, 4), (0x0940e8, 4),
    (0x094100, 4), (0x094108, 4), (0x094110, 4), (0x094118, 4), (0x094140, 4), (0x09418c, 4),
    (0x094194, 4), (0x09419c, 4), (0x0941a4, 4), (0x09427c, 4), (0x094340, 4), (0x094380, 4),
    (0x094388, 4), (0x094390, 4), (0x094398, 4), (0x0943c8, 4), (0x0943e0, 4), (0x0943e8, 4),
    (0x0943f0, 4), (0x0943f8, 4), (0x09442c, 4), (0x094444, 4), (0x09444c, 4), (0x094454, 4),
    (0x09445c, 4), (0x094484, 4), (0x0944cc, 4), (0x0944d4, 4), (0x0944dc, 4), (0x0944e4, 4),
    (0x0945bc, 4), (0x09578c, 4),
)

# The four original builders each select one of five bounded language tables.
FAMILIES = (
    (0x0938fc, (14, 13, 19, 16, 16), (0x398238, 0x39849c, 0x39870c, 0x3989e8, 0x398c58)),
    (0x093c48, (11, 8, 16, 10, 10), (0x3982e0, 0x398538, 0x3987f0, 0x398aa8, 0x398d18)),
    (0x093f60, (15, 17, 15, 17, 16), (0x39810c, 0x398364, 0x398598, 0x3988b0, 0x398b20)),
    (0x0942b8, (10, 9, 16, 9, 10), (0x3981c0, 0x398430, 0x39864c, 0x39897c, 0x398be0)),
)


def project_localized_layouts(segments):
    """Decode the verified ranges without identifying scene or language names."""
    expected = ([(offset, 'reader_literals', length) for offset, length in LITERALS]
                + [(0x3980c0, 'allocation_sizes', 20), (0x3980d4, 'object_lists', 54),
                   (0x39810c, 'layout_rows', 3204), (0x398d90, 'layout_parameters', 20),
                   (0x398da4, 'transition_rows', 68)])
    if [(offset, role, len(raw)) for offset, role, raw in segments] != expected:
        raise ValueError('Unexpected localized layout source ranges')
    targets = [root for _, _, roots in FAMILIES for root in roots for _ in range(4)]
    targets += [0x3980d4, 0x3980ee, 0x3980c0, 0x398d90, 0x398d90, 0x398d90, 0x398d90, 0x398da4]
    words = [value - 0x08000000 for _, _, raw in segments[:-5] for (value,) in struct.iter_unpack('<I', raw)]
    if sorted(words) != sorted(targets):
        raise ValueError('Invalid localized layout reader literals')
    allocations = list(struct.unpack('<5I', segments[-5][2]))
    object_bytes, row_bytes = segments[-4][2], segments[-3][2]
    object_lists = []
    for offset, count in ((0x3980d4, 4), (0x3980ee, 3), (0x398102, 1)):
        position = offset - 0x3980d4
        if struct.unpack_from('<H', object_bytes, position)[0] != count:
            raise ValueError('Invalid localized object list count')
        objects = [list(struct.unpack_from('<3H', object_bytes, position + 2 + 6 * index)) for index in range(count)]
        object_lists.append({'source_offset': offset, 'objects': objects})
    families, ranges = [], []
    for builder, counts, roots in FAMILIES:
        layouts = []
        for language, (offset, count) in enumerate(zip(roots, counts)):
            begin, end = offset - 0x39810c, offset - 0x39810c + 12 * count
            if begin < 0 or end > len(row_bytes):
                raise ValueError('Localized layout exceeds source')
            records = [list(row) for row in struct.iter_unpack('<I4H', row_bytes[begin:end])]
            if len(records) != count or any(row[0] != 0x08398102 or row[2] != 6 or row[4] != 0 for row in records):
                raise ValueError('Invalid localized layout row')
            layouts.append({'language_id': language, 'source_offset': offset, 'rows': records})
            ranges.append((begin, end))
        families.append({'builder_offset': builder, 'layouts': layouts})
    ordered = sorted(ranges)
    if not ordered or ordered[0][0] != 0 or ordered[-1][1] != len(row_bytes) or any(a[1] != b[0] for a, b in zip(ordered, ordered[1:])):
        raise ValueError('Localized layout source is not completely covered')
    parameters = [list(row) for row in struct.iter_unpack('<4B', segments[-2][2])]
    transitions = [list(row) for row in struct.iter_unpack('<4B', segments[-1][2])]
    if transitions != [[16 - i, i, 0, 0] for i in range(17)]:
        raise ValueError('Invalid localized transition rows')
    return {'families': families, 'object_lists': object_lists, 'allocations': allocations,
            'parameters': parameters, 'transitions': transitions, 'transition_offset': 0x398da4}
