"""Exercise source-backed Firaga sequences and their static-list dependencies."""
import copy
import hashlib
import json
import shutil
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from view_rom_data import (FIRAGA_OBJ_SPECS, generate, project_firaga_obj_animation,
                           project_obj_static_lists, viewer_resource_scope)
from test_viewer_static_obj_browser import object_data
from viewer_browser_harness import Chrome


def resources():
    specs = list(FIRAGA_OBJ_SPECS.items())
    def frames(indices, count, indexed=False):
        return [{('object_list' if indexed else 'list'): indices[i % len(indices)], 'duration': i}
                for i in range(count)]
    first, second, third, fourth = (spec for _, spec in specs)
    data = {
        specs[0][0]: {'sequence_pointer': 0x08412050, 'object_pointer': 0x08418e9c,
            'descriptor_fields': [127, 128], 'sequence_relative_offset': 4,
            'sequence': {'source_offset': 0x412054, 'frames': frames([0] + list(range(2, 20)), 19, True)},
            'object_offsets': first['offsets']},
        specs[1][0]: {'caller_literal': 0x0839cc6c, 'descriptor_root': 0x086b3578,
            'selected_pointer': 0x0839cc3c, 'flags': 12, 'split_offset': 284,
            'list_offsets': second['offsets'], 'sequence_relative_offset': 2, 'frames': frames(list(range(10)), 10)},
        specs[2][0]: {'count_literal': 0x0839cc78, 'count': 2, 'first_descriptor_fields': [0] * 10,
            'second_bank_pointer': 0x086b36ac, 'second_descriptor_fields': [0] * 10, 'flags': 12,
            'split_offset': 0xb78, 'list_offsets': third['offsets'], 'sequence_relative_offset': 8,
            'frames': frames(list(range(11)), 17)},
        specs[3][0]: {'count': 1, 'descriptor_pointer': 0x0839a8bc, 'bank_pointer': 0x086c12c8,
            'descriptor_fields': [0] * 10, 'flags': 12, 'split_offset': 0x118,
            'list_offsets': fourth['offsets'], 'sequence_relative_offset': 2, 'frames': frames(list(range(10)), 13)},
    }
    offsets = set()
    for rid, spec in specs:
        item = data[rid]; indexed = spec['key'] == 'firaga_indexed'
        sequence = item['sequence']['frames'] if indexed else item['frames']
        root = spec['root'] + (0 if indexed else 4)
        offsets.update(root + spec['offsets'][f['object_list' if indexed else 'list']] for f in sequence)
    positions = sorted(offsets)
    lists = []
    for index, offset in enumerate(positions):
        length = positions[index + 1] - offset if index + 1 < len(positions) else 8
        lists.append({'source_offset': offset, 'objects': [object_data()] if length >= 8 else []})
    return {'lists': lists}, data


def fixture(work, *, include_static=True, broken=None):
    static, data = resources(); dataset = work / 'dataset'; dataset.mkdir()
    if broken:
        data[broken]['flags'] = 99
    items = [(rid, rid.removeprefix('graphics.'), 1, value) for rid, value in data.items()]
    if include_static:
        items.append(('graphics.obj-static-lists', 'obj-static-lists', 2, static))
    manifest = 'profile="test"\nextractor_revision=1\ndecoder_revision=1\n'
    for rid, name, version, value in items:
        content = json.dumps(value).encode(); (dataset / (name + '.json')).write_bytes(content)
        manifest += (f'[[resources]]\nid="{rid}"\npath="{name}.json"\nformat="{name}-v{version}-json"\n'
                     f'sha256="{hashlib.sha256(content).hexdigest()}"\n')
    (dataset / 'manifest.toml').write_text(manifest)
    return generate(dataset, work / 'view', split=True)


def payload(path):
    return json.JSONDecoder().raw_decode(path.read_text().split('const data = ', 1)[1])[0]


