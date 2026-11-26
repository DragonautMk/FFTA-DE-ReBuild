"""Check the UI source grids and their documented staged/direct forms."""
import copy
import hashlib
import json
import shutil
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from view_rom_data import generate, project_ui_tilemap_rectangles
from viewer_browser_harness import Chrome


def catalog():
    shapes = [(0x3b0e5c, 10, 8), (0x3b0efc, 12, 3), (0x3ba95c, 22, 17)]
    rectangles = []
    for offset, width, height in shapes:
        cells = list(range(width * height))
        cells[0] = 0xffff
        rectangles.append({'width': width, 'height': height,
                           'bank': {'source_offset': offset, 'halfwords': cells}})
    return {'halfword_copies': {'tilemap_rectangles': {
        'caller_literals': [0x083b0e5c, 0x083b0efc, 0x083b0e5c, 0x083b0efc, 0x0871e66c, 0x083ba95c],
        'rectangles': rectangles}}}


def fixture(work, value=None, *, observed=False):
    dataset = work / 'dataset'; dataset.mkdir()
    items = [('graphics.ui-raw-transfers', 'ui-raw-transfers', 1, catalog() if value is None else value)]
    if observed:
        items.append(('graphics.observed-tilemap', 'observed-tilemap', 1,
                      {'source_offset': 2000, 'width': 1, 'height': 1, 'mode': 4,
                       'runs': [{'control': 0x4001, 'value': 0}]}))
    manifest = 'profile="test"\nextractor_revision=1\ndecoder_revision=1\n'
    for rid, name, version, value in items:
        content = json.dumps(value).encode(); (dataset / (name + '.json')).write_bytes(content)
        manifest += (f'[[resources]]\nid="{rid}"\npath="{name}.json"\nformat="{name}-v{version}-json"\n'
                     f'sha256="{hashlib.sha256(content).hexdigest()}"\n')
    (dataset / 'manifest.toml').write_text(manifest)
    index = generate(dataset, work / 'view', split=True)
    return index, index.parent / (index.name + '.parts/world.html')


def payload(path):
    return json.JSONDecoder().raw_decode(path.read_text().split('const data = ', 1)[1])[0]


