"""Exercise localized OBJ source joining and the custom graphics decoder."""
import hashlib
import struct
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from viewer_localized_obj import (SourceBytes, add_verified_source, decode_bank, decode_custom_lz,
                                  encode_obj_list, project_localized_tile_archives)


class LocalizedObjProjectionTest(unittest.TestCase):
    def test_source_guards_and_indexed_animation(self):
        memory = SourceBytes()
        obj = {'x': -7, 'y': -2, 'affine_mode': 0, 'object_mode': 1, 'mosaic': False, 'color_256': False,
               'shape': 2, 'transform': 16, 'size': 1, 'tile': 37, 'priority': 2, 'palette': 3}
        record = encode_obj_list({'objects': [obj]})
        source = [{'offset': 0x104, 'length': len(record), 'sha256': hashlib.sha256(record).hexdigest()}]
        memory.add(0x100, struct.pack('<I', 4)); add_verified_source(memory, source, 0x104, record)
        memory.add(0x200, struct.pack('<IHHH', 4, 1, 0, 7))
        bank = decode_bank(memory, 0x100, 0x200)
        self.assertEqual(bank['lists'][0]['objects'], [obj])
        self.assertEqual(bank['sequences'][0]['frames'], [{'object_list': 0, 'duration': 7}])
        with self.assertRaisesRegex(ValueError, 'SHA-256'): add_verified_source(memory, source, 0x104, bytes(len(record)))
        with self.assertRaisesRegex(ValueError, 'Conflicting'): memory.add(0x100, b'\xff')
        with self.assertRaisesRegex(ValueError, 'Missing'): memory.read(0x1000, 1)
        invalid = SourceBytes(); invalid.add(0x100, struct.pack('<I', 4)); invalid.add(0x104, record)
        invalid.add(0x200, struct.pack('<IHHH', 4, 1, 1, 7))
        with self.assertRaisesRegex(ValueError, 'list reference'): decode_bank(invalid, 0x100, 0x200)
        with self.assertRaisesRegex(ValueError, 'Unaligned'): decode_bank(memory, 0x101, 0x200)

    def test_custom_lz_literal_reference_fill_and_bounds(self):
        # Four literals, short backward copy, zero run and 0xff run.
        stream = b'\x00\x00\x00\x0e\x43ABCD\x80\x03\x20\x01\x02'
        self.assertEqual(decode_custom_lz(stream, 14), b'ABCDABC\x00\x00' + b'\xff' * 5)
        with self.assertRaisesRegex(ValueError, 'Trailing'): decode_custom_lz(stream + b'\x00', 14)
        with self.assertRaisesRegex(ValueError, 'Truncated'): decode_custom_lz(stream[:-1], 14)
        with self.assertRaisesRegex(ValueError, 'size'): decode_custom_lz(stream, 15)
        with self.assertRaisesRegex(ValueError, 'token'): decode_custom_lz(b'\0\0\0\3\x80\0', 3)

    def test_language_archive_links_and_source_guards(self):
        pointers = [0x400000 + i * 0x100 for i in range(5)]
        group = [{'source_offset': 0x71e658, 'pointers': pointers}]
        raw = struct.pack('<5I', *(value + 0x08000000 for value in pointers))
        sources = [{'offset': 0x71e658, 'length': len(raw), 'sha256': hashlib.sha256(raw).hexdigest()}]
        archives = [{'source_offset': pointer, 'fixed_decoded_bytes': 64,
                     'programs': [{'words': [i] * 16} for i in range(3)]} for pointer in pointers]
        catalog = {'families': [{'id': 'base', 'archives': archives}]}
        banks = [{'tile_source': {'kind': 'localized', 'language': 0, 'offset_tiles': 1, 'length_bytes': 64}}]
        output = project_localized_tile_archives(group, catalog, sources, banks)
        self.assertEqual(len(output), 5)
        self.assertEqual(output[0]['words'][8:24], [0] * 8 + [1] * 8)
        with self.assertRaisesRegex(ValueError, 'SHA-256'):
            project_localized_tile_archives(group, catalog, [{'offset': 0x71e658, 'length': 20, 'sha256': '0' * 64}], banks)
        with self.assertRaisesRegex(ValueError, 'outside'):
            project_localized_tile_archives(group, catalog, sources,
                [{'tile_source': {'kind': 'localized', 'language': 0, 'offset_tiles': 5, 'length_bytes': 64}}])
        with self.assertRaisesRegex(ValueError, 'selector'):
            project_localized_tile_archives([], catalog, sources, banks)
