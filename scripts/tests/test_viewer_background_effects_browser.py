"""Inspect authenticated background-effect graphics and manual previews."""
import copy
import hashlib
import json
import shutil
import struct
import sys
import tempfile
import time
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from viewer_browser_harness import Chrome
from view_rom_data import generate, project_background_effect_graphics


def source_parts():
    lengths = [5888] * 24 + [5696]
    assert sum(lengths) == 147008
    descriptors = bytearray(); tiles = bytearray(); maps = bytearray()
    for i, length in enumerate(lengths):
        descriptors.extend(struct.pack('<III', len(tiles), length, 0x08d8f308 + i * 1200))
        words = [0x76543210] * (length // 4)
        words[7] = 0xfedcba98
        words[8:16] = [0x22222222] * 8
        tiles.extend(struct.pack('<' + str(len(words)) + 'I', *words))
        cells = [0x8052, 0x0452, 0x0852, 0x0001, 0x0053, 0x0401, 0x0801, 0x8001] + [0x0052] * 592
        maps.extend(struct.pack('<600H', *cells))
    return [(0x12fa38, 'descriptor-table-literal', struct.pack('<I', 0x083ad318)),
            (0x12fad4, 'tile-bank-literal', struct.pack('<I', 0x08d6b4c8)),
            (0x3ad318, 'graphics-descriptors', bytes(descriptors)),
            (0xd6b4c8, 'tile-blocks', bytes(tiles)), (0xd8f308, 'tilemaps', bytes(maps))]


def source_list(parts):
    return [{'offset': offset, 'role': role, 'length': len(content),
             'sha256': hashlib.sha256(content).hexdigest()} for offset, role, content in parts]


def fixture(work, *, invalid=False):
    dataset = work / 'dataset'; dataset.mkdir()
    parts = source_parts(); content = b''.join(chunk for _, _, chunk in parts)
    (dataset / 'background-effect-graphics.bin').write_bytes(content)
    digest = hashlib.sha256(content).hexdigest()
    manifest = ('profile="europe"\nextractor_revision=356\ndecoder_revision=355\n'
                '[[resources]]\nid="graphics.background-effect-graphics"\npath="background-effect-graphics.bin"\n'
                f'format="background-effect-graphics-v1-bin"\nsha256="{digest}"\n')
    for index, item in enumerate(source_list(parts)):
        sha = '0' * 64 if invalid and index == 4 else item['sha256']
        manifest += (f'[[resources.sources]]\noffset={item["offset"]}\nlength={item["length"]}\n'
                     f'role="{item["role"]}"\nsha256="{sha}"\n')
    banks = [[0] * 16 for _ in range(16)]
    banks[0][1] = 0x7c00; banks[8][1] = 0x001f
    colors = json.dumps({'entries': [{'banks': banks}]}).encode()
    (dataset / 'bg-base-palettes.json').write_bytes(colors)
    manifest += ('[[resources]]\nid="graphics.bg-base-palettes"\npath="bg-base-palettes.json"\n'
                 f'format="bg-base-palettes-v2-json"\nsha256="{hashlib.sha256(colors).hexdigest()}"\n')
    graphics = json.dumps({'descriptor_graphics': [0, 1], 'graphics': [
        {'relative_source': 300, 'destination_offset': 0,
         'words': [0x33333333] * 8 + [0x76543210] * 7 + [0xfedcba98]},
        {'relative_source': 400, 'destination_offset': 0, 'words': [0x44444444] * 16},
    ]}).encode()
    (dataset / 'bg-main-graphics.json').write_bytes(graphics)
    manifest += ('[[resources]]\nid="graphics.bg-main-graphics"\npath="bg-main-graphics.json"\n'
                 f'format="bg-main-graphics-v1-json"\nsha256="{hashlib.sha256(graphics).hexdigest()}"\n')
    (dataset / 'manifest.toml').write_text(manifest)
    return generate(dataset, work / 'view', split=True)


def payload(path):
    return json.JSONDecoder().raw_decode(path.read_text().split('const data = ', 1)[1])[0]


class BackgroundEffectsProjectionTest(unittest.TestCase):
    def test_full_projection_guards_and_scope(self):
        parts = source_parts(); content = b''.join(chunk for _, _, chunk in parts); sources = source_list(parts)
        result = project_background_effect_graphics(content, sources)
        self.assertEqual(len(result['effects']), 25)
        for index, effect in enumerate(result['effects']):
            self.assertEqual(effect['map_offset'], 0xd8f308 + index * 1200)
            self.assertEqual(len(effect['cells']), 600)
            self.assertEqual(effect['cells'][:5], [0x8052, 0x0452, 0x0852, 1, 0x53])
            self.assertEqual(effect['words'][0], 0x76543210)
        bad = copy.deepcopy(sources); bad[0]['sha256'] = '0' * 64
        with self.assertRaisesRegex(ValueError, 'SHA-256'): project_background_effect_graphics(content, bad)
        bad = copy.deepcopy(sources); bad[1]['offset'] += 1
        with self.assertRaisesRegex(ValueError, 'ranges'): project_background_effect_graphics(content, bad)
        changed = bytearray(content); struct.pack_into('<I', changed, 8, 32)
        bad = copy.deepcopy(sources); bad[2]['sha256'] = hashlib.sha256(changed[8:308]).hexdigest()
        with self.assertRaisesRegex(ValueError, 'descriptor'): project_background_effect_graphics(bytes(changed), bad)
        with tempfile.TemporaryDirectory(prefix='viewer-background-effects-') as directory:
            work = Path(directory); index = fixture(work); parts = index.parent / 'index.html.parts'
            data = payload(parts / 'interface-maps.html')
            self.assertEqual(data['background_effect_graphics']['effects'], result['effects'])
            self.assertNotIn('background_effect_graphics', payload(index))
            self.assertNotIn('background_effect_graphics', payload(parts / 'audio.html'))
            self.assertIn('background-effects', [item['id'] for item in next(p for p in payload(index)['wiki']['pages'] if p['id'] == 'interface-maps')['sections']])
            legacy = generate(work / 'dataset', work / 'single-page')
            self.assertEqual(payload(legacy)['background_effect_graphics']['effects'], result['effects'])
        with tempfile.TemporaryDirectory(prefix='viewer-background-effects-invalid-') as directory:
            index = fixture(Path(directory), invalid=True); page = payload(index.parent / 'index.html.parts/interface-maps.html')
            self.assertIsNone(page['background_effect_graphics'])
            self.assertTrue(page['resources'][0]['status'].startswith('Invalid background effect graphics:'))


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class BackgroundEffectsBrowserTest(unittest.TestCase):
    def test_map_tile_flips_palettes_cells_and_playback(self):
        with tempfile.TemporaryDirectory(prefix='viewer-background-effects-browser-') as directory:
            work = Path(directory); index = fixture(work)
            page = index.parent / 'index.html.parts/interface-maps.html'; browser = Chrome(work, page)
            try:
                self.assertTrue(browser.evaluate('''(async()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change')),
                    pixel=(x,y)=>[...q('#background-effect-map').getContext('2d').getImageData(x,y,1,1).data].join(',');
                  const wait=async f=>{const end=Date.now()+5000;while(!f()){
                    if(Date.now()>end)throw Error('timeout');await new Promise(r=>setTimeout(r,10))}};
                  if(q('#background-effects').hidden||q('#background-effect-id').options.length!==25||
                    !q('[data-resource-id="graphics.background-effect-graphics"] a[href="#background-effects"]'))throw Error('section or resource link');
                  if(pixel(1,0)!=='17,17,17,255'||pixel(8,0)!=='119,119,119,255'||pixel(16,0)!=='136,136,136,255'||
                     pixel(24,0)!=='0,0,0,0')throw Error('tile pixels, flips or missing');
                  q('#background-effect-missing').checked=true;change(q('#background-effect-missing'));
                  if(pixel(24,0)!=='255,0,255,255'||!q('#background-effect-description').textContent.includes('4 cells reference'))
                    throw Error('missing tile indication');
                  q('#background-effect-palette').value='0';change(q('#background-effect-palette'));
                  if(pixel(1,0)!=='0,0,255,255')throw Error('manual palette');
                  q('#background-effect-cell-palettes').checked=true;change(q('#background-effect-cell-palettes'));
                  if(pixel(1,0)!=='255,0,0,255')throw Error('cell palette');
                  q('#background-effect-cells').closest('details').open=true;
                  await wait(()=>q('#background-effect-cells').querySelectorAll('button').length===600);
                  q('#background-effect-cells').querySelectorAll('button')[1].click();
                  if(!q('#background-effect-cell').textContent.includes('horizontal flip yes'))throw Error('cell detail');
                  q('#background-effect-base').value='83';q('#background-effect-base').dispatchEvent(new Event('input'));
                  if(!q('#background-effect-cell').textContent.includes('tile not in this block'))throw Error('manual base');
                  q('#background-effect-base').value='82';q('#background-effect-base').dispatchEvent(new Event('input'));
                  q('#background-effect-loop').checked=true;q('#background-effect-rate').value='30';q('#background-effect-play').click();
                  await wait(()=>Number(q('#background-effect-id').value)>0);
                  if(!q('#background-effect-loop').checked||q('#background-effect-stop').disabled)throw Error('playback');
                  q('#background-effect-stop').click();const current=q('#background-effect-id').value;
                  await new Promise(r=>setTimeout(r,70));
                  if(current!==q('#background-effect-id').value||!q('#background-effect-stop').disabled)throw Error('stop');
                  q('#background-effect-id').value='24';change(q('#background-effect-id'));q('#background-effect-play').click();
                  await wait(()=>q('#background-effect-id').value==='0');q('#background-effect-stop').click();
                  return true;})()'''))
                self.assertTrue(browser.evaluate('''(()=>{const e=document.querySelector('#background-effect-id');
                  for(let p=e;p;p=p.parentElement)if(p.tagName==='DETAILS')p.open=true;
                  e.focus();return document.activeElement===e;})()'''))
                browser.click('#background-effect-play')
                self.assertEqual(browser.evaluate('document.activeElement.id'), 'background-effect-id')
                browser.evaluate("document.querySelector('#background-effect-stop').click()")
                browser.command('Page.navigate', {'url': page.as_uri() + '?view=background-effects&select=background-effect-id%3A12#background-effects'})
                deadline = time.monotonic() + 10
                while time.monotonic() < deadline:
                    if browser.evaluate("document.readyState==='complete' && document.querySelector('#background-effect-id').value==='12'"):
                        break
                    time.sleep(.03)
                else:
                    self.fail('Effect bookmark did not restore the selected entry')
                self.assertTrue(browser.evaluate("document.querySelector('#background-effects>details').open"))
            finally:
                browser.close()

    def test_manual_shared_source_base_flips_palettes_and_persistence(self):
        with tempfile.TemporaryDirectory(prefix='viewer-background-effects-shared-') as directory:
            work = Path(directory); index = fixture(work)
            page = index.parent / 'index.html.parts/interface-maps.html'; browser = Chrome(work, page)
            try:
                self.assertTrue(browser.evaluate('''(async()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change')),
                    input=e=>e.dispatchEvent(new Event('input')),
                    pixel=(x,y)=>[...q('#background-effect-map').getContext('2d').getImageData(x,y,1,1).data].join(',');
                  const wait=async f=>{const end=Date.now()+5000;while(!f()){
                    if(Date.now()>end)throw Error('timeout');await new Promise(r=>setTimeout(r,10))}};
                  const source=q('#background-effect-shared'),base=q('#background-effect-shared-base');
                  if(source.value!=='none'||!base.disabled||source.options.length!==3||pixel(25,0)!=='0,0,0,0')
                    throw Error('default or source choices');
                  source.value='0';change(source);
                  if(base.disabled||pixel(25,0)!=='17,17,17,255'||pixel(40,0)!=='119,119,119,255'||
                     pixel(48,0)!=='136,136,136,255'||!q('#background-effect-description').textContent.includes('4 cells use'))
                    throw Error('shared pixels, flips or count');
                  q('#background-effect-cells').closest('details').open=true;
                  await wait(()=>q('#background-effect-cells').querySelectorAll('button').length===600);
                  q('#background-effect-cells').querySelectorAll('button')[3].click();
                  if(!q('#background-effect-cell').textContent.includes('shared tile 1')||
                     !q('#background-effect-cell').textContent.includes('Background graphic 0'))throw Error('shared cell detail');
                  q('#background-effect-palette').value='0';change(q('#background-effect-palette'));
                  q('#background-effect-cell-palettes').checked=true;change(q('#background-effect-cell-palettes'));
                  if(pixel(25,0)!=='0,0,255,255'||pixel(57,0)!=='255,0,0,255')throw Error('shared cell palette');
                  q('#background-effect-palette').value='gray';change(q('#background-effect-palette'));
                  source.value='1';change(source);if(pixel(24,0)!=='68,68,68,255')throw Error('second source');
                  source.value='0';change(source);base.value='82';input(base);
                  if(pixel(1,0)!=='17,17,17,255')throw Error('effect must win overlaps');
                  base.value='';input(base);
                  if(!q('#background-effect-description').textContent.includes('Choose a shared first tile ID'))throw Error('empty base');
                  base.value='1024';input(base);
                  if(!q('#background-effect-description').textContent.includes('Choose a shared first tile ID'))throw Error('invalid base');
                  base.value='0';input(base);
                  q('#background-effect-id').value='1';change(q('#background-effect-id'));
                  if(source.value!=='0'||base.value!=='0'||pixel(25,0)!=='17,17,17,255')throw Error('effect change');
                  q('#background-effect-loop').checked=true;q('#background-effect-rate').value='30';q('#background-effect-play').click();
                  await wait(()=>Number(q('#background-effect-id').value)>1);q('#background-effect-stop').click();
                  if(source.value!=='0'||base.value!=='0'||!q('#background-effect-loop').checked)throw Error('playback settings');
                  source.value='none';change(source);
                  if(!base.disabled||pixel(25,0)!=='0,0,0,0')throw Error('disable shared source');
                  return true;})()'''))
            finally:
                browser.close()
