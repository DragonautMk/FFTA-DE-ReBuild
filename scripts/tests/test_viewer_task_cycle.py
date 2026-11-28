"""Exercise signed task-cycle projection and bounded interactive playback."""
import copy
import hashlib
import json
import shutil
import tempfile
import unittest
from pathlib import Path

from view_rom_data import generate, project_task_cycle_deltas, viewer_section_updates
from viewer_browser_harness import Chrome


def sample():
    values = ([1, -2, 3, 0] * 12)
    catalog = {'source_offset': 0x371034, 'entries': [{'phase': i, 'delta': value} for i, value in enumerate(values)]}
    source = [{'offset': 0x371034, 'length': 48, 'role': 'task-cycle-deltas',
               'sha256': hashlib.sha256(bytes(value & 255 for value in values)).hexdigest()}]
    return catalog, source


def fixture(work, invalid=False):
    dataset = work / 'dataset'; dataset.mkdir(); catalog, source = sample()
    content = json.dumps(catalog).encode(); (dataset / 'task-cycle-deltas.json').write_bytes(content)
    digest = '0' * 64 if invalid else source[0]['sha256']
    manifest = ('profile="europe"\nextractor_revision=361\ndecoder_revision=360\n'
                '[[resources]]\nid="gameplay.task-cycle-deltas"\npath="task-cycle-deltas.json"\n'
                f'format="task-cycle-deltas-v1-json"\nsha256="{hashlib.sha256(content).hexdigest()}"\n'
                f'[[resources.sources]]\noffset={source[0]["offset"]}\nlength=48\n'
                f'role="task-cycle-deltas"\nsha256="{digest}"\n')
    (dataset / 'manifest.toml').write_text(manifest)
    return generate(dataset, work / 'view', split=True)


def payload(path):
    return json.JSONDecoder().raw_decode(path.read_text().split('const data = ', 1)[1])[0]


class TaskCycleProjectionTest(unittest.TestCase):
    def test_signed_values_and_source_binding(self):
        catalog, source = sample(); result = project_task_cycle_deltas(catalog, source)
        self.assertEqual(result['deltas'][:4], [1, -2, 3, 0]); self.assertEqual(len(result['deltas']), 48)
        cases = [lambda c, s: c['entries'].pop(), lambda c, s: c['entries'][1].update(phase=2),
                 lambda c, s: c['entries'][1].update(delta=128), lambda c, s: c['entries'][1].update(delta=True),
                 lambda c, s: s[0].update(sha256='0' * 64), lambda c, s: s[0].update(offset=0),
                 lambda c, s: s.__setitem__(0, None)]
        for mutate in cases:
            c, s = copy.deepcopy(catalog), copy.deepcopy(source); mutate(c, s)
            with self.assertRaises(ValueError): project_task_cycle_deltas(c, s)

    def test_scoped_page_and_invalid_sources(self):
        for invalid in (False, True):
            with self.subTest(invalid=invalid), tempfile.TemporaryDirectory(prefix='viewer-cycle-scope-') as directory:
                index = fixture(Path(directory), invalid); folder = index.parent / (index.name + '.parts')
                home, page, world = map(payload, (index, folder / 'task-cycles.html', folder / 'world.html'))
                self.assertNotIn('task_cycle_deltas', home); self.assertNotIn('task_cycle_deltas', world)
                self.assertEqual(page['task_cycle_deltas'] is None, invalid)
                self.assertEqual(any(s['id'] == 'task-cycle-deltas' for p in home['wiki']['pages']
                                     if p['id'] == 'task-cycles' for s in p.get('sections', [])), not invalid)


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class TaskCycleBrowserTest(unittest.TestCase):
    def test_wrap_sign_phase_playback_loop_focus_and_badge(self):
        with tempfile.TemporaryDirectory(prefix='viewer-cycle-browser-') as directory:
            work = Path(directory); index = fixture(work); page = index.parent / (index.name + '.parts') / 'task-cycles.html'
            browser = Chrome(work, page)
            try:
                expected = viewer_section_updates()['task-cycles']['task-cycle-deltas']
                self.assertTrue(browser.evaluate('''(()=>{const expected=EXPECTED,q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change'));
                  window.pending=new Map();let serial=0;window.setTimeout=(f,ms)=>{const id=++serial;pending.set(id,{f,ms});return id};
                  window.clearTimeout=id=>pending.delete(id);window.fire=()=>{const [id,t]=pending.entries().next().value;pending.delete(id);t.f();return t.ms};
                  const badge=q('#task-cycle-deltas>details>summary time');
                  if(!badge||badge.dateTime!==expected.at||!badge.textContent.startsWith('Added ')||q('#task-cycle-start').options.length!==48||
                    !q('[data-resource-id="gameplay.task-cycle-deltas"] a[href="#task-cycle-deltas"]'))throw Error('section');
                  q('#task-cycle-deltas').closest('.section-group').open=true;q('#task-cycle-deltas>details').open=true;
                  q('#task-cycle-initial').value='65535';q('#task-cycle-initial').dispatchEvent(new Event('input'));
                  q('#task-cycle-step').click();if(!q('#task-cycle-current').textContent.includes('= 0 (0x0) · next phase 1'))throw Error('positive wrap');
                  q('#task-cycle-step').click();if(!q('#task-cycle-current').textContent.includes('= 65534 (0xFFFE) · next phase 2'))throw Error('negative wrap');
                  q('#task-cycle-operation').value='subtract';change(q('#task-cycle-operation'));q('#task-cycle-start').value='1';change(q('#task-cycle-start'));
                  q('#task-cycle-step').click();if(!q('#task-cycle-current').textContent.includes('= 1 (0x1) · next phase 2'))throw Error('subtract negative');
                  q('#task-cycle-start').value='47';change(q('#task-cycle-start'));q('#task-cycle-initial').value='0';q('#task-cycle-initial').dispatchEvent(new Event('input'));
                  q('#task-cycle-rate').value='10';q('#task-cycle-start').focus();
                  const down=new PointerEvent('pointerdown',{bubbles:true,cancelable:true,button:0,isPrimary:true});q('#task-cycle-play').dispatchEvent(down);
                  if(!down.defaultPrevented)throw Error('select focus');q('#task-cycle-play').click();if(fire()!==100||!q('#task-cycle-position').textContent.includes('next phase 0'))throw Error('play phase');
                  for(let i=1;i<48;i++)fire();if(!q('#task-cycle-stop').disabled||q('#task-cycle-progress').value!==48)throw Error('single cycle');
                  q('#task-cycle-loop').checked=true;q('#task-cycle-play').click();fire();if(q('#task-cycle-stop').disabled||q('#task-cycle-progress').value!==1)throw Error('loop');
                  q('#task-cycle-stop').click();q('#task-cycle-initial').value='';q('#task-cycle-initial').dispatchEvent(new Event('input'));
                  q('#task-cycle-play').click();return q('#task-cycle-stop').disabled&&q('#task-cycle-current').textContent.includes('0 to 65535');})()'''.replace('EXPECTED', json.dumps(expected))))
            finally:
                browser.close()


if __name__ == '__main__':
    unittest.main()
