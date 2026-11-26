"""Check bidirectional playback of stored world-road samples."""
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


def fixture(work, *, roads=True):
    dataset = work / 'dataset'; dataset.mkdir()
    empty = {'path_id': 0, 'neighbor': 0}
    nodes = {'nodes': [
        {'index': 0, 'x_tile': 10, 'y_tile': 10, 'links': [{'path_id': 1, 'neighbor': 1}, {'path_id': 2, 'neighbor': 2}, empty, empty]},
        {'index': 1, 'x_tile': 20, 'y_tile': 20, 'links': [{'path_id': 1, 'neighbor': 0}, empty, empty, empty]},
        {'index': 2, 'x_tile': 30, 'y_tile': 30, 'links': [{'path_id': 2, 'neighbor': 0}, empty, empty, empty]}]}
    paths = {'paths': [
        {'index': 0, 'point_count': 3, 'encoded_pointer': 100,
         'points': [{'x': 0, 'y': 0}, {'x': 8, 'y': 16}, {'x': 24, 'y': 32}]},
        {'index': 1, 'point_count': 2, 'encoded_pointer': 200,
         'points': [{'x': 0, 'y': 0}, {'x': -8, 'y': 8}]}]}
    items = [('gameplay.world-nodes', 'world-nodes', nodes)]
    if roads:
        items.append(('gameplay.world-roads', 'world-roads', paths))
    manifest = 'profile="test"\nextractor_revision=1\ndecoder_revision=1\n'
    for rid, name, value in items:
        content = json.dumps(value).encode(); (dataset / (name + '.json')).write_bytes(content)
        manifest += (f'[[resources]]\nid="{rid}"\npath="{name}.json"\nformat="{name}-v1-json"\n'
                     f'sha256="{hashlib.sha256(content).hexdigest()}"\n')
    (dataset / 'manifest.toml').write_text(manifest)
    index = generate(dataset, work / 'view', split=True)
    return index, index.parent / (index.name + '.parts/world.html')


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class WorldRoadPlaybackTest(unittest.TestCase):
    def test_direction_timing_loop_navigation_and_focus(self):
        with tempfile.TemporaryDirectory(prefix='viewer-road-playback-') as directory:
            work = Path(directory); _, world = fixture(work); browser = Chrome(work, world)
            try:
                self.assertEqual(browser.evaluate('''(async()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change')),
                    input=e=>e.dispatchEvent(new Event('input'));
                  if(q('#world-roads-controls').hidden||q('#world-roads-controls').open)throw Error('road disclosure');
                  q('#world>details').open=true;q('[aria-label="Node 1"]').click();
                  q('[aria-label="Show road 1"]').click();await new Promise(r=>setTimeout(r,0));
                  if(!q('#world-roads-controls').open||q('#world-road-direction').value!=='reverse'||
                     q('#world-road-step').value!=='2'||!q('#world-road-detail').textContent.includes('offset (24, 32)'))throw Error('reverse link');
                  const marker=q('#world-road-overlay circle'),expected=4+92*(10+24/8)/30;
                  if(Math.abs(Number(marker.getAttribute('cx'))-expected)>1e-9)throw Error('marker offset');
                  window.roadTimers=new Map();let serial=0;
                  window.setTimeout=(fn,delay)=>{const id=++serial;roadTimers.set(id,{fn,delay});return id};
                  window.clearTimeout=id=>roadTimers.delete(id);
                  window.fireRoadTimer=()=>{const [id,item]=roadTimers.entries().next().value;roadTimers.delete(id);item.fn();return item.delay};
                  q('#world-road-rate').value='20';q('#world-road-loop').checked=true;q('#world-road-play').click();
                  if(fireRoadTimer()!==50||q('#world-road-step').value!=='1')throw Error('reverse advance');
                  fireRoadTimer();if(q('#world-road-step').value!=='0')throw Error('reverse endpoint');
                  fireRoadTimer();if(q('#world-road-step').value!=='2')throw Error('loop wrap');
                  q('#world-road-rate').value='40';input(q('#world-road-rate'));if(fireRoadTimer()!==25)throw Error('rate');
                  q('#world-road-id').value='2';change(q('#world-road-id'));
                  if(q('#world-road-direction').value!=='reverse'||q('#world-road-step').value!=='1'||
                     !q('#world-road-loop').checked||q('#world-road-stop').disabled||roadTimers.size!==1)throw Error('selection settings');
                  q('#world-road-loop').checked=false;fireRoadTimer();
                  if(q('#world-road-step').value!=='0'||!q('#world-road-stop').disabled||roadTimers.size)throw Error('one-shot');
                  q('#world-road-direction').value='forward';change(q('#world-road-direction'));
                  q('#world-road-rate').value='0';q('#world-road-play').click();
                  if(!q('#world-road-stop').disabled||!q('#world-road-status').textContent.includes('Choose a rate'))throw Error('rate guard');
                  q('#world-road-rate').value='10';q('#world-road-play').click();q('#world-road-step').value='0';input(q('#world-road-step'));
                  if(!q('#world-road-stop').disabled||roadTimers.size)throw Error('manual stop');
                  q('#world-road-play').click();q('#world-roads-controls').open=false;q('#world-roads-controls').dispatchEvent(new Event('toggle'));
                  if(!q('#world-road-stop').disabled||roadTimers.size)throw Error('closed stop');
                  q('#world').closest('.section-group').open=true;q('#world-roads-controls').open=true;q('#world-road-id').focus();return 'PASS';
                })()'''), 'PASS')
                browser.click('#world-road-play')
                self.assertEqual(browser.evaluate('document.activeElement.id'), 'world-road-id')
            finally:
                browser.close()

    def test_map_without_roads_remains_available(self):
        with tempfile.TemporaryDirectory(prefix='viewer-road-missing-') as directory:
            work = Path(directory); _, world = fixture(work, roads=False); browser = Chrome(work, world)
            try:
                self.assertTrue(browser.evaluate("!document.querySelector('#world').hidden && document.querySelector('#world-roads-controls').hidden"))
            finally:
                browser.close()
