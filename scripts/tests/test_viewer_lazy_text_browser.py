"""Verify on-demand combined text search and focused-page navigation."""
import json
import shutil
import tempfile
import time
import unittest
from pathlib import Path

from test_viewer_scoped_browser import parse_page
from test_viewer_text_scopes_browser import scoped_text_fixture
from viewer_browser_harness import Chrome
import view_rom_data


def wait_for(browser, expression, timeout=25):
    end = time.monotonic() + timeout
    while time.monotonic() < end:
        try:
            if browser.evaluate(expression):
                return
        except (RuntimeError, OSError):
            pass
        time.sleep(.05)
    raise AssertionError('Timed out waiting for: ' + expression)


class LazyTextAssetTest(unittest.TestCase):
    def test_content_addressed_assets_and_changed_output_guard(self):
        with tempfile.TemporaryDirectory(prefix='viewer-lazy-text-assets-') as directory:
            work = Path(directory)
            index, data = scoped_text_fixture(work)
            parts = index.parent / 'index.html.parts'
            search = parse_page(parts / 'text.html')
            self.assertEqual(set(search['text_search_sources']), {'short', 'pointer', 'relative', 'script'})
            for kind, asset in search['text_search_sources'].items():
                self.assertIn(asset['key'], asset['href'])
                self.assertEqual(asset['languages'], ['0', '1'])
                self.assertTrue((parts / asset['href']).read_text().startswith('globalThis.__FFTA_TEXT_SEARCH__'))
            asset = parts / search['text_search_sources']['short']['href']
            old_page = (parts / 'text.html').read_bytes()
            asset.write_text('changed by another owner')
            template = (Path(__file__).resolve().parents[2] / 'view_rom_data.html').read_text()
            with self.assertRaisesRegex(ValueError, 'text search asset'):
                view_rom_data.write_split_viewer(data, index.parent, 'index.html', template)
            self.assertEqual((parts / 'text.html').read_bytes(), old_page)
            self.assertEqual(asset.read_text(), 'changed by another owner')


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class LazyTextBrowserTest(unittest.TestCase):
    def test_all_results_open_their_focused_entry_with_saved_preferences(self):
        with tempfile.TemporaryDirectory(prefix='viewer-lazy-text-links-') as directory:
            work = Path(directory)
            index, _ = scoped_text_fixture(work)
            parts = index.parent / 'index.html.parts'
            browser = Chrome(work, parts / 'text-short.html')
            try:
                browser.evaluate('''(()=>{const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change'));
                  q('#text-language').value='1';change(q('#text-language'));
                  for(const id of ['text-pin','text-hide-commands','text-normal-font']){q('#'+id).checked=true;change(q('#'+id));}
                })()''')
                browser.command('Page.navigate', {'url': (parts / 'text.html').as_uri()})
                wait_for(browser, "data.wiki.current==='text'")
                browser.evaluate('''(()=>{const q=s=>document.querySelector(s);q('#text-search-language').value='1';
                  q('#text-search-query').value='b d';q('#text-search-form').requestSubmit();})()''')
                wait_for(browser, "document.querySelector('#text-search-status').textContent.startsWith('Searched')")
                links = browser.evaluate("[...document.querySelectorAll('#text-search-rows a')].map(a=>a.href)")
                self.assertEqual(len(links), 4)
                for url, page, prefix in zip(links, ('text-short', 'text-pointers', 'text-relative', 'text-scripts'),
                                             ('text', 'pointer', 'relative', 'script')):
                    browser.command('Page.navigate', {'url': url})
                    wait_for(browser, 'data.wiki.current===' + json.dumps(page) +
                             ' && document.querySelector(' + json.dumps('#' + prefix + '-entry') + ').value===\'1\'')
                    self.assertTrue(browser.evaluate('''(()=>{const q=s=>document.querySelector(s),p=PREFIX;
                      return q('#'+p+'-pin').checked && q('#'+p+'-hide-commands').checked &&
                        q('#'+p+'-normal-font').checked && q('#'+p+'-language').value==='1';})()'''.replace('PREFIX', json.dumps(prefix))))
            finally:
                browser.close()

    def test_selected_loading_results_and_legacy_bookmark(self):
        with tempfile.TemporaryDirectory(prefix='viewer-lazy-text-browser-') as directory:
            work = Path(directory)
            index, _ = scoped_text_fixture(work)
            parts = index.parent / 'index.html.parts'
            browser = Chrome(work, parts / 'text.html')
            try:
                self.assertTrue(browser.evaluate('''(()=>{const q=s=>document.querySelector(s);
                  return !q('#text-search').hidden && !q('#text-search-lazy-note').hidden &&
                    q('#texts').hidden && q('#pointer-texts').hidden &&
                    typeof globalThis.__FFTA_TEXT_SEARCH__==='undefined';})()'''))
                self.assertTrue(browser.evaluate('''(()=>{const row=document.querySelector('#rows [data-resource-id="text.catalogs"]');
                  const hrefs=[...row.querySelectorAll('a')].map(a=>a.href);
                  return hrefs.some(h=>h.includes('/text-pointers.html#pointer-texts')) &&
                    hrefs.some(h=>h.includes('/text-relative.html#relative-texts')) &&
                    !('sources' in data.resources.find(item=>item.id==='text.catalogs'));})()'''))
                browser.evaluate('''(()=>{const q=s=>document.querySelector(s);
                  q('#text-search-scope').value='pointer';q('#text-search-language').value='1';
                  q('#text-search-query').value='b d';q('#text-search-form').requestSubmit();})()''')
                wait_for(browser, "document.querySelector('#text-search-status').textContent.startsWith('Searched')")
                self.assertTrue(browser.evaluate('''(()=>{const q=s=>document.querySelector(s), keys=Object.keys(globalThis.__FFTA_TEXT_SEARCH__);
                  return keys.length===1 && keys[0]==='pointer' && q('#text-search-rows').rows.length===1 &&
                    q('#text-search-rows tr').cells[1].textContent==='Ab d�' &&
                    q('#text-search-rows a').href.includes('/text-pointers.html?') &&
                    q('#text-search-rows a').href.includes('pointer-reader%3A1');})()'''))
                browser.evaluate("document.querySelector('#text-search>details').open=true")
                browser.click('#text-search-rows a')
                wait_for(browser, "data.wiki.current==='text-pointers' && document.querySelector('#pointer-entry').value==='1'")
                self.assertTrue(browser.evaluate('''(()=>{const q=s=>document.querySelector(s);
                  return q('#pointer-reader').value==='1' && q('#pointer-language').value==='1' &&
                    q('#pointer-texts>details').open;})()'''))
                legacy = (parts / 'text.html').as_uri() + '?view=relative-texts&select=relative-reader%3A1&select=relative-language%3A1&select=relative-entry%3A1#relative-texts'
                browser.command('Page.navigate', {'url': legacy})
                wait_for(browser, "data.wiki.current==='text-relative' && document.querySelector('#relative-entry').value==='1'")
                self.assertTrue(browser.evaluate("document.querySelector('#relative-reader').value==='1' && document.querySelector('#relative-language').value==='1' && document.querySelector('#relative-texts>details').open"))
            finally:
                browser.close()

    def test_cancel_while_loading_and_retry_missing_asset(self):
        with tempfile.TemporaryDirectory(prefix='viewer-lazy-text-cancel-') as directory:
            work = Path(directory)
            index, _ = scoped_text_fixture(work)
            parts = index.parent / 'index.html.parts'
            asset = parts / parse_page(parts / 'text.html')['text_search_sources']['short']['href']
            saved = asset.with_suffix('.saved')
            browser = Chrome(work, parts / 'text.html')
            try:
                browser.evaluate('''(()=>{const append=document.head.append.bind(document.head);
                  window.queued=[];window.release=()=>queued.splice(0).forEach(script=>append(script));
                  document.head.append=function(node){if(node.src && node.src.includes('text-search-short-'))queued.push(node);else append(node)};
                  const q=s=>document.querySelector(s);q('#text-search-scope').value='short';q('#text-search-query').value='a';
                  q('#text-search-form').requestSubmit();q('#text-search-cancel').click();})()''')
                self.assertTrue(browser.evaluate("queued.length===1 && document.querySelector('#text-search-status').textContent.startsWith('Search cancelled')"))
                browser.evaluate('release()')
                wait_for(browser, "!!(globalThis.__FFTA_TEXT_SEARCH__||{}).short")
                time.sleep(.1)
                self.assertTrue(browser.evaluate("document.querySelector('#text-search-status').textContent.startsWith('Search cancelled') && document.querySelector('#text-search-results').hidden"))
                asset.rename(saved)
                browser.command('Page.reload', {})
                wait_for(browser, "document.readyState==='complete' && typeof data!=='undefined' && data.wiki.current==='text' && typeof globalThis.__FFTA_TEXT_SEARCH__==='undefined'")
                browser.evaluate('''(()=>{const q=s=>document.querySelector(s);q('#text-search-scope').value='short';
                  q('#text-search-query').value='a';q('#text-search-form').requestSubmit();})()''')
                wait_for(browser, "document.querySelector('#text-search-status').textContent.startsWith('Search unavailable')")
                saved.rename(asset)
                browser.evaluate("document.querySelector('#text-search-form').requestSubmit()")
                wait_for(browser, "document.querySelector('#text-search-status').textContent.startsWith('Searched')")
                self.assertTrue(browser.evaluate("document.querySelector('#text-search-rows').rows.length>0"))
            finally:
                if saved.exists():
                    saved.rename(asset)
                browser.close()