class UiRectangleProjectionTest(unittest.TestCase):
    def test_preserves_exact_cells_and_shapes(self):
        value = catalog(); projected = project_ui_tilemap_rectangles(value)
        originals = value['halfword_copies']['tilemap_rectangles']['rectangles']
        self.assertEqual([item['cells'] for item in projected['maps']], [item['bank']['halfwords'] for item in originals])
        self.assertEqual([item['ui_template'] for item in projected['maps']], [True, True, False])
        self.assertEqual([(item['width'], item['height']) for item in projected['maps']], [(10, 8), (12, 3), (22, 17)])
        self.assertEqual(value, catalog())

    def test_rejects_bad_pointers_shapes_and_cells(self):
        mutations = [lambda c: c['halfword_copies']['tilemap_rectangles']['caller_literals'].__setitem__(0, 0),
                     lambda c: c['halfword_copies']['tilemap_rectangles']['rectangles'][1].__setitem__('width', 14),
                     lambda c: c['halfword_copies']['tilemap_rectangles']['rectangles'][1]['bank']['halfwords'].pop(),
                     lambda c: c['halfword_copies']['tilemap_rectangles']['rectangles'][0]['bank']['halfwords'].__setitem__(0, True),
                     lambda c: c['halfword_copies']['tilemap_rectangles']['rectangles'][2]['bank'].__setitem__('source_offset', 0)]
        for mutate in mutations:
            value = copy.deepcopy(catalog()); mutate(value)
            with self.subTest(mutation=mutate), self.assertRaises(ValueError):
                project_ui_tilemap_rectangles(value)

    def test_world_scope_legacy_and_invalid_isolation(self):
        for kind in ('valid', 'legacy', 'invalid'):
            value = catalog()
            if kind == 'legacy': value['halfword_copies']['tilemap_rectangles'] = None
            if kind == 'invalid': value['halfword_copies']['tilemap_rectangles']['rectangles'][0]['width'] = 2
            with self.subTest(kind=kind), tempfile.TemporaryDirectory(prefix='viewer-ui-rect-scope-') as directory:
                index, world = fixture(Path(directory), value, observed=True)
                data = payload(world)
                self.assertNotIn('extra_tilemaps', payload(index))
                self.assertNotIn('extra_tilemaps', payload(index.parent / (index.name + '.parts/audio.html')))
                self.assertEqual('ui_rectangles' in data['extra_tilemaps'], kind == 'valid')
                self.assertTrue(data['observed_tilemaps'])
                self.assertEqual(data['resources'][0]['status'].startswith('Invalid'), kind == 'invalid')


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class UiRectangleBrowserTest(unittest.TestCase):
    def test_stored_staged_direct_padding_and_navigation(self):
        with tempfile.TemporaryDirectory(prefix='viewer-ui-rect-browser-') as directory:
            work = Path(directory); _, world = fixture(work); browser = Chrome(work, world)
            try:
                self.assertEqual(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change'));
                  const row=q('[data-resource-id="graphics.ui-raw-transfers"]');
                  row.querySelector('a[href="#tilemaps"]').click();
                  const map=q('#tilemap-id'),mode=q('#tilemap-template-view'),canvas=q('#tilemap-canvas');
                  const cells=()=>[...q('#tilemap-cells').querySelectorAll('button')];
                  if(!q('#tilemaps>details').open||map.options.length!==3||map.value!=='0'||
                     q('#tilemap-template-controls').hidden||canvas.width!==80||canvas.height!==64||
                     cells().length!==80||cells()[0].textContent!=='0xFFFF')throw Error('stored grid');
                  mode.value='staged';change(mode);
                  if(cells()[0].textContent!=='0x00BF'||!q('#tilemap-cell-detail').textContent.includes('stored 0xFFFF')||
                     !q('#tilemap-description').textContent.includes('staged interpretation'))throw Error('wrapping staged');
                  map.value='1';change(map);
                  if(mode.value!=='staged'||canvas.width!==112||canvas.height!==24||cells().length!==42||
                     cells()[12].textContent!=='0x0000'||cells()[13].textContent!=='0x0000'||cells()[14].textContent!=='0x00CC')throw Error('padding');
                  cells()[12].click();if(!q('#tilemap-cell-detail').textContent.includes('added zero column'))throw Error('padding label');
                  mode.value='direct';change(mode);
                  if(canvas.width!==96||cells().length!==36||cells()[0].textContent!=='0x02FF'||
                     !q('#tilemap-cell-detail').textContent.includes('stored 0xFFFF'))throw Error('direct draw');
                  map.value='2';change(map);
                  if(!q('#tilemap-template-controls').hidden||canvas.width!==176||canvas.height!==136||
                     cells().length!==374||cells()[0].textContent!=='0xFFFF')throw Error('localized rectangle is stored');
                  map.value='1';change(map);if(mode.value!=='direct'||canvas.width!==96)throw Error('preserved choice');
                  mode.value='stored';change(mode);if(cells()[0].textContent!=='0xFFFF'||!q('#tilemap-cell-detail').textContent.includes('raw 0xFFFF'))throw Error('restore');
                  return 'PASS';
                })()'''), 'PASS')
            finally:
                browser.close()

    def test_missing_or_invalid_ui_rectangles_keep_other_grid(self):
        for kind in ('legacy', 'invalid'):
            value = catalog()
            if kind == 'legacy': value['halfword_copies']['tilemap_rectangles'] = None
            else: value['halfword_copies']['tilemap_rectangles']['rectangles'][0]['width'] = 1
            with self.subTest(kind=kind), tempfile.TemporaryDirectory(prefix='viewer-ui-rect-isolation-') as directory:
                work = Path(directory); _, world = fixture(work, value, observed=True); browser = Chrome(work, world)
                try:
                    self.assertTrue(browser.evaluate('''(()=>{
                      const q=s=>document.querySelector(s),row=q('[data-resource-id="graphics.ui-raw-transfers"]');
                      return !q('#tilemaps').hidden && q('#tilemap-id').options.length===1 &&
                        !row.querySelector('a[href="#tilemaps"]') && q('#tilemap-template-controls').hidden;
                    })()'''))
                finally:
                    browser.close()
