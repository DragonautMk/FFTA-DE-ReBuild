"""Verify source-backed fixed screen maps and scoped visual previews."""
import copy
import hashlib
import json
import shutil
import struct
import tempfile
import unittest
from pathlib import Path

from view_rom_data import generate, project_fixed_screen_maps, viewer_section_updates
from viewer_browser_harness import Chrome

LITERALS = (0x143d00, 0x143d98, 0x143e54, 0x143f78, 0x144018, 0x1440dc, 0x144184, 0x144364, 0x1443c8, 0x14445c)
TARGETS = (0xb8cdd4, 0xb8d354, 0xb8d354, 0xb8cbf0, 0xb8c8f8, 0xb8c778, 0x6df2dc, 0xb8d804, 0xb8d054, 0xb8ca4c)
GEOMETRY = ((0x6df2dc,17,13), (0xb8c778,12,16), (0xb8c8f8,10,17), (0xb8ca4c,15,14),
            (0xb8cbf0,22,11), (0xb8cdd4,10,32), (0xb8d054,12,32), (0xb8d354,30,20), (0xb8d804,30,20))


def chunks():
    result = [(offset, 'reader_literals', struct.pack('<I', 0x08000000 + target)) for offset, target in zip(LITERALS, TARGETS)]
    blocks = []
    for offset, width, height in GEOMETRY:
        values = [5, 0x1405, 0x1805, 0xfc06] * ((width * height + 3) // 4)
        blocks.append((offset, struct.pack('<' + str(width * height) + 'H', *values[:width * height])))
    result.extend([(0x6df2dc, 'tilemap_cells', blocks[0][1]), (0xb8c778, 'tilemap_cells', b''.join(x[1] for x in blocks[1:]))])
    return result


def sources(parts):
    return [{'offset': offset, 'role': role, 'length': len(raw), 'sha256': hashlib.sha256(raw).hexdigest()}
            for offset, role, raw in parts]


def fixture(work, invalid=False):
    dataset = work / 'dataset'; dataset.mkdir(); parts = chunks(); raw = b''.join(p[2] for p in parts)
    graphics = {'descriptor_graphics': [0], 'graphics': [{'relative_source': 400, 'destination_offset': 5 * 32,
                'words': [1] + [0] * 6 + [2] + [0x33333333] * 8}]}
    palette = {'entries': [{'banks': [[0,31,992,31744] + [0]*12, [0,992,31744,31] + [0]*12]}]}
    items = [('graphics.fixed-screen-maps','maps.bin','fixed-screen-maps-v1-bin',raw,sources(parts)),
             ('graphics.bg-main-graphics','graphics.json','bg-main-graphics-v1-json',json.dumps(graphics).encode(),[]),
             ('graphics.bg-base-palettes','palette.json','bg-base-palettes-v2-json',json.dumps(palette).encode(),[])]
    manifest = 'profile="europe"\nextractor_revision=364\ndecoder_revision=363\n'
    for identity, path, fmt, content, entries in items:
        (dataset / path).write_bytes(content)
        manifest += f'[[resources]]\nid="{identity}"\npath="{path}"\nformat="{fmt}"\nsha256="{hashlib.sha256(content).hexdigest()}"\n'
        for index, entry in enumerate(entries):
            digest = '0' * 64 if invalid and index == 11 else entry['sha256']
            manifest += (f'[[resources.sources]]\noffset={entry["offset"]}\nlength={entry["length"]}\n'
                         f'role="{entry["role"]}"\nsha256="{digest}"\n')
    (dataset / 'manifest.toml').write_text(manifest)
    return generate(dataset, work / 'view', split=True)


def payload(path):
    return json.JSONDecoder().raw_decode(path.read_text().split('const data = ', 1)[1])[0]


class FixedScreenMapsProjectionTest(unittest.TestCase):
    def test_geometry_sources_and_shared_caller(self):
        parts = chunks(); result = project_fixed_screen_maps(b''.join(x[2] for x in parts), sources(parts))
        self.assertEqual([(m['source_offset'],m['width'],m['height']) for m in result['maps']], list(GEOMETRY))
        self.assertEqual(sum(len(m['cells']) for m in result['maps']), 2939)
        self.assertEqual(result['maps'][0]['cells'][:4], [5,0x1405,0x1805,0xfc06])
        self.assertEqual(sum(m['source_offset']==0xb8d354 for m in result['maps']), 1)

    def test_reject_hash_pointer_and_extent(self):
        parts = chunks(); raw = b''.join(x[2] for x in parts); bad = sources(parts); bad[-1]['sha256'] = '0'*64
        with self.assertRaises(ValueError): project_fixed_screen_maps(raw, bad)
        bad = copy.deepcopy(parts); value = bytearray(bad[1][2]); value[0] ^= 1; bad[1] = (bad[1][0],bad[1][1],bytes(value))
        with self.assertRaisesRegex(ValueError, 'literal'): project_fixed_screen_maps(b''.join(x[2] for x in bad), sources(bad))
        bad = copy.deepcopy(parts); bad[-1] = (bad[-1][0]+2,bad[-1][1],bad[-1][2])
        with self.assertRaises(ValueError): project_fixed_screen_maps(b''.join(x[2] for x in bad), sources(bad))

    def test_scoped_graphics_and_invalid_data(self):
        for invalid in (False, True):
            with self.subTest(invalid=invalid), tempfile.TemporaryDirectory(prefix='viewer-fixed-maps-scope-') as directory:
                index = fixture(Path(directory), invalid); folder = index.parent / (index.name + '.parts')
                home, page, maps = map(payload,(index,folder/'interface-fixed-maps.html',folder/'interface-maps.html'))
                self.assertNotIn('fixed_screen_maps',home); self.assertNotIn('fixed_screen_maps',maps)
                self.assertNotIn('bg_graphics',page); self.assertNotIn('fixed_map_graphics',page)
                self.assertEqual('fixed_map_graphics_asset' in page, not invalid)
                self.assertEqual(page['fixed_screen_maps'] is None,invalid)
                sections = [s for p in home['wiki']['pages'] if p['id']=='interface-fixed-maps' for s in p['sections']]
                self.assertEqual([s['id'] for s in sections], [] if invalid else ['tilemaps'])


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class FixedScreenMapsBrowserTest(unittest.TestCase):
    def test_scoped_visuals_manual_pairings_and_collapse(self):
        with tempfile.TemporaryDirectory(prefix='viewer-fixed-maps-browser-') as directory:
            work = Path(directory); index = fixture(work); page = index.parent/(index.name+'.parts')/'interface-fixed-maps.html'
            browser = Chrome(work,page)
            try:
                expected = viewer_section_updates()['interface-fixed-maps']['tilemaps']
                self.assertTrue(browser.evaluate('''(async()=>{const expected=EXPECTED,q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change')),
                  canvas=q('#tilemap-canvas'),pixel=(x,y)=>[...canvas.getContext('2d').getImageData(x,y,1,1).data].join(',');
                  const badge=q('#tilemaps>details>summary time');
                  if(!badge||badge.dateTime!==expected.at||!badge.textContent.startsWith('Updated ')||q('#tilemaps>details>summary h2').textContent!=='Fixed-width screen maps')throw Error('badge or title');
                  if(!q('[data-resource-id="graphics.fixed-screen-maps"] a[href="#tilemaps"]')||q('#tilemap-id').options.length!==9||!q('#bg-tiles').hidden)throw Error('scope');
                  if(canvas.width!==136||canvas.height!==104||pixel(0,0)!=='85,85,85,255'||!q('#tilemap-cell-detail').textContent.includes('source 0x6DF2DC'))throw Error('grid');
                  q('#tilemap-load-map-graphics').click();await new Promise((resolve,reject)=>{let n=0;const timer=setInterval(()=>{
                    if(q('#tilemap-load-map-graphics').hidden){clearInterval(timer);resolve();}else if(++n>100){clearInterval(timer);reject(Error('load'));}},20);});
                  q('#tilemap-graphics').value='0';change(q('#tilemap-graphics'));q('#tilemap-palette-source').value='0';change(q('#tilemap-palette-source'));
                  if(pixel(0,0)!=='255,0,0,255'||q('#tilemap-base').value!=='5')throw Error('manual pairing');
                  q('#tilemap-id').value='8';change(q('#tilemap-id'));
                  if(canvas.width!==240||canvas.height!==160||q('#tilemap-graphics').value!=='0'||q('#tilemap-palette-source').value!=='0'||
                     !q('#tilemap-cell-detail').textContent.includes('source 0xB8D804'))throw Error('last map');
                  q('#tilemap-cells button').click();if(!q('#tilemap-cell-detail').textContent.includes('raw 0x0005'))throw Error('cell');
                  q('#tilemaps>details').open=true;q('#collapse-all').click();return !q('#tilemaps>details').open;})()'''.replace('EXPECTED',json.dumps(expected))))
            finally:browser.close()


if __name__ == '__main__':unittest.main()
