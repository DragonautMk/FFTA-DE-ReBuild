"""Check sparse compact OBJ animation indices, playback and manual rendering."""
import copy
import hashlib
import json
import shutil
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from view_rom_data import COMPACT_OBJ_SUFFIXES, generate, project_compact_obj_animations
from viewer_browser_harness import Chrome


def obj(tile):
    return {'x': 0, 'y': 0, 'affine_mode': 0, 'object_mode': 0, 'mosaic': False, 'color_256': False,
            'shape': 0, 'transform': 0, 'size': 0, 'tile': tile, 'priority': 0, 'palette': 0}


def compact(primary=False):
    lists = [{'source_offset': 1008, 'objects': [obj(0)]}, {'source_offset': 1016, 'objects': [obj(1)]}]
    indices = [0, 1] if primary else [11, 19]
    sequences = [{'selector': 2, 'source_offset': 2008, 'frames': [
        {'list': indices[0], 'duration': 2}, {'list': indices[1], 'duration': 3}]},
        {'selector': 7, 'source_offset': 2016, 'frames': [{'list': indices[1], 'duration': 1}]}]
    if primary:
        return {'list_offsets': [8, 16], 'lists': lists, 'sequence_offsets': [8, 16], 'sequences': sequences}
    return {'lists': [{'index': i, 'relative_offset': offset, 'list': item}
                      for i, offset, item in zip(indices, [8, 16], lists)],
            'sequences': [{'relative_offset': offset, 'sequence': item}
                          for offset, item in zip([8, 16], sequences)]}


def fixture(work, *, graphics=True, value=None, split=True):
    dataset = work/'dataset'; dataset.mkdir(); data=[]
    for suffix in COMPACT_OBJ_SUFFIXES:
        data.append(('graphics.conditional-compact-obj-animations'+suffix, 'compact-obj-animations'+suffix,
                     compact(not suffix) if value is None else value))
    if graphics:
        data += [('graphics.obj-custom-banks', 'obj-custom-banks', {'banks': [
                    {'source_offset': 8000, 'words': [0x11111111]*8+[0x22222222]*8}]}),
                 ('graphics.ui-raw-palettes', 'ui-raw-palettes', {'banks': [
                    {'source_offset': 9000, 'colors': [0, 0x001f, 0x03e0]+[0]*13}]})]
    manifest='profile="test"\nextractor_revision=1\ndecoder_revision=1\n'
    for rid,name,contents in data:
        content=json.dumps(contents).encode();(dataset/(name+'.json')).write_bytes(content)
        manifest+=(f'[[resources]]\nid="{rid}"\npath="{name}.json"\nformat="{name}-v1-json"\n'
                   f'sha256="{hashlib.sha256(content).hexdigest()}"\n')
    (dataset/'manifest.toml').write_text(manifest)
    return generate(dataset,work/'view',split=split)


def payload(path):
    return json.JSONDecoder().raw_decode(path.read_text().split('const data = ',1)[1])[0]


class CompactProjectionTest(unittest.TestCase):
    def test_sparse_ids_and_validation(self):
        value=project_compact_obj_animations(compact());bank=value['banks'][0]
        self.assertEqual([v['source_index'] for v in bank['lists']],[11,19])
        self.assertEqual([v['selector'] for v in bank['sequences']],[2,7])
        self.assertEqual(bank['sequences'][0]['frames'][1],{'object_list':1,'list_id':19,'duration':3})
        self.assertEqual(project_compact_obj_animations(compact(True),primary=True)['banks'][0]['lists'][1]['source_index'],1)
        mutations=[lambda c:c['lists'][1].__setitem__('index',11),
                   lambda c:c['sequences'][0]['sequence']['frames'][0].__setitem__('list',12),
                   lambda c:c['sequences'][0]['sequence']['frames'][0].__setitem__('duration',256),
                   lambda c:c['lists'][0].__setitem__('relative_offset',7),
                   lambda c:c['lists'][0]['list']['objects'][0].__setitem__('color_256',1),
                   lambda c:c['sequences'][1]['sequence'].__setitem__('selector',2)]
        for mutate in mutations:
            value=copy.deepcopy(compact());mutate(value)
            with self.subTest(mutation=mutate),self.assertRaises(ValueError):project_compact_obj_animations(value)
        value=compact(True);value['sequence_offsets'].pop()
        with self.assertRaises(ValueError):project_compact_obj_animations(value,primary=True)

    def test_scoped_data_and_missing_projection(self):
        with tempfile.TemporaryDirectory(prefix='viewer-compact-scope-') as directory:
            page=fixture(Path(directory));parts=page.parent/(page.name+'.parts')
            index,objects,world=(payload(p) for p in (page,parts/'objects.html',parts/'world.html'))
            self.assertNotIn('obj_animation_compact',index);self.assertNotIn('obj_animation_compact',world)
            self.assertEqual(len(objects['obj_animation_compact']),8)
            self.assertEqual(sum(r['id'].startswith('graphics.conditional-compact-obj-animations') for r in objects['resources']),8)
            self.assertTrue(all(r['viewer_scope']=='objects' for r in index['resources'] if r['id'].startswith('graphics.conditional-compact-obj-animations')))
        with tempfile.TemporaryDirectory(prefix='viewer-compact-invalid-') as directory:
            bad=compact();bad['lists'][0]['index']=42
            data=payload(fixture(Path(directory),graphics=False,value=bad,split=False))
            self.assertEqual(data['obj_animation_compact'],{})
            self.assertTrue(all(r['status'].startswith('Invalid compact OBJ data:') for r in data['resources']))


