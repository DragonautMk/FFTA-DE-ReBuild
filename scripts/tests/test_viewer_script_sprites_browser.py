"""Exercise conditional script sprite catalogs and source witnesses in Chrome."""
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
class ViewerScriptSpritesTest(unittest.TestCase):
    def test_script_only_catalogs_witness_pages_and_settings(self):
        with tempfile.TemporaryDirectory(prefix='viewer-script-sprites-') as directory:
            work = Path(directory)
            dataset = work / 'dataset'
            dataset.mkdir()
            obj = {'x': 0, 'y': 0, 'shape': 0, 'size': 0, 'tile': 0, 'transform': 0,
                   'affine_mode': 0, 'object_mode': 0, 'mosaic': False, 'color_256': False}
            common = {
                'pixels': [{'source_offset': 0x898a40, 'tiles': [{'indices': [[1] * 8 for _ in range(8)]}]}],
                'objects': [{'source_offset': 0xa50020, 'objects': [obj]}],
                'sequences': [{'source_offset': 500, 'opaque_header': 0, 'frames': [
                    {'word00': 0, 'word04': 0, 'duration': 100, 'operation': 1, 'bytes_0a_13': [0] * 10},
                    {'word00': 0, 'word04': 0, 'duration': 100, 'operation': 6, 'bytes_0a_13': [0] * 10}]}],
            }
            keys = [(10, 0), (10, 1), (10, 4), (11, 0), (12, 0)]
            producers = [dict(mode='normal', sprite_id=10, base=0, selector=i, unit_kind=1,
                              unit_logical_id=2, job_id=3, metadata_source=300,
                              script_sources=[1000 + i, 2000 + i]) for i in range(21)]
            producers += [dict(sprite_id=10, base=4, assignment_source=400, script_sources=[3000]),
                          dict(sprite_id=11, base=0, assignment_source=400, script_sources=[3001]),
                          dict(sprite_id=12, base=0, assignment_source=400, script_sources=[3002])]
            manifest = 'profile="test"\nextractor_revision=1\ndecoder_revision=1\n'
            modes = ('special-normal', 'special-alternate', 'override', 'registry-special', 'registry-generic')
            for mode in modes:
                bindings = []
                for index, (sprite, animation) in enumerate(keys):
                    binding = {'sprite_id': sprite, 'animation': animation, 'part': 0, 'index_source': 100 + index * 4}
                    if mode == 'registry-generic':
                        binding['target'] = ({'kind': 'sequence', 'source_offset': 500} if index < 3 else
                                             {'kind': 'null'} if index == 3 else
                                             {'kind': 'shared-index-root', 'source_offset': 600, 'sprite_id': 20})
                    else:
                        binding['sequence_source'] = 500 if index < 3 else None
                    bindings.append(binding)
                catalog = {**common, 'bindings': bindings, 'producers': producers}
                name = 'unit-obj-script-' + mode
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
                  const modes=['special-normal','special-alternate','override','registry-special','registry-generic'];
                  if(q('#equipment-sprites').hidden || q('#equipment-catalog').options.length!==5)throw Error('script-only catalogs');
                  for(const mode of modes){
                    const row=[...document.querySelectorAll('#rows tr')].find(r=>r.cells[0].textContent.includes('graphics.unit-obj-script-'+mode+')'));
                    if(!row || !row.querySelector('a'))throw Error('resource '+mode);
                    row.querySelector('a').click();if(q('#equipment-catalog').value!==mode)throw Error('navigation '+mode);
                  }
                  if(q('#equipment-witnesses').hidden || q('#equipment-witnesses').open)throw Error('witness disclosure');
                  q('#equipment-witnesses').open=true;
                  if(!q('#equipment-witnesses p').textContent.includes('do not') || !q('#equipment-witness-page').textContent.includes('21 witnesses'))throw Error('conditions');
                  let cells=[...q('#equipment-witness-rows tr').cells].map(c=>c.textContent);
                  if(cells.join('|')!=='normal|0|1|2|3|0|metadata 0x12C|0x3E8, 0x7D0')throw Error('source fields');
                  if(q('#equipment-witness-rows').rows.length!==20 || q('#equipment-witness-next').disabled)throw Error('first page');
                  q('#equipment-witness-next').click();
                  if(q('#equipment-witness-rows').rows.length!==1 || !q('#equipment-witness-page').textContent.includes('page 2 of 2'))throw Error('second page');
                  q('#equipment-binding').value='1';change(q('#equipment-binding'));
                  if(!q('#equipment-witness-page').textContent.includes('21 witnesses') || !q('#equipment-witness-page').textContent.includes('page 1'))throw Error('direction witness');
                  q('#equipment-binding').value='2';change(q('#equipment-binding'));
                  cells=[...q('#equipment-witness-rows tr').cells].map(c=>c.textContent);
                  if(!q('#equipment-witness-page').textContent.includes('1 witnesses') || cells[6]!=='assignment 0x190')throw Error('base witness');
                  q('#equipment-binding').value='0';change(q('#equipment-binding'));
                  if([...q('#equipment-pose').getContext('2d').getImageData(0,0,1,1).data].join(',')!=='17,17,17,255')throw Error('pixel');
                  q('#equipment-loop').checked=true;q('#equipment-rate').value='40';q('#equipment-play').click();
                  q('#equipment-catalog').value='override';change(q('#equipment-catalog'));
                  if(q('#equipment-stop').disabled || !q('#equipment-loop').checked || q('#equipment-rate').value!=='40')throw Error('catalog settings');
                  q('#equipment-catalog').value='registry-generic';change(q('#equipment-catalog'));
                  q('#equipment-sprite').value='12';change(q('#equipment-sprite'));
                  if(!q('#equipment-play').disabled || !q('#equipment-loop').checked || q('#equipment-rate').value!=='40' ||
                     !q('#equipment-binding-detail').textContent.includes('shared index root 0x258') ||
                     !q('#equipment-witness-page').textContent.includes('1 witnesses'))throw Error('shared root');
                  q('#equipment-sprite').value='11';change(q('#equipment-sprite'));
                  if(!q('#equipment-binding-detail').textContent.includes('null binding'))throw Error('null');
                  return 'PASS';
                })()''')
                self.assertEqual(result, 'PASS')
            finally:
                browser.close()
