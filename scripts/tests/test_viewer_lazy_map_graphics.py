"""Check offline, shared, on-demand graphics for the focused map pages."""
import shutil
import tempfile
import unittest
from pathlib import Path

from test_viewer_indexed_ui_tilemaps import fixture, payload
from view_rom_data import generate
from viewer_browser_harness import Chrome


class LazyMapGraphicsProjectionTest(unittest.TestCase):
    def test_shared_asset_is_scoped_and_existing_asset_is_guarded(self):
        with tempfile.TemporaryDirectory(prefix='viewer-lazy-map-scope-') as directory:
            work = Path(directory); index = fixture(work); folder = index.parent / (index.name + '.parts')
            home, indexed, direct, other = map(payload, (index, folder / 'interface-indexed-ui.html',
                                                          folder / 'interface-rectangles.html', folder / 'world.html'))
            asset = indexed['fixed_map_graphics_asset']
            self.assertEqual(asset, direct['fixed_map_graphics_asset'])
            self.assertNotIn('fixed_map_graphics', indexed); self.assertNotIn('fixed_map_graphics_asset', home)
            self.assertNotIn('fixed_map_graphics_asset', other)
            self.assertTrue((folder / asset['href']).is_file())
            before_index = index.read_bytes(); before_page = (folder / 'interface-indexed-ui.html').read_bytes()
            (folder / asset['href']).write_text('unexpected content')
            with self.assertRaisesRegex(ValueError, 'map graphics asset.*identity'):
                generate(work / 'dataset', work / 'view', split=True)
            self.assertEqual(index.read_bytes(), before_index)
            self.assertEqual((folder / 'interface-indexed-ui.html').read_bytes(), before_page)


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class LazyMapGraphicsBrowserTest(unittest.TestCase):
    def test_missing_asset_can_be_retried_without_losing_selection(self):
        with tempfile.TemporaryDirectory(prefix='viewer-lazy-map-browser-') as directory:
            work = Path(directory); index = fixture(work); folder = index.parent / (index.name + '.parts')
            page = folder / 'interface-indexed-ui.html'; asset = folder / payload(page)['fixed_map_graphics_asset']['href']
            saved = asset.with_suffix('.saved'); asset.rename(saved); browser = Chrome(work, page)
            try:
                self.assertTrue(browser.evaluate('''(()=>{const q=s=>document.querySelector(s);
                  q('#tilemap-id').value='1';q('#tilemap-id').dispatchEvent(new Event('change'));
                  return !q('#tilemap-load-map-graphics').hidden&&q('#tilemap-graphics').options.length===1&&
                    q('#tilemap-palette-source').options.length===1&&typeof globalThis.__FFTA_MAP_GRAPHICS__==='undefined';})()'''))
                self.assertTrue(browser.evaluate('''(async()=>{const q=s=>document.querySelector(s),b=q('#tilemap-load-map-graphics');b.click();
                  for(let i=0;i<100;i++){if(!b.disabled&&q('#tilemap-map-graphics-status').textContent.includes('Unable'))return true;
                    await new Promise(r=>setTimeout(r,20));}return false;})()'''))
                saved.rename(asset)
                self.assertTrue(browser.evaluate('''(async()=>{const q=s=>document.querySelector(s),b=q('#tilemap-load-map-graphics');b.click();
                  for(let i=0;i<100&&!b.hidden;i++)await new Promise(r=>setTimeout(r,20));
                  if(!b.hidden||q('#tilemap-id').value!=='1'||q('#tilemap-graphics').options.length!==2||
                    q('#tilemap-palette-source').options.length!==2)throw Error('retry or selection');
                  const change=e=>e.dispatchEvent(new Event('change'));
                  q('#tilemap-graphics').value='0';change(q('#tilemap-graphics'));
                  q('#tilemap-palette-source').value='0';change(q('#tilemap-palette-source'));
                  q('#tilemap-base').value='1022';q('#tilemap-base').dispatchEvent(new Event('input'));
                  const pixel=[...q('#tilemap-canvas').getContext('2d').getImageData(0,0,1,1).data].join(',');
                  q('#tilemap-id').value='2';change(q('#tilemap-id'));b.click();
                  return pixel==='255,0,0,255'&&q('#tilemap-graphics').value==='0'&&q('#tilemap-palette-source').value==='0'&&
                    q('#tilemap-graphics').options.length===2&&q('#tilemap-palette-source').options.length===2;})()'''))
            finally:
                browser.close()

    def test_single_page_keeps_immediate_graphics(self):
        with tempfile.TemporaryDirectory(prefix='viewer-lazy-map-single-') as directory:
            work = Path(directory); page = fixture(work, split=False); browser = Chrome(work, page)
            try:
                self.assertTrue(browser.evaluate('''(()=>{const q=s=>document.querySelector(s);
                  return q('#tilemap-load-map-graphics').hidden&&q('#tilemap-graphics').options.length>1&&
                    q('#tilemap-palette-source').options.length>1;})()'''))
            finally:
                browser.close()


if __name__ == '__main__':
    unittest.main()
