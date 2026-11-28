"""Check source-gated, explicitly tentative mission/text navigation."""
import copy
import hashlib
import json
import shutil
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import view_rom_data as viewer
from test_viewer_mission_events_browser import fixture as mission_fixture
from test_viewer_navbar_split_browser import wait_for
from test_viewer_scoped_browser import parse_page
from viewer_browser_harness import Chrome


def fixture(work, *, split=True, trusted=True):
    mission_fixture(work)
    dataset = work / 'dataset'
    manifest = dataset / 'manifest.toml'
    text = manifest.read_text().replace('profile="test"', 'profile="europe"')
    mission_path = dataset / 'mission-records70.json'
    old = mission_path.read_bytes()
    missions = json.loads(old)
    sample = missions['records'][0]
    missions['source_offset'] = 0x713b3c
    missions['records'] = [{**sample, 'index': i, 'word00': i} for i in range(512)]
    raw = json.dumps(missions).encode(); mission_path.write_bytes(raw)
    mission_digest = hashlib.sha256(raw).hexdigest()
    text = text.replace(hashlib.sha256(old).hexdigest(), mission_digest)
    glyph = lambda index: {'kind': 'glyph', 'index': index}
    strings = [{'source_offset': i, 'encoding': 'compact', 'units': units} for i, units in enumerate([
        [glyph(176)], [glyph(177)], [glyph(176), {'kind': 'control_byte', 'value': 64}, glyph(203)], [glyph(204)]])]
    languages = [{'source_offset': 100 + language, 'entries': [2 if language == 1 and i == 4 else
                  3 if language == 4 and i == 4 else 1 if i == 4 else 0 for i in range(512)]}
                 for language in range(5)]
    catalog = {'glyphs': [{'index': i, 'width': 8, 'tile_words': [1] + [0] * 31} for i in (176, 177, 203, 204)],
               'strings': strings, 'pointer_catalogs': [{'reader': 1, 'languages': copy.deepcopy(languages)},
                                                        {'reader': 0x08016768, 'languages': languages}],
               'archive_catalogs': []}
    raw = json.dumps(catalog).encode(); (dataset / 'text.json').write_bytes(raw)
    text_digest = hashlib.sha256(raw).hexdigest()
    text += ('[[resources]]\nid="text.catalogs"\npath="text.json"\nformat="text-catalogs-v1-json"\n'
             f'sha256="{text_digest}"\n')
    manifest.write_text(text)
    with patch.multiple(viewer, EUROPE_MISSION_LINK_SHA256=mission_digest if trusted else '0' * 64,
                        EUROPE_TEXT_CATALOG_SHA256=text_digest):
        page = viewer.generate(dataset, work / ('split' if split else 'single'), split=split)
    return page, (mission_digest, text_digest)


