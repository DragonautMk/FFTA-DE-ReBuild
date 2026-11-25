"""Exercise normal and alternate actor sprite catalogs in Chrome."""
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


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class ViewerBattleSpritesTest(unittest.TestCase):
    def test_catalogs_commands_nulls_and_shared_roots(self):
        with tempfile.TemporaryDirectory(prefix='viewer-battle-') as directory:
            work = Path(directory)
            dataset = work / 'dataset'
            dataset.mkdir()
            tile = {'indices': [[1] * 8 for _ in range(8)]}
            obj = {'x': 0, 'y': 0, 'shape': 0, 'size': 0, 'tile': 0, 'transform': 0,
                   'affine_mode': 0, 'object_mode': 0, 'mosaic': False, 'color_256': False}
            def frame(op):
                return {'word00': 0, 'word04': 0, 'duration': 100, 'operation': op,
                        'bytes_0a_13': [0, 0, 0x34, 0x12, 0x78, 0x56, 0, 0, 0, 0]}
            common = {'pixels': [{'source_offset': 0x898a40, 'tiles': [tile]}],
                      'objects': [{'source_offset': 0xa50020, 'objects': [obj]}],
                      'sequences': [{'source_offset': 500, 'opaque_header': 2,
                                     'frames': [frame(1), frame(2), frame(3), frame(6)]}]}
            normal = {**common, 'bindings': [
                {'sprite_id': 10, 'animation': 0, 'part': 0, 'index_source': 100, 'sequence_source': 500},
                {'sprite_id': 10, 'animation': 4, 'part': 1, 'index_source': 104, 'sequence_source': None},
                {'sprite_id': 11, 'animation': 0, 'part': 0, 'index_source': 108, 'sequence_source': 500}]}
            alternate = {**common, 'bindings': [
                {'sprite_id': 10, 'animation': 0, 'part': 0, 'index_source': 200,
                 'target': {'kind': 'sequence', 'source_offset': 500}},
                {'sprite_id': 10, 'animation': 4, 'part': 1, 'index_source': 204,
                 'target': {'kind': 'shared-index-root', 'source_offset': 600, 'sprite_id': 11}},
                {'sprite_id': 11, 'animation': 0, 'part': 0, 'index_source': 208, 'target': {'kind': 'null'}}]}
            manifest = 'profile="test"\nextractor_revision=1\ndecoder_revision=1\n'
            for mode, catalog in [('normal', normal), ('alternate', alternate), ('equipment', normal)]:
                name = f'unit-obj-{mode}-battle' if mode != 'equipment' else 'unit-obj-equipment'
                content = json.dumps(catalog).encode()
                (dataset / (name + '.json')).write_bytes(content)
                manifest += (f'[[resources]]\nid="graphics.{name}"\npath="{name}.json"\n'
                             f'format="{name}-v1-json"\nsha256="{hashlib.sha256(content).hexdigest()}"\n')
            (dataset / 'manifest.toml').write_text(manifest)
            page = generate(dataset, work / 'view')
            browser = Chrome(work, page)
            try:
                result = browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change'));
                  const pixel=()=>[...q('#equipment-pose').getContext('2d').getImageData(0,0,1,1).data].join(',');
                  q('#equipment-sprites>details').open=true;
                  if(q('#equipment-catalog').options.length!==3 || q('#equipment-catalog').value!=='equipment')throw Error('catalogs');
                  q('#equipment-loop').checked=true;q('#equipment-rate').value='40';q('#equipment-play').click();
                  q('#equipment-catalog').value='normal';change(q('#equipment-catalog'));
                  if(q('#equipment-sprite').value!=='10' || q('#equipment-stop').disabled || !q('#equipment-loop').checked || q('#equipment-rate').value!=='40')throw Error('settings across catalogs');
                  q('#equipment-stop').click();q('#equipment-frame').value='1';change(q('#equipment-frame'));
                  if(pixel()!=='17,17,17,255' || !q('#equipment-frame-detail').textContent.includes('0x1234, 0x5678'))throw Error('task halfwords');
                  q('#equipment-frame').value='2';change(q('#equipment-frame'));
                  if(pixel()!=='17,17,17,255' || !q('#equipment-frame-detail').textContent.includes('toggle task flag'))throw Error('task flag');
                  q('#equipment-frame').value='3';change(q('#equipment-frame'));if(pixel()!=='0,0,0,0')throw Error('clear command');
                  q('#equipment-catalog').value='alternate';change(q('#equipment-catalog'));
                  q('#equipment-binding').value='1';change(q('#equipment-binding'));
                  if(!q('#equipment-play').disabled || !q('#equipment-binding-detail').textContent.includes('shared index root 0x258') ||
                     !q('#equipment-binding-detail').textContent.includes('sprite 11') || !q('#equipment-status').textContent.includes('not a sequence'))throw Error('shared index');
                  q('#equipment-sprite').value='11';change(q('#equipment-sprite'));
                  if(!q('#equipment-binding-detail').textContent.includes('null binding'))throw Error('null');
                  const rows=[...document.querySelectorAll('#rows tr')];
                  const link=rows.find(row=>row.cells[0].textContent.includes('graphics.unit-obj-normal-battle')).querySelector('a');link.click();
                  if(q('#equipment-catalog').value!=='normal' || !q('#equipment-sprites>details').open)throw Error('resource navigation');
                  return 'PASS';
                })()''')
                self.assertEqual(result, 'PASS')
            finally:
                browser.close()
