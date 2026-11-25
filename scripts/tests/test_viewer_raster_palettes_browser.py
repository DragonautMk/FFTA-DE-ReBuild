"""Exercise raster palette bands and descriptor navigation in Chrome."""
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


def fixture(work, *, include_background):
    dataset = work / 'dataset'
    dataset.mkdir()
    header = {'color_count': 2, 'range_length': 2, 'range_start': 0}
    raster = {'source_offset': 100, 'programs': [
        {'id': 0, 'source_offset': 200, 'header': header, 'colors': [0x001f, 0x83e0]},
        {'id': 1, 'source_offset': 210, 'header': header, 'colors': [0x7c00, 0xffff]},
    ], 'descriptor_programs': [[0, 1], [1, 0]]}
    background = {'descriptor_graphics': [0, 0], 'graphics': [
        {'relative_source': 1234, 'destination_offset': 0, 'words': [0x21] + [0] * 7}]}
    resources = [('graphics.map-raster-palettes', 'map-raster-palettes', raster)]
    if include_background:
        resources.append(('graphics.bg-main-graphics', 'bg-main-graphics', background))
    manifest = 'profile="test"\nextractor_revision=1\ndecoder_revision=1\n'
    for resource_id, name, value in resources:
        content = json.dumps(value).encode()
        (dataset / (name + '.json')).write_bytes(content)
        manifest += (f'[[resources]]\nid="{resource_id}"\npath="{name}.json"\n'
                     f'format="{name}-v1-json"\nsha256="{hashlib.sha256(content).hexdigest()}"\n')
    (dataset / 'manifest.toml').write_text(manifest)
    return generate(dataset, work / 'view')


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class ViewerRasterPalettesTest(unittest.TestCase):
    def test_colors_descriptor_links_and_background_navigation(self):
        with tempfile.TemporaryDirectory(prefix='viewer-raster-') as directory:
            work = Path(directory)
            browser = Chrome(work, fixture(work, include_background=True))
            try:
                self.assertEqual(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change'));
                  const pixel=y=>[...q('#raster-bands').getContext('2d').getImageData(0,y,1,1).data].join(',');
                  if(q('#raster-palettes').hidden || q('#raster-palettes>details').open)throw Error('disclosure');
                  const row=[...document.querySelectorAll('#rows tr')].find(r=>r.cells[0].textContent.includes('graphics.map-raster-palettes'));
                  row.querySelector('a').click();if(!q('#raster-palettes>details').open)throw Error('resource');
                  if(pixel(0)!=='255,0,0,255' || pixel(1)!=='0,255,0,255')throw Error('program colors');
                  q('#raster-color').value='1';change(q('#raster-color'));
                  if(!q('#raster-color-detail').textContent.includes('0x83E0') || !q('#raster-color-detail').textContent.includes('bit 15 1'))throw Error('raw color');
                  q('#bg-descriptor').value='1';change(q('#bg-descriptor'));q('#bg-raster-open').click();
                  if(q('#raster-descriptor').value!=='1' || q('#raster-program').value!=='1' ||
                     q('#raster-program').options.length!==2 || pixel(0)!=='0,0,255,255')throw Error('background to raster');
                  q('#raster-program').value='0';change(q('#raster-program'));
                  if(pixel(0)!=='255,0,0,255' || q('#raster-color').value!=='1')throw Error('second slot');
                  q('#raster-bg-open').click();
                  if(q('#bg-descriptor').value!=='1' || !q('#bg-tiles>details').open)throw Error('raster to background');
                  q('#raster-descriptor').value='all';change(q('#raster-descriptor'));
                  if(!q('#raster-bg-navigation').hidden || q('#raster-program').options.length!==2)throw Error('all programs');
                  return 'PASS';
                })()'''), 'PASS')
            finally:
                browser.close()

    def test_raster_only_dataset(self):
        with tempfile.TemporaryDirectory(prefix='viewer-raster-only-') as directory:
            work = Path(directory)
            browser = Chrome(work, fixture(work, include_background=False))
            try:
                self.assertEqual(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s),d=q('#raster-descriptor');d.value='0';d.dispatchEvent(new Event('change'));
                  if(q('#raster-palettes').hidden || !q('#bg-tiles').hidden || !q('#raster-bg-navigation').hidden ||
                     q('#raster-program').options.length!==2)throw Error('independent raster programs');
                  return 'PASS';
                })()'''), 'PASS')
            finally:
                browser.close()
