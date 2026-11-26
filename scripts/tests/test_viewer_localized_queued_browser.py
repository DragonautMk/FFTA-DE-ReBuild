"""Check localized queued rectangles, byte order, and manual preview navigation."""
import copy
import hashlib
import json
import shutil
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from view_rom_data import generate, project_localized_queued_grids
from viewer_browser_harness import Chrome


ROOTS = (0x438d00, 0x449f00, 0x45baf0, 0x46cd8c, 0x47e434)


def fixture(work, *, profile='europe', include_queued=True):
    dataset = work / 'dataset'; dataset.mkdir()
    banks, pointers = [], []
    for language, root in enumerate(ROOTS):
        halves = [(language * 700 + index) & 0xffff for index in range(576)]
        words = [halves[index] | halves[index + 1] << 16 for index in range(0, 576, 2)]
        banks.append({'bank_offset': root, 'relative_offset': 0, 'words': words})
        pointers.extend(0x08000000 + root + index * 192 for index in range(6))
    queued = {'pointers': pointers, 'lengths': [192] * 30, 'literals': [0x08397fbc, 0x08398034], 'banks': banks}
    catalog = {'rows': [], 'banks': [], 'bank_starts': [], 'localized_queued': queued if include_queued else None}
    content = json.dumps(catalog).encode(); (dataset / 'dynamic-raw-fragments.json').write_bytes(content)
    manifest = (f'profile="{profile}"\nextractor_revision=1\ndecoder_revision=1\n'
                '[[resources]]\nid="graphics.dynamic-raw-fragments"\npath="dynamic-raw-fragments.json"\n'
                'format="dynamic-raw-fragments-v3-json"\n'
                f'sha256="{hashlib.sha256(content).hexdigest()}"\n')
    (dataset / 'manifest.toml').write_text(manifest)
    return dataset, catalog


def payload(page):
    return json.JSONDecoder().raw_decode(page.read_text().split('const data = ', 1)[1])[0]


class LocalizedQueuedProjectionTest(unittest.TestCase):
    def test_grid_byte_order_source_and_scope(self):
        with tempfile.TemporaryDirectory(prefix='viewer-localized-grid-') as directory:
            work = Path(directory); dataset, catalog = fixture(work)
            index = generate(dataset, work / 'view', split=True)
            world = payload(index.parent / 'index.html.parts/world.html')
            maps = world['extra_tilemaps']['localized_queued']['maps']
            self.assertEqual(len(maps), 30)
            for language in range(5):
                for entry in range(6):
                    item = maps[language * 6 + entry]
                    self.assertEqual((item['source_offset'], item['width'], item['height']),
                                     (ROOTS[language] + entry * 192, 24, 4))
                    self.assertEqual(item['cells'], [(language * 700 + entry * 96 + i) & 0xffff for i in range(96)])
                    self.assertEqual((item['metadata']['source_row_bytes'], item['metadata']['destination_row_bytes']), (48, 64))
            self.assertNotIn('extra_tilemaps', payload(index))
            self.assertNotIn('extra_tilemaps', payload(index.parent / 'index.html.parts/audio.html'))
            self.assertEqual(next(row['status'] for row in world['resources']
                                  if row['id'] == 'graphics.dynamic-raw-fragments'), 'Available: localized transfer rectangles')
            mutations = [lambda c: c['localized_queued']['pointers'].pop(),
                         lambda c: c['localized_queued']['pointers'].__setitem__(0, 0),
                         lambda c: c['localized_queued']['lengths'].__setitem__(0, True),
                         lambda c: c['localized_queued']['literals'].__setitem__(0, 0),
                         lambda c: c['localized_queued']['banks'][0].update(bank_offset=0),
                         lambda c: c['localized_queued']['banks'][0].update(relative_offset=True),
                         lambda c: c['localized_queued']['banks'][0]['words'].pop(),
                         lambda c: c['localized_queued']['banks'][0]['words'].__setitem__(0, -1)]
            for mutate in mutations:
                bad = copy.deepcopy(catalog); mutate(bad)
                with self.assertRaises(ValueError): project_localized_queued_grids(bad)

    def test_missing_and_other_profile_do_not_infer_data(self):
        for profile, include in [('europe', False), ('usa', True)]:
            with self.subTest(profile=profile), tempfile.TemporaryDirectory(prefix='viewer-localized-partial-') as directory:
                work = Path(directory); dataset, _ = fixture(work, profile=profile, include_queued=include)
                index = generate(dataset, work / 'view', split=True)
                world = payload(index.parent / 'index.html.parts/world.html')
                self.assertNotIn('localized_queued', world['extra_tilemaps'])
                self.assertTrue(next(row['status'] for row in world['resources']
                                     if row['id'] == 'graphics.dynamic-raw-fragments').startswith('Preview unavailable'))


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class LocalizedQueuedBrowserTest(unittest.TestCase):
    def test_resource_link_values_metadata_and_manual_preview(self):
        with tempfile.TemporaryDirectory(prefix='viewer-localized-grid-browser-') as directory:
            work = Path(directory); dataset, _ = fixture(work)
            index = generate(dataset, work / 'view', split=True)
            browser = Chrome(work, index.parent / 'index.html.parts/world.html')
            try:
                self.assertEqual(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change'));
                  const row=q('[data-resource-id="graphics.dynamic-raw-fragments"]');
                  row.querySelector('a[href="#tilemaps"]').click();
                  if(q('#tilemaps').hidden || !q('#tilemaps>details').open || q('#tilemap-id').options.length!==30 ||
                     !q('#tilemap-id').selectedOptions[0].textContent.includes('language 0 · entry 0'))throw Error('source link');
                  q('#tilemap-id').value='29';change(q('#tilemap-id'));
                  if(!q('#tilemap-id').selectedOptions[0].textContent.includes('language 4 · entry 5') ||
                     q('#tilemap-cells').rows.length!==4 || q('#tilemap-cells').rows[0].querySelectorAll('button').length!==24 ||
                     q('#tilemap-canvas').width!==192 || q('#tilemap-canvas').height!==32)throw Error('rectangle dimensions');
                  const expected=4*700+5*96;
                  if(q('#tilemap-cells').rows[0].querySelector('button').textContent!=='0x'+expected.toString(16).toUpperCase().padStart(4,'0') ||
                     !q('#tilemap-source-fields').textContent.includes('"destination_row_bytes":64'))throw Error('source values');
                  const pixel=[...q('#tilemap-canvas').getContext('2d').getImageData(0,0,1,1).data],shade=(expected&15)*17;
                  if(pixel.join(',')!==[shade,shade,shade,255].join(','))throw Error('manual preview');
                  q('#tilemap-cells').rows[3].querySelectorAll('button')[2].click();
                  if(!q('#tilemap-cell-detail').textContent.includes('Cell (2, 3)'))throw Error('cell selection');
                  if(!q('#firaga-cell-controls').hidden || !q('#tilemap-template-controls').hidden)throw Error('unrelated controls');
                  return 'PASS';
                })()'''), 'PASS')
            finally:
                browser.close()
