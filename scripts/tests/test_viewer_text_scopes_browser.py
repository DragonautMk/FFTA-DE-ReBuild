"""Verify focused text pages, combined search and shared display preferences."""
import json
import shutil
import sys
import tempfile
import time
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from view_rom_data import viewer_resource_scope, write_split_viewer
from test_viewer_scoped_browser import parse_page, prepared
from test_viewer_text_search_browser import fixture as text_fixture
from viewer_browser_harness import Chrome


REPO = Path(__file__).resolve().parents[2]


def scoped_text_fixture(work):
    data, _ = prepared(work)
    text = parse_page(text_fixture(work))
    for key in ('short_texts', 'pointer_texts', 'relative_texts', 'script_texts'):
        data[key] = text[key]
    for identity, path in (('text.catalogs', 'catalog.json'), ('text.script-archives', 'scripts.json')):
        data['resources'].append({'id': identity, 'path': path, 'format': 'synthetic',
                                  'description': 'Synthetic text archive', 'status': 'Available'})
    template = (REPO / 'view_rom_data.html').read_text()
    return write_split_viewer(data, work / 'text-scopes', 'index.html', template), data


class TextScopePartitionTest(unittest.TestCase):
    def test_payload_and_resource_routing(self):
        with tempfile.TemporaryDirectory(prefix='viewer-text-scopes-') as directory:
            index, original = scoped_text_fixture(Path(directory)); parts = index.parent / 'index.html.parts'
            home = parse_page(index)
            scopes = {name: parse_page(parts / (name + '.html'))
                      for name in ('text', 'text-pointers', 'text-relative', 'text-scripts')}
            for key in ('short_texts', 'pointer_texts', 'relative_texts', 'script_texts'):
                self.assertEqual(scopes['text'][key], original[key])
            for scope, own in (('text-pointers', 'pointer_texts'), ('text-relative', 'relative_texts'),
                               ('text-scripts', 'script_texts')):
                self.assertEqual(scopes[scope][own], original[own])
                for other in {'short_texts', 'pointer_texts', 'relative_texts', 'script_texts'} - {own}:
                    self.assertNotIn(other, scopes[scope])
            index_rows = {row['id']: row for row in home['resources']}
            self.assertEqual(index_rows['text.catalogs']['viewer_scope'], 'text-pointers')
            self.assertEqual(index_rows['text.script-archives']['viewer_scope'], 'text-scripts')
            self.assertEqual(viewer_resource_scope('text.short-strings'), 'text')
            self.assertEqual([row['id'] for row in scopes['text-pointers']['resources']], ['text.catalogs'])
            self.assertEqual([row['id'] for row in scopes['text-relative']['resources']], ['text.catalogs'])
            self.assertEqual([row['id'] for row in scopes['text-scripts']['resources']], ['text.script-archives'])
            self.assertTrue({'text.catalogs', 'text.script-archives'} <= {row['id'] for row in scopes['text']['resources']})


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class TextScopeBrowserTest(unittest.TestCase):
    def test_preferences_and_search_follow_page_navigation(self):
        with tempfile.TemporaryDirectory(prefix='viewer-text-scopes-browser-') as directory:
            work = Path(directory); index, _ = scoped_text_fixture(work)
            browser = Chrome(work, index.parent / 'index.html.parts/text-pointers.html')
            try:
                def wait_for(current):
                    end = time.monotonic() + 30
                    while time.monotonic() < end:
                        try:
                            if browser.evaluate("document.readyState==='complete' && typeof data!=='undefined' && data.wiki.current===" + json.dumps(current)):
                                return
                        except (RuntimeError, OSError):
                            pass
                        time.sleep(.05)
                    self.fail('Text scope did not load: ' + current)
                def click_page(current):
                    hit = browser.evaluate('''(()=>{
                      const a=document.querySelector(SELECTOR);
                      for(let p=a.parentElement;p;p=p.parentElement)if(p.tagName==='DETAILS')p.open=true;
                      a.scrollIntoView({block:'center'});
                      const r=a.getClientRects()[0],x=r.left+r.width/2,y=r.top+r.height/2,e=document.elementFromPoint(x,y);
                      return {x,y,match:e===a||a.contains(e)};
                    })()'''.replace('SELECTOR', json.dumps('#wiki-page-links a[href$="/' + current + '.html"]')))
                    self.assertTrue(hit['match'], 'Page link is not clickable: ' + current)
                    point = {'x': hit['x'], 'y': hit['y']}
                    for event in ({'type': 'mouseMoved'}, {'type': 'mousePressed', 'button': 'left', 'clickCount': 1},
                                  {'type': 'mouseReleased', 'button': 'left', 'clickCount': 1}):
                        browser.command('Input.dispatchMouseEvent', {**event, **point})
                self.assertTrue(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change'));
                  if(q('#pointer-texts').hidden || !q('#relative-texts').hidden || !q('#script-texts').hidden ||
                     q('#text-search-scope-note').hidden || !q('#text-search-all-link').href.includes('/text.html#text-search') ||
                     q('#text-search-scope').options.length!==2) return false;
                  q('#pointer-language').value='1';change(q('#pointer-language'));
                  for(const id of ['pointer-pin','pointer-hide-commands','pointer-normal-font']){q('#'+id).checked=true;change(q('#'+id));}
                  return true;
                })()'''))
                for page, prefix in (('text-relative', 'relative'), ('text-scripts', 'script'), ('text', 'text')):
                    click_page(page); wait_for(page)
                    self.assertTrue(browser.evaluate('''(()=>{
                      const q=s=>document.querySelector(s),p=PREFIX;
                      return q('#'+p+'-pin').checked && q('#'+p+'-hide-commands').checked && q('#'+p+'-normal-font').checked &&
                        q('#'+p+'-language').selectedOptions[0].textContent==='1' &&
                        (p!=='text' || (q('#text-search-scope').options.length===5 && q('#text-search-scope-note').hidden));
                    })()'''.replace('PREFIX', json.dumps(prefix))))
                self.assertTrue(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change'));
                  q('#text-pin').checked=false;change(q('#text-pin'));
                  q('#text-hide-commands').checked=false;change(q('#text-hide-commands'));
                  q('#text-normal-font').checked=false;change(q('#text-normal-font'));return true;
                })()'''))
                click_page('text-pointers'); wait_for('text-pointers')
                self.assertTrue(browser.evaluate("!document.querySelector('#pointer-pin').checked && "
                                                 "!document.querySelector('#pointer-hide-commands').checked && "
                                                 "!document.querySelector('#pointer-normal-font').checked"))
            finally:
                browser.close()

    def test_storage_unavailable_keeps_the_viewer_usable(self):
        with tempfile.TemporaryDirectory(prefix='viewer-text-storage-') as directory:
            work = Path(directory); index, _ = scoped_text_fixture(work)
            browser = Chrome(work, index.parent / 'index.html.parts/text-pointers.html')
            try:
                browser.command('Page.addScriptToEvaluateOnNewDocument', {'source':
                    "Object.defineProperty(window,'localStorage',{get(){throw Error('blocked')}});"})
                previous = browser.evaluate('performance.timeOrigin')
                browser.command('Page.reload', {})
                end = time.monotonic() + 30
                while time.monotonic() < end:
                    try:
                        if browser.evaluate('document.readyState === "complete" && performance.timeOrigin !== ' + json.dumps(previous)):
                            break
                    except (RuntimeError, OSError):
                        pass
                    time.sleep(.05)
                else:
                    self.fail('Viewer did not reload')
                self.assertTrue(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s);q('#pointer-pin').checked=true;
                  q('#pointer-pin').dispatchEvent(new Event('change'));
                  return !q('#pointer-texts').hidden && q('#pointer-pin').checked;
                })()'''))
            finally:
                browser.close()


if __name__ == '__main__':
    unittest.main()
