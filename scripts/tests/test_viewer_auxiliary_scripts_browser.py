"""Check auxiliary script entries, source links and independent directory views."""
import copy
import hashlib
import json
import shutil
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from view_rom_data import generate, project_auxiliary_script_archive
from viewer_browser_harness import Chrome


def catalogs():
    instructions = [{'source_offset': 0x1100 + 3 * index, 'command': 1, 'parameters': [index, 0]}
                    for index in range(55)]
    instructions.append({'source_offset': 0x1100 + 3 * 55, 'command': 0, 'parameters': []})
    archive = {'source_offset': 0x1000, 'banks': [
        {'id': 0, 'source_offset': 0x1010,
         'entries': [{'program_id': 10, 'instruction': 0}, {'program_id': 10, 'instruction': 53}],
         'programs': [{'id': 10, 'source_offset': 0x1100, 'instructions': instructions},
                      {'id': 11, 'source_offset': 0x1200, 'instructions': [
                          {'source_offset': 0x1200, 'command': 0, 'parameters': []}]}],
         'alignment_after': [255]},
        {'id': 1, 'source_offset': 0x1300, 'entries': [{'program_id': 20, 'instruction': 0}],
         'programs': [{'id': 20, 'source_offset': 0x1302, 'instructions': [
             {'source_offset': 0x1302, 'command': 2, 'parameters': [42]},
             {'source_offset': 0x1304, 'command': 0, 'parameters': []}]}], 'alignment_after': []}]}
    commands = {'source_offset': 0x2000, 'commands': [
        {'opcode': index, 'advance': advance, 'callback': {'rom_offset': 0x3000 + index * 2}}
        for index, advance in enumerate((1, 3, 2))]}
    return archive, commands


def fixture(work, *, archive=True, directory=True, invalid_archive=False, invalid_directory=False):
    dataset = work / 'dataset'; dataset.mkdir()
    archive_data, command_data = catalogs()
    if invalid_archive:
        archive_data['banks'][0]['entries'][1]['instruction'] = 99
    if invalid_directory:
        command_data['commands'][0]['opcode'] = True
    resources = []
    if archive:
        resources.append(('auxiliary-script-archive', archive_data))
    if directory:
        resources.append(('auxiliary-script-command-directory', command_data))
    manifest = 'profile="europe"\nextractor_revision=1\ndecoder_revision=1\n'
    for name, value in resources:
        content = json.dumps(value).encode(); (dataset / (name + '.json')).write_bytes(content)
        manifest += (f'[[resources]]\nid="engine.{name}"\npath="{name}.json"\n'
                     f'format="{name}-v1-json"\nsha256="{hashlib.sha256(content).hexdigest()}"\n')
    (dataset / 'manifest.toml').write_text(manifest)
    return generate(dataset, work / 'view', split=True)


def payload(path):
    return json.JSONDecoder().raw_decode(path.read_text().split('const data = ', 1)[1])[0]


