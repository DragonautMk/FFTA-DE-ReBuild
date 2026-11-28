"""Verify focused world/interface pages and the links between their views."""
import json
import shutil
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from test_viewer_navbar_split_browser import wait_for
from test_viewer_scoped_browser import parse_page
from test_viewer_terrain_browser import fixture as terrain_fixture
from test_viewer_ui_context_browser import fixture as ui_fixture
from view_rom_data import generate
from viewer_browser_harness import Chrome


def click_first_line(browser, selector):
    # A wrapped inline link's bounding-box center can lie outside both lines.
    point = browser.evaluate('''(()=>{const e=document.querySelector(%s);
      for(let p=e.parentElement;p;p=p.parentElement)if(p.tagName==='DETAILS')p.open=true;
      e.scrollIntoView({block:'center'});const r=e.getClientRects()[0];
      return {x:r.left+r.width/2,y:r.top+r.height/2};})()''' % json.dumps(selector))
    browser.command('Input.dispatchMouseEvent', {'type': 'mouseMoved', **point})
    browser.command('Input.dispatchMouseEvent', {'type': 'mousePressed', **point, 'button': 'left', 'clickCount': 1})
    browser.command('Input.dispatchMouseEvent', {'type': 'mouseReleased', **point, 'button': 'left', 'clickCount': 1})


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class FocusedScopesBrowserTest(unittest.TestCase):
    def test_world_terrain_roundtrip_and_old_bookmark(self):
        with tempfile.TemporaryDirectory(prefix='viewer-focused-world-') as directory:
            work = Path(directory)
            terrain_fixture(work)
            page = generate(work / 'dataset', work / 'split', split=True)
            parts = page.parent / (page.name + '.parts')
            world, maps = map(parse_page, (parts / 'world.html', parts / 'world-map.html'))
            self.assertNotIn('terrain_seeds', world)
            self.assertNotIn('encounters', maps)
            self.assertEqual(set(maps['terrain_encounters']), {'headers', 'coordinates'})
            self.assertNotIn('units', maps['terrain_encounters'])
            browser = Chrome(work, parts / 'world-map.html')
            try:
                browser.evaluate('''(()=>{const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change'));
                  q('#terrain-descriptor').value='1';change(q('#terrain-descriptor'));
                  q('#terrain-encounter').value='1';change(q('#terrain-encounter'));})()''')
                browser.click('#terrain-encounter-open')
                wait_for(browser, "data.wiki.current==='world-encounters' && document.querySelector('#encounter-id').value==='1'")
                self.assertTrue(browser.evaluate("!document.querySelector('#encounter-terrain-navigation').hidden"))
                click_first_line(browser, '#encounter-terrain-open')
                wait_for(browser, "data.wiki.current==='world-map' && document.querySelector('#terrain-descriptor').value==='1' && document.querySelector('#terrain-encounter').value==='1'")
                self.assertEqual(browser.evaluate("document.querySelector('#terrain-overlay-list').value"), '0')
                legacy = (parts / 'world.html').as_uri() + '?view=terrain-seeds&select=terrain-descriptor%3A1#terrain-seeds'
                browser.command('Page.navigate', {'url': legacy})
                wait_for(browser, "data.wiki.current==='world-map' && document.querySelector('#terrain-descriptor').value==='1'")
                self.assertTrue(browser.evaluate("document.querySelector('#terrain-seeds>details').open"))
            finally:
                browser.close()

    def test_interface_context_scope_and_old_bookmark(self):
        with tempfile.TemporaryDirectory(prefix='viewer-focused-interface-') as directory:
            work = Path(directory); page = ui_fixture(work); parts = page.parent / (page.name + '.parts')
            interface, ui, maps = map(parse_page, (parts / 'interface.html', parts / 'interface-ui.html', parts / 'interface-maps.html'))
            self.assertNotIn('ui_context_templates', interface)
            self.assertNotIn('ui_context_templates', maps)
            self.assertTrue(ui['ui_context_templates'])
            browser = Chrome(work, parts / 'interface.html')
            try:
                legacy = (parts / 'interface.html').as_uri() + '?view=ui-context-templates&select=ui-context-template%3A6#ui-context-templates'
                browser.command('Page.navigate', {'url': legacy})
                wait_for(browser, "data.wiki.current==='interface-ui' && document.querySelector('#ui-context-template').value==='6'")
                self.assertTrue(browser.evaluate("document.querySelector('#ui-context-templates>details').open"))
            finally:
                browser.close()
