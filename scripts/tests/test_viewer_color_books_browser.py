"""Check source-preserving Color-Mode palette fragment browsing."""
import copy
import hashlib
import json
import shutil
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from view_rom_data import generate, project_color_mode_palettes
from viewer_browser_harness import Chrome


def catalog():
    return {'spans': [
        {'id': 'unit', 'book': 'unit-base', 'variant': 0, 'source_offset': 0x100,
         'colors': [0x001f, 0x83e0, 0x7c00] + [0] * 253},
        {'id': 'partial', 'book': 'supplemental', 'variant': 1, 'source_offset': 0x400,
         'colors': [0x8000, 0x03e0, 0x7c00]},
        {'id': 'other', 'book': 'supplemental', 'variant': 1, 'source_offset': 0x410,
         'colors': [0x1234]},
        {'id': 'companion', 'book': 'companion-base', 'variant': 0, 'source_offset': 0x600,
         'colors': [0x2222]}], 'slice_offsets': [2, 4, 6, 8]}


def fixture(work, value=None):
    dataset = work / 'dataset'; dataset.mkdir()
    content = json.dumps(catalog() if value is None else value).encode()
    (dataset / 'colors.json').write_bytes(content)
    (dataset / 'manifest.toml').write_text(
        'profile="test"\nextractor_revision=1\ndecoder_revision=1\n'
        '[[resources]]\nid="graphics.color-mode-palettes"\npath="colors.json"\n'
        'format="color-mode-palettes-v1-json"\nsha256="' + hashlib.sha256(content).hexdigest() + '"\n')
    index = generate(dataset, work / 'view', split=True)
    return index, index.parent / (index.name + '.parts/actors.html')


def payload(path):
    return json.JSONDecoder().raw_decode(path.read_text().split('const data = ', 1)[1])[0]


class ColorBooksProjectionTest(unittest.TestCase):
    def test_projection_and_invalid_fragments(self):
        original = catalog(); self.assertEqual(project_color_mode_palettes(original), original)
        self.assertEqual(original, catalog())
        mutations = [lambda c: c['spans'][0].__setitem__('variant', True),
                     lambda c: c['spans'][0].__setitem__('book', 'unknown'),
                     lambda c: c['spans'][0].__setitem__('source_offset', 1),
                     lambda c: c['spans'][0]['colors'].__setitem__(0, 65536),
                     lambda c: c['spans'][1].__setitem__('id', 'unit'),
                     lambda c: c['spans'][2].__setitem__('source_offset', 0x402),
                     lambda c: c.__setitem__('slice_offsets', [1, 2, 3, True])]
        for mutate in mutations:
            bad = copy.deepcopy(original); mutate(bad)
            with self.subTest(mutation=mutate), self.assertRaises(ValueError):
                project_color_mode_palettes(bad)

    def test_scope_and_partial_failure_isolation(self):
        with tempfile.TemporaryDirectory(prefix='viewer-color-books-scope-') as directory:
            index, actors = fixture(Path(directory)); data = payload(actors)
            self.assertEqual(len(data['color_mode_palettes']['spans']), 4)
            self.assertEqual(len(data['unit_palettes']), 16)
            self.assertNotIn('color_mode_palettes', payload(index))
            self.assertNotIn('color_mode_palettes', payload(index.parent / (index.name + '.parts/world.html')))
        with tempfile.TemporaryDirectory(prefix='viewer-color-books-bad-') as directory:
            bad = catalog(); bad['spans'][1]['colors'][0] = -1
            _, actors = fixture(Path(directory), bad); data = payload(actors)
            self.assertIsNone(data['color_mode_palettes']); self.assertEqual(len(data['unit_palettes']), 16)
            self.assertIn('invalid color fragments', data['resources'][0]['status'])
        with tempfile.TemporaryDirectory(prefix='viewer-color-books-unit-bad-') as directory:
            bad = catalog(); bad['spans'][0]['colors'].pop()
            _, actors = fixture(Path(directory), bad); data = payload(actors)
            self.assertIsNone(data['unit_palettes']); self.assertEqual(len(data['color_mode_palettes']['spans']), 4)
            self.assertTrue(data['resources'][0]['status'].startswith('Available: color fragments'))


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class ColorBooksBrowserTest(unittest.TestCase):
    def test_filters_paging_swatches_and_source_positions(self):
        with tempfile.TemporaryDirectory(prefix='viewer-color-books-browser-') as directory:
            work = Path(directory); _, actors = fixture(work); browser = Chrome(work, actors)
            try:
                self.assertEqual(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change'));
                  if(q('#color-books').hidden||q('#color-books>details').open)throw Error('disclosure');
                  const row=q('[data-resource-id="graphics.color-mode-palettes"]');
                  [...row.querySelectorAll('a')].find(a=>a.textContent==='Open color fragments').click();
                  if(!q('#color-books>details').open||q('#color-book-swatches').children.length!==16)throw Error('navigation/page');
                  const swatch=n=>q('#color-book-swatches').children[n];
                  if(swatch(0).querySelector('.color').style.backgroundColor!=='rgb(255, 0, 0)'||
                     swatch(1).querySelector('.color').style.backgroundColor!=='rgb(0, 255, 0)'||
                     !swatch(1).textContent.includes('0x83E0')||!swatch(1).textContent.includes('bit 15 1'))throw Error('BGR555');
                  swatch(1).click();if(!q('#color-book-selection').textContent.includes('source 0x102')||
                    !q('#color-book-selection').textContent.includes('spans[0].colors[1]'))throw Error('source selection');
                  q('#color-page').value='15';change(q('#color-page'));
                  if(!swatch(0).textContent.includes('Color 240'))throw Error('page');
                  q('#color-book').value='supplemental';change(q('#color-book'));
                  if(q('#color-fragment').options.length!==2||q('#color-page').options.length!==1||q('#color-book-swatches').children.length!==3||
                     !q('#color-book-source').textContent.includes('source 0x400'))throw Error('partial fragment');
                  q('#color-fragment').value='2';change(q('#color-fragment'));
                  if(q('#color-book-swatches').children.length!==1||!q('#color-book-source').textContent.includes('source 0x410'))throw Error('gap');
                  q('#color-variant').value='0';change(q('#color-variant'));
                  if(!q('#color-fragment').disabled||q('#color-book-swatches').children.length||
                     !q('#color-book-source').textContent.includes('No stored physical fragments'))throw Error('empty filter');
                  q('#color-book').value='companion-base';change(q('#color-book'));
                  if(q('#color-fragment').disabled||q('#color-book-swatches').children.length!==1||
                     q('#color-slices').rows.length!==4||q('#color-slices').rows[3].cells[1].textContent!=='0x08')throw Error('companion or selectors');
                  return 'PASS';
                })()'''), 'PASS')
            finally:
                browser.close()
