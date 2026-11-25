"""Exercise word-task, layered and title OBJ animation catalogs."""
import shutil
import tempfile
import unittest
from pathlib import Path

from test_viewer_obj_animations_browser import fixture
from viewer_browser_harness import Chrome


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class ViewerObjFamiliesTest(unittest.TestCase):
    def test_catalog_links_titles_and_preserved_settings(self):
        with tempfile.TemporaryDirectory(prefix='viewer-obj-families-') as directory:
            work = Path(directory)
            browser = Chrome(work, fixture(work, extra=True))
            try:
                self.assertEqual(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change'));
                  const open=id=>{const row=[...document.querySelectorAll('#rows tr')].find(r=>r.cells[0].textContent.includes(id));
                    if(!row || !row.querySelector('a'))throw Error('resource '+id);row.querySelector('a').click()};
                  window.pending=new Map();let serial=0;
                  window.setTimeout=(fn,delay)=>{const id=++serial;pending.set(id,{fn,delay});return id};
                  window.clearTimeout=id=>pending.delete(id);
                  if(q('#obj-animation-catalog').options.length!==3)throw Error('families');
                  open('graphics.obj-animation-banks');
                  q('#obj-animation-graphics').value='0';change(q('#obj-animation-graphics'));
                  q('#obj-animation-palette').value='0';change(q('#obj-animation-palette'));
                  q('#obj-animation-rate').value='10';q('#obj-animation-loop').checked=true;q('#obj-animation-play').click();
                  open('graphics.word-task-obj-animations');
                  if(q('#obj-animation-catalog').value!=='word_task' || q('#obj-animation-bank').options.length!==1 ||
                     q('#obj-animation-stop').disabled || !q('#obj-animation-selector').disabled)throw Error('word task');
                  open('graphics.layered-obj-animations');
                  if(q('#obj-animation-catalog').value!=='layered' || q('#obj-animation-bank').options.length!==3 ||
                     !q('#obj-animation-source').textContent.includes('descriptor bytes 0x00 0x01') ||
                     q('#obj-animation-stop').disabled || !q('#obj-animation-loop').checked ||
                     q('#obj-animation-rate').value!=='10' || q('#obj-animation-graphics').value!=='0' ||
                     q('#obj-animation-palette').value!=='0')throw Error('layered settings');
                  q('#obj-animation-bank').value='1';change(q('#obj-animation-bank'));
                  if(!q('#obj-animation-bank').selectedOptions[0].textContent.includes('Title language') ||
                     q('#obj-animation-source').textContent.includes('descriptor bytes'))throw Error('title language');
                  q('#obj-animation-bank').value='2';change(q('#obj-animation-bank'));
                  if(!q('#obj-animation-bank').selectedOptions[0].textContent.includes('Title main'))throw Error('title main');
                  open('graphics.obj-animation-banks');
                  if(q('#obj-animation-catalog').value!=='indexed' || q('#obj-animation-selector-rows').rows.length!==2 ||
                     q('#obj-animation-stop').disabled)throw Error('indexed return');
                  q('#obj-animation-stop').click();return 'PASS';
                })()'''), 'PASS')
            finally:
                browser.close()

    def test_extra_catalogs_without_indexed_resource(self):
        with tempfile.TemporaryDirectory(prefix='viewer-obj-families-only-') as directory:
            work = Path(directory)
            browser = Chrome(work, fixture(work, graphics=False, extra=True, indexed=False))
            try:
                self.assertEqual(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s);
                  if(q('#obj-animations').hidden || q('#obj-animation-catalog').options.length!==2 ||
                     q('#obj-animation-catalog').value!=='word_task' || q('#obj-animation-objects').rows.length!==1)throw Error('standalone catalogs');
                  return 'PASS';
                })()'''), 'PASS')
            finally:
                browser.close()
