"""Validate physical unit source spans, directory aliases and offline animation."""
import copy
import hashlib
import json
import shutil
import struct
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import view_rom_data as viewer
from viewer_physical_unit import (DIRECTORY, INDEX_START, OBJECTS, PIXELS, SEQUENCES,
                                  parse_physical_unit_bank)
from test_viewer_navbar_split_browser import wait_for
from test_viewer_scoped_browser import parse_page
from viewer_browser_harness import Chrome


def bank():
    roots = [INDEX_START + 24 * i for i in range(244)] + [DIRECTORY - 24]
    parts = [('pose_index', roots[0] + 4, struct.pack('<II', 0x08000000 + SEQUENCES, 0)),
             ('sprite_directory', DIRECTORY + 244 * 4,
              struct.pack('<IIII', *(0x08000000 + root for root in (roots[244], roots[0], roots[1], roots[2])))),
             ('pixel_tiles', PIXELS + 32, bytes([0x22] * 32 + [0x33] * 32)),
             ('object_lists', OBJECTS + 8, struct.pack('<HHHH', 1, 0, 0, 0)),
             ('pose_sequences', SEQUENCES, struct.pack('<HH', 3, 0) +
              b''.join(struct.pack('<IIBB10B', pixel, 8, 1, op, *([0] * 10))
                       for pixel, op in ((32, 1), (64, 1), (0, 6))))]
    sources = [{'role': role, 'offset': offset, 'length': len(data), 'sha256': hashlib.sha256(data).hexdigest()}
               for role, offset, data in parts]
    return roots, b''.join(data for _, _, data in parts), sources


def fixture(work, *, trusted=True, missing_root=False):
    roots, content, sources = bank(); dataset = work / 'dataset'; dataset.mkdir()
    obj = {'x': 0, 'y': 0, 'affine_mode': 0, 'object_mode': 0, 'mosaic': False, 'color_256': False,
           'shape': 0, 'transform': 0, 'size': 0, 'tile': 0, 'priority': 0, 'palette': 0}
    prior = {'roots': [], 'pixels': [{'source_offset': PIXELS, 'tiles': [{'indices': [[1] * 8 for _ in range(8)]}]}],
             'objects': [{'source_offset': OBJECTS, 'objects': [obj]}],
             'sequences': [{'source_offset': SEQUENCES + 0x100, 'opaque_header': 0,
                            'frames': [{'word00': 0, 'word04': 0, 'duration': 1, 'operation': 1,
                                        'bytes_0a_13': [0] * 10}]}],
             'bindings': [{'sprite_id': i, 'animation': 0, 'part': 0, 'index_source': root,
                           'sequence_source': SEQUENCES + 0x100}
                          for i, root in enumerate(roots[:243 if missing_root else 244])]}
    raw_prior = json.dumps(prior).encode(); (dataset / 'prior.json').write_bytes(raw_prior)
    (dataset / 'physical.bin').write_bytes(content)
    manifest = ('profile="europe"\nextractor_revision=370\ndecoder_revision=369\n'
                '[[resources]]\nid="graphics.unit-obj-normal-battle"\npath="prior.json"\n'
                'format="unit-obj-normal-battle-v1-json"\nsha256="' + hashlib.sha256(raw_prior).hexdigest() + '"\n'
                '[[resources]]\nid="graphics.physical-unit-obj-bank"\npath="physical.bin"\n'
                'format="physical-unit-obj-bank-v1-bin"\nsha256="' + hashlib.sha256(content).hexdigest() + '"\n')
    for source in sources:
        manifest += (f'[[resources.sources]]\nrole="{source["role"]}"\noffset={source["offset"]}\n'
                     f'length={source["length"]}\nsha256="{source["sha256"]}"\n')
    (dataset / 'manifest.toml').write_text(manifest)
    with patch.object(viewer, 'EUROPE_PHYSICAL_UNIT_SHA256', hashlib.sha256(content).hexdigest() if trusted else '0' * 64):
        page = viewer.generate(dataset, work / 'view', split=True)
    return page


