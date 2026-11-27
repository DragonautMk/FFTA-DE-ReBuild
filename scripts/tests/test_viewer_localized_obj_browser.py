"""Exercise the localized OBJ catalog in the existing offline animation player."""
import copy
import json
import shutil
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from test_viewer_firaga_obj_browser import fixture
from test_viewer_static_obj_browser import object_data
from viewer_browser_harness import Chrome


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class LocalizedObjBrowserTest(unittest.TestCase):
    def test_catalog_selection_graphics_animation_and_settings(self):
        with tempfile.TemporaryDirectory(prefix='viewer-localized-obj-') as directory:
            work = Path(directory); index = fixture(work)
            page = index.parent / (index.name + '.parts/objects.html')
            text = page.read_text(); prefix, rest = text.split('const data = ', 1)
            data, end = json.JSONDecoder().raw_decode(rest)
            obj = object_data(); obj.update({'x': 0, 'y': 0, 'shape': 0, 'size': 0, 'tile': 0})
            bank = {'object_root': 0x100, 'sequence_root': 0x200, 'object_offsets': [4], 'sequence_offsets': [4, 14],
                    'lists': [{'source_offset': 0x104, 'objects': [obj]}],
                    'sequences': [{'source_offset': 0x204, 'frames': [{'object_list': 0, 'duration': 1}, {'object_list': 0, 'duration': 2}]},
                                  {'source_offset': 0x20e, 'frames': []}]}
            bank['tile_source'] = {'kind': 'localized', 'language': 0, 'mode': 2, 'offset_tiles': 1, 'length_bytes': 64}
            other = copy.deepcopy(bank); other['lists'][0]['objects'][0]['tile'] = 5
            other['tile_source'] = {'kind': 'localized', 'language': 1, 'mode': 2, 'offset_tiles': 0, 'length_bytes': 64}
            direct = copy.deepcopy(bank); direct['lists'][0]['objects'][0]['tile'] = 10
            direct['tile_source'] = {'kind': 'direct', 'source_offset': 0x430c5c}
            missing = copy.deepcopy(other)
            missing['tile_source']['language'] = 4
            archives = [{'source_offset': 0x400000, 'words': [0] * 8 + [0x22222222] * 16},
                        {'source_offset': 0x410000, 'words': [0x33333333] * 16}]
            data['obj_animation_extra']['localized'] = {'path': 'synthetic.bin', 'banks': [bank, other, direct, missing], 'tile_archives': archives,
                'labels': ['Language 0 · mode 2', 'Language 4 · mode 27', 'Direct', 'Missing source'], 'selector_tables': [],
                'graphics': [{'source_offset': 0x430c5c, 'words': [0x11111111] * 8}]}
            data['resources'].append({'id': 'graphics.localized-obj-animation-banks', 'path': 'synthetic.bin', 'description': 'Localized objects',
                                      'format': 'localized-obj-animation-banks-v1-bin', 'status': 'Available'})
            page.write_text(prefix + 'const data = ' + json.dumps(data, ensure_ascii=True).replace('<', '\\u003c') + rest[end:])
            browser = Chrome(work, page)
            try:
                self.assertTrue(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change',{bubbles:true}));
                  window.pending=new Map();let serial=0;
                  window.setTimeout=(fn,delay)=>{const id=++serial;pending.set(id,{fn,delay});return id};
                  window.clearTimeout=id=>pending.delete(id);
                  const fire=()=>{const [id,item]=pending.entries().next().value;pending.delete(id);item.fn();return item.delay};
                  const row=q('[data-resource-id="graphics.localized-obj-animation-banks"]');
                  if(!row||!row.querySelector('a[href="#obj-animations"]'))throw Error('resource link');
                  row.querySelector('a[href="#obj-animations"]').click();
                  if(q('#obj-animation-catalog').value!=='localized'||q('#obj-animation-bank').options.length!==4||
                     !q('#obj-animation-bank').options[1].textContent.includes('Language 4')||
                     q('#obj-animation-frame').options.length!==2)throw Error('catalog');
                  const choices=[...q('#obj-animation-graphics').options],source=choices.find(o=>o.textContent.includes('Localized direct graphics'));
                  if(!source)throw Error('graphics source');q('#obj-animation-graphics').value=source.value;change(q('#obj-animation-graphics'));
                  q('#obj-animation-outlines').checked=false;change(q('#obj-animation-outlines'));
                  if(!q('#obj-animation-detail').textContent.includes('1 rendered'))throw Error('graphics preview');
                  q('#obj-animation-rate').value='10';q('#obj-animation-loop').checked=true;q('#obj-animation-play').click();
                  if(fire()!==100||q('#obj-animation-frame').value!=='1'||fire()!==200||q('#obj-animation-frame').value!=='0')throw Error('play and loop');
                  q('#obj-animation-stop').click();q('#obj-animation-bank').value='1';change(q('#obj-animation-bank'));
                  if(!q('#obj-animation-loop').checked||q('#obj-animation-graphics').value!==source.value)throw Error('retained settings');
                  q('#obj-animation-sequence').value='1';change(q('#obj-animation-sequence'));
                  if(!q('#obj-animation-play').disabled||q('#obj-animation-frame').options.length)throw Error('empty sequence');
                  q('#obj-animation-bank').value='0';change(q('#obj-animation-bank'));
                  q('#obj-animation-sequence').value='0';change(q('#obj-animation-sequence'));
                  q('#obj-animation-use-linked').click();
                  const pixel=()=>[...q('#obj-animation-canvas').getContext('2d').getImageData(8,8,1,1).data].join(',');
                  if(pixel()!=='34,34,34,255'||!q('#obj-animation-linked-status').textContent.includes('Language 0'))throw Error('linked slice');
                  q('#obj-animation-bank').value='1';change(q('#obj-animation-bank'));
                  if(!q('#obj-animation-linked-status').textContent.includes('Showing linked tiles from Language 0') ||
                     !q('#obj-animation-detail').textContent.includes('1 unrendered'))
                    throw Error('previous bank source is identified');
                  q('#obj-animation-follow-linked').checked=true;change(q('#obj-animation-follow-linked'));
                  if(!q('#obj-animation-follow-linked').checked||q('#obj-animation-base').value!=='5'||pixel()!=='51,51,51,255'||
                     !q('#obj-animation-loop').checked)throw Error('follow selected bank');
                  q('#obj-animation-bank').value='2';change(q('#obj-animation-bank'));
                  if(!q('#obj-animation-follow-linked').checked||q('#obj-animation-base').value!=='10'||pixel()!=='17,17,17,255'||
                     !q('#obj-animation-linked-status').textContent.includes('Direct graphics'))throw Error('direct linked source');
                  q('#obj-animation-bank').value='3';change(q('#obj-animation-bank'));
                  if(!q('#obj-animation-linked-status').textContent.includes('unavailable') ||
                     !q('#obj-animation-detail').textContent.includes('1 unrendered') ||
                     !q('#obj-animation-graphics').selectedOptions[0].textContent.includes('unavailable'))throw Error('missing linked source');
                  q('#obj-animation-bank').value='2';change(q('#obj-animation-bank'));
                  if(pixel()!=='17,17,17,255')throw Error('restore linked source');
                  q('#obj-animation-base').value='4';q('#obj-animation-base').dispatchEvent(new Event('input'));
                  if(q('#obj-animation-follow-linked').checked)throw Error('manual override');
                  return true;})()'''))
            finally:
                browser.close()
