"""Exercise alternate background palettes and linked tile sheets in Chrome."""
import hashlib
import json
import shutil
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from view_rom_data import generate
from viewer_browser_harness import Chrome


def fixture(work, *, alternate=True, background=True):
    dataset = work / 'dataset'
    dataset.mkdir()
    def entry(word):
        return {'banks': [[0, word] + [0] * 14 for _ in range(8)], 'encoded_source': [0, 0, 1, 0]}
    alternates = {'descriptor_keys': [0, 255, 1], 'relative_sources': [[4, 8], [4, 8]],
                  'catalogs': [[entry(0x801f), entry(0x7c00)], [entry(0x83e0), entry(0x03ff)]]}
    graphics = {'descriptor_graphics': [0, 0, 0], 'graphics': [
        {'relative_source': 100, 'destination_offset': 0, 'words': [0x21] + [0] * 7}]}
    base = {'entries': [{'banks': [[0, 0, 0] + [0] * 13 for _ in range(8)]} for _ in range(3)]}
    resources = []
    if alternate:
        resources.append(('graphics.bg-alternate-palettes', 'bg-alternate-palettes', 1, alternates))
    if background:
        resources.extend([('graphics.bg-main-graphics', 'bg-main-graphics', 1, graphics),
                          ('graphics.bg-base-palettes', 'bg-base-palettes', 2, base)])
    manifest = 'profile="test"\nextractor_revision=1\ndecoder_revision=1\n'
    for resource_id, name, version, value in resources:
        content = json.dumps(value).encode()
        (dataset / (name + '.json')).write_bytes(content)
        manifest += (f'[[resources]]\nid="{resource_id}"\npath="{name}.json"\n'
                     f'format="{name}-v{version}-json"\nsha256="{hashlib.sha256(content).hexdigest()}"\n')
    (dataset / 'manifest.toml').write_text(manifest)
    return generate(dataset, work / 'view')


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class ViewerAlternatePalettesTest(unittest.TestCase):
    def test_alternate_colors_tile_pixels_sentinel_and_navigation(self):
        with tempfile.TemporaryDirectory(prefix='viewer-alternate-palettes-') as directory:
            work = Path(directory)
            browser = Chrome(work, fixture(work))
            try:
                self.assertEqual(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change'));
                  const pixel=()=>[...q('#bg-sheet').getContext('2d').getImageData(0,0,1,1).data].join(',');
                  if(q('#alternate-palettes').hidden || q('#alternate-palettes>details').open)throw Error('disclosure');
                  const row=[...document.querySelectorAll('#rows tr')].find(r=>r.cells[0].textContent.includes('graphics.bg-alternate-palettes'));
                  if(!row || row.querySelectorAll('a').length!==2)throw Error('resource links');row.querySelector('a').click();
                  q('#alternate-group').value='1';change(q('#alternate-group'));
                  q('#alternate-bank').value='1';change(q('#alternate-bank'));
                  if(q('#alternate-colors').children.length!==16 || !q('#alternate-colors').children[1].textContent.includes('0x83E0') ||
                     !q('#alternate-source').textContent.includes('relative source 0x4') ||
                     !q('#alternate-source').textContent.includes('environment low bits 1'))throw Error('swatches');
                  q('#alternate-bg-open').click();
                  if(!q('#bg-tiles>details').open || q('#bg-palette').value!=='alt-1-1' || pixel()!=='0,255,0,255')throw Error('alternate tile pixel');
                  q('#bg-alternate-open').click();
                  if(q('#alternate-group').value!=='1' || q('#alternate-bank').value!=='1')throw Error('return navigation');
                  q('#bg-descriptor').value='1';change(q('#bg-descriptor'));
                  if([...q('#bg-palette').options].some(o=>o.value.startsWith('alt-')) ||
                     !q('#bg-detail').textContent.includes('unavailable (key 0xFF)'))throw Error('sentinel tile');
                  q('#bg-alternate-open').click();
                  if(!q('#alternate-bank').disabled || q('#alternate-colors').children.length ||
                     !q('#alternate-source').textContent.includes('key 0xFF') || !q('#alternate-bg-navigation').hidden)throw Error('sentinel view');
                  q('#alternate-descriptor').value='2';change(q('#alternate-descriptor'));
                  if(q('#alternate-bank').disabled || q('#alternate-group').value!=='1' || q('#alternate-bank').value!=='1')throw Error('settings');
                  q('#alternate-bg-open').click();if(q('#bg-palette').value!=='alt-1-1' || pixel()!=='255,255,0,255')throw Error('other descriptor');
                  return 'PASS';
                })()'''), 'PASS')
            finally:
                browser.close()

    def test_catalogs_work_independently(self):
        for alternate, background in ((True, False), (False, True)):
            with self.subTest(alternate=alternate), tempfile.TemporaryDirectory(prefix='viewer-alternate-only-') as directory:
                work = Path(directory)
                browser = Chrome(work, fixture(work, alternate=alternate, background=background))
                try:
                    expression = '''(()=>{const q=s=>document.querySelector(s);
                      if(q('#alternate-palettes').hidden!==ALTERNATE_HIDDEN || q('#bg-tiles').hidden!==BACKGROUND_HIDDEN)throw Error('independent sections');
                      if(!q('#alternate-bg-navigation').hidden || !q('#bg-alternate-navigation').hidden)throw Error('missing links');
                      return 'PASS';})()'''.replace('ALTERNATE_HIDDEN', str(not alternate).lower()).replace('BACKGROUND_HIDDEN', str(not background).lower())
                    self.assertEqual(browser.evaluate(expression), 'PASS')
                finally:
                    browser.close()
