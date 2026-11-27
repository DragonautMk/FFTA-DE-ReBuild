"""Exercise LZ77 validation and manual graphics previews across wiki scopes."""
import copy
import hashlib
import json
import shutil
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from view_rom_data import decode_gba_lz77, generate, project_observed_lz
from test_viewer_static_obj_browser import object_data
from viewer_browser_harness import Chrome


def literal_stream(data):
    result = [0x10, len(data) & 255, len(data) >> 8 & 255, len(data) >> 16 & 255]
    for index in range(0, len(data), 8):
        result += [0] + data[index:index + 8]
    return result


def block(offset, data):
    return {'source_offset': offset, 'encoded_source': literal_stream(data), 'decoded': data}


def catalog():
    first = block(0xaf4998, [0x12, 0x34] + [0] * 94)
    first['additional_blocks'] = [block(0xaff6e0, [0x12] * 32 + [0x45, 0x67, 0x89])]
    vram_blocks = [block(0xb4fa4c + 0x100 * i, [i + 1] * 32) for i in range(3)]
    first['vram_bank'] = {'source_literals': [list((0x08000000 + item['source_offset']).to_bytes(4, 'little'))
                                             for item in vram_blocks], 'blocks': vram_blocks}
    return first


def fixture(work, *, invalid=False, graphics=False):
    dataset = work / 'dataset'; dataset.mkdir(); value = catalog()
    if invalid:
        value['decoded'][0] = 0
    items = [('graphics.observed-lz-block', 'observed-lz-block', 2, value),
             ('graphics.obj-static-lists', 'obj-static-lists', 2,
              {'lists': [{'source_offset': 1000, 'objects': [object_data()]}]}),
             ('graphics.observed-tilemap', 'observed-tilemap', 1,
              {'source_offset': 2000, 'width': 1, 'height': 1, 'mode': 4,
               'runs': [{'control': 0x4001, 'value': 0}]}),
             ('graphics.ui-raw-palettes', 'ui-raw-palettes', 1,
              {'banks': [{'source_offset': 9000, 'colors': [0, 31] + [0] * 14}]})]
    if graphics:
        items.append(('graphics.obj-custom-banks', 'obj-custom-banks', 1,
                      {'banks': [{'source_offset': 8000, 'words': [0x11111111] * 8}]}))
    manifest = 'profile="test"\nextractor_revision=1\ndecoder_revision=1\n'
    for rid, name, version, value in items:
        content = json.dumps(value).encode(); (dataset / (name + '.json')).write_bytes(content)
        manifest += (f'[[resources]]\nid="{rid}"\npath="{name}.json"\nformat="{name}-v{version}-json"\n'
                     f'sha256="{hashlib.sha256(content).hexdigest()}"\n')
    (dataset / 'manifest.toml').write_text(manifest)
    return generate(dataset, work / 'view', split=True)


def payload(path):
    return json.JSONDecoder().raw_decode(path.read_text().split('const data = ', 1)[1])[0]


