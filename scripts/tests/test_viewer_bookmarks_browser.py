"""Check offline links for stored selector positions in wiki sections."""
import json
import shutil
import tempfile
import time
import unittest
from pathlib import Path

from test_viewer_instruments_browser import fixture as instrument_fixture
from test_viewer_obj_animations_browser import fixture as object_fixture
from test_viewer_text_search_browser import fixture as text_fixture
from viewer_browser_harness import Chrome


def navigate(browser, url):
    previous = browser.evaluate('performance.timeOrigin')
    browser.command('Page.navigate', {'url': url})
    deadline = time.monotonic() + 30
    while time.monotonic() < deadline:
        try:
            if browser.evaluate('document.readyState === "complete" && performance.timeOrigin !== ' + json.dumps(previous)):
                return
        except (RuntimeError, OSError):
            pass
        time.sleep(.05)
    raise AssertionError('Linked page did not reload')


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class ViewerBookmarksTest(unittest.TestCase):
    def test_text_link_restores_cascading_selectors_and_rejects_invalid_values(self):
        with tempfile.TemporaryDirectory(prefix='viewer-bookmark-text-') as directory:
            work = Path(directory)
            page = text_fixture(work)
            browser = Chrome(work, page)
            try:
                link = browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change',{bubbles:true}));
                  q('#pointer-reader').value='1';change(q('#pointer-reader'));
                  q('#pointer-language').value='1';change(q('#pointer-language'));
                  q('#pointer-entry').value='1';change(q('#pointer-entry'));
                  const a=q('#pointer-texts>details>p>a');
                  if(!a || !a.href.includes('view=pointer-texts') || a.href.includes('Ab'))throw Error('link or privacy');
                  return a.href;
                })()''')
                navigate(browser, link)
                self.assertTrue(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s);
                  return q('#pointer-texts>details').open && q('#pointer-reader').value==='1' &&
                    q('#pointer-language').value==='1' && q('#pointer-entry').value==='1' &&
                    q('#pointer-source').textContent.includes('string 1');
                })()'''))
                invalid = (page.as_uri() + '?view=pointer-texts&select=pointer-reader%3A9999999'
                           '&select=script-language%3A1&select=pointer-entry%3A-1#pointer-texts')
                navigate(browser, invalid)
                self.assertTrue(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s);
                  return q('#pointer-reader').value==='0' && q('#pointer-entry').value==='0' &&
                    q('#script-language').value==='0' && q('#pointer-texts>details').open;
                })()'''))
            finally:
                browser.close()

    def test_instrument_link_restores_program_and_key(self):
        with tempfile.TemporaryDirectory(prefix='viewer-bookmark-instrument-') as directory:
            work = Path(directory)
            browser = Chrome(work, instrument_fixture(work))
            try:
                link = browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change',{bubbles:true}));
                  q('#instrument-program').value='4';change(q('#instrument-program'));
                  q('#instrument-key').value='61';change(q('#instrument-key'));
                  return q('#instruments>details>p>a').href;
                })()''')
                navigate(browser, link)
                self.assertTrue(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s);
                  return q('#instruments>details').open && q('#instrument-program').value==='4' &&
                    q('#instrument-key').value==='61' && q('#instrument-status').textContent.includes('PCM sample 10');
                })()'''))
            finally:
                browser.close()

    def test_object_animation_link_restores_catalog_bank_and_frame(self):
        with tempfile.TemporaryDirectory(prefix='viewer-bookmark-object-') as directory:
            work = Path(directory)
            browser = Chrome(work, object_fixture(work, extra=True))
            try:
                link = browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change',{bubbles:true}));
                  q('#obj-animation-catalog').value='layered';change(q('#obj-animation-catalog'));
                  q('#obj-animation-bank').value='1';change(q('#obj-animation-bank'));
                  q('#obj-animation-graphics').value='0';change(q('#obj-animation-graphics'));
                  return q('#obj-animations>details>p>a').href;
                })()''')
                navigate(browser, link)
                self.assertTrue(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s);
                  return q('#obj-animations>details').open && q('#obj-animation-catalog').value==='layered' &&
                    q('#obj-animation-bank').value==='1' && q('#obj-animation-graphics').value==='0';
                })()'''))
            finally:
                browser.close()
