"""Check source-backed effect, event and task cross references."""
import copy
import hashlib
import json
import shutil
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from view_rom_data import generate, project_effect_event_directories, project_task_descriptors
from test_viewer_event_scripts_browser import catalog as script_catalog
from viewer_browser_harness import Chrome


def catalogs():
    def descriptor(index, callback):
        return {'ordinal': index, 'callback': None if callback is None else {'rom_offset': callback},
                'context_size': 16 + index, 'metadata_raw': 0x8000 + index}
    tasks = {'primary': {'source_offset': 0x1000, 'entries': [descriptor(0, 0x2000)]},
             'auxiliary': {'source_offset': 0x1010, 'entries': [descriptor(0, 0x2010)]},
             'script': {'source_offset': 0x1020, 'entries': [descriptor(0, None), descriptor(1, 0x2020)]}}
    def event(index, opcode, task):
        return {'index': index, 'opcode': opcode, 'byte1_raw': 1, 'id_raw': 364 if task == 1 else 7,
                'post_delay': 2, 'byte5_raw': 3, 'half6_raw': 0x4567, 'arg8_raw': 0x12345678,
                'arg12_raw': 0x87654321, 'script_descriptor': task}
    effects = {'event_source_offset': 0x3000, 'events': [event(0, 0, 1), event(1, 1, None)],
               'selector_source_offset': 0x3100,
               'selectors': [{'index': i, 'kind': i, 'parameters': [i, 2, 3], 'script_descriptor': i} for i in range(2)],
               'event_dispatch': {'source_offset': 0x3200, 'slots': [
                   {'selector': i, 'target': {'rom_offset': 0x4000 + 2 * i}} for i in range(2)]},
               'selector_dispatch': {'source_offset': 0x3300, 'slots': [
                   {'selector': i, 'target': {'rom_offset': 0x4100 + 2 * i}} for i in range(2)]}}
    return tasks, effects


def fixture(work, *, tasks=True, effects=True, scripts=True, invalid_tasks=False, invalid_effects=False):
    dataset = work / 'dataset'; dataset.mkdir(); task_data, effect_data = catalogs()
    if invalid_tasks:
        task_data['script']['entries'][0]['callback'] = {'rom_offset': 3}
    if invalid_effects:
        effect_data['selectors'][0]['script_descriptor'] = 1
    resources = []
    if tasks: resources.append(('task-descriptors', task_data))
    if effects: resources.append(('effect-event-directories', effect_data))
    if scripts: resources.append(('script-archive', script_catalog()))
    manifest = 'profile="europe"\nextractor_revision=1\ndecoder_revision=1\n'
    for name, value in resources:
        content = json.dumps(value).encode(); (dataset / (name + '.json')).write_bytes(content)
        manifest += (f'[[resources]]\nid="engine.{name}"\npath="{name}.json"\n'
                     f'format="{name}-v1-json"\nsha256="{hashlib.sha256(content).hexdigest()}"\n')
    (dataset / 'manifest.toml').write_text(manifest)
    return generate(dataset, work / 'view', split=True)


def payload(path):
    return json.JSONDecoder().raw_decode(path.read_text().split('const data = ', 1)[1])[0]


