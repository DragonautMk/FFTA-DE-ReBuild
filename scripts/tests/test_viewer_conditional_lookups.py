"""Exercise documented conditional lookup projections and bounds."""
import copy
import hashlib
import json
import tempfile
import unittest
from pathlib import Path

from test_viewer_scoped_browser import parse_page
from view_rom_data import generate
from viewer_conditional_lookups import CONDITIONAL_LOOKUPS


def fixture_catalogs():
    record = {'field0': 1, 'field2': 2, 'field4': 1, 'kind': 5, 'byte7': 3}
    banks, offset = [], 0x6df5b4
    for index in range(1, 24):
        count = 29 if index < 23 else 46
        banks.append({'index': index, 'source_offset': offset,
                      'records': [dict(record, field4=1 + (row % 33)) for row in range(count)]})
        offset += count * 8
    return {
        'banks': {'pointer_offset': 0x6df4d8, 'banks': banks},
        'zero': {'pointer_offset': 0x6df4d4, 'bank_root': 0x6df534, 'first_row': 1,
                 'records': [dict(record) for _ in range(15)]},
        'packed': {'source_offset': 0x6e6ee8, 'first_index': 1,
                   'records': [{'byte0': i, 'low5': i % 32, 'high3': 3, 'low2': 2, 'high6': 17, 'byte3': 7}
                               for i in range(33)]},
        'four': {'source_offset': 0x6e6dbc, 'records': [{'first': i, 'second': 65535 - i} for i in range(74)]},
        'masks': {'source_offset': 0x6e0b44, 'words': [0x80000001] + list(range(34))},
    }


def fixture(work, *, invalid=None):
    dataset = work / 'dataset'; dataset.mkdir()
    catalogs = fixture_catalogs()
    manifest = 'profile="europe"\nextractor_revision=1\ndecoder_revision=1\n'
    for identity, (format_name, key, _, _) in CONDITIONAL_LOOKUPS.items():
        value = copy.deepcopy(catalogs[key])
        if invalid == key:
            if key == 'masks': value['words'].pop()
            else: value['source_offset' if key in ('four', 'packed') else 'pointer_offset'] = 0
        content = json.dumps(value).encode(); name = key + '.json'
        (dataset / name).write_bytes(content)
        manifest += (f'[[resources]]\nid="{identity}"\npath="{name}"\nformat="{format_name}"\n'
                     f'sha256="{hashlib.sha256(content).hexdigest()}"\n')
    (dataset / 'manifest.toml').write_text(manifest)
    return generate(dataset, work / 'view', split=True), catalogs


class ConditionalLookupProjectionTest(unittest.TestCase):
    def test_each_source_is_bounded_and_preserves_stored_fields(self):
        catalogs = fixture_catalogs()
        for _, (_, key, _, project) in CONDITIONAL_LOOKUPS.items():
            with self.subTest(key=key):
                projected = project(copy.deepcopy(catalogs[key]))
                self.assertEqual(projected, catalogs[key])
        bad = copy.deepcopy(catalogs['banks']); bad['banks'][1]['source_offset'] = bad['banks'][0]['source_offset']
        with self.assertRaises(ValueError): CONDITIONAL_LOOKUPS['gameplay.conditional-descriptor-banks'][3](bad)
        bad = copy.deepcopy(catalogs['zero']); bad['records'][0]['kind'] = True
        with self.assertRaises(ValueError): CONDITIONAL_LOOKUPS['gameplay.conditional-descriptor-bank-zero'][3](bad)
        bad = copy.deepcopy(catalogs['packed']); bad['records'][0]['high3'] = 8
        with self.assertRaises(ValueError): CONDITIONAL_LOOKUPS['gameplay.conditional-packed-lookup'][3](bad)
        bad = copy.deepcopy(catalogs['four']); bad['records'].pop()
        with self.assertRaises(ValueError): CONDITIONAL_LOOKUPS['gameplay.conditional-four-byte-lookup'][3](bad)
        bad = copy.deepcopy(catalogs['masks']); bad['words'][0] = 0x100000000
        with self.assertRaises(ValueError): CONDITIONAL_LOOKUPS['gameplay.conditional-property-masks'][3](bad)

    def test_generated_scope_keeps_valid_sources_if_one_is_invalid(self):
        with tempfile.TemporaryDirectory(prefix='viewer-conditional-lookups-') as directory:
            index, _ = fixture(Path(directory), invalid='masks')
            parts = index.parent / (index.name + '.parts')
            home, world, audio = map(parse_page, (index, parts / 'world.html', parts / 'audio.html'))
            self.assertEqual(set(world['conditional_lookups']), {'banks', 'zero', 'packed', 'four'})
            self.assertNotIn('conditional_lookups', home)
            self.assertNotIn('conditional_lookups', audio)
            self.assertTrue(all(not value['reader_verified'] for value in world['conditional_lookups'].values()))
            mask = next(row for row in home['resources'] if row['id'] == 'gameplay.conditional-property-masks')
            self.assertTrue(mask['status'].startswith('Invalid conditional lookup'))
            page = next(page for page in home['wiki']['pages'] if page['id'] == 'world')
            self.assertTrue(any(section['id'] == 'conditional-lookups' for section in page['sections']))


if __name__ == '__main__':
    unittest.main()