class FiragaObjProjectionTest(unittest.TestCase):
    def test_all_sequences_preserve_ids_durations_and_source_lists(self):
        raw_lists, raw = resources(); static = project_obj_static_lists(raw_lists)
        for rid, spec in FIRAGA_OBJ_SPECS.items():
            projected = project_firaga_obj_animation(raw[rid], static, rid)
            bank = projected['banks'][0]; frames = bank['sequences'][0]['frames']
            original = raw[rid]['sequence']['frames'] if spec['key'] == 'firaga_indexed' else raw[rid]['frames']
            self.assertEqual(len(frames), spec['frame_count'])
            for frame, old in zip(frames, original):
                index = old['object_list' if spec['key'] == 'firaga_indexed' else 'list']
                self.assertEqual(frame['list_id'], index)
                self.assertEqual(frame['duration'], old['duration'])
                self.assertEqual(bank['lists'][frame['object_list']]['source_offset'], bank['object_root'] + spec['offsets'][index])
            self.assertEqual(viewer_resource_scope(rid), 'objects')
        self.assertNotIn(0x418e9c + 0x54, [item['source_offset'] for item in static['lists']])

    def test_missing_or_invalid_bindings_do_not_create_animations(self):
        raw_lists, raw = resources(); static = project_obj_static_lists(raw_lists)
        rid = 'graphics.firaga-compact-obj-bank'
        for mutate in (lambda v: v.__setitem__('flags', True), lambda v: v['frames'][0].__setitem__('list', 10),
                       lambda v: v['frames'][0].__setitem__('duration', 256),
                       lambda v: v['list_offsets'].__setitem__(0, 22),
                       lambda v: v.__setitem__('selected_pointer', 0)):
            value = copy.deepcopy(raw[rid]); mutate(value)
            with self.subTest(mutation=mutate), self.assertRaises(ValueError):
                project_firaga_obj_animation(value, static, rid)
        with self.assertRaisesRegex(ValueError, 'unavailable'):
            project_firaga_obj_animation(raw[rid], None, rid)
        missing = copy.deepcopy(static); missing['lists'] = [v for v in missing['lists'] if v['source_offset'] != 0x6b3590]
        with self.assertRaisesRegex(ValueError, 'unavailable'):
            project_firaga_obj_animation(raw[rid], missing, rid)

    def test_scope_and_partial_inputs(self):
        for mode in ('all', 'missing', 'broken'):
            with self.subTest(mode=mode), tempfile.TemporaryDirectory(prefix='viewer-firaga-scope-') as directory:
                bad = 'graphics.firaga-compact-obj-bank'
                index = fixture(Path(directory), include_static=mode != 'missing', broken=bad if mode == 'broken' else None)
                parts = index.parent / (index.name + '.parts'); home = payload(index); objects = payload(parts / 'objects.html')
                self.assertNotIn('obj_animation_extra', home)
                self.assertTrue(all(r['viewer_scope'] == 'objects' for r in home['resources']))
                self.assertEqual(payload(parts / 'world.html')['resources'], [])
                available = [rid for rid, spec in FIRAGA_OBJ_SPECS.items() if spec['key'] in objects['obj_animation_extra']]
                self.assertEqual(len(available), {'all': 4, 'missing': 0, 'broken': 3}[mode])
                if mode == 'missing':
                    self.assertTrue(all('unavailable' in row['status'].lower() for row in objects['resources']))


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class FiragaObjBrowserTest(unittest.TestCase):
    def test_each_sequence_links_frames_and_metadata(self):
        with tempfile.TemporaryDirectory(prefix='viewer-firaga-obj-') as directory:
            work = Path(directory); index = fixture(work)
            browser = Chrome(work, index.parent / (index.name + '.parts/objects.html'))
            try:
                self.assertEqual(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change',{bubbles:true}));
                  window.pending=new Map();let serial=0;
                  window.setTimeout=(fn,delay)=>{const id=++serial;pending.set(id,{fn,delay});return id};
                  window.clearTimeout=id=>pending.delete(id);
                  const fire=()=>{const [id,item]=pending.entries().next().value;pending.delete(id);item.fn();return item.delay};
                  const modes={'graphics.firaga-indexed-obj-bank':'firaga_indexed',
                    'graphics.firaga-compact-obj-bank':'firaga_compact',
                    'graphics.firaga-adjacent-compact-obj-bank':'firaga_adjacent',
                    'graphics.firaga-6c12c8-obj-bank':'firaga_6c12c8'};
                  let total=0;
                  for(const [id,key] of Object.entries(modes)){
                    const row=[...q('#rows').rows].find(r=>r.dataset.resourceId===id);
                    if(!row||!row.querySelector('a'))throw Error('resource link '+id);row.querySelector('a').click();
                    const bank=data.obj_animation_extra[key].banks[0],sequence=bank.sequences[0];
                    if(q('#obj-animation-catalog').value!==key||!q('#obj-animations>details').open||
                       q('#obj-animation-play').disabled||q('#obj-animation-frame').options.length!==sequence.frames.length||
                       q('#obj-animation-metadata').hidden||!q('#obj-animation-metadata-rows').rows.length)throw Error('catalog '+id);
                    for(let i=0;i<sequence.frames.length;i++){
                      q('#obj-animation-frame').value=String(i);change(q('#obj-animation-frame'));
                      const part=sequence.frames[i],list=bank.lists[part.object_list];
                      if(!q('#obj-animation-frame').selectedOptions[0].textContent.includes('list '+part.list_id)||
                         !q('#obj-animation-position').textContent.includes('duration '+part.duration)||
                         q('#obj-animation-objects').rows.length!==list.objects.length)throw Error('frame '+id);
                      total++;
                    }
                  }
                  q('#obj-animation-rate').value='10';q('#obj-animation-loop').checked=true;
                  q('#obj-animation-frame').value='0';change(q('#obj-animation-frame'));q('#obj-animation-play').click();
                  if(fire()!==100||q('#obj-animation-frame').value!=='1')throw Error('zero duration');q('#obj-animation-stop').click();
                  q('#obj-animation-catalog').value='static_lists';change(q('#obj-animation-catalog'));
                  if(!q('#obj-animation-metadata').hidden||!q('#obj-animation-play').disabled)throw Error('static mode');
                  if(total!==59)throw Error('frame total');return 'PASS';
                })()'''), 'PASS')
            finally: browser.close()