class MissionTextProjectionTest(unittest.TestCase):
    def test_source_guards_and_scoped_projection(self):
        with tempfile.TemporaryDirectory(prefix='viewer-mission-text-') as directory:
            page, digests = fixture(Path(directory)); parts = page.parent / 'index.html.parts'
            home, world, text = map(parse_page, (page, parts / 'world.html', parts / 'text-pointers.html'))
            source, missions, pointer = world['mission_text_candidates'], world['mission_events']['missions'], text['pointer_texts']
            self.assertEqual((len(source['rows']), source['reader_index'], source['languages']), (512, 1, list(range(5))))
            self.assertEqual(source['rows'][4]['names'][1], [176, [64], 203])
            self.assertNotIn('mission_text_candidates', home)
            self.assertNotIn('mission_text_candidates', text)
            self.assertNotIn('pointer_texts', world)
            self.assertEqual(text['mission_text_reference'], {'reader_index': 1, 'count': 512})
            with patch.multiple(viewer, EUROPE_MISSION_LINK_SHA256=digests[0], EUROPE_TEXT_CATALOG_SHA256=digests[1]):
                project = viewer.project_mission_text_candidates
                args = ('europe', *digests, missions, pointer)
                self.assertEqual(project(*args), source)
                for index, value in ((0, 'usa'), (1, 'wrong'), (2, 'wrong'), (3, None), (4, None)):
                    bad = list(args); bad[index] = value; self.assertIsNone(project(*bad))
                for mutate in (
                    lambda m, p: m.pop('source_offset'),
                    lambda m, p: m.update(source_offset=1),
                    lambda m, p: m['records'].pop(),
                    lambda m, p: m['records'][4].update(word00=9),
                    lambda m, p: p['catalogs'].append(copy.deepcopy(p['catalogs'][1])),
                    lambda m, p: p['catalogs'][1]['languages'].pop(),
                    lambda m, p: p['catalogs'][1]['languages'][0]['entries'].pop(),
                    lambda m, p: p['catalogs'][1]['languages'][0]['entries'].__setitem__(4, 9999),
                ):
                    modified_missions, modified_pointer = copy.deepcopy(missions), copy.deepcopy(pointer)
                    mutate(modified_missions, modified_pointer)
                    self.assertIsNone(project(*args[:3], modified_missions, modified_pointer))
            entries = viewer.project_wiki_name_index({'profile': 'europe', 'mission_events': {'missions': missions},
                                                       'mission_text_candidates': source})
            self.assertEqual(len(entries), 512)
            self.assertEqual(entries[4]['names']['1']['text'], 'A b')
            self.assertEqual(viewer.project_wiki_name_index({'profile': 'europe', 'mission_events': {},
                                                               'mission_text_candidates': source}), [])
            asset = page.parent / home['wiki']['name_search']['href']
            self.assertTrue(asset.is_file())
            self.assertNotIn('A b', page.read_text())

    def test_untrusted_source_does_not_create_candidate(self):
        with tempfile.TemporaryDirectory(prefix='viewer-mission-text-untrusted-') as directory:
            page, _ = fixture(Path(directory), trusted=False)
            parts = page.parent / 'index.html.parts'
            self.assertIsNone(parse_page(parts / 'world.html')['mission_text_candidates'])
            self.assertNotIn('mission_text_reference', parse_page(parts / 'text-pointers.html'))
            self.assertNotIn('name_search', parse_page(page)['wiki'])


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class MissionTextBrowserTest(unittest.TestCase):
    def test_search_settings_and_bidirectional_navigation(self):
        with tempfile.TemporaryDirectory(prefix='viewer-mission-text-browser-') as directory:
            work = Path(directory); page, _ = fixture(work); browser = Chrome(work, page)
            try:
                self.assertTrue(browser.evaluate("!window.__FFTA_WIKI_NAMES__ && document.querySelector('#wiki-name-search>details>summary time').dateTime==='2026-11-27T21:42:54Z'"))
                browser.evaluate("document.querySelector('#wiki-name-search>details').open=true")
                wait_for(browser, "!!window.__FFTA_WIKI_NAMES__ && document.querySelector('#wiki-name-rows').rows.length>0")
                target = browser.evaluate('''(()=>{const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change'));
                  q('#wiki-name-kind').value='mission';change(q('#wiki-name-kind'));
                  q('#wiki-name-language').value='1';change(q('#wiki-name-language'));
                  q('#wiki-name-query').value='A b';q('#wiki-name-query').dispatchEvent(new Event('input'));
                  const row=q('#wiki-name-rows tr[data-record-id="4"]');
                  if(!row||row.cells[0].textContent!=='Mission text candidates'||row.cells[2].textContent!=='A b')throw Error('search');
                  return row.querySelector('a').href;})()''')
                browser.command('Page.navigate', {'url': target})
                wait_for(browser, "data.wiki.current==='world' && document.querySelector('#mission-record-id').value==='4'")
                self.assertTrue(browser.evaluate('''(()=>{const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change'));
                  if(!q('#mission-record-panel').open||!q('#mission-name-panel').open||q('#mission-name-language').value!=='1'||
                     q('#mission-records>details>summary time').dateTime!=='2026-11-27T21:58:37Z'||
                     q('#mission-name-panel>summary time').dateTime!=='2026-11-27T21:42:54Z'||
                     !q('#mission-name-panel>summary time').textContent.startsWith('Added ')||
                     !q('#mission-name-panel').textContent.includes('does not prove'))throw Error('mission');
                  for(const id of ['mission-name-pin','mission-name-hide-commands','mission-name-normal-font']){
                    q('#'+id).checked=true;change(q('#'+id));}
                  if(q('#mission-name-preview').textContent!=='A b')throw Error('format');
                  return q('#mission-record-id').selectedOptions[0].textContent.includes('A b');})()'''))
                target = browser.evaluate("document.querySelector('#mission-name-source a').href")
                browser.command('Page.navigate', {'url': target})
                wait_for(browser, "data.wiki.current==='text-pointers' && document.querySelector('#pointer-entry').value==='4'")
                self.assertTrue(browser.evaluate('''(()=>{const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change'));
                  if(q('#pointer-reader').value!=='1'||q('#pointer-language').value!=='1'||q('#pointer-preview').textContent!=='A b'||
                     !q('#pointer-pin').checked||!q('#pointer-hide-commands').checked||!q('#pointer-normal-font').checked||
                     q('#pointer-mission-navigation').hidden||q('#pointer-texts>details>summary time').dateTime!=='2026-11-27T21:42:54Z')throw Error('text link');
                  q('#pointer-reader').value='0';change(q('#pointer-reader'));if(!q('#pointer-mission-navigation').hidden)throw Error('other reader');
                  q('#pointer-reader').value='1';change(q('#pointer-reader'));q('#pointer-entry').value='4';change(q('#pointer-entry'));
                  return !q('#pointer-mission-navigation').hidden;})()'''))
                browser.command('Page.navigate', {'url': browser.evaluate("document.querySelector('#pointer-mission-open').href")})
                wait_for(browser, "data.wiki.current==='world' && document.querySelector('#mission-record-id').value==='4'")
                self.assertTrue(browser.evaluate("document.querySelector('#mission-name-panel').open && document.querySelector('#mission-name-pin').checked && document.querySelector('#mission-name-preview').textContent==='A b'"))
            finally:
                browser.close()

    def test_single_page_and_untrusted(self):
        for trusted in (True, False):
            with self.subTest(trusted=trusted), tempfile.TemporaryDirectory(prefix='viewer-mission-text-single-') as directory:
                work = Path(directory); page, _ = fixture(work, split=False, trusted=trusted); browser = Chrome(work, page)
                try:
                    if trusted:
                        self.assertTrue(browser.evaluate('''(()=>{const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change'));
                          q('#mission-record-id').value='4';change(q('#mission-record-id'));q('#mission-name-language').value='1';change(q('#mission-name-language'));
                          q('#mission-name-source a').click();
                          return q('#pointer-reader').value==='1'&&q('#pointer-language').value==='1'&&q('#pointer-entry').value==='4'&&
                            !q('#pointer-mission-navigation').hidden;})()'''))
                    else:
                        self.assertTrue(browser.evaluate("document.querySelector('#mission-name-panel').hidden && document.querySelector('#pointer-mission-navigation').hidden"))
                finally:
                    browser.close()


if __name__ == '__main__':
    unittest.main()
