"""Exercise world record fields and documented mission joins."""
import copy
import hashlib
import json
import shutil
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import view_rom_data
from test_viewer_mission_events_browser import fixture as mission_fixture
from test_viewer_scoped_browser import parse_page
from viewer_browser_harness import Chrome


def world_record(index):
    return {'index': index, 'opaque00': index, 'byte01': 11, 'byte02': 0x34, 'byte03': 0x12,
            'value04': 0x4567, 'opaque06': 0xfe,
            'thresholds07': [255 if offset == 30 else offset for offset in range(31)]}


def fixture(work, *, profile='europe', with_world=True, with_missions=True, invalid=False, split=False):
    mission_fixture(work)
    dataset = work / 'dataset'
    mission_path = dataset / 'mission-records70.json'
    missions = json.loads(mission_path.read_text())
    for record in missions['records']:
        index = 5 if record['index'] < 25 else 30
        record['packed02'] = [((index & 3) << 6) | 3, ((index >> 2) & 15) | 0xa0, 7]
    mission_path.write_text(json.dumps(missions))
    world = {'source_offset': 0x3000, 'records': [world_record(i) for i in range(31)]}
    if invalid:
        world['records'][0]['thresholds07'].pop()
    (dataset / 'world-records38.json').write_text(json.dumps(world))
    names = []
    if with_missions:
        names += ['parameter-records27', 'mission-records70', 'event-records12', 'encounter-headers']
    if with_world:
        names.append('world-records38')
    manifest = f'profile="{profile}"\nextractor_revision=1\ndecoder_revision=1\n'
    for name in names:
        content = (dataset / (name + '.json')).read_bytes()
        manifest += (f'[[resources]]\nid="gameplay.{name}"\npath="{name}.json"\n'
                     f'format="{name}-v1-json"\nsha256="{hashlib.sha256(content).hexdigest()}"\n')
    (dataset / 'manifest.toml').write_text(manifest)
    return view_rom_data.generate(dataset, work / 'world-view', split=split)


class WorldRecordProjectionTest(unittest.TestCase):
    def test_source_fields_and_validation(self):
        catalog = {'source_offset': 0x3000, 'records': [world_record(i) for i in range(31)]}
        self.assertEqual(view_rom_data.project_world_records(catalog), catalog)
        for change in ('source', 'extent', 'empty', 'too_many', 'index', 'byte', 'word', 'threshold', 'threshold_count'):
            value = copy.deepcopy(catalog)
            if change == 'source': value['source_offset'] = True
            elif change == 'extent': value['source_offset'] = 0x1fffffc
            elif change == 'empty': value['records'] = []
            elif change == 'too_many': value['records'] = [world_record(i) for i in range(65)]
            elif change == 'index': value['records'][2]['index'] = 1
            elif change == 'byte': value['records'][0]['byte01'] = -1
            elif change == 'word': value['records'][0]['value04'] = 65536
            elif change == 'threshold': value['records'][0]['thresholds07'][0] = True
            elif change == 'threshold_count': value['records'][0]['thresholds07'].pop()
            with self.subTest(change=change), self.assertRaises(ValueError):
                view_rom_data.project_world_records(value)

    def test_split_scope_and_malformed_catalog(self):
        with tempfile.TemporaryDirectory() as directory:
            page = fixture(Path(directory), split=True)
            parts = page.parent / (page.name + '.parts')
            world, index = parse_page(parts / 'world.html'), parse_page(page)
            self.assertEqual(len(world['world_records']['records']), 31)
            self.assertNotIn('world_records', parse_page(parts / 'audio.html'))
            row = next(row for row in index['resources'] if row['id'] == 'gameplay.world-records38')
            self.assertEqual(row['viewer_scope'], 'world')
            self.assertTrue(row['status'].startswith('Available'))
        with tempfile.TemporaryDirectory() as directory:
            data = parse_page(fixture(Path(directory), invalid=True))
            self.assertIsNone(data['world_records'])
            self.assertEqual(len(data['mission_events']['missions']['records']), 26)
            row = next(row for row in data['resources'] if row['id'] == 'gameplay.world-records38')
            self.assertTrue(row['status'].startswith('Invalid world record'))


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class WorldRecordBrowserTest(unittest.TestCase):
    def test_fields_thresholds_and_bidirectional_navigation(self):
        with tempfile.TemporaryDirectory() as directory:
            work = Path(directory); page = fixture(work, split=True)
            browser = Chrome(work, page.parent / (page.name + '.parts/world.html'))
            try:
                self.assertEqual(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change'));
                  if(q('#world-records').hidden || q('#world-records>details').open ||
                     q('#mission-world-navigation').hidden || q('#world-record-id').options.length!==31)throw Error('initial views');
                  q('#mission-world-open').click();
                  if(q('#world-record-id').value!=='5' || !q('#world-records>details').open ||
                     !q('#world-record-fields').textContent.includes('4660 (0x1234)') ||
                     !q('#world-record-fields').textContent.includes('359 (0x167)'))throw Error('fields');
                  if(q('#world-threshold-rows').rows.length!==31 || q('#world-threshold-rows tr:last-child meter').value!==255 ||
                     q('#world-mission-rows').rows.length!==20 || !q('#world-mission-page').textContent.includes('25 references'))throw Error('rows');
                  q('#world-mission-next').click();if(q('#world-mission-rows').rows.length!==5)throw Error('pagination');
                  q('#world-mission-filter').value='22';q('#world-mission-filter').dispatchEvent(new Event('input'));
                  if(q('#world-mission-rows').rows.length!==1 || q('#world-mission-rows a').textContent!=='22')throw Error('filter');
                  q('#world-mission-rows a').click();
                  if(q('#mission-record-id').value!=='22' || !q('#mission-record-panel').open ||
                     !q('#mission-records>details').open)throw Error('reverse');
                  q('#world-mission-filter').value='';q('#world-mission-filter').dispatchEvent(new Event('input'));
                  q('#mission-record-id').value='25';change(q('#mission-record-id'));q('#mission-world-open').click();
                  if(q('#world-record-id').value!=='30' || q('#world-mission-rows').rows.length!==1 ||
                     q('#world-mission-rows a').textContent!=='25')throw Error('different world record');
                  return 'PASS';
                })()'''), 'PASS')
            finally:
                browser.close()

    def test_missing_inputs_and_profile_gate(self):
        for kwargs in ({'with_missions': False}, {'with_world': False}, {'invalid': True}, {'profile': 'other'}):
            with self.subTest(kwargs=kwargs), tempfile.TemporaryDirectory() as directory:
                work = Path(directory); browser = Chrome(work, fixture(work, **kwargs))
                try:
                    result = browser.evaluate('''(()=>{
                      const q=s=>document.querySelector(s);
                      return {world:q('#world-records').hidden, mission:q('#mission-records').hidden,
                        link:q('#mission-world-navigation').hidden, text:q('#world-mission-page').textContent};
                    })()''')
                    if not kwargs.get('with_missions', True):
                        self.assertFalse(result['world']); self.assertTrue(result['mission'])
                        self.assertIn('unavailable', result['text'])
                    elif not kwargs.get('with_world', True) or kwargs.get('invalid'):
                        self.assertTrue(result['world']); self.assertFalse(result['mission']); self.assertTrue(result['link'])
                    else:
                        self.assertFalse(result['world']); self.assertTrue(result['link'])
                        self.assertIn('European profile', result['text'])
                finally:
                    browser.close()
