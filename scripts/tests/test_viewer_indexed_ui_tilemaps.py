"""Verify EUR indexed UI map geometry, aliases and scoped navigation."""
import copy
import hashlib
import json
import shutil
import struct
import tempfile
import unittest
from pathlib import Path

from test_viewer_direct_rectangles import chunks as direct_chunks
from view_rom_data import generate, project_indexed_ui_tilemaps, viewer_section_updates
from viewer_browser_harness import Chrome

GEOMETRY = ([(0xb543a8, 27, 2), (0xb54414, 27, 2), (0xb54480, 27, 1)]
            + [(0xb544b6 + 60 * index, 15, 2) for index in range(20)]
            + [(0xb54966 + 30 * index, 15, 1) for index in range(8)])


def chunks():
    targets = [offset for offset, _, _ in GEOMETRY[3:]]
    directories = [targets[:10], targets[10:18] + targets[16:18], targets[10:20], targets[20:28] + targets[26:28]]
    directory_bytes = struct.pack('<40I', *(0x08000000 + address for row in directories for address in row))
    cells = b''.join(struct.pack('<' + str(width * height) + 'H',
                    *([0xfffe, index, 0x1405] * ((width * height + 2) // 3))[:width * height])
                    for index, (_, width, height) in enumerate(GEOMETRY))
    return [(0x03a88c, 'reader_literals', struct.pack('<2I', 0x08395a44, 0x08395a6c)),
            (0x03a8f8, 'reader_literals', struct.pack('<I', 0x08395a94)),
            (0x03aaf8, 'reader_literals', struct.pack('<I', 0x08395a44)),
            (0x03ab00, 'reader_literals', struct.pack('<I', 0x08395abc)),
            (0x03aeec, 'reader_literals', struct.pack('<I', 0x08b543a8)),
            (0x03b084, 'reader_literals', struct.pack('<2I', 0x08b54414, 0x08b54480)),
            (0x395a44, 'indexed_directories', directory_bytes), (0xb543a8, 'tilemap_cells', cells)]


def sources(parts):
    return [{'offset': offset, 'role': role, 'length': len(raw), 'sha256': hashlib.sha256(raw).hexdigest()}
            for offset, role, raw in parts]


def project(parts):
    return project_indexed_ui_tilemaps(b''.join(part[2] for part in parts), sources(parts))


def fixture(work, invalid=False, profile='europe', split=True):
    dataset = work / 'dataset'; dataset.mkdir(); parts = chunks(); prior = direct_chunks()
    graphics = {'descriptor_graphics': [0], 'graphics': [{'relative_source': 400, 'destination_offset': 0,
                'words': [0x11111111] * 8}]}
    palette = {'entries': [{'banks': [[0, 31] + [0] * 14]}]}
    items = [('graphics.indexed-ui-tilemaps', 'indexed.bin', 'indexed-ui-tilemaps-v1-bin', b''.join(p[2] for p in parts), sources(parts)),
             ('graphics.direct-rectangle-directory', 'direct.bin', 'direct-rectangle-directory-v1-bin', b''.join(p[2] for p in prior), sources(prior)),
             ('graphics.bg-main-graphics', 'graphics.json', 'bg-main-graphics-v1-json', json.dumps(graphics).encode(), []),
             ('graphics.bg-base-palettes', 'palette.json', 'bg-base-palettes-v2-json', json.dumps(palette).encode(), [])]
    manifest = f'profile="{profile}"\nextractor_revision=367\ndecoder_revision=366\n'
    for identity, path, fmt, content, entries in items:
        (dataset / path).write_bytes(content)
        manifest += f'[[resources]]\nid="{identity}"\npath="{path}"\nformat="{fmt}"\nsha256="{hashlib.sha256(content).hexdigest()}"\n'
        for index, entry in enumerate(entries):
            digest = '0' * 64 if invalid and identity == 'graphics.indexed-ui-tilemaps' and index == 7 else entry['sha256']
            manifest += (f'[[resources.sources]]\noffset={entry["offset"]}\nlength={entry["length"]}\n'
                         f'role="{entry["role"]}"\nsha256="{digest}"\n')
    (dataset / 'manifest.toml').write_text(manifest)
    return generate(dataset, work / 'view', split=split)


def payload(path):
    return json.JSONDecoder().raw_decode(path.read_text().split('const data = ', 1)[1])[0]


class IndexedUiProjectionTest(unittest.TestCase):
    def test_geometry_cells_and_directory_aliases(self):
        result = project(chunks()); maps, dirs = result['maps'], result['directories']
        self.assertEqual([(m['source_offset'], m['width'], m['height']) for m in maps], GEOMETRY)
        self.assertEqual(sum(len(m['cells']) for m in maps), 855)
        self.assertEqual([len(d['maps']) for d in dirs], [10] * 4)
        self.assertEqual(dirs[1]['maps'][6], dirs[1]['maps'][8])
        self.assertEqual(set(i for directory in dirs for i in directory['maps']), set(range(3, 31)))
        self.assertEqual(maps[0]['cells'][:3], [0xfffe, 0, 0x1405])

    def test_reject_bad_hash_literals_targets_and_incomplete_coverage(self):
        parts = chunks(); raw = b''.join(part[2] for part in parts); bad = sources(parts); bad[-1]['sha256'] = '0' * 64
        with self.assertRaisesRegex(ValueError, 'SHA-256'): project_indexed_ui_tilemaps(raw, bad)
        bad = copy.deepcopy(parts); value = bytearray(bad[0][2]); value[0] ^= 1; bad[0] = (*bad[0][:2], bytes(value))
        with self.assertRaisesRegex(ValueError, 'literals'): project(bad)
        for pointer, message in ((0, 'target'), (0x08000000 + GEOMETRY[23][0], 'target'),
                                 (0x08000000 + GEOMETRY[4][0], 'coverage')):
            bad = copy.deepcopy(parts); value = bytearray(bad[6][2]); struct.pack_into('<I', value, 0, pointer)
            bad[6] = (*bad[6][:2], bytes(value))
            with self.subTest(pointer=pointer), self.assertRaisesRegex(ValueError, message): project(bad)
        bad = chunks(); bad[-1] = (bad[-1][0] + 2, *bad[-1][1:])
        with self.assertRaisesRegex(ValueError, 'sources'): project(bad)

    def test_scope_and_unverified_sources(self):
        for invalid, profile in ((False, 'europe'), (True, 'europe'), (False, 'usa')):
            with self.subTest(invalid=invalid, profile=profile), tempfile.TemporaryDirectory(prefix='viewer-indexed-ui-scope-') as directory:
                index = fixture(Path(directory), invalid, profile); folder = index.parent / (index.name + '.parts')
                home, page, other = map(payload, (index, folder / 'interface-indexed-ui.html', folder / 'interface-rectangles.html'))
                self.assertNotIn('indexed_ui_tilemaps', home); self.assertNotIn('indexed_ui_tilemaps', other)
                self.assertNotIn('bg_graphics', page); self.assertNotIn('fixed_map_graphics', page)
                valid = not invalid and profile == 'europe'
                self.assertEqual('fixed_map_graphics_asset' in page, valid)
                self.assertEqual(page['indexed_ui_tilemaps'] is not None, valid)
                sections = [s['id'] for p in home['wiki']['pages'] if p['id'] == 'interface-indexed-ui' for s in p['sections']]
                self.assertEqual(sections, ['indexed-ui-directories', 'tilemaps'] if valid else [])


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class IndexedUiBrowserTest(unittest.TestCase):
    def test_directories_aliases_addend_and_badges(self):
        with tempfile.TemporaryDirectory(prefix='viewer-indexed-ui-browser-') as directory:
            work = Path(directory); index = fixture(work); page = index.parent / (index.name + '.parts') / 'interface-indexed-ui.html'
            browser = Chrome(work, page)
            try:
                expected = viewer_section_updates()['interface-indexed-ui']
                self.assertTrue(browser.evaluate('''(()=>{const expected=EXPECTED,q=s=>document.querySelector(s),event=(e,n)=>e.dispatchEvent(new Event(n));
                  for(const id of ['tilemaps','indexed-ui-directories']){const badge=q('#'+id+'>details>summary time');
                    if(!badge||badge.dateTime!==expected[id].at||!badge.textContent.startsWith(expected[id].kind==='added'?'Added ':'Updated '))throw Error('badge '+id);}
                  if(q('#tilemaps>details>summary h2').textContent!=='Indexed UI map cells'||q('#tilemap-id').options.length!==31||!q('#bg-tiles').hidden)throw Error('scope');
                  if(q('#indexed-ui-directory-rows').rows.length!==4||q('#indexed-ui-directory-rows').querySelectorAll('a').length!==40)throw Error('directories');
                  if(!q('#indexed-ui-map-references').textContent.includes('language-byte')||!q('#tilemap-cell-detail').textContent.includes('source 0xB543A8'))throw Error('direct');
                  const row=q('#indexed-ui-directory-rows').rows[1];row.cells[7].querySelector('a').click();const first=q('#tilemap-id').value;
                  if(!q('#indexed-ui-map-references').textContent.includes('slot 6')||!q('#indexed-ui-map-references').textContent.includes('slot 8'))throw Error('aliases');
                  row.cells[9].querySelector('a').click();if(q('#tilemap-id').value!==first||!q('#tilemaps>details').open)throw Error('alias link');
                  q('#indexed-ui-addend').value='3';event(q('#indexed-ui-addend'),'input');
                  if(!q('#tilemap-cell-detail').textContent.includes('shown 0x0001')||!q('#tilemap-cell-detail').textContent.includes('stored 0xFFFE'))throw Error('wrapping');
                  q('#indexed-ui-addend').value='65536';event(q('#indexed-ui-addend'),'input');
                  if(!q('#indexed-ui-addend-status').textContent.includes('retains 3')||!q('#tilemap-cell-detail').textContent.includes('shown 0x0001'))throw Error('invalid addend');
                  q('#indexed-ui-addend').value='0';event(q('#indexed-ui-addend'),'input');
                  if(!q('#tilemap-cell-detail').textContent.includes('raw 0xFFFE'))throw Error('stored');
                  q('#collapse-all').click();return !q('#tilemaps>details').open&&!q('#indexed-ui-directories>details').open;})()'''.replace('EXPECTED', json.dumps(expected))))
            finally:
                browser.close()

    def test_single_page_resolves_map_after_other_sources(self):
        with tempfile.TemporaryDirectory(prefix='viewer-indexed-ui-single-') as directory:
            work = Path(directory); page = fixture(work, split=False); browser = Chrome(work, page)
            try:
                self.assertTrue(browser.evaluate('''(()=>{const q=s=>document.querySelector(s),a=q('#indexed-ui-directory-rows').rows[0].cells[1].querySelector('a');
                  a.click();return q('#tilemap-id').value==='8' && q('#tilemap-cell-detail').textContent.includes('source 0xB544B6');})()'''))
            finally:
                browser.close()


if __name__ == '__main__':
    unittest.main()
