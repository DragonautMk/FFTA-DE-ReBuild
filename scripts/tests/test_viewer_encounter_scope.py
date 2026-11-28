"""Verify encounter page partitioning and cross-page reference navigation."""
import hashlib
import json
import shutil
import tempfile
import unittest
from pathlib import Path

import view_rom_data as viewer
from test_viewer_encounters_browser import fixture as encounter_fixture
from test_viewer_mission_events_browser import fixture as mission_fixture
from test_viewer_navbar_split_browser import wait_for
from test_viewer_scoped_browser import parse_page
from viewer_browser_harness import Chrome


def fixture(work, *, partial=False):
    encounter_fixture(work, coordinates_only=partial)
    other = work / 'events'; other.mkdir(); mission_fixture(other)
    raw = (other / 'dataset/event-records12.json').read_bytes()
    dataset = work / 'dataset'; (dataset / 'event-records12.json').write_bytes(raw)
    with (dataset / 'manifest.toml').open('a') as manifest:
        manifest.write('[[resources]]\nid="gameplay.event-records12"\npath="event-records12.json"\n'
                       'format="event-records12-v1-json"\nsha256="' + hashlib.sha256(raw).hexdigest() + '"\n')
    return viewer.generate(dataset, work / 'split', split=True)


class EncounterScopeProjectionTest(unittest.TestCase):
    def test_partition_resources_and_partial_inputs(self):
        for partial in (False, True):
            with self.subTest(partial=partial), tempfile.TemporaryDirectory(prefix='viewer-encounter-scope-') as directory:
                page = fixture(Path(directory), partial=partial); parts = page.parent / 'index.html.parts'
                home, world, encounters = map(parse_page, (page, parts / 'world.html', parts / 'world-encounters.html'))
                self.assertNotIn('encounters', world)
                self.assertNotIn('encounters', home)
                self.assertNotIn('mission_events', encounters)
                self.assertNotIn('items', encounters)
                refs = world['encounter_references']; support = encounters['encounter_support']
                self.assertEqual(refs['units_available'], not partial)
                self.assertEqual(len(refs['units']), 0 if partial else 24)
                self.assertEqual(support['events'], [{'index': i, 'primary02': value}
                                                      for i, value in enumerate((0, 2, 200))])
                self.assertTrue(support['events_available'])
                row = next(row for row in home['resources'] if row['id'] == 'gameplay.encounter-coordinates')
                self.assertEqual(row['viewer_scope'], 'world-encounters')
                self.assertIn('coordinates', encounters['encounters'])
                self.assertEqual(viewer.viewer_resource_scope('gameplay.encounter-unit-templates'), 'world-encounters')
                self.assertEqual(refs['headers'], [] if partial else [1, 2])
                page_info = next(page for page in home['wiki']['pages'] if page['id'] == 'world-encounters')
                section = next(section for section in page_info['sections'] if section['id'] == 'encounters')
                self.assertEqual(section['change'], {'kind': 'added', 'at': '2026-11-27T21:58:37Z'})


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class EncounterScopeBrowserTest(unittest.TestCase):
    def test_item_event_navigation_and_old_bookmark(self):
        with tempfile.TemporaryDirectory(prefix='viewer-encounter-scope-browser-') as directory:
            work = Path(directory); page = fixture(work); parts = page.parent / 'index.html.parts'
            browser = Chrome(work, parts / 'world-encounters.html')
            try:
                self.assertTrue(browser.evaluate('''(()=>{const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change'));
                  if(q('#encounters').hidden || q('#encounters>details>summary time').dateTime!=='2026-11-27T21:58:37Z')throw Error('section');
                  q('#encounter-id').value='2';change(q('#encounter-id'));q('#encounter-unit').value='20';change(q('#encounter-unit'));
                  return q('#encounter-equipment a').href.includes('world.html?');})()'''))
                browser.command('Page.navigate', {'url': browser.evaluate("document.querySelector('#encounter-equipment a').href")})
                wait_for(browser, "data.wiki.current==='world' && document.querySelector('#item-id').value==='1'")
                self.assertTrue(browser.evaluate('''(()=>{const q=s=>document.querySelector(s);
                  if(q('#item-encounter-references').hidden||q('#item-encounter-rows').rows.length!==20||
                     !q('#item-encounter-page').textContent.includes('24 references'))throw Error('item refs');
                  q('#item-encounter-next').click();
                  return q('#item-encounter-rows').rows.length===4;})()'''))
                browser.command('Page.navigate', {'url': browser.evaluate("document.querySelector('#item-encounter-rows a').href")})
                wait_for(browser, "data.wiki.current==='world-encounters' && document.querySelector('#encounter-id').value==='2' && document.querySelector('#encounter-unit').value==='20'")
                self.assertTrue(browser.evaluate("document.querySelector('#encounter-unit-details').open && !document.querySelector('#encounter-event-references').hidden"))
                browser.command('Page.navigate', {'url': browser.evaluate("document.querySelector('#encounter-event-rows a').href")})
                wait_for(browser, "data.wiki.current==='world' && document.querySelector('#event-record-id').value==='1'")
                self.assertTrue(browser.evaluate("document.querySelector('#event-record-panel').open && !document.querySelector('#event-encounter-navigation').hidden"))
                browser.command('Page.navigate', {'url': browser.evaluate("document.querySelector('#event-encounter-open').href")})
                wait_for(browser, "data.wiki.current==='world-encounters' && document.querySelector('#encounter-id').value==='2'")
                legacy = (parts / 'world.html').as_uri() + '?view=encounters&select=encounter-id%3A1&select=encounter-unit%3A19#encounters'
                browser.command('Page.navigate', {'url': legacy})
                wait_for(browser, "data.wiki.current==='world-encounters' && document.querySelector('#encounter-id').value==='2' && document.querySelector('#encounter-unit').value==='20'")
                self.assertTrue(browser.evaluate("document.querySelector('#encounter-unit-details').open && document.querySelector('#encounters>details').open"))
            finally:
                browser.close()

    def test_partial_catalog_and_missing_items(self):
        with tempfile.TemporaryDirectory(prefix='viewer-encounter-scope-partial-') as directory:
            work = Path(directory); page = fixture(work, partial=True); parts = page.parent / 'index.html.parts'
            browser = Chrome(work, parts / 'world-encounters.html')
            try:
                self.assertTrue(browser.evaluate('''(()=>{const q=s=>document.querySelector(s);
                  return !q('#encounters').hidden && q('#encounter-id').value==='1' &&
                    q('#encounter-unit-status').textContent.includes('unavailable') && q('#encounter-points').rows.length===2 &&
                    q('#encounter-name-language-control').hidden;})()'''))
            finally:
                browser.close()


if __name__ == '__main__':
    unittest.main()
