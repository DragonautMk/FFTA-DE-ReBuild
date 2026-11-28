"""Verify the scoped European rectangle and selector-preset views."""
import copy
import hashlib
import json
import shutil
import struct
import tempfile
import unittest
from pathlib import Path

from view_rom_data import generate, project_direct_rectangle_directory, viewer_section_updates
from viewer_browser_harness import Chrome

OFFSETS = (0xae9110, 0xae92f4, 0xae94d8, 0xae96bc, 0xae98a0)


def chunks():
    cells = [5, 0x1405, 0x1805, 0xfc06] * 60
    rectangle = bytes((16, 15)) + struct.pack('<240H', *cells)
    return [(0x124144, 'reader_literals', struct.pack('<I', 0x083ac9f8)),
            (0x12414c, 'reader_literals', struct.pack('<I', 0x083ac9e4)),
            (0x3ac9e4, 'selector_presets', bytes(range(20))),
            (0x3ac9f8, 'rectangle_directory', struct.pack('<5I', *(0x08000000 + x for x in OFFSETS))),
            *[(offset, 'rectangle_cells', rectangle) for offset in OFFSETS]]


def sources(parts):
    return [{'offset': offset, 'role': role, 'length': len(raw), 'sha256': hashlib.sha256(raw).hexdigest()}
            for offset, role, raw in parts]


def project(parts):
    return project_direct_rectangle_directory(b''.join(part[2] for part in parts), sources(parts))


def fixture(work, invalid=False, profile='europe', split=True):
    dataset = work / 'dataset'; dataset.mkdir(); parts = chunks(); raw = b''.join(part[2] for part in parts)
    graphics = {'descriptor_graphics': [0], 'graphics': [{'relative_source': 400, 'destination_offset': 5 * 32,
                'words': [1] + [0] * 6 + [2] + [0x33333333] * 8}]}
    palette = {'entries': [{'banks': [[0, 31, 992, 31744] + [0] * 12]}]}
    items = [('graphics.direct-rectangle-directory', 'rectangles.bin', 'direct-rectangle-directory-v1-bin', raw, sources(parts)),
             ('graphics.bg-main-graphics', 'graphics.json', 'bg-main-graphics-v1-json', json.dumps(graphics).encode(), []),
             ('graphics.bg-base-palettes', 'palette.json', 'bg-base-palettes-v2-json', json.dumps(palette).encode(), [])]
    manifest = f'profile="{profile}"\nextractor_revision=366\ndecoder_revision=365\n'
    for identity, path, fmt, content, entries in items:
        (dataset / path).write_bytes(content)
        manifest += f'[[resources]]\nid="{identity}"\npath="{path}"\nformat="{fmt}"\nsha256="{hashlib.sha256(content).hexdigest()}"\n'
        for index, entry in enumerate(entries):
            digest = '0' * 64 if invalid and index == 8 else entry['sha256']
            manifest += (f'[[resources.sources]]\noffset={entry["offset"]}\nlength={entry["length"]}\n'
                         f'role="{entry["role"]}"\nsha256="{digest}"\n')
    (dataset / 'manifest.toml').write_text(manifest)
    return generate(dataset, work / 'view', split=split)


def payload(path):
    return json.JSONDecoder().raw_decode(path.read_text().split('const data = ', 1)[1])[0]


