"""Exercise script path animation and manual terrain backdrops in Chrome."""
import hashlib
import json
import shutil
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from view_rom_data import generate
from viewer_browser_harness import Chrome


def fixture(work, *, paths_only=False):
    dataset = work / 'dataset'
    dataset.mkdir()
    paths = {'source_offset': 100, 'paths': [
        {'id': 0, 'source_offset': 108, 'points': [{'x': 2, 'y': 3}, {'x': 20, 'y': 40}],
         'alignment_after': [255, 255]},
        {'id': 1, 'source_offset': 116, 'points': [{'x': 1, 'y': 1}, {'x': 2, 'y': 1}, {'x': 2, 'y': 2}],
         'alignment_after': []}],
        'script_links': [{'instruction_offset': 900, 'script_opcode': 0x44, 'path_id': 0},
                         {'instruction_offset': 901, 'script_opcode': 0x45, 'path_id': 0},
                         {'instruction_offset': 902, 'script_opcode': 0x44, 'path_id': 1}]}
    terrain = {'auxiliary': {'descriptor_programs': [0], 'programs': [
        {'relative_source': 100, 'parent_descriptor': None, 'writes': {'kind': 'halfwords', 'packets': [
            {'byte_offset': 0, 'values': [0x0102, 0x0205]}]}}]}}
    resources = [('engine.script-coordinate-paths', 'script-coordinate-paths', paths)]
    if not paths_only:
        resources.append(('graphics.bg-packet-programs', 'bg-packet-programs', terrain))
    manifest = 'profile="test"\nextractor_revision=1\ndecoder_revision=1\n'
    for resource_id, name, value in resources:
        content = json.dumps(value).encode()
        (dataset / (name + '.json')).write_bytes(content)
        manifest += (f'[[resources]]\nid="{resource_id}"\npath="{name}.json"\n'
                     f'format="{name}-v1-json"\nsha256="{hashlib.sha256(content).hexdigest()}"\n')
    (dataset / 'manifest.toml').write_text(manifest)
    return generate(dataset, work / 'view')


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class ViewerScriptPathsTest(unittest.TestCase):
    def test_paths_playback_settings_references_and_manual_terrain(self):
        with tempfile.TemporaryDirectory(prefix='viewer-script-paths-') as directory:
            work = Path(directory)
            browser = Chrome(work, fixture(work))
            try:
                self.assertEqual(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change'));
                  window.pending=new Map();let serial=0;
                  window.setTimeout=(fn,delay)=>{const id=++serial;pending.set(id,{fn,delay});return id};
                  window.clearTimeout=id=>pending.delete(id);
                  window.firePathTimer=()=>{const [id,item]=pending.entries().next().value;pending.delete(id);item.fn();return item.delay};
                  if(q('#script-paths').hidden || q('#script-paths>details').open)throw Error('disclosure');
                  const row=[...document.querySelectorAll('#rows tr')].find(r=>r.cells[0].textContent.includes('engine.script-coordinate-paths'));
                  if(!row || !row.querySelector('a'))throw Error('resource');row.querySelector('a').click();
                  if(!q('#script-paths>details').open || q('#script-path-links').rows.length!==2 ||
                     !q('#script-path-links').textContent.includes('0x44') || !q('#script-path-source').textContent.includes('0xFF 0xFF'))throw Error('source references');
                  q('#script-path-terrain').value='0';change(q('#script-path-terrain'));
                  if(!q('#script-path-source').textContent.includes('1 of 2 points visible') || q('#script-path-terrain-navigation').hidden)throw Error('manual backdrop');
                  q('#script-path-points').rows[1].querySelector('button').click();
                  if(q('#script-path-step').value!=='1' || q('#script-path-progress').value!==2 || !q('#script-path-position').textContent.includes('(20, 40)'))throw Error('point selection');
                  q('#script-path-rate').value='20';q('#script-path-rate').dispatchEvent(new Event('input'));
                  q('#script-path-loop').checked=true;q('#script-path-play').click();
                  if(q('#script-path-stop').disabled || firePathTimer()!==50 || q('#script-path-step').value!=='0')throw Error('loop playback');
                  q('#script-path-id').value='1';change(q('#script-path-id'));
                  if(q('#script-path-stop').disabled || !q('#script-path-loop').checked || q('#script-path-rate').value!=='20' ||
                     q('#script-path-terrain').value!=='0' || q('#script-path-links').rows.length!==1)throw Error('path settings');
                  firePathTimer();if(q('#script-path-step').value!=='1')throw Error('continued playback');
                  q('#script-path-loop').checked=false;firePathTimer();firePathTimer();
                  if(!q('#script-path-stop').disabled || !q('#script-path-status').textContent.includes('finished'))throw Error('path finish');
                  q('#script-path-terrain-open').click();if(q('#terrain-descriptor').value!=='0' || !q('#terrain-seeds>details').open)throw Error('path to terrain');
                  q('#terrain-path-open').click();if(q('#script-path-terrain').value!=='0' || q('#script-path-id').value!=='1')throw Error('terrain to path');
                  return 'PASS';
                })()'''), 'PASS')
                self.assertTrue(browser.evaluate('''(()=>{const s=document.querySelector('#script-path-id');s.focus();return document.activeElement===s})()'''))
                browser.click('#script-path-play')
                self.assertEqual(browser.evaluate('document.activeElement.id'), 'script-path-id')
            finally:
                browser.close()

    def test_paths_without_terrain_catalog(self):
        with tempfile.TemporaryDirectory(prefix='viewer-script-paths-only-') as directory:
            work = Path(directory)
            browser = Chrome(work, fixture(work, paths_only=True))
            try:
                self.assertEqual(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s);
                  if(q('#script-paths').hidden || !q('#script-path-terrain').disabled ||
                     !q('#script-path-source').textContent.includes('2 of 2 points visible') ||
                     !q('#script-path-terrain-navigation').hidden)throw Error('standalone paths');
                  return 'PASS';
                })()'''), 'PASS')
            finally:
                browser.close()
