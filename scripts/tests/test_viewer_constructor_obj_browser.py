"""Exercise constructor phase playback and its static-list dependencies."""
import copy
import hashlib
import json
import shutil
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from view_rom_data import generate, project_constructor_obj_animation, project_obj_static_lists, viewer_resource_scope
from test_viewer_static_obj_browser import object_data
from viewer_browser_harness import Chrome


ROOTS = (0x394f82, 0x394f90, 0x394f9e, 0x394fac)


def resources():
    animation = {'source_offset': 0x394fbc, 'frames': [
        {'threshold': threshold, 'list_offset': offset} for threshold, offset in zip((1, 3, 4, 4), ROOTS)]}
    static = {'lists': [{'source_offset': offset, 'objects': [object_data(), object_data(x=8)]} for offset in ROOTS]}
    return animation, static


def fixture(work, *, include_static=True, animation=None, static=None, profile='europe', split=True):
    original, lists = resources(); animation = original if animation is None else animation; static = lists if static is None else static
    dataset = work / 'dataset'; dataset.mkdir()
    items = [('graphics.constructor-obj-animation', 'constructor-obj-animation', 1, animation)]
    if include_static: items.append(('graphics.obj-static-lists', 'obj-static-lists', 2, static))
    items.append(('graphics.obj-custom-banks', 'obj-custom-banks', 1,
                  {'banks': [{'source_offset': 0x5000, 'words': [0x11111111] * 8}]}))
    manifest = f'profile="{profile}"\nextractor_revision=338\ndecoder_revision=337\n'
    for rid, name, version, value in items:
        content = json.dumps(value).encode(); (dataset / (name + '.json')).write_bytes(content)
        manifest += (f'[[resources]]\nid="{rid}"\npath="{name}.json"\nformat="{name}-v{version}-json"\n'
                     f'sha256="{hashlib.sha256(content).hexdigest()}"\n')
    (dataset / 'manifest.toml').write_text(manifest)
    return generate(dataset, work / 'view', split=split)


def payload(page):
    return json.JSONDecoder().raw_decode(page.read_text().split('const data = ', 1)[1])[0]