class DirectRectangleProjectionTest(unittest.TestCase):
    def test_cells_presets_and_distinct_sources(self):
        result = project(chunks())
        self.assertTrue(result['identical_copies'])
        self.assertEqual(result['preset_offset'], 0x3ac9e4)
        self.assertEqual([(m['source_offset'], m['cell_offset'], m['width'], m['height']) for m in result['maps']],
                         [(offset, offset + 2, 15, 16) for offset in OFFSETS])
        self.assertEqual([m['presets'] for m in result['maps']], [list(range(i, i + 4)) for i in range(0, 20, 4)])
        self.assertEqual(sum(len(m['cells']) for m in result['maps']), 1200)
        self.assertEqual(result['maps'][4]['cells'][:4], [5, 0x1405, 0x1805, 0xfc06])
        changed = chunks(); raw = bytearray(changed[-1][2]); raw[-1] ^= 1
        changed[-1] = (*changed[-1][:2], bytes(raw))
        self.assertFalse(project(changed)['identical_copies'])

    def test_reject_bad_hash_pointers_dimensions_and_extents(self):
        parts = chunks(); raw = b''.join(part[2] for part in parts); bad = sources(parts); bad[-1]['sha256'] = '0' * 64
        with self.assertRaisesRegex(ValueError, 'SHA-256'): project_direct_rectangle_directory(raw, bad)
        for index, position, message in ((0, 0, 'pointers'), (3, 0, 'pointers'), (4, 0, 'dimensions')):
            bad = copy.deepcopy(parts); raw = bytearray(bad[index][2]); raw[position] ^= 1
            bad[index] = (*bad[index][:2], bytes(raw))
            with self.subTest(index=index), self.assertRaisesRegex(ValueError, message): project(bad)
        bad = chunks(); bad[-1] = (bad[-1][0] + 2, *bad[-1][1:])
        with self.assertRaisesRegex(ValueError, 'sources'): project(bad)

    def test_scoped_and_unverified_resources(self):
        for invalid, profile in ((False, 'europe'), (True, 'europe'), (False, 'usa')):
            with self.subTest(invalid=invalid, profile=profile), tempfile.TemporaryDirectory(prefix='viewer-direct-scope-') as directory:
                index = fixture(Path(directory), invalid, profile); folder = index.parent / (index.name + '.parts')
                home, page, maps = map(payload, (index, folder / 'interface-rectangles.html', folder / 'interface-maps.html'))
                self.assertNotIn('direct_rectangle_directory', home)
                self.assertNotIn('direct_rectangle_directory', maps)
                self.assertNotIn('bg_graphics', page)
                valid = not invalid and profile == 'europe'
                self.assertNotIn('fixed_map_graphics', page)
                self.assertEqual('fixed_map_graphics_asset' in page, valid)
                self.assertEqual(page['direct_rectangle_directory'] is not None, valid)
                sections = [s['id'] for p in home['wiki']['pages'] if p['id'] == 'interface-rectangles' for s in p['sections']]
                self.assertEqual(sections, ['direct-rectangle-presets', 'tilemaps'] if valid else [])


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class DirectRectangleBrowserTest(unittest.TestCase):
    def test_single_page_keeps_rectangle_navigation(self):
        with tempfile.TemporaryDirectory(prefix='viewer-direct-single-') as directory:
            work = Path(directory); page = fixture(work, split=False); browser = Chrome(work, page)
            try:
                self.assertTrue(browser.evaluate('''(()=>{const q=s=>document.querySelector(s);
                  q('#direct-rectangle-preset-rows').rows[3].querySelector('a').click();
                  return !q('#direct-rectangle-presets').hidden && !q('#tilemaps').hidden &&
                    q('#tilemap-id').value==='3' && q('#tilemap-cell-detail').textContent.includes('source 0xAE96BE');})()'''))
            finally:
                browser.close()

    def test_scoped_visuals_preset_links_and_badges(self):
        with tempfile.TemporaryDirectory(prefix='viewer-direct-browser-') as directory:
            work = Path(directory); index = fixture(work); page = index.parent / (index.name + '.parts') / 'interface-rectangles.html'
            browser = Chrome(work, page)
            try:
                expected = viewer_section_updates()['interface-rectangles']
                self.assertTrue(browser.evaluate('''(async()=>{const expected=EXPECTED,q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change')),
                  canvas=q('#tilemap-canvas'),pixel=(x,y)=>[...canvas.getContext('2d').getImageData(x,y,1,1).data].join(',');
                  for(const id of ['tilemaps','direct-rectangle-presets']){const badge=q('#'+id+'>details>summary time');
                    if(!badge||badge.dateTime!==expected[id].at||!badge.textContent.startsWith(expected[id].kind==='added'?'Added ':'Updated '))throw Error('badge '+id);}
                  if(q('#tilemaps>details>summary h2').textContent!=='Direct rectangle cells'||q('#tilemap-id').options.length!==5||!q('#bg-tiles').hidden)throw Error('scope');
                  if(q('#direct-rectangle-preset-rows').rows.length!==5||q('#direct-rectangle-preset-rows').rows[4].cells[5].textContent!=='19 → 99')throw Error('presets');
                  if(!q('#direct-rectangle-copies').textContent.includes('byte-identical')||canvas.width!==120||canvas.height!==128||pixel(0,0)!=='85,85,85,255'||
                    !q('#tilemap-cell-detail').textContent.includes('source 0xAE9112'))throw Error('initial cells');
                  q('#tilemap-load-map-graphics').click();await new Promise((resolve,reject)=>{let n=0;const timer=setInterval(()=>{
                    if(q('#tilemap-load-map-graphics').hidden){clearInterval(timer);resolve();}else if(++n>100){clearInterval(timer);reject(Error('load'));}},20);});
                  q('#tilemap-graphics').value='0';change(q('#tilemap-graphics'));q('#tilemap-palette-source').value='0';change(q('#tilemap-palette-source'));
                  if(pixel(0,0)!=='255,0,0,255'||q('#tilemap-base').value!=='5')throw Error('manual graphics');
                  q('#direct-rectangle-preset-rows').rows[4].querySelector('a').click();
                  if(q('#tilemap-id').value!=='4'||q('#tilemap-graphics').value!=='0'||q('#tilemap-palette-source').value!=='0'||
                    !q('#tilemap-cell-detail').textContent.includes('source 0xAE98A2')||!q('#tilemaps>details').open)throw Error('preset link');
                  q('#tilemap-cells button:last-child').click();q('#collapse-all').click();
                  return !q('#tilemaps>details').open&&!q('#direct-rectangle-presets>details').open;})()'''.replace('EXPECTED', json.dumps(expected))))
            finally:
                browser.close()


if __name__ == '__main__':
    unittest.main()
