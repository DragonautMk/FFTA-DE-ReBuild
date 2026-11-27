"""Match compact sequences by their source and displayed contents."""
import copy
import unittest

from viewer_physical_compact import project_witnessed_links


def catalogs():
    obj = {'x': 2, 'tile': 5}
    def item(offset, index, value):
        return {'source_offset': offset, 'source_index': index, 'relative_offset': offset - 0x1004, 'objects': [value]}
    def seq(selector, index):
        return {'selector': selector, 'source_offset': 0x1100 + selector * 4, 'relative_offset': selector * 4,
                'frames': [{'object_list': index, 'list_id': 2, 'duration': 3}]}
    old = {'object_root': 0x1004, 'sequence_root': 0x1100, 'flags': 12,
           'lists': [item(0x1030, 2, obj)], 'sequences': [seq(0, 0), seq(6, 0)]}
    new = {'object_root': 0x1004, 'sequence_root': 0x1100, 'flags': 12,
           'lists': [item(0x1010, 0, {}), item(0x1020, 1, {}), item(0x1030, 2, copy.deepcopy(obj))],
           'sequences': [seq(0, 2), seq(6, 2)]}
    return {'banks': [old]}, {'banks': [new]}


class WitnessedCompactLinksTest(unittest.TestCase):
    def test_sparse_and_physical_list_indices_can_differ(self):
        old, new = catalogs()
        self.assertEqual(project_witnessed_links(old, new), [{'root': 0x1000, 'selector': 0},
                                                             {'root': 0x1000, 'selector': 6}])
        self.assertEqual(project_witnessed_links(None, new), [])

    def test_source_frame_and_object_mismatches_are_not_linked(self):
        mutations = [lambda bank: bank.update(flags=4),
                     lambda bank: bank['sequences'][0].update(source_offset=42),
                     lambda bank: bank['sequences'][0]['frames'][0].update(duration=4),
                     lambda bank: bank['sequences'][0]['frames'][0].update(list_id=1),
                     lambda bank: bank['sequences'][0]['frames'][0].update(object_list=-1),
                     lambda bank: bank['lists'][2].update(source_index=1),
                     lambda bank: bank['lists'][2]['objects'][0].update(tile=99)]
        for mutate in mutations:
            old, new = catalogs(); mutate(new['banks'][0])
            self.assertNotIn({'root': 0x1000, 'selector': 0}, project_witnessed_links(old, new))

    def test_ambiguous_roots_and_selectors_are_not_linked(self):
        old, new = catalogs(); new['banks'].append(copy.deepcopy(new['banks'][0]))
        self.assertEqual(project_witnessed_links(old, new), [])
        old, new = catalogs(); new['banks'][0]['sequences'].append(copy.deepcopy(new['banks'][0]['sequences'][0]))
        self.assertEqual(project_witnessed_links(old, new), [{'root': 0x1000, 'selector': 6}])


if __name__ == '__main__':
    unittest.main()
