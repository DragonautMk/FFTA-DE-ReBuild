"""Exercise indexed OBJ frames, manual graphics and animation controls."""
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


def fixture(work, *, graphics=True, extra=False, indexed=True):
    dataset = work / 'dataset'
    dataset.mkdir()
    def obj(**overrides):
        value = {'x': 0, 'y': 0, 'affine_mode': 0, 'object_mode': 0, 'mosaic': False, 'color_256': False,
                 'shape': 0, 'transform': 0, 'size': 0, 'tile': 0, 'priority': 0, 'palette': 1}
        value.update(overrides)
        return value
    banks = {'banks': [
        {'object_root': 1000, 'object_offsets': [12, 20, 28], 'lists': [
            {'source_offset': 1012, 'objects': [obj(transform=8)]},
            {'source_offset': 1020, 'objects': [obj(color_256=True, tile=2)]},
            {'source_offset': 1028, 'objects': [obj(shape=3)]}],
         'sequence_root': 2000, 'sequence_offsets': [12, 22, 28], 'sequences': [
            {'source_offset': 2012, 'frames': [{'object_list': 0, 'duration': 2}, {'object_list': 1, 'duration': 0}]},
            {'source_offset': 2022, 'frames': [{'object_list': 2, 'duration': 1}]},
            {'source_offset': 2028, 'frames': [], 'zero_count_hold': {'object_list': 1, 'duration_source_offset': 2032}}]},
        {'object_root': 3000, 'object_offsets': [4], 'lists': [{'source_offset': 3004, 'objects': [obj()]}],
         'sequence_root': 4000, 'sequence_offsets': [4], 'sequences': [
            {'source_offset': 4004, 'frames': [{'object_list': 0, 'duration': 1}]}]}],
        'selector_tables': [{'selector_address': 5000, 'object_table_offset': 6000, 'sequence_table_offset': 7000,
                             'object_roots': [1000, 3000], 'sequence_roots': [2000, 4000]}]}
    words = [0x11111112] * 8 + [0] * 8 + [0x22222222] * 16
    graphic = {'banks': [{'source_offset': 8000, 'words': words}]}
    full = [0] * 256
    full[1], full[2], full[17], full[34] = 0x001f, 0x03e0, 0x7c00, 0x03e0
    palettes = {'banks': [{'source_offset': 9000, 'colors': full}, {'source_offset': 10000, 'colors': [0] * 16}]}
    resources = [('graphics.obj-animation-banks', 'obj-animation-banks', banks)] if indexed else []
    if extra:
        word = {'banks': [banks['banks'][1]]}
        layered_bank = json.loads(json.dumps(banks['banks'][0]))
        layered_bank['lists'][1]['objects'][0]['x'] = 8
        layered_bank['sequences'][1]['frames'] = [{'object_list': 1, 'duration': 0}, {'object_list': 0, 'duration': 1}]
        layered_bank['sequence_offsets'][2] = 32
        layered_bank['sequences'][2]['source_offset'] = 2032
        layered_bank['sequences'][2]['zero_count_hold']['duration_source_offset'] = 2036
        layered = {'banks': [layered_bank],
                   'descriptors': [{'object_root': 1000, 'sequence_root': 2000, 'opaque': list(range(20))}],
                   'title_language_bank': banks['banks'][1], 'title_main_bank': banks['banks'][1]}
        resources += [('graphics.word-task-obj-animations', 'word-task-obj-animations', word),
                      ('graphics.layered-obj-animations', 'layered-obj-animations', layered)]
    if graphics:
        resources += [('graphics.obj-custom-banks', 'obj-custom-banks', graphic),
                      ('graphics.ui-raw-palettes', 'ui-raw-palettes', palettes)]
    manifest = 'profile="test"\nextractor_revision=1\ndecoder_revision=1\n'
    for resource_id, name, value in resources:
        content = json.dumps(value).encode()
        (dataset / (name + '.json')).write_bytes(content)
        manifest += (f'[[resources]]\nid="{resource_id}"\npath="{name}.json"\n'
                     f'format="{name}-v1-json"\nsha256="{hashlib.sha256(content).hexdigest()}"\n')
    (dataset / 'manifest.toml').write_text(manifest)
    return generate(dataset, work / 'view')


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class ViewerObjAnimationsTest(unittest.TestCase):
    def test_layout_manual_pixels_timing_settings_and_empty_frames(self):
        with tempfile.TemporaryDirectory(prefix='viewer-obj-animations-') as directory:
            work = Path(directory)
            browser = Chrome(work, fixture(work))
            try:
                self.assertEqual(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change'));
                  const pixel=(x,y)=>[...q('#obj-animation-canvas').getContext('2d').getImageData(x,y,1,1).data].join(',');
                  window.pending=new Map();let serial=0;
                  window.setTimeout=(fn,delay)=>{const id=++serial;pending.set(id,{fn,delay});return id};
                  window.clearTimeout=id=>pending.delete(id);
                  window.fireObjTimer=()=>{const [id,item]=pending.entries().next().value;pending.delete(id);item.fn();return item.delay};
                  if(q('#obj-animations').hidden || q('#obj-animations>details').open)throw Error('disclosure');
                  const row=[...document.querySelectorAll('#rows tr')].find(r=>r.cells[0].textContent.includes('graphics.obj-animation-banks'));
                  if(!row || !row.querySelector('a'))throw Error('resource link');row.querySelector('a').click();
                  if(!q('#obj-animations>details').open || q('#obj-animation-objects').rows.length!==1 ||
                     q('#obj-animation-selector-rows').rows.length!==2)throw Error('layout');
                  q('#obj-animation-graphics').value='0';change(q('#obj-animation-graphics'));
                  q('#obj-animation-palette').value='0';change(q('#obj-animation-palette'));
                  q('#obj-animation-outlines').checked=false;change(q('#obj-animation-outlines'));
                  if(pixel(9,9)!=='255,0,0,255' || pixel(15,9)!=='0,255,0,255')throw Error('4-bit pixels and flip');
                  q('#obj-animation-object-palettes').checked=true;change(q('#obj-animation-object-palettes'));
                  if(pixel(9,9)!=='0,0,255,255')throw Error('object palette ID');
                  q('#obj-animation-object-palettes').checked=false;change(q('#obj-animation-object-palettes'));
                  q('#obj-animation-base').value='1';q('#obj-animation-base').dispatchEvent(new Event('input'));
                  if(!q('#obj-animation-detail').textContent.includes('1 unrendered'))throw Error('missing tile range');
                  q('#obj-animation-base').value='0';q('#obj-animation-base').dispatchEvent(new Event('input'));
                  q('#obj-animation-frame').value='1';change(q('#obj-animation-frame'));
                  if(pixel(9,9)!=='0,255,0,255' || q('#obj-animation-progress').value!==2 ||
                     !q('#obj-animation-position').textContent.includes('duration 0'))throw Error('8-bit pixels');
                  q('#obj-animation-palette').value='1';change(q('#obj-animation-palette'));
                  if(pixel(9,9)!=='255,0,255,255' || !q('#obj-animation-detail').textContent.includes('lack palette colors'))throw Error('missing palette');
                  q('#obj-animation-palette').value='0';change(q('#obj-animation-palette'));
                  q('#obj-animation-sequence').value='1';change(q('#obj-animation-sequence'));
                  if(q('#obj-animation-objects').rows[0].cells[3].textContent!=='3' ||
                     !q('#obj-animation-detail').textContent.includes('1 unrendered'))throw Error('reserved shape');
                  q('#obj-animation-sequence').value='2';change(q('#obj-animation-sequence'));
                  if(!q('#obj-animation-play').disabled || !q('#obj-animation-frame').disabled ||
                     !q('#obj-animation-source').textContent.includes('zero-count hold list 1'))throw Error('empty hold');
                  q('#obj-animation-sequence').value='0';change(q('#obj-animation-sequence'));
                  q('#obj-animation-frame').value='0';change(q('#obj-animation-frame'));
                  q('#obj-animation-rate').value='10';q('#obj-animation-loop').checked=true;q('#obj-animation-play').click();
                  if(fireObjTimer()!==200 || q('#obj-animation-frame').value!=='1' || fireObjTimer()!==100 || q('#obj-animation-frame').value!=='0')throw Error('timing and loop');
                  q('#obj-animation-selector-rows').rows[1].querySelector('button').click();
                  if(q('#obj-animation-bank').value!=='1' || q('#obj-animation-stop').disabled || !q('#obj-animation-loop').checked ||
                     q('#obj-animation-rate').value!=='10' || q('#obj-animation-graphics').value!=='0' || q('#obj-animation-palette').value!=='0')throw Error('retained settings');
                  q('#obj-animation-stop').click();return 'PASS';
                })()'''), 'PASS')
                self.assertTrue(browser.evaluate('''(()=>{const s=document.querySelector('#obj-animation-frame');s.focus();return document.activeElement===s})()'''))
                browser.click('#obj-animation-play')
                self.assertEqual(browser.evaluate('document.activeElement.id'), 'obj-animation-frame')
            finally:
                browser.close()

    def test_layout_without_graphics_or_palettes(self):
        with tempfile.TemporaryDirectory(prefix='viewer-obj-layout-') as directory:
            work = Path(directory)
            browser = Chrome(work, fixture(work, graphics=False))
            try:
                self.assertEqual(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s);
                  if(q('#obj-animations').hidden || q('#obj-animation-graphics').options.length!==1 ||
                     q('#obj-animation-palette').options.length!==1 || q('#obj-animation-objects').rows.length!==1)throw Error('standalone layout');
                  return 'PASS';
                })()'''), 'PASS')
            finally:
                browser.close()
