"""Verify the EUR queued map bank, original paths and bounded row preview."""
import copy
import hashlib
import json
import shutil
import struct
import tempfile
import unittest
from pathlib import Path

from test_viewer_direct_rectangles import chunks as direct_chunks
from view_rom_data import generate, project_queued_ui_maps, viewer_section_updates
from viewer_browser_harness import Chrome

LITERALS = (0x093c2c, 0x093f50, 0x094298, 0x0945d8, 0x09525c, 0x12d4ec, 0x12d674)
TARGETS = (0x3ba3d0, 0x3ba3d0, 0x3ba3d0, 0x3ba3d0, 0x3ba358, 0x3ba4c0, 0x3ba4c0, 0x3ba7ac, 0x3ba720)
GEOMETRY = ((0x3ba358, 20, 3), (0x3ba3d0, 30, 4), (0x3ba4c0, 19, 16),
            (0x3ba720, 10, 7), (0x3ba7ac, 10, 7))


def chunks():
    result = [(offset, 'queue_literals', struct.pack('<I', 0x08000000 + target))
              for offset, target in zip(LITERALS, TARGETS[:7])]
    result.append((0x12d7cc, 'queue_literals', struct.pack('<2I', *(0x08000000 + target for target in TARGETS[7:]))))
    cells = b''.join(struct.pack('<' + str(width * height) + 'H',
                     *([5, 0x1405, 0x1805, 0xfc06] * ((width * height + 3) // 4))[:width * height])
                     for _, width, height in GEOMETRY)
    result.append((0x3ba358, 'tilemap_cells', cells))
    return result


def sources(parts):
    return [{'offset': offset, 'role': role, 'length': len(raw), 'sha256': hashlib.sha256(raw).hexdigest()}
            for offset, role, raw in parts]


def project(parts):
    return project_queued_ui_maps(b''.join(part[2] for part in parts), sources(parts))


def fixture(work, invalid=False, profile='europe', split=True):
    dataset = work / 'dataset'; dataset.mkdir(); parts = chunks(); prior = direct_chunks()
    graphics = {'descriptor_graphics': [0], 'graphics': [{'relative_source': 400, 'destination_offset': 5 * 32,
                'words': [1] + [0] * 6 + [2] + [0x33333333] * 8}]}
    palette = {'entries': [{'banks': [[0, 31, 992, 31744] + [0] * 12]}]}
    items = [('graphics.queued-ui-maps', 'queued.bin', 'queued-ui-maps-v1-bin', b''.join(p[2] for p in parts), sources(parts)),
             ('graphics.direct-rectangle-directory', 'direct.bin', 'direct-rectangle-directory-v1-bin', b''.join(p[2] for p in prior), sources(prior)),
             ('graphics.bg-main-graphics', 'graphics.json', 'bg-main-graphics-v1-json', json.dumps(graphics).encode(), []),
             ('graphics.bg-base-palettes', 'palette.json', 'bg-base-palettes-v2-json', json.dumps(palette).encode(), [])]
    manifest = f'profile="{profile}"\nextractor_revision=369\ndecoder_revision=368\n'
    for identity, path, fmt, content, entries in items:
        (dataset / path).write_bytes(content)
        manifest += f'[[resources]]\nid="{identity}"\npath="{path}"\nformat="{fmt}"\nsha256="{hashlib.sha256(content).hexdigest()}"\n'
        for index, entry in enumerate(entries):
            digest = '0' * 64 if invalid and identity == 'graphics.queued-ui-maps' and index == 8 else entry['sha256']
            manifest += (f'[[resources.sources]]\noffset={entry["offset"]}\nlength={entry["length"]}\n'
                         f'role="{entry["role"]}"\nsha256="{digest}"\n')
    (dataset / 'manifest.toml').write_text(manifest)
    return generate(dataset, work / 'view', split=split)


def payload(path):
    return json.JSONDecoder().raw_decode(path.read_text().split('const data = ', 1)[1])[0]


class QueuedUiProjectionTest(unittest.TestCase):
    def test_geometry_cells_and_original_paths(self):
        result = project(chunks()); maps, paths = result['maps'], result['paths']
        self.assertEqual([(m['source_offset'], m['width'], m['height']) for m in maps], list(GEOMETRY))
        self.assertEqual(sum(len(m['cells']) for m in maps), 624)
        self.assertEqual([maps[p['map_index']]['source_offset'] for p in paths], list(TARGETS))
        self.assertEqual([m['queued_ui_live'] for m in maps], [False, False, True, False, False])
        self.assertEqual(result['destination_stride'], 64)
        self.assertEqual(maps[2]['cells'][:4], [5, 0x1405, 0x1805, 0xfc06])

    def test_reject_bad_hash_literals_and_extents(self):
        parts = chunks(); raw = b''.join(p[2] for p in parts); bad = sources(parts); bad[-1]['sha256'] = '0' * 64
        with self.assertRaisesRegex(ValueError, 'SHA-256'): project_queued_ui_maps(raw, bad)
        for index in (0, 7):
            bad = copy.deepcopy(parts); raw = bytearray(bad[index][2]); raw[0] ^= 1; bad[index] = (*bad[index][:2], bytes(raw))
            with self.subTest(index=index), self.assertRaisesRegex(ValueError, 'literals'): project(bad)
        bad = chunks(); bad[-1] = (bad[-1][0] + 2, *bad[-1][1:])
        with self.assertRaisesRegex(ValueError, 'sources'): project(bad)

    def test_scope_and_unverified_sources(self):
        for invalid, profile in ((False, 'europe'), (True, 'europe'), (False, 'usa')):
            with self.subTest(invalid=invalid, profile=profile), tempfile.TemporaryDirectory(prefix='viewer-queued-scope-') as directory:
                index = fixture(Path(directory), invalid, profile); folder = index.parent / (index.name + '.parts')
                home, page, direct = map(payload, (index, folder / 'interface-queued-maps.html', folder / 'interface-rectangles.html'))
                self.assertNotIn('queued_ui_maps', home); self.assertNotIn('queued_ui_maps', direct)
                valid = not invalid and profile == 'europe'
                self.assertEqual(page['queued_ui_maps'] is not None, valid)
                self.assertEqual('fixed_map_graphics_asset' in page, valid)
                if valid: self.assertEqual(page['fixed_map_graphics_asset'], direct['fixed_map_graphics_asset'])
                sections = [s['id'] for p in home['wiki']['pages'] if p['id'] == 'interface-queued-maps' for s in p['sections']]
                self.assertEqual(sections, ['queued-ui-paths', 'tilemaps'] if valid else [])


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class QueuedUiBrowserTest(unittest.TestCase):
    def test_paths_live_rows_and_manual_graphics(self):
        with tempfile.TemporaryDirectory(prefix='viewer-queued-browser-') as directory:
            work = Path(directory); index = fixture(work); page = index.parent / (index.name + '.parts') / 'interface-queued-maps.html'
            browser = Chrome(work, page)
            try:
                expected = viewer_section_updates()['interface-queued-maps']
                self.assertTrue(browser.evaluate('''(async()=>{const expected=EXPECTED,q=s=>document.querySelector(s),event=(e,n)=>e.dispatchEvent(new Event(n));
                  for(const id of ['tilemaps','queued-ui-paths']){const badge=q('#'+id+'>details>summary time');
                    if(!badge||badge.dateTime!==expected[id].at||!badge.textContent.startsWith('Added '))throw Error('badge '+id);}
                  if(q('#tilemaps>details>summary h2').textContent!=='Queued UI map cells'||q('#tilemap-id').options.length!==5||q('#queued-ui-path-rows').rows.length!==9)throw Error('scope');
                  const rows=q('#queued-ui-path-rows').rows;rows[5].cells[2].querySelector('a').click();
                  if(q('#tilemap-id').value!=='2'||q('#queued-ui-row-controls').hidden||q('#tilemap-canvas').height!==128)throw Error('live path');
                  q('#queued-ui-rows').value='1';event(q('#queued-ui-rows'),'input');
                  if(q('#tilemap-canvas').height!==8||q('#tilemap-cells').querySelectorAll('button').length!==19)throw Error('one row');
                  q('#queued-ui-rows').value='17';event(q('#queued-ui-rows'),'input');
                  if(!q('#queued-ui-row-status').textContent.includes('retains 1')||q('#tilemap-canvas').height!==8)throw Error('bounds');
                  q('#queued-ui-rows').value='3';event(q('#queued-ui-rows'),'input');
                  rows[7].cells[2].querySelector('a').click();if(q('#tilemap-id').value!=='4'||!q('#queued-ui-row-controls').hidden)throw Error('branch');
                  rows[8].cells[2].querySelector('a').click();if(q('#tilemap-id').value!=='3')throw Error('other branch');
                  rows[6].cells[2].querySelector('a').click();if(q('#tilemap-id').value!=='2'||q('#tilemap-canvas').height!==24)throw Error('preserve rows');
                  q('#tilemap-load-map-graphics').click();for(let i=0;i<100&&!q('#tilemap-load-map-graphics').hidden;i++)await new Promise(r=>setTimeout(r,20));
                  q('#tilemap-graphics').value='0';event(q('#tilemap-graphics'),'change');q('#tilemap-palette-source').value='0';event(q('#tilemap-palette-source'),'change');
                  const pixel=[...q('#tilemap-canvas').getContext('2d').getImageData(0,0,1,1).data].join(',');
                  q('#tilemap-cells button').click();if(pixel!=='255,0,0,255'||!q('#tilemap-cell-detail').textContent.includes('source 0x3BA4C0'))throw Error('graphics or source');
                  q('#collapse-all').click();return !q('#tilemaps>details').open&&!q('#queued-ui-paths>details').open;})()'''.replace('EXPECTED', json.dumps(expected))))
            finally:
                browser.close()

    def test_single_page_path_selects_correct_map(self):
        with tempfile.TemporaryDirectory(prefix='viewer-queued-single-') as directory:
            work = Path(directory); page = fixture(work, split=False); browser = Chrome(work, page)
            try:
                self.assertTrue(browser.evaluate('''(()=>{const q=s=>document.querySelector(s);q('#queued-ui-path-rows').rows[5].cells[2].querySelector('a').click();
                  return q('#tilemap-id').value==='7'&&q('#tilemap-cell-detail').textContent.includes('source 0x3BA4C0')&&
                    q('#tilemap-load-map-graphics').hidden;})()'''))
            finally:
                browser.close()


if __name__ == '__main__':
    unittest.main()