class PhysicalUnitProjectionTest(unittest.TestCase):
    def test_decodes_source_spans_and_rejects_corruption(self):
        roots, content, sources = bank(); result = parse_physical_unit_bank(content, sources)
        self.assertEqual(len(result['cells']), 2)
        self.assertEqual(result['cells'][0]['target']['source_offset'], SEQUENCES)
        self.assertEqual(result['cells'][1]['target'], {'kind': 'null'})
        self.assertEqual(result['directory_roots'][245], roots[0])
        self.assertEqual(result['pixels'][0]['tiles'][1]['indices'][7][7], 3)
        self.assertEqual(result['objects'][0]['objects'][0]['shape'], 0)
        self.assertEqual([frame['operation'] for frame in result['sequences'][0]['frames']], [1, 1, 6])
        for mutate in (lambda s: s[0].update(sha256='0' * 64),
                       lambda s: s[0].update(offset=0),
                       lambda s: s[0].update(length=999999),
                       lambda s: s[0].update(role='unknown'),
                       lambda s: s.append(copy.deepcopy(s[0]))):
            altered = copy.deepcopy(sources); mutate(altered)
            with self.assertRaises(ValueError):
                parse_physical_unit_bank(content, altered)

    def test_generation_source_guards_and_aliases(self):
        for options in ({}, {'trusted': False}, {'missing_root': True}):
            with self.subTest(options=options), tempfile.TemporaryDirectory(prefix='viewer-physical-unit-') as directory:
                page = fixture(Path(directory), **options); parts = page.parent / 'index.html.parts'
                home, physical = map(parse_page, (page, parts / 'physical-unit-sprites.html'))
                row = next(row for row in home['resources'] if row['id'] == 'graphics.physical-unit-obj-bank')
                self.assertEqual(row['viewer_scope'], 'physical-unit-sprites')
                if options:
                    self.assertFalse(physical['physical_unit_sprites'])
                    self.assertTrue(row['status'].startswith('Preview unavailable'))
                    continue
                catalog = physical['physical_unit_sprites']['physical']
                self.assertNotIn('physical_unit_sprites', home)
                self.assertEqual([(b['sprite_id'], b['animation'], b['part']) for b in catalog['bindings']],
                                 [(0, 0, 1), (0, 0, 2), (245, 0, 1), (245, 0, 2)])
                self.assertEqual(len(catalog['sequences']), 1)
                self.assertEqual(catalog['sequences'][0]['frames'][0]['block'], 0)
                self.assertEqual(catalog['sequences'][0]['frames'][1]['block'], 1)
                self.assertEqual(len(catalog['cells']), 2)
                self.assertNotIn('tiles', catalog['blocks'][0])
                self.assertTrue((parts / catalog['blocks'][0]['tiles_asset']['href']).is_file())
                self.assertIn('2/2 graphics frames resolved', row['status'])


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class PhysicalUnitBrowserTest(unittest.TestCase):
    def test_real_sprite_ids_aliases_and_looped_playback(self):
        with tempfile.TemporaryDirectory(prefix='viewer-physical-unit-browser-') as directory:
            work = Path(directory); page = fixture(work); parts = page.parent / 'index.html.parts'
            browser = Chrome(work, parts / 'physical-unit-sprites.html')
            try:
                wait_for(browser, "Array.isArray(data.physical_unit_sprites.physical.blocks[0].tiles)")
                self.assertTrue(browser.evaluate('''(()=>{const q=s=>document.querySelector(s);
                  return q('#equipment-sprite-label').textContent==='Sprite ID' && q('#equipment-sprite').value==='0' &&
                    [...q('#equipment-sprite').options].some(o=>o.value==='245') && !q('#equipment-index-cells').hidden &&
                    q('#equipment-sprites>details>summary time').dateTime==='2026-11-27T22:15:32Z' &&
                    q('#equipment-source-note').textContent.includes('Directions 0/3');})()'''))
                self.assertTrue(browser.evaluate("loadSpriteBlock(data.physical_unit_sprites.physical.blocks[1]).then(()=>true)"))
                self.assertTrue(browser.evaluate('''(()=>{const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change'));
                  const pixel=()=>[...q('#equipment-pose').getContext('2d').getImageData(0,0,1,1).data].join(',');
                  if(pixel()!=='34,34,34,255')throw Error('first frame');
                  q('#equipment-loop').checked=true;change(q('#equipment-loop'));q('#equipment-sprite').value='245';change(q('#equipment-sprite'));
                  if(!q('#equipment-loop').checked||q('#equipment-frame').value!=='0')throw Error('alias settings');
                  const timers=new Map();let serial=0;window.setTimeout=f=>{timers.set(++serial,f);return serial};window.clearTimeout=id=>timers.delete(id);
                  const step=()=>{const [id,f]=timers.entries().next().value;timers.delete(id);f();};
                  q('#equipment-play').click();step();if(q('#equipment-frame').value!=='1'||pixel()!=='51,51,51,255')throw Error('second frame');
                  step();if(q('#equipment-frame').value!=='2')throw Error('clear frame');step();
                  if(q('#equipment-frame').value!=='0'||!q('#equipment-loop').checked)throw Error('loop');
                  q('#equipment-stop').click();q('#equipment-sprite').value='0';change(q('#equipment-sprite'));
                  return q('#equipment-loop').checked && q('#equipment-sprite').value==='0';})()'''))
            finally:
                browser.close()


if __name__ == '__main__':
    unittest.main()
