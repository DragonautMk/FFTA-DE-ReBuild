"""Exercise the documented compact sequence joined to separately owned lists."""
import copy
import hashlib
import json
import shutil
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from view_rom_data import (generate, project_compact_catalog_auxiliary,
                           project_compact_obj_catalog, project_obj_static_lists,
                           project_soil_evidence_animation)
from test_viewer_compact_catalog_browser import catalog as base_catalog
from test_viewer_static_obj_browser import object_data
from viewer_browser_harness import Chrome


def inputs():
    offsets = [40 + 8 * i for i in range(20)]
    static = {'lists': [{'source_offset': 0x6c1740 + offset,
                         'objects': [object_data(x=i)]} for i, offset in enumerate(offsets)]}
    compact = base_catalog()
    compact['selector_tables'] = {}
    compact['soil_evidence_metadata'] = {'flags': 12, 'sequence_relative_offset': 0x278,
        'selector0_relative_offset': 2, 'list_offsets': offsets,
        'frames': [{'list': i, 'duration': i + 1} for i in range(20)]}
    return static, compact


def projected():
    static, compact = inputs()
    lists = project_obj_static_lists(static); lists['path'] = 'static.json'
    banks = project_compact_obj_catalog(compact); banks['path'] = 'compact.json'
    banks['auxiliary'] = project_compact_catalog_auxiliary(compact, banks['banks'])
    return lists, banks


def fixture(work, *, include_static=True):
    static, compact = inputs()
    dataset = work / 'dataset'; dataset.mkdir()
    items = [('graphics.conditional-compact-obj-catalog', 'compact-obj-catalog', 15, compact)]
    if include_static:
        items.append(('graphics.obj-static-lists', 'obj-static-lists', 2, static))
    manifest = 'profile="test"\nextractor_revision=1\ndecoder_revision=1\n'
    for rid, name, version, value in items:
        content = json.dumps(value).encode(); (dataset / (name + '.json')).write_bytes(content)
        manifest += (f'[[resources]]\nid="{rid}"\npath="{name}.json"\nformat="{name}-v{version}-json"\n'
                     f'sha256="{hashlib.sha256(content).hexdigest()}"\n')
    (dataset / 'manifest.toml').write_text(manifest)
    return generate(dataset, work / 'view', split=True)


class SoilProjectionTest(unittest.TestCase):
    def test_join_retains_sources_frames_and_durations(self):
        static, compact = projected(); result = project_soil_evidence_animation(static, compact)
        bank = result['banks'][0]; self.assertEqual(len(bank['lists']), 20)
        self.assertEqual(bank['lists'], static['lists'])
        self.assertEqual(bank['sequences'][0]['source_offset'], 0x6c19b6)
        self.assertEqual(bank['sequences'][0]['frames'],
                         [{'object_list': i, 'list_id': i, 'duration': i + 1} for i in range(20)])

    def test_join_refuses_incomplete_or_inconsistent_inputs(self):
        static, compact = projected()
        self.assertIsNone(project_soil_evidence_animation(None, compact))
        self.assertIsNone(project_soil_evidence_animation(static, {'banks': []}))
        def group(c, name):
            return next(g for g in c['auxiliary']['groups'] if g['id'] == name)['rows']
        mutations = [lambda s, c: s['lists'].pop(),
            lambda s, c: group(c, 'soil_evidence_header')[0].__setitem__('flags', 0),
            lambda s, c: group(c, 'soil_evidence_lists')[1].__setitem__('relative_offset', 40),
            lambda s, c: group(c, 'soil_evidence_frames')[1].__setitem__('list', 0),
            lambda s, c: group(c, 'soil_evidence_frames')[1].__setitem__('duration', 0)]
        for mutate in mutations:
            s, c = copy.deepcopy(static), copy.deepcopy(compact); mutate(s, c)
            with self.subTest(mutation=mutate): self.assertIsNone(project_soil_evidence_animation(s, c))


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class SoilBrowserTest(unittest.TestCase):
    def test_joined_sequence_playback_loop_and_source_navigation(self):
        with tempfile.TemporaryDirectory(prefix='viewer-soil-animation-') as directory:
            work = Path(directory); index = fixture(work)
            browser = Chrome(work, index.parent / (index.name + '.parts/objects.html'))
            try:
                self.assertEqual(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change',{bubbles:true}));
                  window.pending=new Map();let serial=0;
                  window.setTimeout=(fn,delay)=>{const id=++serial;pending.set(id,{fn,delay});return id};
                  window.clearTimeout=id=>pending.delete(id);
                  const fire=()=>{const [id,item]=pending.entries().next().value;pending.delete(id);item.fn();return item.delay};
                  if(q('#compact-soil-animation-link').hidden)throw Error('missing source link');
                  q('#compact-soil-animation-link a').click();
                  if(q('#obj-animation-catalog').value!=='soil_evidence'||!q('#obj-animations>details').open||
                     q('#obj-animation-frame').options.length!==20||q('#obj-animation-play').disabled)throw Error('joined animation');
                  if(!q('#obj-animation-detail').textContent.includes('0x6C1768')||
                     !q('#obj-animation-position').textContent.includes('duration 1'))throw Error('first source');
                  q('#obj-animation-rate').value='10';q('#obj-animation-loop').checked=true;q('#obj-animation-play').click();
                  if(fire()!==100||q('#obj-animation-frame').value!=='1'||
                     !q('#obj-animation-position').textContent.includes('duration 2'))throw Error('timing');
                  q('#obj-animation-frame').value='19';change(q('#obj-animation-frame'));
                  if(fire()!==2000||q('#obj-animation-frame').value!=='0')throw Error('loop');
                  q('#obj-animation-stop').click();
                  const back=[...q('#obj-animation-catalog-references').querySelectorAll('a')]
                    .find(a=>a.textContent.includes('soil evidence frames #0'));
                  if(!back)throw Error('missing reverse source');back.click();
                  if(q('#compact-table-group').selectedOptions[0].textContent!=='Soil Evidence Frames')throw Error('reverse source');
                  return 'PASS';
                })()'''), 'PASS')
            finally: browser.close()

    def test_missing_static_resource_keeps_other_catalogs_available(self):
        with tempfile.TemporaryDirectory(prefix='viewer-soil-missing-') as directory:
            work = Path(directory); index = fixture(work, include_static=False)
            browser = Chrome(work, index.parent / (index.name + '.parts/objects.html'))
            try:
                self.assertTrue(browser.evaluate("document.querySelector('#compact-soil-animation-link').hidden && !document.querySelector('#obj-animations').hidden && ![...document.querySelector('#obj-animation-catalog').options].some(o=>o.value==='soil_evidence')"))
            finally: browser.close()