@unittest.skipUnless(shutil.which('google-chrome'),'Google Chrome is unavailable')
class CompactObjBrowserTest(unittest.TestCase):
    def test_sparse_frames_render_play_loop_and_preserve_focus(self):
        with tempfile.TemporaryDirectory(prefix='viewer-compact-obj-') as directory:
            work=Path(directory);index=fixture(work);browser=Chrome(work,index.parent/(index.name+'.parts/objects.html'))
            try:
                self.assertEqual(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change'));
                  const pixel=()=>[...q('#obj-animation-canvas').getContext('2d').getImageData(9,9,1,1).data].join(',');
                  window.pending=new Map();let serial=0;
                  window.setTimeout=(fn,delay)=>{const id=++serial;pending.set(id,{fn,delay});return id};
                  window.clearTimeout=id=>pending.delete(id);
                  window.fireCompactTimer=()=>{const [id,item]=pending.entries().next().value;pending.delete(id);item.fn();return item.delay};
                  const row=[...q('#rows').rows].find(r=>r.dataset.resourceId==='graphics.conditional-compact-obj-animations-secondary');
                  if(!row||!row.querySelector('a'))throw Error('resource link');row.querySelector('a').click();
                  if(!q('#obj-animations>details').open||q('#obj-animation-catalog').value!=='compact_secondary'||
                     q('#obj-animation-catalog').options.length!==8||!q('#obj-animation-sequence').selectedOptions[0].textContent.startsWith('2 ·')||
                     !q('#obj-animation-frame').selectedOptions[0].textContent.includes('list 11'))throw Error('sparse source IDs');
                  q('#obj-animation-graphics').value='0';change(q('#obj-animation-graphics'));
                  q('#obj-animation-palette').value='0';change(q('#obj-animation-palette'));
                  q('#obj-animation-outlines').checked=false;change(q('#obj-animation-outlines'));
                  if(pixel()!=='255,0,0,255')throw Error('first frame pixel');
                  q('#obj-animation-frame').value='1';change(q('#obj-animation-frame'));
                  if(pixel()!=='0,255,0,255'||!q('#obj-animation-detail').textContent.includes('Object list 19'))throw Error('second frame pixel');
                  q('#obj-animation-rate').value='10';q('#obj-animation-loop').checked=true;q('#obj-animation-play').click();
                  if(fireCompactTimer()!==300||q('#obj-animation-frame').value!=='0'||pixel()!=='255,0,0,255')throw Error('play loop');
                  q('#obj-animation-catalog').value='compact_tertiary';change(q('#obj-animation-catalog'));
                  if(q('#obj-animation-stop').disabled||!q('#obj-animation-loop').checked||q('#obj-animation-rate').value!=='10'||
                     q('#obj-animation-graphics').value!=='0'||q('#obj-animation-palette').value!=='0')throw Error('preserved settings');
                  q('#obj-animation-loop').checked=false;fireCompactTimer();fireCompactTimer();
                  if(!q('#obj-animation-stop').disabled||!q('#obj-animation-status').textContent.includes('finished'))throw Error('finish');
                  q('#obj-animation-catalog').value='compact';change(q('#obj-animation-catalog'));
                  if(!q('#obj-animation-frame').selectedOptions[0].textContent.includes('list 1') ||
                     !q('#obj-animation-source').textContent.includes('sequence 2'))throw Error('primary IDs');
                  return 'PASS';
                })()'''),'PASS')
                self.assertTrue(browser.evaluate('''(()=>{const s=document.querySelector('#obj-animation-sequence');s.focus();return document.activeElement===s})()'''))
                browser.click('#obj-animation-play')
                self.assertEqual(browser.evaluate('document.activeElement.id'),'obj-animation-sequence')
            finally:browser.close()

    def test_compact_without_graphics_is_browsable(self):
        with tempfile.TemporaryDirectory(prefix='viewer-compact-alone-') as directory:
            work=Path(directory);index=fixture(work,graphics=False);browser=Chrome(work,index.parent/(index.name+'.parts/objects.html'))
            try:
                self.assertTrue(browser.evaluate('''(()=>{const q=s=>document.querySelector(s);return !q('#obj-animations').hidden&&
                 q('#obj-animation-graphics').value==='none'&&q('#obj-animation-detail').textContent.includes('outline only');})()'''))
            finally:browser.close()
