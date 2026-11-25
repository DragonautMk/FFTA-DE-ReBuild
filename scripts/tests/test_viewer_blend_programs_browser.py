"""Check stored blend programs, linked producers and illustrative playback."""
import copy
import hashlib
import json
import shutil
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from view_rom_data import generate, project_blend_programs
from viewer_browser_harness import Chrome


def catalog():
    def program(offset, mode, first, second, commands):
        return {'source_offset': offset,
                'header': {'control': {'first_targets': 33, 'mode': mode, 'second_targets': 18, 'reserved': 2},
                           'initial': {'first': first, 'first_reserved': 1, 'second': second, 'second_reserved': 3},
                           'command_count': len(commands)}, 'commands': commands}
    return {'programs': [
        program(100, 1, 8, 8, [{'duration': 65535, 'first': 0, 'second': 16},
                               {'duration': 9, 'first': 33, 'second': 255}]),
        program(200, 2, 8, 0, [{'duration': 2, 'first': 16, 'second': 0}]),
        program(300, 3, 16, 0, []), program(400, 0, 0, 0, [])],
        'producers': [{'source_offset': 1000, 'programs': [0, 1, 0]},
                      {'source_offset': 2000, 'programs': [2]}]}


def fixture(work, value=None, *, split=False):
    dataset = work / 'dataset'
    dataset.mkdir()
    content = json.dumps(catalog() if value is None else value).encode()
    (dataset / 'blends.json').write_bytes(content)
    (dataset / 'manifest.toml').write_text(
        'profile="test"\nextractor_revision=1\ndecoder_revision=1\n'
        '[[resources]]\nid="graphics.blend-programs"\npath="blends.json"\n'
        'format="blend-programs-v1-json"\nsha256="' + hashlib.sha256(content).hexdigest() + '"\n')
    return generate(dataset, work / 'view', split=split)


