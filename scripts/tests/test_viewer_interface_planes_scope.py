"""Verify focused background planes and cross-page source navigation."""
import hashlib
import json
import shutil
import tempfile
import unittest
from pathlib import Path

from test_viewer_bg_descriptors_browser import catalog as descriptor_catalog
from test_viewer_bg_planes_browser import fixture as plane_fixture, wait_for
from test_viewer_scoped_browser import parse_page
from view_rom_data import generate, viewer_resource_scope
from viewer_browser_harness import Chrome


def fixture(work, *, invalid=None):
    plane_fixture(work, invalid=invalid)
    dataset = work / 'dataset'; content = json.dumps(descriptor_catalog()).encode()
    (dataset / 'descriptors.json').write_bytes(content)
    with (dataset / 'manifest.toml').open('a') as manifest:
        manifest.write('[[resources]]\nid="graphics.bg-descriptors"\npath="descriptors.json"\n'
                       'format="bg-descriptors-json-v2"\nsha256="' + hashlib.sha256(content).hexdigest() + '"\n')
    return generate(dataset, work / 'focused', split=True)


class InterfacePlanesProjectionTest(unittest.TestCase):
    def test_partition_support_and_resource_routing(self):
        for invalid in (None, 'paired', 'terrain'):
            with self.subTest(invalid=invalid), tempfile.TemporaryDirectory(prefix='viewer-interface-planes-') as directory:
                page = fixture(Path(directory), invalid=invalid); parts = page.parent / 'index.html.parts'
                home, main, planes, world = map(parse_page, (page, parts / 'interface.html',
                                                             parts / 'interface-planes.html', parts / 'world-map.html'))
                self.assertNotIn('bg_plane_programs', home)
                self.assertNotIn('bg_plane_programs', main)
                self.assertNotIn('bg_graphics', planes)
                if invalid == 'paired':
                    self.assertIsNone(planes['plane_support'])
                else:
                    self.assertEqual(planes['plane_support']['bg_graphics'], main['bg_graphics'])
                    self.assertEqual(planes['plane_support']['palettes'], main['palettes'])
                self.assertEqual(bool(planes['bg_plane_programs']), invalid != 'paired')
                self.assertEqual(bool(world['terrain_seeds']), invalid != 'terrain')
                row = next(row for row in home['resources'] if row['id'] == 'graphics.bg-packet-programs')
                self.assertEqual(row['viewer_scope'], 'interface-planes')
                self.assertEqual(viewer_resource_scope('graphics.bg-packet-programs'), 'interface-planes')
                self.assertIn('graphics.bg-packet-programs', [row['id'] for row in main['resources']])
                self.assertIn('graphics.bg-packet-programs', [row['id'] for row in world['resources']])
                info = next(page for page in home['wiki']['pages'] if page['id'] == 'interface-planes')
                if invalid != 'paired':
                    section = next(section for section in info['sections'] if section['id'] == 'bg-planes')
                    self.assertEqual(section['count'], 3)
                    self.assertEqual(section['change'], {'kind': 'added', 'at': '2026-11-27T22:33:35Z'})


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class InterfacePlanesBrowserTest(unittest.TestCase):
    def test_descriptor_tile_and_old_bookmarks(self):
        with tempfile.TemporaryDirectory(prefix='viewer-interface-planes-browser-') as directory:
            work = Path(directory); page = fixture(work); parts = page.parent / 'index.html.parts'
            browser = Chrome(work, parts / 'interface.html')
            try:
                target = browser.evaluate('''(()=>{const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change'));
                  q('#bg-descriptor-id').value='1';change(q('#bg-descriptor-id'));
                  const link=[...document.querySelectorAll('#bg-descriptor-related a')].find(a=>a.textContent==='Initial planes');
                  if(!link||!link.href.includes('/interface-planes.html?'))throw Error('descriptor link');
                  const row=q('[data-resource-id="graphics.bg-packet-programs"]');
                  if(![...row.querySelectorAll('a')].some(a=>a.textContent==='Open background planes'&&a.href.includes('/interface-planes.html#')))throw Error('resource link');
                  return link.href;})()''')
                browser.command('Page.navigate', {'url': target})
                wait_for(browser, "data.wiki.current==='interface-planes' && document.querySelector('#bg-plane-descriptor').value==='1'")
                self.assertTrue(browser.evaluate('''(()=>{const q=s=>document.querySelector(s);
                  return q('#bg-planes>details').open&&q('#bg-plane-source').textContent.includes('inheritance 0 → 1')&&
                    q('#bg-planes>details>summary time').dateTime==='2026-11-27T22:33:35Z'&&
                    typeof data.bg_graphics==='undefined'&&data.plane_support.bg_graphics&&
                    !q('#bg-plane-tile-navigation').hidden;})()'''))
                browser.command('Page.navigate', {'url': browser.evaluate("document.querySelector('#bg-plane-tile-open').href")})
                wait_for(browser, "data.wiki.current==='interface' && document.querySelector('#bg-descriptor').value==='1'")
                browser.command('Page.navigate', {'url': browser.evaluate("document.querySelector('#bg-plane-open').href")})
                wait_for(browser, "data.wiki.current==='interface-planes' && document.querySelector('#bg-plane-descriptor').value==='1'")
                for old in ('interface.html', 'world.html'):
                    url=(parts / old).as_uri()+'?view=bg-planes&select=bg-plane-descriptor%3A1#bg-planes'
                    browser.command('Page.navigate', {'url': url})
                    wait_for(browser, "data.wiki.current==='interface-planes' && document.querySelector('#bg-plane-descriptor').value==='1'")
                self.assertTrue(browser.evaluate("document.querySelector('#bg-planes>details').open"))
            finally:
                browser.close()

    def test_invalid_paired_data_keeps_terrain_link(self):
        with tempfile.TemporaryDirectory(prefix='viewer-interface-planes-partial-') as directory:
            work = Path(directory); page = fixture(work, invalid='paired'); parts = page.parent / 'index.html.parts'
            browser = Chrome(work, parts / 'interface.html')
            try:
                self.assertTrue(browser.evaluate('''(()=>{const q=s=>document.querySelector(s),row=q('[data-resource-id="graphics.bg-packet-programs"]');
                  return q('#bg-plane-navigation').hidden && [...row.querySelectorAll('a')].some(a=>
                    a.textContent==='Open terrain seeds' && a.href.includes('/world-map.html#terrain-seeds')) &&
                    ![...row.querySelectorAll('a')].some(a=>a.textContent==='Open background planes');})()'''))
            finally:
                browser.close()

    def test_legacy_single_page_keeps_local_plane_links(self):
        with tempfile.TemporaryDirectory(prefix='viewer-interface-planes-single-') as directory:
            work = Path(directory); fixture(work)
            page = generate(work / 'dataset', work / 'single', split=False); browser = Chrome(work, page)
            try:
                self.assertTrue(browser.evaluate('''(()=>{const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change'));
                  q('#bg-descriptor').value='1';change(q('#bg-descriptor'));q('#bg-plane-open').click();
                  if(q('#bg-plane-descriptor').value!=='1'||!q('#bg-planes>details').open)throw Error('plane link');
                  q('#bg-plane-tile-open').click();return q('#bg-descriptor').value==='1'&&q('#bg-tiles>details').open;})()'''))
            finally:
                browser.close()


if __name__ == '__main__':
    unittest.main()
