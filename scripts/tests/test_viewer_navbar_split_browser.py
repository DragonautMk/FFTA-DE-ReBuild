"""Exercise direct navbar views, separated scopes and old deep links."""
import shutil
import sys
import tempfile
import time
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from test_viewer_scoped_browser import prepared, parse_page, REPO
from test_viewer_bg_planes_browser import fixture as background_fixture
from view_rom_data import write_split_viewer
from viewer_browser_harness import Chrome


def wait_for(browser, condition):
    deadline = time.monotonic() + 30
    while time.monotonic() < deadline:
        try:
            if browser.evaluate('document.readyState === "complete" && ' + condition):
                return
        except (RuntimeError, OSError):
            pass
        time.sleep(.05)
    raise AssertionError('Page or selection did not finish loading: ' + condition)


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class NavbarSplitBrowserTest(unittest.TestCase):
    def test_audio_menu_goes_directly_to_available_subsection(self):
        with tempfile.TemporaryDirectory(prefix='viewer-navbar-audio-') as directory:
            work = Path(directory)
            data, page = prepared(work)
            data['audio_noise'] = {'path': 'noise.json', 'source_offset': 0, 'entries': [0] * 60}
            write_split_viewer(data, page.parent, page.name, (REPO / 'view_rom_data.html').read_text())
            browser = Chrome(work, page)
            try:
                self.assertTrue(browser.evaluate('''(()=>{
                  const nav=document.querySelector('#wiki-pages'), menu=nav.querySelector('[data-category="Audio"]');
                  const links=[...menu.querySelectorAll('a')];
                  return !nav.hidden && getComputedStyle(nav).position==='sticky' && !menu.open &&
                    links.some(a=>a.textContent==='PCM waveforms' && a.href.endsWith('/audio-samples.html#audio')) &&
                    links.some(a=>a.textContent==='CGB noise register codes' && a.href.endsWith('/audio-noise.html#cgb-noise')) &&
                    !links.some(a=>a.textContent==='Timed sound sources');
                })()'''))
                browser.command('Emulation.setDeviceMetricsOverride',
                                {'width':375,'height':850,'deviceScaleFactor':1,'mobile':True})
                self.assertTrue(browser.evaluate('''(()=>{
                  for(const group of document.querySelectorAll('#wiki-page-links > details')) {
                    group.open=true;const rect=group.querySelector('.wiki-menu').getBoundingClientRect();
                    if(rect.left<0 || rect.right>innerWidth || rect.width<=0)return false;
                    group.open=false;
                  }
                  return document.documentElement.scrollWidth<=innerWidth+2;
                })()'''))
                browser.command('Emulation.setDeviceMetricsOverride',
                                {'width':1280,'height':900,'deviceScaleFactor':1,'mobile':False})
                browser.evaluate("document.querySelector('[data-category=\"Audio\"] > summary').click()")
                self.assertTrue(browser.evaluate("document.querySelector('[data-category=\"Audio\"]').open"))
                self.assertTrue(browser.evaluate('''(()=>{
                  const menu=document.querySelector('[data-category="Audio"]');
                  const noise=[...menu.querySelectorAll('a')].find(a=>a.textContent==='CGB noise register codes');
                  return !menu.querySelector('details') && !!noise && noise.getClientRects().length>0;
                })()'''))
                browser.click('#wiki-page-links a[href$="/audio-noise.html#cgb-noise"]')
                wait_for(browser, "typeof data !== 'undefined' && data.wiki.current === 'audio-noise' && location.hash === '#cgb-noise'")
                self.assertTrue(browser.evaluate('''(()=>{const s=document.querySelector('#cgb-noise');
                  return !s.hidden && s.querySelector(':scope > details').open && s.closest('.section-group').open &&
                    typeof data.audio==='undefined' && typeof data.sequences==='undefined';})()'''))
                legacy = (page.parent / (page.name + '.parts') / 'audio.html').as_uri() + '#cgb-noise'
                browser.command('Page.navigate', {'url': legacy})
                wait_for(browser, "typeof data !== 'undefined' && data.wiki.current === 'audio-noise' && location.hash === '#cgb-noise'")
                browser.evaluate("document.querySelector('[data-category=\"Audio\"] > summary').focus(); document.querySelector('[data-category=\"Audio\"] > summary').click()")
                browser.command('Input.dispatchKeyEvent', {'type':'keyDown','key':'Escape','code':'Escape','windowsVirtualKeyCode':27})
                browser.command('Input.dispatchKeyEvent', {'type':'keyUp','key':'Escape','code':'Escape','windowsVirtualKeyCode':27})
                self.assertFalse(browser.evaluate("document.querySelector('[data-category=\"Audio\"]').open"))
                self.assertTrue(browser.evaluate("document.activeElement.matches('[data-category=\"Audio\"] > summary')"))
            finally:
                browser.close()

    def test_scope_partition_cross_links_and_legacy_bookmark(self):
        with tempfile.TemporaryDirectory(prefix='viewer-navbar-world-') as directory:
            work = Path(directory)
            index, interface = background_fixture(work)
            world = interface.with_name('world-map.html')
            home, world_data, interface_data = map(parse_page, (index, world, interface))
            self.assertIn('terrain_seeds', world_data)
            self.assertNotIn('bg_graphics', world_data)
            self.assertIn('bg_graphics', interface_data)
            self.assertNotIn('terrain_seeds', interface_data)
            rows = {row['id']: row for row in home['resources']}
            self.assertEqual(rows['graphics.bg-main-graphics']['viewer_scope'], 'interface')
            self.assertEqual(rows['graphics.bg-packet-programs']['viewer_scope'], 'interface-planes')
            self.assertIn('graphics.bg-packet-programs', [row['id'] for row in world_data['resources']])
            self.assertNotIn('bg_plane_programs', interface_data)
            self.assertIn('bg_plane_programs', parse_page(interface.with_name('interface-planes.html')))
            browser = Chrome(work, interface)
            try:
                self.assertTrue(browser.evaluate('''(()=>{const q=s=>document.querySelector(s);
                  q('#bg-descriptor').value='1';q('#bg-descriptor').dispatchEvent(new Event('change'));
                  return !q('#bg-terrain-navigation').hidden && q('#bg-terrain-open').href.includes('/world-map.html?');})()'''))
                browser.click('#bg-terrain-open')
                wait_for(browser, "typeof data !== 'undefined' && data.wiki.current === 'world-map' && document.querySelector('#terrain-descriptor').value === '1'")
                self.assertTrue(browser.evaluate("document.querySelector('#terrain-seeds>details').open && !document.querySelector('#terrain-bg-navigation').hidden"))
                browser.click('#terrain-bg-open')
                wait_for(browser, "typeof data !== 'undefined' && data.wiki.current === 'interface' && document.querySelector('#bg-descriptor').value === '1'")
                self.assertTrue(browser.evaluate("document.querySelector('#bg-tiles>details').open"))
                legacy = world.with_name('world.html').as_uri() + '?view=bg-tiles&select=bg-descriptor%3A1#bg-tiles'
                browser.command('Page.navigate', {'url': legacy})
                wait_for(browser, "typeof data !== 'undefined' && data.wiki.current === 'interface' && document.querySelector('#bg-descriptor').value === '1'")
                self.assertTrue(browser.evaluate("location.hash==='#bg-tiles' && document.querySelector('#bg-tiles>details').open"))
            finally:
                browser.close()
