"""Check lazy transfer graphics in the offline object-animation viewer."""
import hashlib
import json
import shutil
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from view_rom_data import generate
from test_viewer_constructor_obj_browser import resources as constructor_resources
from test_viewer_transfer_graphics_lazy_browser import fixture as transfer_fixture, payload, wait_for
from viewer_browser_harness import Chrome


def fixture(work, *, split=True, include_type3=True):
    _, dataset = transfer_fixture(work, split=split, include_type3=include_type3)
    animation, static = constructor_resources()
    manifest = (dataset / 'manifest.toml').read_text()
    for rid, name, version, value in [('graphics.constructor-obj-animation', 'constructor-obj-animation', 1, animation),
                                      ('graphics.obj-static-lists', 'obj-static-lists', 2, static)]:
        content = json.dumps(value).encode(); (dataset / (name + '.json')).write_bytes(content)
        manifest += (f'[[resources]]\nid="{rid}"\npath="{name}.json"\nformat="{name}-v{version}-json"\n'
                     f'sha256="{hashlib.sha256(content).hexdigest()}"\n')
    (dataset / 'manifest.toml').write_text(manifest)
    return generate(dataset, work / 'view', split=split)


class ObjectTransferGenerationTest(unittest.TestCase):
    def test_only_relevant_scopes_reference_shared_asset(self):
        with tempfile.TemporaryDirectory(prefix='viewer-obj-transfer-scope-') as directory:
            index = fixture(Path(directory)); parts = index.parent / 'index.html.parts'
            home, world, interface, objects, audio = (payload(path) for path in (index, parts / 'world.html', parts / 'interface.html', parts / 'objects.html', parts / 'audio.html'))
            self.assertNotIn('transfer_graphics', home['wiki']); self.assertNotIn('transfer_graphics', audio['wiki'])
            self.assertNotIn('transfer_graphics', world['wiki'])
            self.assertEqual(interface['wiki']['transfer_graphics'], objects['wiki']['transfer_graphics'])
            self.assertNotIn('type3_payloads', objects); self.assertTrue((parts / objects['wiki']['transfer_graphics']['href']).is_file())
        with tempfile.TemporaryDirectory(prefix='viewer-obj-transfer-no-source-') as directory:
            index = fixture(Path(directory), include_type3=False)
            self.assertNotIn('transfer_graphics', payload(index.parent / 'index.html.parts/objects.html')['wiki'])


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class ObjectTransferBrowserTest(unittest.TestCase):
    def test_lazy_loading_pixels_selection_and_bookmark(self):
        with tempfile.TemporaryDirectory(prefix='viewer-obj-transfer-browser-') as directory:
            work = Path(directory); index = fixture(work); browser = Chrome(work, index.parent / 'index.html.parts/objects.html')
            try:
                self.assertTrue(browser.evaluate('''(()=>{const q=s=>document.querySelector(s);
                  return !q('#obj-animation-load-transfer').hidden && q('#obj-animation-graphics').options.length===2 &&
                    typeof globalThis.__FFTA_TYPE3_PAYLOADS__==='undefined' && !q('script[data-transfer-graphics]');})()'''))
                browser.evaluate('''(()=>{const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change',{bubbles:true}));
                  q('[data-resource-id="graphics.constructor-obj-animation"] a[href="#obj-animations"]').click();
                  q('#obj-animation-graphics').value='0';change(q('#obj-animation-graphics'));
                  q('#obj-animation-outlines').checked=false;change(q('#obj-animation-outlines'));
                  q('#obj-animation-load-transfer').click();})()''')
                wait_for(browser, "document.querySelector('#obj-animation-load-transfer').disabled && document.querySelector('#obj-animation-graphics').options.length===4")
                bookmark = browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change',{bubbles:true})),
                    input=e=>e.dispatchEvent(new Event('input',{bubbles:true})),pixel=()=>[...q('#obj-animation-canvas').getContext('2d').getImageData(10,10,1,1).data];
                  if(q('#obj-animation-graphics').value!=='0'||!q('#obj-animation-transfer-load-status').textContent.includes('1 raw')||
                     document.querySelectorAll('script[data-transfer-graphics]').length!==1)throw Error('lazy loading or selection');
                  q('#obj-animation-graphics').value='1';change(q('#obj-animation-graphics'));
                  if(pixel().join(',')!=='17,17,17,255')throw Error('compressed pixel');
                  q('#obj-animation-graphics').value='2';change(q('#obj-animation-graphics'));
                  if(q('#obj-animation-lz-offset').disabled||pixel().join(',')!=='85,85,85,255')throw Error('raw pixel');
                  q('#obj-animation-lz-offset').value='9999';input(q('#obj-animation-lz-offset'));
                  if(!q('#obj-animation-detail').textContent.includes('valid raw byte offset'))throw Error('raw offset error');
                  q('#obj-animation-lz-offset').value='0';input(q('#obj-animation-lz-offset'));
                  q('#obj-animation-rate').value='20';q('#obj-animation-loop').checked=true;
                  q('#obj-animation-catalog').value='static_lists';change(q('#obj-animation-catalog'));
                  q('#obj-animation-catalog').value='constructor_phases';change(q('#obj-animation-catalog'));
                  if(q('#obj-animation-graphics').value!=='2'||q('#obj-animation-rate').value!=='20'||!q('#obj-animation-loop').checked)throw Error('settings');
                  const link=q('#obj-animations>details>p>a');link.dispatchEvent(new Event('focus'));return link.href;
                })()''')
                browser.command('Page.navigate', {'url': bookmark})
                wait_for(browser, "document.readyState==='complete' && document.querySelector('#obj-animation-graphics').options.length===4 && document.querySelector('#obj-animation-graphics').value==='2'")
                self.assertTrue(browser.evaluate("document.querySelector('#obj-animations>details').open && document.querySelectorAll('script[data-transfer-graphics]').length===1"))
            finally: browser.close()

    def test_missing_and_mismatched_assets_can_be_retried(self):
        with tempfile.TemporaryDirectory(prefix='viewer-obj-transfer-retry-') as directory:
            work = Path(directory); index = fixture(work); parts = index.parent / 'index.html.parts'
            asset = parts / payload(parts / 'objects.html')['wiki']['transfer_graphics']['href']; saved = asset.read_bytes(); asset.unlink()
            browser = Chrome(work, parts / 'objects.html')
            try:
                browser.evaluate("document.querySelector('#obj-animation-load-transfer').click()")
                wait_for(browser, "document.querySelector('#obj-animation-transfer-load-status').textContent.includes('could not be opened')")
                self.assertTrue(browser.evaluate("!document.querySelector('#obj-animation-load-transfer').disabled && document.querySelector('#obj-animation-graphics').options.length===2"))
                asset.write_text('globalThis.__FFTA_TYPE3_PAYLOADS__={key:"wrong",payload:{streams:[],raw_streams:[]}};')
                browser.evaluate("document.querySelector('#obj-animation-load-transfer').click()")
                wait_for(browser, "document.querySelector('#obj-animation-transfer-load-status').textContent.includes('does not match')")
                asset.write_bytes(saved); browser.evaluate("document.querySelector('#obj-animation-load-transfer').click()")
                wait_for(browser, "document.querySelector('#obj-animation-graphics').options.length===4")
            finally: browser.close()

    def test_single_page_shares_asset_with_world_views(self):
        with tempfile.TemporaryDirectory(prefix='viewer-obj-transfer-single-') as directory:
            work = Path(directory); index = fixture(work, split=False); browser = Chrome(work, index)
            try:
                browser.evaluate('''(()=>{const change=e=>e.dispatchEvent(new Event('change',{bubbles:true}));
                  for(const id of ['obj-animation-graphics','tilemap-graphics','compact-cell-graphics']){
                    const select=document.getElementById(id);select.value='0';change(select);}
                  document.querySelector('#obj-animation-load-transfer').click();})()''')
                wait_for(browser, "['obj-animation-load-transfer','tilemap-load-transfer','compact-cell-load-transfer'].every(id=>document.getElementById(id).disabled)")
                self.assertTrue(browser.evaluate('''(()=>{
                  const ids=['obj-animation-graphics','tilemap-graphics','compact-cell-graphics'];
                  return ids.every(id=>document.getElementById(id).options.length===4&&document.getElementById(id).value==='0')&&
                    !document.querySelector('script[data-transfer-graphics]');})()'''))
            finally: browser.close()
        with tempfile.TemporaryDirectory(prefix='viewer-obj-transfer-absent-') as directory:
            work = Path(directory); index = fixture(work, include_type3=False); browser = Chrome(work, index.parent / 'index.html.parts/objects.html')
            try: self.assertTrue(browser.evaluate("document.querySelector('#obj-animation-load-transfer').hidden"))
            finally: browser.close()
