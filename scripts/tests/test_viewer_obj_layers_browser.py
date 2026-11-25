"""Exercise simultaneous layered OBJ previews in Chrome."""
import shutil
import tempfile
import unittest
from pathlib import Path

from test_viewer_obj_animations_browser import fixture
from viewer_browser_harness import Chrome


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class ViewerObjLayersTest(unittest.TestCase):
    def test_overlay_pixels_independent_frames_and_settings(self):
        with tempfile.TemporaryDirectory(prefix='viewer-obj-layers-') as directory:
            work = Path(directory)
            browser = Chrome(work, fixture(work, extra=True))
            try:
                self.assertEqual(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change'));
                  const open=id=>{const r=[...document.querySelectorAll('#rows tr')].find(x=>x.cells[0].textContent.includes(id));
                    if(!r||!r.querySelector('a'))throw Error('resource');r.querySelector('a').click()};
                  const pixel=(x,y)=>[...q('#obj-animation-canvas').getContext('2d').getImageData(x,y,1,1).data].join(',');
                  window.pending=new Map();let serial=0;
                  window.setTimeout=(fn,delay)=>{const id=++serial;pending.set(id,{fn,delay});return id};
                  window.clearTimeout=id=>pending.delete(id);
                  const fire=()=>{const [id,item]=pending.entries().next().value;pending.delete(id);item.fn();return item.delay};
                  open('graphics.layered-obj-animations');
                  q('#obj-animation-graphics').value='0';change(q('#obj-animation-graphics'));
                  q('#obj-animation-palette').value='0';change(q('#obj-animation-palette'));
                  q('#obj-animation-outlines').checked=false;change(q('#obj-animation-outlines'));
                  q('#obj-animation-composite').checked=true;change(q('#obj-animation-composite'));
                  if(q('#obj-animation-composite-control').hidden || q('#obj-animation-layers').hidden ||
                     q('#obj-animation-layer-rows').rows.length!==2 || q('#obj-animation-objects').rows.length!==2 ||
                     pixel(9,9)!=='255,0,0,255' || pixel(17,9)!=='0,255,0,255')throw Error('overlay pixels');
                  q('#obj-animation-rate').value='10';q('#obj-animation-loop').checked=true;q('#obj-animation-play').click();
                  if(fire()!==100 || q('#obj-animation-layer-rows').rows[0].cells[1].textContent!=='0' ||
                     q('#obj-animation-layer-rows').rows[1].cells[1].textContent!=='1')throw Error('independent layer tick');
                  fire();fire();
                  if(q('#obj-animation-frame').value!=='1' || q('#obj-animation-layer-rows').rows[1].cells[1].textContent!=='0')throw Error('layer loops');
                  q('#obj-animation-loop').checked=false;change(q('#obj-animation-loop'));fire();
                  if(!q('#obj-animation-stop').disabled || !q('#obj-animation-status').textContent.includes('finished'))throw Error('overlay finish');
                  q('#obj-animation-sequence').value='2';change(q('#obj-animation-sequence'));
                  if(!q('#obj-animation-frame').disabled || q('#obj-animation-play').disabled ||
                     q('#obj-animation-layer-rows').rows.length!==2 || !q('#obj-animation-source').textContent.includes('zero-count hold'))throw Error('empty selected layer');
                  q('#obj-animation-play').click();open('graphics.word-task-obj-animations');
                  if(!q('#obj-animation-composite-control').hidden || !q('#obj-animation-composite').checked || q('#obj-animation-stop').disabled)throw Error('hidden setting');
                  open('graphics.layered-obj-animations');
                  if(q('#obj-animation-composite-control').hidden || !q('#obj-animation-composite').checked ||
                     q('#obj-animation-layers').hidden || q('#obj-animation-stop').disabled || q('#obj-animation-rate').value!=='10')throw Error('restored setting');
                  q('#obj-animation-stop').click();return 'PASS';
                })()'''), 'PASS')
            finally:
                browser.close()