class ConstructorObjProjectionTest(unittest.TestCase):
    def test_phases_thresholds_and_exact_lists(self):
        source, static = resources()
        projected = project_constructor_obj_animation(source, project_obj_static_lists(static))
        bank = projected['banks'][0]; frames = bank['sequences'][0]['frames']
        self.assertEqual([frame['threshold'] for frame in frames], [1, 3, 4, 4])
        self.assertEqual([frame['duration'] for frame in frames], [2, 4, 5, 5])
        self.assertEqual([item['source_offset'] for item in bank['lists']], list(ROOTS))
        self.assertEqual([len(item['objects']) for item in bank['lists']], [2] * 4)
        self.assertEqual(viewer_resource_scope('graphics.constructor-obj-animation'), 'objects')
        self.assertEqual((source, static), resources())

    def test_unsigned_threshold_and_dependency_guards(self):
        source, static = resources(); static = project_obj_static_lists(static)
        for value, ticks, stalled in [(0, 1, False), (254, 255, False), (255, 1, True), (0xffffffff, 1, True)]:
            sample = copy.deepcopy(source); sample['frames'][0]['threshold'] = value
            frame = project_constructor_obj_animation(sample, static)['banks'][0]['sequences'][0]['frames'][0]
            self.assertEqual((frame['duration'], frame['counter_stalls']), (ticks, stalled))
        for mutate in (lambda value: value.update(source_offset=0), lambda value: value['frames'].pop(),
                       lambda value: value['frames'][0].update(threshold=True),
                       lambda value: value['frames'][0].update(threshold=-1),
                       lambda value: value['frames'][0].update(threshold=0x100000000),
                       lambda value: value['frames'][0].update(list_offset=0)):
            bad = copy.deepcopy(source); mutate(bad)
            with self.subTest(mutation=mutate), self.assertRaises(ValueError):
                project_constructor_obj_animation(bad, static)
        with self.assertRaisesRegex(ValueError, 'unavailable'): project_constructor_obj_animation(source, None)
        missing = copy.deepcopy(static); missing['lists'].pop()
        with self.assertRaisesRegex(ValueError, 'unavailable'): project_constructor_obj_animation(source, missing)
        invalid = copy.deepcopy(static); invalid['lists'][0]['objects'][0]['shape'] = 3
        with self.assertRaisesRegex(ValueError, 'Invalid constructor object'): project_constructor_obj_animation(source, invalid)

    def test_scopes_missing_and_other_regions(self):
        for mode in ('valid', 'missing', 'other-profile', 'bad-list'):
            source, static = resources()
            if mode == 'bad-list': static['lists'][0]['objects'].pop()
            with self.subTest(mode=mode), tempfile.TemporaryDirectory(prefix='viewer-constructor-obj-') as directory:
                index = fixture(Path(directory), include_static=mode != 'missing', static=static,
                                profile='usa' if mode == 'other-profile' else 'europe')
                parts = index.parent / 'index.html.parts'; home = payload(index); objects = payload(parts / 'objects.html')
                self.assertNotIn('obj_animation_extra', home)
                self.assertEqual(home['resources'][0]['viewer_scope'], 'objects')
                self.assertEqual('constructor_phases' in objects['obj_animation_extra'], mode == 'valid')
                self.assertNotIn('obj_animation_extra', payload(parts / 'world.html'))
                status = next(row['status'] for row in objects['resources'] if row['id'] == 'graphics.constructor-obj-animation')
                self.assertEqual(status.startswith('Available'), mode == 'valid')


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class ConstructorObjBrowserTest(unittest.TestCase):
    def test_missing_dependency_does_not_create_a_phantom_animation(self):
        with tempfile.TemporaryDirectory(prefix='viewer-constructor-missing-') as directory:
            work = Path(directory); index = fixture(work, include_static=False)
            browser = Chrome(work, index.parent / 'index.html.parts/objects.html')
            try:
                self.assertTrue(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s),row=q('[data-resource-id="graphics.constructor-obj-animation"]');
                  return q('#obj-animations').hidden && !row.querySelector('a[href="#obj-animations"]') &&
                    !!q('#contents a[href="#resource-index"]') && !q('#wiki-pages').hidden;
                })()'''))
            finally: browser.close()

    def test_frames_pixels_timing_focus_and_loop(self):
        with tempfile.TemporaryDirectory(prefix='viewer-constructor-browser-') as directory:
            work = Path(directory); index = fixture(work); browser = Chrome(work, index.parent / 'index.html.parts/objects.html')
            try:
                result = browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change',{bubbles:true}));
                  let next=0;window.pending=new Map();window.setTimeout=(fn,delay)=>{pending.set(++next,{fn,delay});return next};
                  window.clearTimeout=id=>pending.delete(id);
                  const fire=()=>{const [id,item]=pending.entries().next().value;pending.delete(id);item.fn();return item.delay};
                  const row=q('[data-resource-id="graphics.constructor-obj-animation"]');
                  row.querySelector('a[href="#obj-animations"]').click();
                  const frame=q('#obj-animation-frame'),family=q('#obj-animation-catalog');
                  if(family.value!=='constructor_phases'||!q('#obj-animations>details').open||frame.options.length!==4||
                     !q('#obj-animation-source').textContent.includes('counter starts at zero'))throw Error('navigation');
                  for(let i=0;i<4;i++){
                    frame.value=String(i);change(frame);
                    if(q('#obj-animation-objects').rows.length!==2||!frame.selectedOptions[0].textContent.includes('threshold ' + [1,3,4,4][i]))throw Error('phase '+i);
                  }
                  q('#obj-animation-graphics').value='0';change(q('#obj-animation-graphics'));
                  q('#obj-animation-outlines').checked=false;change(q('#obj-animation-outlines'));
                  const pixel=[...q('#obj-animation-canvas').getContext('2d').getImageData(10,10,1,1).data];
                  if(pixel.join(',')!=='17,17,17,255')throw Error('manual graphics');
                  q('#obj-animation-rate').value='10';q('#obj-animation-loop').checked=true;
                  frame.value='2';change(frame);frame.focus();q('#obj-animation-play').click();
                  if(document.activeElement!==frame||fire()!==500||frame.value!=='3'||fire()!==500||frame.value!=='0')throw Error('timing loop or focus');
                  q('#obj-animation-stop').click();
                  family.value='static_lists';change(family);family.value='constructor_phases';change(family);
                  if(!q('#obj-animation-loop').checked||q('#obj-animation-rate').value!=='10'||q('#obj-animation-graphics').value!=='0')throw Error('settings');
                  return 'PASS';
                })()''')
                self.assertEqual(result, 'PASS')
            finally: browser.close()

    def test_counter_wrap_stops_and_next_frame_remains_selectable(self):
        with tempfile.TemporaryDirectory(prefix='viewer-constructor-wrap-') as directory:
            work = Path(directory); animation, _ = resources(); animation['frames'][0]['threshold'] = 255
            index = fixture(work, animation=animation); browser = Chrome(work, index.parent / 'index.html.parts/objects.html')
            try:
                self.assertTrue(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change',{bubbles:true}));
                  q('[data-resource-id="graphics.constructor-obj-animation"] a[href="#obj-animations"]').click();
                  q('#obj-animation-play').click();
                  if(!q('#obj-animation-stop').disabled||!q('#obj-animation-status').textContent.includes('wraps'))return false;
                  q('#obj-animation-frame').value='1';change(q('#obj-animation-frame'));q('#obj-animation-play').click();
                  const result=!q('#obj-animation-stop').disabled;q('#obj-animation-stop').click();return result;
                })()'''))
            finally: browser.close()
