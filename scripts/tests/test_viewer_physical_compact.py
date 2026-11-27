"""Check compact list and both frame-directory widths on synthetic bytes."""
import struct
import unittest

from viewer_localized_obj import SourceBytes, encode_obj_list
from viewer_physical_compact import _directory, add_observed_contexts, annotate_authenticated_contexts, project_physical_descriptors


class PhysicalCompactDirectoryTest(unittest.TestCase):
    def test_lists_and_both_sequence_formats(self):
        memory = SourceBytes(); root = 0x1000
        attrs = (0xC0FE, 0xC1FD, 0xFFFF)
        memory.add(root, struct.pack('<HHHHHHH', 4, 6, 0, 1, *attrs))
        lists, end = _directory(memory, root, root + 14, 2, lists=True)
        self.assertEqual(end, root + 14)
        self.assertEqual([item['source_index'] for item in lists], [0, 1])
        self.assertEqual(lists[0]['objects'], [])
        self.assertEqual(encode_obj_list(lists[1]), struct.pack('<HHHH', 1, *attrs))
        narrow = SourceBytes(); narrow.add(0x2000, struct.pack('<HHHBB', 4, 8, 1, 1, 9) + struct.pack('<H', 0))
        seq, end = _directory(narrow, 0x2000, 0x200a, 2, lists=False, list_count=2)
        self.assertEqual(end, 0x200a)
        self.assertEqual(seq[0]['frames'][0], {'object_list': 1, 'list_id': 1, 'duration': 9})
        self.assertEqual(seq[1]['frames'], [])
        wide = SourceBytes(); wide.add(0x3000, struct.pack('<IHHH', 4, 1, 1, 500))
        seq, end = _directory(wide, 0x3000, 0x300a, 4, lists=False, list_count=2)
        self.assertEqual((end, seq[0]['frames'][0]['duration']), (0x300a, 500))

    def test_rejects_wrong_index_and_frame_reference(self):
        bad = SourceBytes(); bad.add(0x2000, struct.pack('<HHHBB', 4, 10, 1, 0, 1) + b'\0\0\0\0')
        with self.assertRaises(ValueError): _directory(bad, 0x2000, 0x200c, 2, lists=False, list_count=2)
        bad = SourceBytes(); bad.add(0x3000, struct.pack('<IHHH', 4, 1, 2, 10))
        with self.assertRaises(ValueError): _directory(bad, 0x3000, 0x300a, 4, lists=False, list_count=2)

    def test_contexts_are_scoped_to_documented_roots_and_selectors(self):
        def bank(root, selectors):
            return {'object_root': root + 4, 'sequences': [{'selector': index} for index in selectors]}
        data = {'banks': [bank(0x6c1e78, [5, 6]), bank(0x6babc4, [0]), bank(0x6b3428, [0])]}
        add_observed_contexts(data)
        self.assertEqual(data['banks'][0]['observed_contexts'], [{'name': 'Red Spring', 'selector': 5},
                                                                 {'name': 'Blue Screw', 'selector': 6}])
        self.assertEqual(data['banks'][1]['observed_contexts'], [{'name': 'Freezeblink', 'selector': None}])
        self.assertNotIn('observed_contexts', data['banks'][2])
        unverified = {'banks': [bank(0x6babc4, [0])]}
        self.assertFalse(annotate_authenticated_contexts(unverified, 'other-rom', 'other-source'))
        self.assertNotIn('observed_contexts', unverified['banks'][0])
        self.assertTrue(annotate_authenticated_contexts(unverified,
            '8fab1e390cb9d5e695bfdb86e81246892e1aa8c0cb4014db8235ff76538ea632',
            '975eadd350ac4295d2ac32a59251e7c8a1f074401e446168fa978075b8ca861e'))
        self.assertEqual(unverified['banks'][0]['observed_contexts'][0]['name'], 'Freezeblink')
        with self.assertRaises(ValueError): add_observed_contexts({'banks': [bank(0x6c1e78, [5])]})
        with self.assertRaises(ValueError): project_physical_descriptors([(0, 'unexpected', b'x')], {}, {'banks': []})


if __name__ == '__main__':
    unittest.main()
