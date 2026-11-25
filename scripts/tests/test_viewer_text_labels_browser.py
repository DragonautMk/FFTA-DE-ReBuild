"""Check concise, source-derived entry labels in all text browsers."""
import shutil
import tempfile
import unittest
from pathlib import Path

from test_viewer_text_search_browser import fixture
from viewer_browser_harness import Chrome


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class ViewerTextLabelsTest(unittest.TestCase):
    def test_labels_follow_entries_languages_and_fallbacks(self):
        with tempfile.TemporaryDirectory(prefix='viewer-text-labels-') as directory:
            work = Path(directory)
            browser = Chrome(work, fixture(work))
            try:
                self.assertEqual(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change'));
                  const set=(id,value)=>{q('#'+id).value=String(value);change(q('#'+id))};
                  set('text-language',1);set('pointer-reader',1);set('pointer-language',1);
                  set('relative-reader',1);set('relative-language',1);set('script-language',1);set('script-bank',1);
                  for(const prefix of ['text','pointer','relative','script']){
                    const select=q('#'+prefix+'-entry');
                    if(select.options[1].value!=='1' || select.options[1].textContent!=='1 · Ab d�')throw Error('label '+prefix);
                    select.value='1';change(select);
                    if(select.value!=='1' || !q('#'+prefix+'-preview').textContent.includes('command') && prefix!=='text' && prefix!=='pointer')throw Error('selection '+prefix);
                  }
                  q('#text-pin').checked=true;change(q('#text-pin'));
                  q('#text-hide-commands').checked=true;change(q('#text-hide-commands'));
                  q('#text-normal-font').checked=true;change(q('#text-normal-font'));
                  if(!['text','pointer','relative','script'].every(prefix=>q('#'+prefix+'-entry').options[1].textContent==='1 · Ab d�'))throw Error('preferences');
                  q('#text-pin').checked=false;change(q('#text-pin'));
                  data.short_texts.strings[1].units=Array(70).fill({kind:'glyph',index:176});set('text-language',1);
                  const long=q('#text-entry').options[1].textContent;
                  if(!long.endsWith('…') || long.length>55)throw Error('bounded preview');
                  data.short_texts.strings[1].units=[{kind:'glyph',index:1000}];set('text-language',1);
                  if(q('#text-entry').options[1].textContent!=='1')throw Error('unknown-only fallback');
                  data.profile='other';set('pointer-language',1);set('relative-language',1);set('script-language',1);set('script-bank',1);
                  if(!['pointer','relative','script'].every(prefix=>q('#'+prefix+'-entry').options[1].textContent==='1'))throw Error('profile fallback');
                  return 'PASS';
                })()'''), 'PASS')
            finally:
                browser.close()