class EffectTaskProjectionTest(unittest.TestCase):
    def test_shapes_bounds_callbacks_and_relationships(self):
        tasks, effects = catalogs()
        self.assertEqual(project_task_descriptors(tasks), tasks)
        self.assertEqual(project_effect_event_directories(effects), effects)
        task_mutations = [lambda x: x['primary']['entries'][0].update(ordinal=True),
                          lambda x: x['script']['entries'][1]['callback'].update(rom_offset=3),
                          lambda x: x['auxiliary'].update(source_offset=0x1000),
                          lambda x: x['script']['entries'][0].update(context_size=-1)]
        effect_mutations = [lambda x: x['events'][0].update(id_raw=363),
                            lambda x: x['events'][1].update(script_descriptor=0),
                            lambda x: x['selectors'][0].update(script_descriptor=1),
                            lambda x: x['selectors'][0].update(parameters=[0, 1, 256]),
                            lambda x: x.update(event_source_offset=0xffffff),
                            lambda x: x['selector_dispatch'].update(source_offset=0xffffff),
                            lambda x: x['event_dispatch']['slots'][0]['target'].update(rom_offset=1)]
        for value, mutations, project in ((tasks, task_mutations, project_task_descriptors),
                                          (effects, effect_mutations, project_effect_event_directories)):
            for mutate in mutations:
                bad = copy.deepcopy(value); mutate(bad)
                with self.subTest(mutate=mutate), self.assertRaises(ValueError): project(bad)

    def test_scopes_and_independent_malformed_resources(self):
        for options in ({}, {'tasks': False}, {'effects': False}, {'invalid_tasks': True}, {'invalid_effects': True}):
            with self.subTest(options=options), tempfile.TemporaryDirectory(prefix='viewer-effect-task-') as directory:
                index = fixture(Path(directory), **options); parts = index.parent / 'index.html.parts'
                scripts = payload(parts / 'scripts.html'); overview = payload(index)
                self.assertNotIn('task_descriptors', overview); self.assertNotIn('effect_events', payload(parts / 'world.html'))
                self.assertEqual(bool(scripts['task_descriptors']), options.get('tasks', True) and not options.get('invalid_tasks'))
                self.assertEqual(bool(scripts['effect_events']), options.get('effects', True) and not options.get('invalid_effects'))
                self.assertTrue(all(item['viewer_scope'] == 'scripts' for item in overview['resources']))


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class EffectTaskBrowserTest(unittest.TestCase):
    def test_script_selector_task_and_reverse_navigation(self):
        with tempfile.TemporaryDirectory(prefix='viewer-effect-task-browser-') as directory:
            work = Path(directory); index = fixture(work); browser = Chrome(work, index.parent / 'index.html.parts/scripts.html')
            try:
                self.assertEqual(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change',{bubbles:true}));
                  q('[data-resource-id="engine.script-archive"] a[href="#event-scripts"]').click();
                  q('#event-script-program').value='1';change(q('#event-script-program'));
                  q('#event-script-rows').rows[0].cells[4].querySelector('button').click();
                  if(q('#effect-selector').value!=='1'||!q('#effect-events>details').open||
                     !q('#effect-selector-detail').textContent.includes('secondary task ID 364'))throw Error('script selector');
                  q('#effect-selector-task').click();
                  if(q('#task-catalog').value!=='script'||q('#task-descriptor').value!=='1'||
                     !q('#task-descriptor-detail').textContent.includes('primary task ID 363 · secondary task ID 364')||
                     !q('#task-descriptor-detail').textContent.includes('0x08002021'))throw Error('descriptor link');
                  const buttons=[...q('#task-references').querySelectorAll('button')];
                  if(buttons.map(b=>b.textContent).join(',')!=='Selector 1,Event 0')throw Error('reverse links');
                  buttons[1].click();
                  if(q('#effect-event').value!=='0'||!q('#effect-event-detail').textContent.includes('0x12345678')||
                     !q('#effect-event-detail').textContent.includes('0x08004000'))throw Error('event raw and target');
                  q('#effect-event').value='1';change(q('#effect-event'));if(!q('#effect-event-task').disabled)throw Error('no link');
                  q('#task-descriptor').value='0';change(q('#task-descriptor'));
                  if(!q('#task-descriptor-detail').textContent.includes('primary task ID 362 · secondary task ID 363')||
                     !q('#task-descriptor-detail').textContent.includes('callback null'))throw Error('alias or null');
                  q('#effect-dispatch-table').value='selector_dispatch';change(q('#effect-dispatch-table'));
                  if(q('#effect-dispatch-rows').rows[1].cells[2].textContent!=='0x08004102')throw Error('even target');
                  return 'PASS';
                })()'''), 'PASS')
            finally: browser.close()

    def test_missing_or_invalid_neighbors_stay_browsable(self):
        for options in ({'tasks': False}, {'effects': False}, {'invalid_tasks': True}, {'invalid_effects': True}):
            with self.subTest(options=options), tempfile.TemporaryDirectory(prefix='viewer-effect-task-partial-') as directory:
                work = Path(directory); index = fixture(work, **options); browser = Chrome(work, index.parent / 'index.html.parts/scripts.html')
                try:
                    task = options.get('tasks', True) and not options.get('invalid_tasks')
                    effect = options.get('effects', True) and not options.get('invalid_effects')
                    self.assertEqual(browser.evaluate("!document.querySelector('#task-descriptors').hidden"), task)
                    self.assertEqual(browser.evaluate("!document.querySelector('#effect-events').hidden"), effect)
                    if effect and not task:
                        self.assertTrue(browser.evaluate("document.querySelector('#effect-selector-task').disabled"))
                finally: browser.close()