class BlendProjectionTest(unittest.TestCase):
    def test_rejects_invalid_counts_references_and_field_ranges(self):
        self.assertEqual(project_blend_programs(catalog()), catalog())
        mutations = [
            lambda c: c['programs'][0]['header'].__setitem__('command_count', 1),
            lambda c: c['programs'][0]['header']['control'].__setitem__('mode', 4),
            lambda c: c['programs'][0]['header']['initial'].__setitem__('first', True),
            lambda c: c['programs'][0]['commands'][0].__setitem__('duration', -1),
            lambda c: c['programs'][0]['commands'][0].__setitem__('first', 256),
            lambda c: c['producers'][0]['programs'].__setitem__(0, 4),
            lambda c: c['programs'][1].__setitem__('source_offset', 100),
        ]
        for mutate in mutations:
            value = copy.deepcopy(catalog())
            mutate(value)
            with self.subTest(value=value), self.assertRaises(ValueError):
                project_blend_programs(value)

    def test_split_data_stays_in_world_scope_and_invalid_catalog_has_no_view(self):
        with tempfile.TemporaryDirectory(prefix='viewer-blend-scopes-') as directory:
            root = Path(directory)
            index = fixture(root, split=True)
            def payload(path):
                text = path.read_text()
                return json.JSONDecoder().raw_decode(text.split('const data = ', 1)[1])[0]
            parts = index.parent / (index.name + '.parts')
            self.assertNotIn('blend_programs', payload(index))
            world = payload(parts / 'world.html')
            self.assertEqual(world['blend_programs']['programs'], catalog()['programs'])
            self.assertNotIn('blend_programs', payload(parts / 'audio.html'))
        with tempfile.TemporaryDirectory(prefix='viewer-blend-invalid-') as directory:
            root = Path(directory)
            bad = catalog(); bad['producers'][0]['programs'][0] = -1
            page = fixture(root, bad)
            data = json.JSONDecoder().raw_decode(page.read_text().split('const data = ', 1)[1])[0]
            self.assertIsNone(data['blend_programs'])
            self.assertTrue(data['resources'][0]['status'].startswith('Invalid blend program data:'))


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class ViewerBlendProgramsTest(unittest.TestCase):
    def test_steps_samples_producers_loop_and_focus(self):
        with tempfile.TemporaryDirectory(prefix='viewer-blend-programs-') as directory:
            work = Path(directory)
            browser = Chrome(work, fixture(work))
            try:
                self.assertEqual(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s), change=e=>e.dispatchEvent(new Event('change'));
                  window.pending=new Map();let serial=0;
                  window.setTimeout=(fn,delay)=>{const id=++serial;pending.set(id,{fn,delay});return id};
                  window.clearTimeout=id=>pending.delete(id);
                  window.fireBlendTimer=()=>{const [id,item]=pending.entries().next().value;pending.delete(id);item.fn();return item.delay};
                  if(q('#blend-programs').hidden || q('#blend-programs>details').open)throw Error('disclosure');
                  const row=[...q('#rows').rows].find(r=>r.cells[0].textContent.includes('graphics.blend-programs'));
                  if(!row || !row.querySelector('a'))throw Error('resource link');row.querySelector('a').click();
                  if(!q('#blend-programs>details').open || q('#blend-program-references').rows.length!==2 ||
                     !q('#blend-program-control').textContent.includes('BG0, Backdrop') ||
                     !q('#blend-program-commands').rows[0].textContent.includes('0xFFFF (immediate)'))throw Error('metadata');
                  q('#blend-first-color').value='#ff0000';q('#blend-first-color').dispatchEvent(new Event('input'));
                  q('#blend-second-color').value='#0000ff';q('#blend-second-color').dispatchEvent(new Event('input'));
                  if(q('#blend-program-sample').style.backgroundColor!=='rgb(128, 0, 128)')throw Error('alpha sample');
                  q('#blend-program-commands').rows[0].querySelector('button').click();
                  if(q('#blend-program-step').value!=='1' || q('#blend-program-sample').style.backgroundColor!=='rgb(0, 0, 255)' ||
                     q('#blend-program-progress').value!==2)throw Error('command selection');
                  q('#blend-program-rate').value='20';q('#blend-program-loop').checked=true;q('#blend-program-play').click();
                  if(fireBlendTimer()!==50 || q('#blend-program-step').value!=='2' ||
                     !q('#blend-program-detail').textContent.includes('5-bit coefficients 1, 31'))throw Error('play and masks');
                  fireBlendTimer();if(q('#blend-program-step').value!=='0')throw Error('loop');
                  q('#blend-program-id').value='1';change(q('#blend-program-id'));
                  if(q('#blend-program-stop').disabled || !q('#blend-program-loop').checked || q('#blend-program-rate').value!=='20' ||
                     q('#blend-first-color').value!=='#ff0000' || q('#blend-program-sample').style.backgroundColor!=='rgb(255, 128, 128)')throw Error('preserved state and brighten');
                  q('#blend-program-loop').checked=false;fireBlendTimer();fireBlendTimer();
                  if(!q('#blend-program-stop').disabled || !q('#blend-program-status').textContent.includes('finished'))throw Error('finish');
                  q('#blend-program-id').value='2';change(q('#blend-program-id'));
                  if(q('#blend-program-sample').style.backgroundColor!=='rgb(0, 0, 0)' || q('#blend-program-step').options.length!==1)throw Error('darken and empty commands');
                  q('#blend-program-references').rows[0].querySelector('button').click();
                  if(q('#blend-program-producer').value!=='1' || !q('#blend-program-producer').closest('details').open)throw Error('reference navigation');
                  q('#blend-program-producer').value='0';change(q('#blend-program-producer'));
                  q('#blend-program-producer-rows').rows[1].querySelector('button').click();
                  if(q('#blend-program-id').value!=='1')throw Error('producer navigation');
                  q('#blend-program-id').value='3';change(q('#blend-program-id'));
                  if(q('#blend-program-sample').style.backgroundColor!=='rgb(255, 0, 0)')throw Error('no effect');
                  return 'PASS';
                })()'''), 'PASS')
                self.assertTrue(browser.evaluate('''(()=>{const s=document.querySelector('#blend-program-id');s.focus();return document.activeElement===s})()'''))
                browser.click('#blend-program-play')
                self.assertEqual(browser.evaluate('document.activeElement.id'), 'blend-program-id')
            finally:
                browser.close()
