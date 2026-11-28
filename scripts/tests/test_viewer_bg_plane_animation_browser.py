"""Exercise background-plane transfer composition and playback in a browser."""
import hashlib
import json
import shutil
import tempfile
import unittest
from pathlib import Path

from test_viewer_bg_planes_browser import fixture as plane_fixture
from test_viewer_bg_transfers_browser import catalog as transfer_catalog
from view_rom_data import generate
from viewer_browser_harness import Chrome


def fixture(work):
    plane_fixture(work)
    dataset = work / 'dataset'
    content = json.dumps(transfer_catalog()).encode()
    (dataset / 'bg-transfer-programs.json').write_bytes(content)
    with (dataset / 'manifest.toml').open('a') as manifest:
        manifest.write('[[resources]]\nid="graphics.bg-transfer-programs"\n'
                       'path="bg-transfer-programs.json"\nformat="bg-transfer-programs-v1-json"\n'
                       f'sha256="{hashlib.sha256(content).hexdigest()}"\n')
    index = generate(dataset, work / 'animated-view', split=True)
    return index.parent / (index.name + '.parts/interface-planes.html')


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class BackgroundPlaneAnimationTest(unittest.TestCase):
    def test_composition_respects_offsets_slot_order_and_clipping(self):
        with tempfile.TemporaryDirectory(prefix='viewer-plane-composition-') as directory:
            work = Path(directory); browser = Chrome(work, fixture(work))
            try:
                self.assertEqual(browser.evaluate('''(()=>{
                  const graphic={destination_offset:32,words:Array(8).fill(0x11111111)};
                  const transfers={descriptor_links:[[
                    {control:0,tiles:0,destination:0x6000020},
                    {control:1,tiles:1,destination:0x6000020},
                    {control:2,tiles:2,destination:0x6007fe0}]],
                    controls:[{transfer_bytes:32,frames:[{source_tile:0}]},
                              {transfer_bytes:32,frames:[{source_tile:0}]},
                              {transfer_bytes:64,frames:[{source_tile:0}]}],
                    tiles:[{words:Array(8).fill(0x22222222)}, {words:Array(8).fill(0x33333333)},
                           {words:Array(8).fill(0x44444444).concat(Array(8).fill(0x55555555))}]};
                  const before=JSON.stringify([graphic,transfers]);
                  const base=composeBackgroundTiles(graphic,transfers,0,[]);
                  if(base.present[0]||!base.present[1]||base.words[8]!==0x11111111)throw Error('base offset');
                  const result=composeBackgroundTiles(graphic,transfers,0,[{slot:2,frame:0},{slot:1,frame:0},{slot:0,frame:0}]);
                  if(result.words[8]!==0x33333333||result.words[1023*8]!==0x44444444||
                     !result.present[1023]||result.applied!==3||result.clipped!==1)throw Error('ordered writes or clipping');
                  if(before!==JSON.stringify([graphic,transfers]))throw Error('source mutation');
                  return 'PASS';
                })()'''), 'PASS')
            finally:
                browser.close()

    def test_slots_run_independently_and_closing_stops_playback(self):
        with tempfile.TemporaryDirectory(prefix='viewer-plane-timers-') as directory:
            work = Path(directory); browser = Chrome(work, fixture(work))
            try:
                self.assertEqual(browser.evaluate('''(async()=>{
                  const q=s=>document.querySelector(s),input=e=>e.dispatchEvent(new Event('input'));
                  const frame=n=>q('[aria-label="Frame for transfer slot '+n+'"]');
                  const details=q('#bg-planes>details');details.open=true;await new Promise(r=>setTimeout(r,0));
                  window.planeTimers=new Map();let serial=0;
                  window.setTimeout=(fn,delay)=>{const id=++serial;planeTimers.set(id,{fn,delay});return id};
                  window.clearTimeout=id=>planeTimers.delete(id);
                  const fire=delay=>{const pair=[...planeTimers].find(([,item])=>item.delay===delay);
                    if(!pair)throw Error('missing timer '+delay);planeTimers.delete(pair[0]);pair[1].fn()};
                  q('#bg-plane-rate').value='10';q('#bg-plane-play').click();
                  if(planeTimers.size!==2)throw Error('independent timers');fire(200);
                  if(frame(0).value!=='1'||frame(2).value!=='0'||planeTimers.size!==2)throw Error('independent advance');
                  q('#bg-plane-rate').value='20';input(q('#bg-plane-rate'));
                  fire(150);if(q('#bg-plane-stop').disabled||planeTimers.size!==1)throw Error('first slot finished');
                  fire(3276800);if(!q('#bg-plane-stop').disabled||planeTimers.size)throw Error('all slots finished');
                  q('#bg-plane-play').click();details.open=false;details.dispatchEvent(new Event('toggle'));
                  if(!q('#bg-plane-stop').disabled||planeTimers.size)throw Error('closed preview still running');
                  return 'PASS';
                })()'''), 'PASS')
            finally:
                browser.close()

    def test_animation_controls_frames_and_descriptor_changes(self):
        with tempfile.TemporaryDirectory(prefix='viewer-plane-animation-') as directory:
            work = Path(directory); browser = Chrome(work, fixture(work))
            try:
                self.assertEqual(browser.evaluate('''(async()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change')),
                    input=e=>e.dispatchEvent(new Event('input'));
                  const slot=n=>q('[aria-label="Enable transfer slot '+n+'"]');
                  const frame=n=>q('[aria-label="Frame for transfer slot '+n+'"]');
                  const pixel=()=>[...q('#bg-plane-canvas').getContext('2d').getImageData(0,0,1,1).data].join(',');
                  if(!q('#bg-planes>details')||q('#bg-planes>details').open||q('#bg-plane-source').textContent)throw Error('lazy preview');
                  q('#bg-planes>details').open=true;await new Promise(r=>setTimeout(r,0));
                  window.planeTimers=new Map();let serial=0;
                  window.setTimeout=(fn,delay)=>{const id=++serial;planeTimers.set(id,{fn,delay});return id};
                  window.clearTimeout=id=>planeTimers.delete(id);
                  window.firePlaneTimer=()=>{const [id,item]=planeTimers.entries().next().value;planeTimers.delete(id);item.fn();return item.delay};
                  if(q('#bg-plane-animation').hidden||!slot(0).checked||!slot(2).checked||pixel()!=='255,0,0,255')throw Error('initial controls');
                  frame(0).value='1';change(frame(0));q('#bg-plane-apply').checked=true;change(q('#bg-plane-apply'));
                  if(pixel()!=='0,255,0,255'||!q('#bg-plane-source').textContent.includes('transferred tiles'))throw Error('selected frame');
                  q('#bg-plane-apply').checked=false;change(q('#bg-plane-apply'));if(pixel()!=='255,0,0,255')throw Error('disable overlay');
                  q('#bg-plane-graphics').value='transfers';change(q('#bg-plane-graphics'));
                  if(pixel()!=='255,0,255,255')throw Error('missing transfer tiles');
                  q('#bg-plane-apply').checked=true;change(q('#bg-plane-apply'));if(pixel()!=='0,255,0,255')throw Error('transfers only');
                  q('#bg-plane-graphics').value='linked';change(q('#bg-plane-graphics'));
                  slot(2).checked=false;change(slot(2));q('#bg-plane-rate').value='10';q('#bg-plane-loop').checked=true;
                  q('#bg-plane-play').click();if(firePlaneTimer()!==300||frame(0).value!=='0'||pixel()!=='255,0,0,255')throw Error('loop or duration');
                  q('#bg-plane-descriptor').value='1';change(q('#bg-plane-descriptor'));
                  if(q('#bg-plane-stop').disabled||!q('#bg-plane-loop').checked||q('#bg-plane-rate').value!=='10'||!q('#bg-plane-apply').checked||!slot(1))throw Error('descriptor settings');
                  if(firePlaneTimer()!==100||frame(1).value!=='1')throw Error('descriptor playback');
                  slot(1).checked=false;change(slot(1));if(!q('#bg-plane-stop').disabled||planeTimers.size)throw Error('disable all');
                  q('#bg-plane-descriptor').value='0';change(q('#bg-plane-descriptor'));
                  if(slot(2).checked||frame(0).value!=='0'||!q('#bg-plane-loop').checked)throw Error('slot settings');
                  slot(0).checked=false;change(slot(0));slot(2).checked=true;change(slot(2));q('#bg-plane-play').click();
                  if(firePlaneTimer()!==6553600||frame(2).value!=='0')throw Error('zero countdown');
                  q('#bg-plane-descriptor').value='2';change(q('#bg-plane-descriptor'));
                  if(!q('#bg-plane-animation').hidden||!q('#bg-plane-stop').disabled||planeTimers.size)throw Error('descriptor without transfers');
                  q('#bg-plane-descriptor').value='0';change(q('#bg-plane-descriptor'));
                  if(slot(0).checked||!slot(2).checked||!q('#bg-plane-apply').checked||!q('#bg-plane-loop').checked)throw Error('preserve choices');
                  slot(0).checked=true;change(slot(0));slot(2).checked=false;change(slot(2));frame(0).value='1';change(frame(0));
                  q('#bg-plane-loop').checked=false;q('#bg-plane-play').click();firePlaneTimer();
                  if(!q('#bg-plane-stop').disabled||frame(0).value!=='1'||!q('#bg-plane-animation-status').textContent.includes('finished'))throw Error('one-shot completion');
                  q('#bg-planes').closest('.section-group').open=true;q('#bg-plane-animation').open=true;frame(0).focus();return 'PASS';
                })()'''), 'PASS')
                browser.click('#bg-plane-play')
                self.assertEqual(browser.evaluate("document.activeElement.getAttribute('aria-label')"), 'Frame for transfer slot 0')
            finally:
                browser.close()
