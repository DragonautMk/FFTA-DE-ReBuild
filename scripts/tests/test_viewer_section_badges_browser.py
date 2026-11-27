"""Check stable, local-time badges for added and updated wiki sections."""
import json
import shutil
import tempfile
import unittest
from pathlib import Path

from test_viewer_conditional_lookups import fixture
from test_viewer_scoped_browser import REPO, parse_page, prepared
from view_rom_data import generate, viewer_section_updates, write_split_viewer
from viewer_browser_harness import Chrome


class SectionBadgeMetadataTest(unittest.TestCase):
    def test_registered_timestamp_is_stable_across_regeneration(self):
        with tempfile.TemporaryDirectory(prefix='viewer-section-badge-') as directory:
            work = Path(directory); index, _ = fixture(work)
            before = parse_page(index)
            world = next(page for page in before['wiki']['pages'] if page['id'] == 'world')
            section = next(item for item in world['sections'] if item['id'] == 'conditional-lookups')
            self.assertEqual(section['change'], viewer_section_updates()['world']['conditional-lookups'])
            self.assertNotIn('objects-physical', [page['id'] for page in before['wiki']['pages']])
            second = generate(work / 'dataset', work / 'again', split=True)
            after = parse_page(second)
            world_after = next(page for page in after['wiki']['pages'] if page['id'] == 'world')
            self.assertEqual(world['sections'], world_after['sections'])
            self.assertEqual(viewer_section_updates()['world']['conditional-lookups'], section['change'])


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class SectionBadgeBrowserTest(unittest.TestCase):
    def test_index_sections_can_show_updated_badges(self):
        with tempfile.TemporaryDirectory(prefix='viewer-index-badge-browser-') as directory:
            work = Path(directory); data, index = prepared(work)
            data['section_updates'] = {'index': {'resource-index': {'kind': 'updated', 'at': '2026-11-27T18:00:00Z'}}}
            index = write_split_viewer(data, index.parent, index.name, (REPO / 'view_rom_data.html').read_text())
            browser = Chrome(work, index)
            try:
                self.assertTrue(browser.evaluate('''(()=>{const t=document.querySelector('#resource-index > details > summary time');
                  return !!t && t.dateTime==='2026-11-27T18:00:00Z' && t.textContent.startsWith('Updated ');})()'''))
            finally:
                browser.close()

    def test_badges_mark_navbar_group_and_section_with_datetime(self):
        with tempfile.TemporaryDirectory(prefix='viewer-section-badge-browser-') as directory:
            work = Path(directory); index, _ = fixture(work)
            parts = index.parent / (index.name + '.parts')
            browser = Chrome(work, parts / 'world.html')
            try:
                expected = viewer_section_updates()['world']['conditional-lookups']
                self.assertTrue(browser.evaluate('''(()=>{
                  const expected=EXPECTED_BADGE,q=s=>document.querySelector(s),at=expected.at;
                  const section=q('#conditional-lookups > details > summary time');
                  const group=q('.section-group[data-category="Conditions and flags"] > summary time');
                  const nav=q('.wiki-page-category[data-category="World and other data"] > summary time');
                  const sidebar=q('#contents a[href="#conditional-lookups"] time');
                  if(![section,group,nav,sidebar].every(x=>x && x.dateTime===at && /\\d{2}:\\d{2}/.test(x.textContent)))return false;
                  return section.textContent.startsWith(expected.kind==='added'?'Added ':'Updated ') && group.textContent.startsWith('Updated ') &&
                    section.title===at && !q('#conditional-lookups > details').open;
                })()'''.replace('EXPECTED_BADGE', json.dumps(expected))))
                browser.command('Emulation.setDeviceMetricsOverride',
                                {'width':320,'height':850,'deviceScaleFactor':1,'mobile':True})
                self.assertTrue(browser.evaluate("document.documentElement.scrollWidth<=innerWidth+2"))
            finally:
                browser.close()
            single = generate(work / 'dataset', work / 'single')
            separate = work / 'single-browser'; separate.mkdir()
            browser = Chrome(separate, single)
            try:
                self.assertTrue(browser.evaluate('''(()=>{const expected=EXPECTED_BADGE,t=document.querySelector('#conditional-lookups > details > summary time');
                  return !!t && t.dateTime===expected.at && t.textContent.startsWith(expected.kind==='added'?'Added ':'Updated ');})()'''.replace('EXPECTED_BADGE', json.dumps(expected))))
            finally:
                browser.close()


if __name__ == '__main__':
    unittest.main()