class ObservedLzProjectionTest(unittest.TestCase):
    def test_decode_overlap_and_validate_streams(self):
        source = bytes([0x10, 20, 0, 0, 0x40, 65, 0xf0, 0, 66])
        self.assertEqual(decode_gba_lz77(source), b'A' * 19 + b'B')
        result = project_observed_lz(catalog())
        self.assertEqual(len(result['streams']), 5)
        self.assertEqual(result['streams'][0]['words'][0], 0x3412)
        self.assertEqual(result['streams'][0]['context'], 'Firaga observation')
        self.assertEqual(result['streams'][1]['decoded_length'], 35)
        self.assertEqual(len(result['streams'][1]['words']), 8)
        self.assertEqual(result['streams'][1]['tail'], [0x45, 0x67, 0x89])

    def test_rejects_malformed_or_inconsistent_data(self):
        for raw in (bytes([0x10, 0, 0, 0]), bytes([0x10, 1, 0, 0, 0x80, 0, 0]),
                    bytes([0x10, 1, 0, 0, 0, 1, 2]), bytes([0x10, 1, 0, 0, 0])):
            with self.subTest(raw=raw), self.assertRaises(ValueError): decode_gba_lz77(raw)
        mutations = [lambda c: c['decoded'].__setitem__(0, 0),
                     lambda c: c['encoded_source'].append(0),
                     lambda c: c['decoded'].__setitem__(0, True),
                     lambda c: c['additional_blocks'][0].__setitem__('source_offset', c['source_offset']),
                     lambda c: c['vram_bank']['source_literals'][0].__setitem__(0, 0)]
        for mutate in mutations:
            value = copy.deepcopy(catalog()); mutate(value)
            with self.subTest(mutation=mutate), self.assertRaises(ValueError): project_observed_lz(value)

    def test_scoping_and_invalid_source(self):
        for invalid in (False, True):
            with self.subTest(invalid=invalid), tempfile.TemporaryDirectory(prefix='viewer-lz-scope-') as directory:
                index = fixture(Path(directory), invalid=invalid); parts = index.parent / (index.name + '.parts')
                self.assertNotIn('lz_graphics', payload(index))
                world, objects = payload(parts / 'interface-maps.html'), payload(parts / 'objects.html')
                self.assertEqual(bool(world['lz_graphics']), not invalid)
                self.assertEqual(bool(objects['lz_graphics']), not invalid)
                row = next(r for r in world['resources'] if r['id'] == 'graphics.observed-lz-block')
                self.assertEqual(row['status'].startswith('Invalid'), invalid)


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class ObservedLzBrowserTest(unittest.TestCase):
    def test_lz_tiles_4_and_8_bit_and_tilemap_source(self):
        with tempfile.TemporaryDirectory(prefix='viewer-lz-world-') as directory:
            work = Path(directory); index = fixture(work); browser = Chrome(work, index.parent / (index.name + '.parts/interface-maps.html'))
            try:
                self.assertEqual(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change',{bubbles:true}));
                  const pixel=(canvas,x,y)=>[...canvas.getContext('2d').getImageData(x,y,1,1).data].join(',');
                  const row=[...q('#rows').rows].find(r=>r.dataset.resourceId==='graphics.observed-lz-block');
                  if(!row||!row.querySelector('a'))throw Error('resource link');row.querySelector('a').click();
                  if(!q('#obj-tiles>details').open||q('#obj-catalog').value!=='lz'||q('#obj-bank').options.length!==5||
                     !q('#obj-bank').options[0].textContent.includes('Firaga observation'))throw Error('LZ catalog');
                  const sheet=q('#obj-sheet');
                  if(pixel(sheet,0,0)!=='34,34,34,255'||pixel(sheet,1,0)!=='17,17,17,255')throw Error('4-bit pixels');
                  q('#obj-lz-offset').value='1';q('#obj-lz-offset').dispatchEvent(new Event('input'));
                  if(pixel(sheet,0,0)!=='68,68,68,255'||pixel(sheet,1,0)!=='51,51,51,255')throw Error('4-bit byte offset');
                  q('#obj-depth').value='8';change(q('#obj-depth'));
                  if(pixel(sheet,0,0)!=='52,52,52,255')throw Error('8-bit byte offset');
                  q('#obj-lz-offset').value='0';q('#obj-lz-offset').dispatchEvent(new Event('input'));
                  if(pixel(sheet,0,0)!=='18,18,18,255'||pixel(sheet,1,0)!=='52,52,52,255')throw Error('8-bit pixels');
                  q('#obj-palette').value='0';change(q('#obj-palette'));
                  if(pixel(sheet,0,0)!=='255,0,255,255'||!q('#obj-detail').textContent.includes('lack palette colors'))throw Error('short palette');
                  q('#obj-bank').value='1';change(q('#obj-bank'));q('#obj-depth').value='4';change(q('#obj-depth'));
                  if(!q('#obj-detail').textContent.includes('3 trailing bytes'))throw Error('partial tile');
                  q('#obj-lz-offset').value='3';q('#obj-lz-offset').dispatchEvent(new Event('input'));
                  if(lzWordsAt(data.lz_graphics.streams[1],3).at(-1)!==0x89674512||
                     q('#obj-detail').textContent.includes('trailing bytes'))throw Error('tail bytes');
                  q('#obj-lz-offset').value='36';q('#obj-lz-offset').dispatchEvent(new Event('input'));
                  if(!q('#obj-detail').textContent.includes('Choose an LZ byte offset'))throw Error('invalid offset');
                  q('#tilemap-graphics').value='0';change(q('#tilemap-graphics'));
                  if(pixel(q('#tilemap-canvas'),0,0)!=='34,34,34,255')throw Error('tilemap source');
                  q('#tilemap-lz-offset').value='1';q('#tilemap-lz-offset').dispatchEvent(new Event('input'));
                  if(pixel(q('#tilemap-canvas'),0,0)!=='68,68,68,255')throw Error('tilemap byte offset');
                  q('#tilemap-lz-offset').value='97';q('#tilemap-lz-offset').dispatchEvent(new Event('input'));
                  if(!q('#tilemap-description').textContent.includes('Choose an LZ byte offset'))throw Error('tilemap invalid offset');
                  return 'PASS';
                })()'''), 'PASS')
            finally: browser.close()

    def test_lz_as_manual_object_animation_source(self):
        with tempfile.TemporaryDirectory(prefix='viewer-lz-objects-') as directory:
            work = Path(directory); index = fixture(work); browser = Chrome(work, index.parent / (index.name + '.parts/objects.html'))
            try:
                self.assertTrue(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change'));
                  if(q('#obj-animations').hidden||q('#obj-animation-graphics').options.length!==6)return false;
                  q('#obj-animation-graphics').value='0';change(q('#obj-animation-graphics'));
                  q('#obj-animation-outlines').checked=false;change(q('#obj-animation-outlines'));
                  const pixel=[...q('#obj-animation-canvas').getContext('2d').getImageData(8,8,1,1).data].join(',');
                  if(pixel!=='34,34,34,255'||!q('#obj-animation-graphics').options[1].textContent.includes('Firaga observation'))return false;
                  q('#obj-animation-lz-offset').value='1';q('#obj-animation-lz-offset').dispatchEvent(new Event('input'));
                  const shifted=[...q('#obj-animation-canvas').getContext('2d').getImageData(8,8,1,1).data].join(',');
                  if(shifted!=='68,68,68,255')return false;
                  q('#obj-animation-lz-offset').value='97';q('#obj-animation-lz-offset').dispatchEvent(new Event('input'));
                  return q('#obj-animation-detail').textContent.includes('choose a valid LZ byte offset');
                })()'''))
            finally: browser.close()

    def test_invalid_lz_does_not_link_to_other_graphics(self):
        with tempfile.TemporaryDirectory(prefix='viewer-lz-invalid-') as directory:
            work = Path(directory); index = fixture(work, invalid=True, graphics=True)
            browser = Chrome(work, index.parent / (index.name + '.parts/interface-maps.html'))
            try:
                self.assertTrue(browser.evaluate('''(()=>{
                  const row=id=>[...document.querySelector('#rows').rows].find(r=>r.dataset.resourceId===id);
                  return !row('graphics.observed-lz-block').querySelector('a[href="#obj-tiles"]')&&
                    !document.querySelector('#obj-tiles').hidden;
                })()'''))
            finally: browser.close()
            second = work / 'second-browser'; second.mkdir()
            browser = Chrome(second, index.parent / (index.name + '.parts/objects.html'))
            try:
                self.assertTrue(browser.evaluate('''(()=>{
                  const row=[...document.querySelector('#rows').rows].find(r=>r.dataset.resourceId==='graphics.obj-custom-banks');
                  return !!row.querySelector('a[href="#obj-tiles"]');
                })()'''))
            finally: browser.close()
