"""Exercise static OBJ layouts without presenting them as animations."""
import copy
import hashlib
import json
import shutil
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from view_rom_data import generate, project_obj_static_lists
from viewer_browser_harness import Chrome


def object_data(**values):
    obj = {'x': 0, 'y': 0, 'affine_mode': 0, 'object_mode': 0, 'mosaic': False,
           'color_256': False, 'shape': 0, 'transform': 0, 'size': 0,
           'tile': 0, 'priority': 0, 'palette': 0}
    obj.update(values)
    return obj


def static_data():
    return {'lists': [{'source_offset': 1000, 'objects': [object_data()]},
                      {'source_offset': 1008, 'objects': []},
                      {'source_offset': 1010, 'objects': [object_data(x=-5, y=3, transform=8)]}],
            'actor_lookup': [11, 22]}


def fixture(work, *, include_animation=True):
    dataset = work / 'dataset'
    dataset.mkdir()
    graphic = {'banks': [{'source_offset': 8000, 'words': [0x11111111] * 8}]}
    palette = {'banks': [{'source_offset': 9000, 'colors': [0, 0x001f] + [0] * 14}]}
    animation = {'banks': [{'object_root': 2000, 'object_offsets': [4],
        'lists': [{'source_offset': 2004, 'objects': [object_data()]}],
        'sequence_root': 3000, 'sequence_offsets': [4],
        'sequences': [{'source_offset': 3004, 'frames': [{'object_list': 0, 'duration': 2}]}]}],
        'selector_tables': []}
    resources = [('graphics.obj-static-lists', 'obj-static-lists', 2, static_data()),
                 ('graphics.obj-custom-banks', 'obj-custom-banks', 1, graphic),
                 ('graphics.ui-raw-palettes', 'ui-raw-palettes', 1, palette)]
    if include_animation:
        resources.append(('graphics.obj-animation-banks', 'obj-animation-banks', 1, animation))
    manifest = 'profile="test"\nextractor_revision=1\ndecoder_revision=1\n'
    for resource_id, name, version, value in resources:
        content = json.dumps(value).encode()
        (dataset / (name + '.json')).write_bytes(content)
        manifest += (f'[[resources]]\nid="{resource_id}"\npath="{name}.json"\n'
                     f'format="{name}-v{version}-json"\nsha256="{hashlib.sha256(content).hexdigest()}"\n')
    (dataset / 'manifest.toml').write_text(manifest)
    return generate(dataset, work / 'view', split=True)


class StaticProjectionTest(unittest.TestCase):
    def test_preserves_lists_and_rejects_bad_attributes_or_extents(self):
        raw = static_data()
        self.assertEqual(project_obj_static_lists(raw)['lists'], raw['lists'])
        for mutate in (lambda c: c['lists'][1].__setitem__('source_offset', 1006),
                       lambda c: c['lists'][0]['objects'][0].__setitem__('tile', 1024),
                       lambda c: c['lists'][0]['objects'][0].__setitem__('mosaic', 1),
                       lambda c: c['lists'][2].__setitem__('source_offset', 1011)):
            invalid = copy.deepcopy(raw)
            mutate(invalid)
            with self.subTest(mutation=mutate), self.assertRaises(ValueError):
                project_obj_static_lists(invalid)

    def test_scoped_projection_is_not_embedded_in_index(self):
        with tempfile.TemporaryDirectory(prefix='viewer-static-obj-scope-') as directory:
            index = fixture(Path(directory))
            def data(path):
                return json.JSONDecoder().raw_decode(path.read_text().split('const data = ', 1)[1])[0]
            objects = data(index.parent / (index.name + '.parts/objects.html'))
            self.assertEqual(objects['obj_animation_extra']['static_lists']['lists'], static_data()['lists'])
            self.assertNotIn('obj_animation_extra', data(index))


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class StaticBrowserTest(unittest.TestCase):
    def test_static_layout_selection_pixels_and_animation_switch(self):
        with tempfile.TemporaryDirectory(prefix='viewer-static-obj-') as directory:
            work = Path(directory)
            index = fixture(work)
            browser = Chrome(work, index.parent / (index.name + '.parts/objects.html'))
            try:
                self.assertEqual(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change',{bubbles:true}));
                  const row=[...q('#rows').rows].find(r=>r.dataset.resourceId==='graphics.obj-static-lists');
                  if(!row||!row.querySelector('a'))throw Error('static resource link');row.querySelector('a').click();
                  if(!q('#obj-animations>details').open||q('#obj-animation-catalog').value!=='static_lists'||
                     q('#obj-animation-sequence').options.length!==3||q('#obj-animation-bank').selectedOptions[0].textContent!=='All static lists')throw Error('catalog');
                  if(!q('#obj-animation-play').disabled||!q('#obj-animation-frame').disabled||
                     !q('#obj-animation-position').textContent.includes('no animation')||
                     !q('#obj-animation-source').textContent.includes('static list 0'))throw Error('static semantics');
                  q('#obj-animation-graphics').value='0';change(q('#obj-animation-graphics'));
                  q('#obj-animation-palette').value='0';change(q('#obj-animation-palette'));
                  q('#obj-animation-outlines').checked=false;change(q('#obj-animation-outlines'));
                  const pixel=[...q('#obj-animation-canvas').getContext('2d').getImageData(8,8,1,1).data].join(',');
                  if(pixel!=='255,0,0,255'||q('#obj-animation-objects').rows.length!==1)throw Error('static pixel');
                  q('#obj-animation-sequence').value='1';change(q('#obj-animation-sequence'));
                  if(q('#obj-animation-objects').rows.length!==0||!q('#obj-animation-source').textContent.includes('0x3F0'))throw Error('empty layout');
                  q('#obj-animation-sequence').value='2';change(q('#obj-animation-sequence'));
                  if(q('#obj-animation-objects').rows[0].cells[1].textContent!=='-5'||q('#obj-animation-graphics').value!=='0'||
                     q('#obj-animation-palette').value!=='0')throw Error('selection or settings');
                  q('#obj-animation-catalog').value='indexed';change(q('#obj-animation-catalog'));
                  if(q('#obj-animation-play').disabled||q('#obj-animation-frame').disabled||
                     !q('#obj-animation-position').textContent.includes('duration 2'))throw Error('animation restored');
                  q('#obj-animation-catalog').value='static_lists';change(q('#obj-animation-catalog'));
                  if(!q('#obj-animation-play').disabled||q('#obj-animation-graphics').value!=='0')throw Error('static restored');
                  return 'PASS';
                })()'''), 'PASS')
            finally:
                browser.close()

    def test_static_layout_without_animation_family(self):
        with tempfile.TemporaryDirectory(prefix='viewer-static-obj-only-') as directory:
            work = Path(directory)
            index = fixture(work, include_animation=False)
            browser = Chrome(work, index.parent / (index.name + '.parts/objects.html'))
            try:
                self.assertTrue(browser.evaluate("!document.querySelector('#obj-animations').hidden && document.querySelector('#obj-animation-catalog').value === 'static_lists' && document.querySelector('#obj-animation-play').disabled"))
            finally:
                browser.close()