class AuxiliaryScriptProjectionTest(unittest.TestCase):
    def test_invalid_instruction_entries_and_overlaps_are_rejected(self):
        archive, _ = catalogs()
        self.assertEqual(project_auxiliary_script_archive(archive), archive)
        mutations = [
            lambda item: item['banks'][0]['entries'][1].update(instruction=True),
            lambda item: item['banks'][0]['entries'][1].update(instruction=56),
            lambda item: item['banks'][0]['entries'][1].update(program_id=20),
            lambda item: item['banks'][0]['programs'][0]['instructions'][1].update(source_offset=0x1102),
            lambda item: item['banks'][0]['programs'][0]['instructions'][0].update(parameters=[256]),
            lambda item: item['banks'][0]['programs'][1].update(id=10),
            lambda item: item['banks'][1].update(id=0),
            lambda item: item['banks'][0]['programs'][1].update(source_offset=0x1100,
                instructions=[{'source_offset': 0x1100, 'command': 0, 'parameters': []}]),
        ]
        for mutate in mutations:
            bad = copy.deepcopy(archive); mutate(bad)
            with self.subTest(bad=bad), self.assertRaises(ValueError):
                project_auxiliary_script_archive(bad)

    def test_scope_and_missing_or_invalid_resources_are_independent(self):
        for options in ({}, {'archive': False}, {'directory': False},
                        {'invalid_archive': True}, {'invalid_directory': True}):
            with self.subTest(options=options), tempfile.TemporaryDirectory(prefix='viewer-aux-projection-') as directory:
                index = fixture(Path(directory), **options); parts = index.parent / 'index.html.parts'
                scripts = payload(parts / 'scripts.html'); overview = payload(index)
                self.assertNotIn('auxiliary_script_archive', overview)
                self.assertNotIn('auxiliary_script_archive', payload(parts / 'world.html'))
                self.assertEqual(bool(scripts['auxiliary_script_archive']), options.get('archive', True) and not options.get('invalid_archive'))
                self.assertEqual(bool(scripts['auxiliary_script_commands']), options.get('directory', True) and not options.get('invalid_directory'))
                self.assertTrue(all(row['viewer_scope'] == 'scripts' for row in overview['resources']))


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class AuxiliaryScriptBrowserTest(unittest.TestCase):
    def test_interior_references_filter_directory_and_programs(self):
        with tempfile.TemporaryDirectory(prefix='viewer-aux-browser-') as directory:
            index = fixture(Path(directory)); browser = Chrome(Path(directory), index.parent / 'index.html.parts/scripts.html')
            try:
                self.assertEqual(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change',{bubbles:true}));
                  q('[data-resource-id="engine.auxiliary-script-archive"] a[href="#auxiliary-scripts"]').click();
                  if(q('#auxiliary-scripts').hidden || !q('#auxiliary-scripts > details').open || q('#auxiliary-rows').rows.length!==50 ||
                     q('#auxiliary-next').disabled)throw Error('archive navigation');
                  q('#auxiliary-slot').value='1';change(q('#auxiliary-slot'));
                  if(!q('#auxiliary-target').textContent.includes('instruction 53') || !q('#auxiliary-target').textContent.includes('interior') ||
                     q('#auxiliary-rows').rows[3].cells[4].textContent!=='← Slot entry' || q('#auxiliary-next').disabled!==true)throw Error('interior target');
                  q('#auxiliary-rows').rows[3].cells[2].querySelector('button').click();
                  if(q('#auxiliary-command-opcode').value!=='1' || !q('#auxiliary-commands > details').open ||
                     !q('#auxiliary-command-detail').textContent.includes('0x08003003'))throw Error('directory link');
                  q('#auxiliary-filter').value='0';change(q('#auxiliary-filter'));
                  if(q('#auxiliary-rows').rows.length!==1 || q('#auxiliary-rows').rows[0].cells[0].textContent!=='55')throw Error('opcode filter');
                  q('#auxiliary-references button:nth-child(2)').click();
                  if(q('#auxiliary-filter').value!=='all' || q('#auxiliary-rows').rows[3].cells[0].textContent!=='53')throw Error('reference link');
                  q('#auxiliary-program').value='11';change(q('#auxiliary-program'));
                  if(!q('#auxiliary-target').textContent.includes('independent') || !q('#auxiliary-references').textContent.includes('No stored slot') ||
                     q('#auxiliary-rows').rows[0].cells[4].textContent!=='—')throw Error('unreferenced physical program');
                  q('#auxiliary-bank').value='1';change(q('#auxiliary-bank'));
                  if(q('#auxiliary-program').value!=='20' || !q('#auxiliary-target').textContent.includes('slot 0'))throw Error('bank change');
                  return 'PASS';
                })()'''), 'PASS')
            finally:
                browser.close()

    def test_archive_works_without_usable_directory(self):
        for options in ({'directory': False}, {'invalid_directory': True}):
            with self.subTest(options=options), tempfile.TemporaryDirectory(prefix='viewer-aux-partial-') as directory:
                index = fixture(Path(directory), **options); browser = Chrome(Path(directory), index.parent / 'index.html.parts/scripts.html')
                try:
                    self.assertTrue(browser.evaluate("!document.querySelector('#auxiliary-scripts').hidden && document.querySelector('#auxiliary-commands').hidden && document.querySelector('#auxiliary-rows button').disabled"))
                finally:
                    browser.close()

    def test_directory_works_without_usable_archive(self):
        with tempfile.TemporaryDirectory(prefix='viewer-aux-directory-') as directory:
            index = fixture(Path(directory), invalid_archive=True); browser = Chrome(Path(directory), index.parent / 'index.html.parts/scripts.html')
            try:
                self.assertTrue(browser.evaluate("document.querySelector('#auxiliary-scripts').hidden && !document.querySelector('#auxiliary-commands').hidden && document.querySelector('#auxiliary-command-opcode').options.length===3"))
            finally:
                browser.close()
