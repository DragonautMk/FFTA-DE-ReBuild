"""Inspect and preview authenticated background screen banks."""
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
from view_rom_data import generate, project_background_screen_banks
from viewer_browser_harness import Chrome


def source_parts():
    sizes = [13120] * 4 + [13184]; dims = [(15, 38), (18, 32), (37, 23), (16, 50), (18, 20)]
    tiles = bytearray(); before = bytearray(); after = bytearray(); tile_pointers = []; map_pointers = []
    address = 0x6dd9e8
    for index, (size, (width, height)) in enumerate(zip(sizes, dims)):
        tile_pointers.append(0x086cd968 + len(tiles)); map_pointers.append(0x08000000 + address)
        words = [0x76543210] * (size // 4); words[7] = 0xfedcba98; words[8:16] = [0x33333333] * 8
        tiles.extend(struct.pack('<' + str(len(words)) + 'I', *words))
        cells = [0x03ff, 0x23ff, 0x8000, 0x0400, 0x0800, 0x0001] + [0] * (width * height - 6)
        encoded = struct.pack('<' + str(len(cells)) + 'H', *cells)
        (before if index <= 2 else after).extend(encoded); address += len(encoded)
        if index == 2: address += 74
    return [(0x123618, 'tile-directory-literals', struct.pack('<2I', 0x083ac770, 0x083ac784)),
            (0x1236bc, 'map-construction-literals', struct.pack('<4I', 0x01000800, 0x083ac7a4, 0x083ac790, 0x03ff)),
            (0x123724, 'map-upload-literals', struct.pack('<2I', 0x083ac7a4, 0x06006000)),
            (0x3ac770, 'tile-pointers', struct.pack('<5I', *tile_pointers)),
            (0x3ac784, 'tile-byte-counts', struct.pack('<5H', *sizes)),
            (0x3ac790, 'map-pointers', struct.pack('<5I', *map_pointers)),
            (0x3ac7a4, 'map-dimensions', b''.join(struct.pack('<I', w | h << 8) for w, h in dims)),
            (0x6cd968, 'tile-blocks', bytes(tiles)), (0x6dd9e8, 'tilemaps-before-extra-row', bytes(before)),
            (0x6de9cc, 'tilemaps-after-extra-row', bytes(after))]


def source_list(parts):
    return [{'offset': offset, 'role': role, 'length': len(content), 'sha256': hashlib.sha256(content).hexdigest()}
            for offset, role, content in parts]


def fixture(work, *, invalid=False):
    dataset = work / 'dataset'; dataset.mkdir(); parts = source_parts(); content = b''.join(chunk for _, _, chunk in parts)
    (dataset / 'background-screen-banks.bin').write_bytes(content)
    manifest = ('profile="europe"\nextractor_revision=358\ndecoder_revision=357\n'
                '[[resources]]\nid="graphics.background-screen-banks"\npath="background-screen-banks.bin"\n'
                f'format="background-screen-banks-v1-bin"\nsha256="{hashlib.sha256(content).hexdigest()}"\n')
    for index, item in enumerate(source_list(parts)):
        digest = '0' * 64 if invalid and index == 8 else item['sha256']
        manifest += (f'[[resources.sources]]\noffset={item["offset"]}\nlength={item["length"]}\n'
                     f'role="{item["role"]}"\nsha256="{digest}"\n')
    banks = [[0] * 16 for _ in range(16)]; banks[0][1] = 0x7c00; banks[8][1] = 0x001f
    colors = json.dumps({'entries': [{'banks': banks}]}).encode()
    (dataset / 'bg-base-palettes.json').write_bytes(colors)
    manifest += ('[[resources]]\nid="graphics.bg-base-palettes"\npath="bg-base-palettes.json"\n'
                 f'format="bg-base-palettes-v2-json"\nsha256="{hashlib.sha256(colors).hexdigest()}"\n')
    (dataset / 'manifest.toml').write_text(manifest)
    return generate(dataset, work / 'view', split=True)


def payload(path):
    return json.JSONDecoder().raw_decode(path.read_text().split('const data = ', 1)[1])[0]


class BackgroundScreenProjectionTest(unittest.TestCase):
    def test_projection_scope_and_source_guards(self):
        parts = source_parts(); content = b''.join(chunk for _, _, chunk in parts); sources = source_list(parts)
        result = project_background_screen_banks(content, sources)
        self.assertEqual([(s['width'], s['height']) for s in result['screens']], [(15, 38), (18, 32), (37, 23), (16, 50), (18, 20)])
        self.assertEqual(sum(len(s['cells']) for s in result['screens']), 3157)
        self.assertEqual(result['screens'][2]['map_offset'] + len(result['screens'][2]['cells']) * 2 + 74, result['screens'][3]['map_offset'])
        self.assertEqual(result['screens'][0]['cells'][:6], [0x03ff, 0x23ff, 0x8000, 0x0400, 0x0800, 1])
        bad = copy.deepcopy(sources); bad[8]['sha256'] = '0' * 64
        with self.assertRaisesRegex(ValueError, 'SHA-256'): project_background_screen_banks(content, bad)
        bad = copy.deepcopy(sources); bad[0]['offset'] += 1
        with self.assertRaisesRegex(ValueError, 'ranges'): project_background_screen_banks(content, bad)
        badparts = copy.deepcopy(parts); offset, role, data = badparts[6]
        changed = bytearray(data); struct.pack_into('<I', changed, 0, 0)
        badparts[6] = (offset, role, bytes(changed)); bad = source_list(badparts)
        with self.assertRaisesRegex(ValueError, 'descriptor'): project_background_screen_banks(b''.join(v for _, _, v in badparts), bad)
        with tempfile.TemporaryDirectory(prefix='viewer-background-screen-') as directory:
            work = Path(directory); index = fixture(work); parts_dir = index.parent / 'index.html.parts'
            screen = payload(parts_dir / 'interface-maps.html')
            self.assertEqual(screen['background_screen_banks']['screens'], result['screens'])
            self.assertNotIn('background_screen_banks', payload(index)); self.assertNotIn('background_screen_banks', payload(parts_dir / 'audio.html'))
            self.assertIn('background-screens', [s['id'] for s in next(p for p in payload(index)['wiki']['pages'] if p['id'] == 'interface-maps')['sections']])
            single = generate(work / 'dataset', work / 'single')
            self.assertEqual(payload(single)['background_screen_banks']['screens'], result['screens'])
        with tempfile.TemporaryDirectory(prefix='viewer-background-screen-invalid-') as directory:
            index = fixture(Path(directory), invalid=True); page = payload(index.parent / 'index.html.parts/interface-maps.html')
            self.assertIsNone(page['background_screen_banks'])
            self.assertTrue(page['resources'][0]['status'].startswith('Invalid background screen banks:'))


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class BackgroundScreenBrowserTest(unittest.TestCase):
    def test_screen_pixels_palette_clear_depth_cells_and_navigation(self):
        with tempfile.TemporaryDirectory(prefix='viewer-background-screen-browser-') as directory:
            work = Path(directory); index = fixture(work); page = index.parent / 'index.html.parts/interface-maps.html'
            browser = Chrome(work, page)
            try:
                self.assertTrue(browser.evaluate('''(async()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change')),
                    pixel=(x,y)=>[...q('#background-screen-map').getContext('2d').getImageData(x,y,1,1).data].join(',');
                  const wait=async f=>{const end=Date.now()+8000;while(!f()){
                    if(Date.now()>end)throw Error('timeout');await new Promise(r=>setTimeout(r,10))}};
                  if(q('#background-screens').hidden||q('#background-screen-id').options.length!==5||
                     !q('[data-resource-id="graphics.background-screen-banks"] a[href="#background-screens"]'))throw Error('section or resource link');
                  if(pixel(1,0)!=='17,17,17,255'||pixel(8,0)!=='0,0,0,0'||pixel(17,0)!=='17,17,17,255'||
                     pixel(24,0)!=='119,119,119,255'||pixel(32,0)!=='136,136,136,255')throw Error('tiles, flips or empty');
                  q('#background-screen-missing').checked=true;change(q('#background-screen-missing'));
                  if(pixel(1,0)!=='17,17,17,255'||pixel(8,0)!=='255,0,255,255'||!q('#background-screen-description').textContent.includes('1 replaced cells'))
                    throw Error('exact empty value');
                  q('#background-screen-clear').checked=false;change(q('#background-screen-clear'));
                  if(pixel(0,0)!=='255,0,255,255')throw Error('raw empty tile');
                  q('#background-screen-clear').checked=true;change(q('#background-screen-clear'));
                  q('#background-screen-palette').value='0';change(q('#background-screen-palette'));
                  if(pixel(17,0)!=='0,0,255,255')throw Error('manual palette');
                  q('#background-screen-cell-palettes').checked=true;change(q('#background-screen-cell-palettes'));
                  if(pixel(17,0)!=='255,0,0,255')throw Error('per-cell palette');
                  q('#background-screen-palette').value='gray';change(q('#background-screen-palette'));
                  q('#background-screen-depth').value='8';change(q('#background-screen-depth'));
                  if(pixel(16,0)!=='16,16,16,255'||pixel(17,0)!=='50,50,50,255'||!q('#background-screen-cell-palettes').disabled)throw Error('8-bit decoding');
                  q('#background-screen-depth').value='4';change(q('#background-screen-depth'));
                  q('#background-screen-id').value='2';change(q('#background-screen-id'));
                  if(!q('#background-screen-missing').checked||!q('#background-screen-cell-palettes').checked||q('#background-screen-cell-palettes').disabled)
                    throw Error('settings retained');
                  q('#background-screen-cells').closest('details').open=true;
                  await wait(()=>q('#background-screen-cells').querySelectorAll('button').length===851);
                  q('#background-screen-cells').querySelectorAll('button')[32].click();
                  if(!q('#background-screen-cell').textContent.includes('destination halfword 1024'))throw Error('second screen block');
                  q('#background-screen-cells').querySelectorAll('button')[1].click();
                  if(!q('#background-screen-cell').textContent.includes('raw 0x23FF')||
                     q('#background-screen-cell').textContent.includes('becomes 0x0000'))throw Error('flagged 1023 cell');
                  q('#background-screen-base').value='';q('#background-screen-base').dispatchEvent(new Event('input'));
                  if(!q('#background-screen-description').textContent.includes('Choose a first tile ID'))throw Error('invalid base');
                  return true;})()'''))
                browser.command('Page.navigate', {'url': page.as_uri() + '?view=background-screens&select=background-screen-id%3A2#background-screens'})
                deadline = time.monotonic() + 10
                while time.monotonic() < deadline:
                    if browser.evaluate("document.readyState==='complete' && document.querySelector('#background-screen-id').value==='2'"):
                        break
                    time.sleep(.03)
                else:
                    self.fail('Screen bookmark did not restore selection')
                self.assertTrue(browser.evaluate("document.querySelector('#background-screens>details').open"))
            finally:
                browser.close()
