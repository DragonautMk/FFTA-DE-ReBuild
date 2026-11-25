"""Check scoped event-script browsing and links to localized text banks."""
import copy
import hashlib
import json
import shutil
import sys
import tempfile
import time
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from view_rom_data import generate, project_script_archive, project_script_commands, project_script_text_refs
from viewer_browser_harness import Chrome


def catalog():
    instructions = [{'source_offset': 1000 + i * 2, 'command': i % 2,
                     'parameters': [i % 256], 'effect_selector': None} for i in range(120)]
    def key(i, bank, slot, text):
        return {'id': i, 'script_bank': bank, 'script_index': slot, 'text_bank': text, 'byte3_raw': 7}
    return {'archive_offset': 100, 'key_offset': 200,
            'keys': [key(0, 0, 0, 0), key(1, 1, 0, 0), key(2, 1, 1, 1), key(3, 2, 0, 0), key(4, 2, 1, 0)],
            'banks': [{'ordinal': 1, 'source_offset': 300, 'slots': [
                {'index': 0, 'target': {'kind': 'program', 'ordinal': 0}},
                {'index': 1, 'target': {'kind': 'header', 'bank': 2}},
                {'index': 2, 'target': {'kind': 'program', 'ordinal': 0}}]},
                {'ordinal': 2, 'source_offset': 400, 'slots': [
                    {'index': 0, 'target': {'kind': 'fill', 'ordinal': 0}},
                    {'index': 1, 'target': {'kind': 'program', 'ordinal': 1}}]}],
            'programs': [{'ordinal': 0, 'source_offset': 1000, 'instructions': instructions},
                {'ordinal': 1, 'source_offset': 3000, 'instructions': [
                    {'source_offset': 3000, 'command': 90, 'parameters': [2], 'effect_selector': 1}]},
                {'ordinal': 2, 'source_offset': 4000, 'instructions': [
                    {'source_offset': 4000, 'command': 1, 'parameters': [], 'effect_selector': None}]}],
            'padding': [{'source_offset': 2999, 'length': 1, 'value': 255}]}


def commands():
    return {'source_offset': 500, 'commands': [
        {'opcode': i, 'advance': 2, 'callback': {'rom_offset': 0x1000 + i * 2}} for i in range(91)]}


def text_catalog():
    def lang(index):
        banks = [{'source_offset': 6000 + index * 100 + n * 10, 'sentinel': 65535,
                  'programs': [{'source_offset': 6004 + index * 100 + n * 10, 'header': 0, 'units': []}]} for n in range(2)]
        return {'id': index, 'source_offset': 6000 + index * 100, 'end': 6100 + index * 100,
                'top': [{'kind': 'bank', 'index': 1 - index}, {'kind': 'variants'}],
                'variants': {'source_offset': 6050 + index * 100, 'bank_indices': [0, 1]}, 'banks': banks}
    return {'languages': [lang(0), lang(1)], 'alignment': []}


def fixture(work, *, archive=True, command=True, texts=True, value=None, split=True):
    dataset = work / 'dataset'; dataset.mkdir()
    data = []
    if archive: data.append(('engine.script-archive', 'script-archive', catalog() if value is None else value))
    if command: data.append(('engine.script-command-directory', 'script-command-directory', commands()))
    if texts: data.append(('text.script-archives', 'script-text', text_catalog()))
    manifest = 'profile="europe"\nextractor_revision=1\ndecoder_revision=1\n'
    for rid, name, contents in data:
        content = json.dumps(contents).encode(); (dataset / (name + '.json')).write_bytes(content)
        manifest += (f'[[resources]]\nid="{rid}"\npath="{name}.json"\nformat="{name}-v1-json"\n'
                     f'sha256="{hashlib.sha256(content).hexdigest()}"\n')
    (dataset / 'manifest.toml').write_text(manifest)
    return generate(dataset, work / 'view', split=split)


def payload(path):
    return json.JSONDecoder().raw_decode(path.read_text().split('const data = ', 1)[1])[0]


def navigate(browser, url):
    previous = browser.evaluate('performance.timeOrigin')
    browser.command('Page.navigate', {'url': url})
    end = time.monotonic() + 30
    while time.monotonic() < end:
        try:
            if browser.evaluate('document.readyState === "complete" && performance.timeOrigin !== ' + json.dumps(previous)):
                return
        except (RuntimeError, OSError):
            pass
        time.sleep(.05)
    raise AssertionError('Text page did not load')


