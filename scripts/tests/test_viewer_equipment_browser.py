"""Check equipment frames, commands, null bindings and playback in Chrome."""
import json
import shutil
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from view_rom_data import project_equipment_sprites
from viewer_browser_harness import Chrome


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class ViewerEquipmentTest(unittest.TestCase):
    def test_equipment_preview_and_animation(self):
        with tempfile.TemporaryDirectory(prefix='viewer-equipment-') as directory:
            work = Path(directory)
            repo = Path(__file__).resolve().parents[2]
            tile = lambda value: {'indices': [[value] * 8 for _ in range(8)]}
            obj = {'x': 0, 'y': 0, 'shape': 0, 'size': 0, 'tile': 0, 'transform': 0,
                   'affine_mode': 0, 'object_mode': 0, 'mosaic': False, 'color_256': False}
            def frame(op, word=0, duration=1):
                return {'word00': word, 'word04': 0, 'duration': duration, 'operation': op,
                        'bytes_0a_13': [7] + [0] * 9}
            source = {'pixels': [{'source_offset': 0x898a40, 'tiles': [tile(1)]},
                                 {'source_offset': 0x898a60, 'tiles': [tile(2)]}],
                      'objects': [{'source_offset': 0xa50020, 'objects': [obj]}],
                      'sequences': [{'source_offset': 500, 'opaque_header': 2,
                                     'frames': [frame(1), frame(8), frame(6), frame(1, 32), frame(0)]}],
                      'bindings': [{'sprite_id': 3, 'animation': 4, 'part': 0, 'index_source': 100, 'sequence_source': 500},
                                   {'sprite_id': 3, 'animation': 4, 'part': 1, 'index_source': 104, 'sequence_source': None},
                                   {'sprite_id': 4, 'animation': 4, 'part': 0, 'index_source': 108, 'sequence_source': 500}]}
            catalog = project_equipment_sprites(source)
            catalog['path'] = 'equipment.json'
            data = {'profile': 'synthetic', 'extractor': 1, 'decoder': 1, 'resources': [], 'equipment_sprites': catalog}
            template = (repo / 'view_rom_data.html').read_text()
            page = work / 'equipment.html'
            page.write_text(template.replace('/* DATA */', json.dumps(data)))
            browser = Chrome(work, page)
            try:
                result = browser.evaluate('''(async()=>{
                  const q=s=>document.querySelector(s), change=e=>e.dispatchEvent(new Event('change'));
                  const wait=async f=>{let end=Date.now()+2000;while(!f()){if(Date.now()>end)throw Error('timeout');await new Promise(r=>setTimeout(r,10))}};
                  const pixel=()=>[...q('#equipment-pose').getContext('2d').getImageData(0,0,1,1).data].join(',');
                  q('#equipment-sprites>details').open=true;
                  if(q('#equipment-sprites').hidden || q('#equipment-sprite').options.length!==2 || pixel()!=='17,17,17,255')throw Error('initial graphics');
                  q('#equipment-frame').value='1';change(q('#equipment-frame'));
                  if(pixel()!=='17,17,17,255' || !q('#equipment-frame-detail').textContent.includes('event byte 0x07'))throw Error('event holds graphic');
                  q('#equipment-frame').value='2';change(q('#equipment-frame'));
                  if(pixel()!=='0,0,0,0' || !q('#equipment-pose-detail').textContent.includes('No graphics'))throw Error('clear');
                  q('#equipment-frame').value='3';change(q('#equipment-frame'));if(pixel()!=='34,34,34,255')throw Error('second graphic');
                  q('#equipment-frame').value='4';change(q('#equipment-frame'));if(pixel()!=='34,34,34,255')throw Error('default holds graphic');
                  q('#equipment-binding').value='1';change(q('#equipment-binding'));
                  if(!q('#equipment-play').disabled || !q('#equipment-binding-detail').textContent.includes('null binding') || pixel()!=='0,0,0,0')throw Error('null');
                  q('#equipment-binding').value='0';change(q('#equipment-binding'));q('#equipment-rate').value='120';
                  q('#equipment-frame').value='3';change(q('#equipment-frame'));q('#equipment-play').click();
                  await wait(()=>q('#equipment-status').textContent.includes('finished'));
                  if(q('#equipment-frame').value!=='4' || !q('#equipment-stop').disabled)throw Error('one-shot');
                  q('#equipment-loop').checked=true;q('#equipment-play').click();q('#equipment-sprite').value='4';change(q('#equipment-sprite'));
                  if(!q('#equipment-loop').checked || q('#equipment-rate').value!=='120' || q('#equipment-stop').disabled || q('#equipment-frame').value!=='4')throw Error('settings');
                  await wait(()=>q('#equipment-frame').value==='0');q('#equipment-sprites>details').open=false;
                  await wait(()=>q('#equipment-stop').disabled);return 'PASS';
                })()''')
                self.assertEqual(result, 'PASS')
            finally:
                browser.close()