class ScriptProjectionTest(unittest.TestCase):
    def test_validates_programs_references_and_text_mappings(self):
        self.assertEqual(project_script_archive(catalog()), catalog())
        self.assertEqual(project_script_commands(commands()), commands())
        refs = project_script_text_refs(text_catalog()['languages'])
        self.assertEqual(refs[0]['top'], [1, None]); self.assertEqual(refs[1]['variants'], [0, 1])
        without_top = text_catalog()['languages']; del without_top[0]['top']
        self.assertIsNone(project_script_text_refs(without_top))
        mutations = [lambda c: c['banks'][0]['slots'][0]['target'].__setitem__('ordinal', 99),
                     lambda c: c['banks'][0]['slots'][1]['target'].__setitem__('bank', 0),
                     lambda c: c['keys'][1].__setitem__('script_index', 200),
                     lambda c: c['programs'][0]['instructions'][0]['parameters'].__setitem__(0, 256),
                     lambda c: c['programs'][0]['instructions'][0].__setitem__('effect_selector', True),
                     lambda c: c['padding'][0].__setitem__('length', 0)]
        for mutate in mutations:
            value = copy.deepcopy(catalog()); mutate(value)
            with self.subTest(mutation=mutate), self.assertRaises(ValueError): project_script_archive(value)
        bad = commands(); bad['commands'][0]['callback']['rom_offset'] = 1
        with self.assertRaises(ValueError): project_script_commands(bad)

    def test_scope_isolated_and_invalid_archive(self):
        with tempfile.TemporaryDirectory(prefix='viewer-scripts-scope-') as directory:
            index = fixture(Path(directory)); parts = index.parent / (index.name + '.parts')
            home, scripts, text, world = (payload(p) for p in (index, parts/'scripts.html', parts/'text.html', parts/'world.html'))
            self.assertNotIn('script_archive', home); self.assertNotIn('script_archive', world); self.assertNotIn('script_texts', scripts)
            self.assertEqual(len(scripts['script_archive']['programs']), 3); self.assertEqual(len(text['script_texts']['languages']), 2)
            self.assertEqual([row['id'] for row in scripts['resources']], ['engine.script-archive', 'engine.script-command-directory'])
        with tempfile.TemporaryDirectory(prefix='viewer-scripts-invalid-') as directory:
            bad = catalog(); bad['keys'][1]['script_bank'] = 99
            page = fixture(Path(directory), value=bad, split=False); data = payload(page)
            self.assertIsNone(data['script_archive']); self.assertTrue(data['resources'][0]['status'].startswith('Invalid script archive data:'))


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class EventScriptsBrowserTest(unittest.TestCase):
    def test_keys_aliases_pagination_filter_commands_and_text_links(self):
        with tempfile.TemporaryDirectory(prefix='viewer-event-scripts-') as directory:
            work = Path(directory); index = fixture(work); page = index.parent / (index.name + '.parts/scripts.html')
            browser = Chrome(work, page)
            try:
                links = browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change',{bubbles:true}));
                  if(q('#event-scripts').hidden||q('#script-commands').hidden||q('#event-scripts>details').open)throw Error('sections');
                  const row=[...q('#rows').rows].find(r=>r.dataset.resourceId==='engine.script-archive');
                  if(!row||!row.querySelector('a'))throw Error('resource link');row.querySelector('a').click();
                  if(q('#event-script-rows').rows.length!==50||!q('#event-script-page').textContent.includes('1–50 of 120'))throw Error('first page');
                  q('#event-script-next').click();q('#event-script-next').click();
                  if(q('#event-script-rows').rows.length!==20||q('#event-script-rows').rows[0].cells[0].textContent!=='100')throw Error('last page');
                  q('#event-script-filter').value='1';change(q('#event-script-filter'));
                  if(q('#event-script-rows').rows.length!==50||!q('#event-script-page').textContent.includes('of 60')||q('#event-script-rows').rows[0].cells[0].textContent!=='1')throw Error('filter');
                  q('#event-script-rows').rows[0].cells[2].querySelector('button').click();
                  if(!q('#script-commands>details').open||q('#script-command-opcode').value!=='1'||!q('#script-command-detail').textContent.includes('stored advance 2'))throw Error('command link');
                  q('#event-script-key').value='2';change(q('#event-script-key'));
                  if(!q('#event-script-program-view').hidden||!q('#event-script-target').textContent.includes('bank header 2')||q('#event-script-variant-label').hidden)throw Error('header/variant');
                  q('#event-script-variant').value='1';change(q('#event-script-variant'));
                  q('#event-script-language').value='1';change(q('#event-script-language'));
                  const link=q('#event-script-text-open').href;
                  if(!link.includes('text.html')||!link.includes('script-bank%3A1')||!link.includes('script-language%3A1'))throw Error('text link');
                  q('#event-script-target button').click();
                  if(q('#event-script-bank').value!=='2'||!q('#event-script-target').textContent.includes('fill at'))throw Error('header navigation/fill');
                  q('#event-script-program').value='2';change(q('#event-script-program'));
                  if(q('#event-script-program-view').hidden||!q('#event-script-target').textContent.includes('independent'))throw Error('independent segment');
                  q('#event-script-key').value='2';change(q('#event-script-key'));
                  const bookmark=q('#event-scripts>details>p>a').href;
                  q('#event-script-key').value='1';change(q('#event-script-key'));
                  if(q('#event-script-program').value!=='0'||q('#event-script-text-links').hidden||!q('#event-script-references').textContent.includes('1:2'))throw Error('key/alias');
                  return {link,bookmark};
                })()''')
                navigate(browser, links['bookmark'])
                self.assertTrue(browser.evaluate('''(()=>{const q=s=>document.querySelector(s);return q('#event-script-key').value==='2'&&
                  q('#event-script-program-view').hidden&&q('#event-script-target').textContent.includes('bank header 2');})()'''))
                navigate(browser, links['link'])
                self.assertTrue(browser.evaluate('''(()=>{const q=s=>document.querySelector(s);return data.wiki.current==='text'&&
                  q('#script-texts>details').open&&q('#script-language').value==='1'&&q('#script-bank').value==='1';})()'''))
            finally:
                browser.close()

    def test_script_archive_without_optional_dependencies(self):
        with tempfile.TemporaryDirectory(prefix='viewer-scripts-alone-') as directory:
            work = Path(directory); index = fixture(work, command=False, texts=False); page = index.parent/(index.name+'.parts/scripts.html')
            browser = Chrome(work, page)
            try:
                self.assertTrue(browser.evaluate('''(()=>{const q=s=>document.querySelector(s);q('#event-script-key').value='1';
                  q('#event-script-key').dispatchEvent(new Event('change'));return !q('#event-scripts').hidden&&q('#script-commands').hidden&&
                  q('#event-script-text-links').hidden&&q('#event-script-rows').rows[0].cells[2].querySelector('button').disabled;})()'''))
            finally: browser.close()
