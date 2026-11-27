"""Check offline on-demand sharing of transfer graphics across map views."""
import hashlib
import json
import shutil
import sys
import tempfile
import time
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from view_rom_data import generate
from test_viewer_compact_cell_bank_browser import segments as compact_segments
from test_viewer_type3_payloads_browser import segments as type3_segments
from viewer_browser_harness import Chrome


def fixture(work, *, split=True, include_type3=True):
    dataset = work / 'dataset'; dataset.mkdir()
    manifest = 'profile="europe"\nextractor_revision=1\ndecoder_revision=1\n'
    binary = [('compact-cell-bank', compact_segments())]
    if include_type3: binary.append(('tilemap-type3-transfers', type3_segments()))
    for name, parts in binary:
        content = b''.join(chunk for _, _, chunk in parts); (dataset / (name + '.bin')).write_bytes(content)
        manifest += (f'[[resources]]\nid="graphics.{name}"\npath="{name}.bin"\n'
                     f'format="{name}-v1-bin"\nsha256="{hashlib.sha256(content).hexdigest()}"\n')
        for off, role, chunk in parts:
            manifest += (f'[[resources.sources]]\noffset={off}\nlength={len(chunk)}\nrole="{role}"\n'
                         f'sha256="{hashlib.sha256(chunk).hexdigest()}"\n')
    observed = {'source_offset': 0x7000, 'width': 1, 'height': 1, 'mode': 4,
                'runs': [{'control': 0x4001, 'value': 0}]}
    content = json.dumps(observed).encode(); (dataset / 'observed-tilemap.json').write_bytes(content)
    manifest += ('[[resources]]\nid="graphics.observed-tilemap"\npath="observed-tilemap.json"\n'
                 f'format="observed-tilemap-v1-json"\nsha256="{hashlib.sha256(content).hexdigest()}"\n')
    graphics = {'banks': [{'source_offset': 0x8000, 'words': [0x33333333] * 8}]}
    content = json.dumps(graphics).encode(); (dataset / 'obj-custom-banks.json').write_bytes(content)
    manifest += ('[[resources]]\nid="graphics.obj-custom-banks"\npath="obj-custom-banks.json"\n'
                 f'format="obj-custom-banks-v1-json"\nsha256="{hashlib.sha256(content).hexdigest()}"\n')
    (dataset / 'manifest.toml').write_text(manifest)
    return generate(dataset, work / 'view', split=split), dataset


def payload(path):
    return json.JSONDecoder().raw_decode(path.read_text().split('const data = ', 1)[1])[0]


def wait_for(browser, expression):
    deadline = time.monotonic() + 25
    while time.monotonic() < deadline:
        try:
            if browser.evaluate(expression): return
        except (RuntimeError, OSError):
            pass
        time.sleep(.05)
    raise AssertionError('Browser condition not reached: ' + expression)


class TransferGraphicsGenerationTest(unittest.TestCase):
    def test_scope_and_content_addressed_asset_ownership(self):
        with tempfile.TemporaryDirectory(prefix='viewer-lazy-transfer-') as directory:
            work = Path(directory); index, dataset = fixture(work); parts = index.parent / 'index.html.parts'
            home, world, transfer = payload(index), payload(parts / 'interface-maps.html'), payload(parts / 'transfer-graphics.html')
            self.assertNotIn('type3_payloads', world); self.assertNotIn('transfer_graphics', home['wiki'])
            self.assertEqual(len(transfer['type3_payloads']['streams']), 1)
            asset = world['wiki']['transfer_graphics']; path = parts / asset['href']
            self.assertTrue(path.is_file()); self.assertIn(asset['key'], path.name)
            original = path.read_bytes(); self.assertIn(asset['key'].encode(), original)
            path.write_text('user file')
            with self.assertRaisesRegex(ValueError, 'asset does not match'):
                generate(dataset, work / 'view', split=True)
            self.assertEqual(path.read_text(), 'user file')
        with tempfile.TemporaryDirectory(prefix='viewer-lazy-transfer-none-') as directory:
            work = Path(directory); index, _ = fixture(work, include_type3=False)
            self.assertNotIn('transfer_graphics', payload(index.parent / 'index.html.parts/interface-maps.html')['wiki'])


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class TransferGraphicsBrowserTest(unittest.TestCase):
    def test_shared_loading_pixels_and_bookmark(self):
        with tempfile.TemporaryDirectory(prefix='viewer-lazy-transfer-browser-') as directory:
            work = Path(directory); index, _ = fixture(work); browser = Chrome(work, index.parent / 'index.html.parts/interface-maps.html')
            try:
                self.assertTrue(browser.evaluate('''(()=>{const q=s=>document.querySelector(s);return !q('#tilemap-load-transfer').hidden &&
                  !q('#compact-cell-load-transfer').hidden && typeof globalThis.__FFTA_TYPE3_PAYLOADS__==='undefined' &&
                  !document.querySelector('script[data-transfer-graphics]') && q('#tilemap-graphics').options.length===2;})()'''))
                browser.evaluate('''(()=>{for(const id of ['tilemap-graphics','compact-cell-graphics']) {
                  const select=document.getElementById(id); select.value='0';
                  select.dispatchEvent(new Event('change',{bubbles:true}));
                }})()''')
                browser.evaluate("document.querySelector('#tilemap-load-transfer').click()")
                wait_for(browser, "document.querySelector('#tilemap-load-transfer').disabled && document.querySelector('#compact-cell-load-transfer').disabled")
                bookmark = browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change',{bubbles:true})),
                    pixel=s=>[...q(s).getContext('2d').getImageData(0,0,1,1).data];
                  if(document.querySelectorAll('script[data-transfer-graphics]').length!==1||q('#tilemap-graphics').options.length!==4||
                     q('#compact-cell-graphics').options.length!==4||!q('#compact-cell-transfer-load-status').textContent.includes('1 raw'))throw Error('shared loading');
                  if(q('#tilemap-graphics').value!=='0'||q('#compact-cell-graphics').value!=='0')throw Error('existing selection changed');
                  q('#tilemap-graphics').value='1';change(q('#tilemap-graphics'));
                  if(pixel('#tilemap-canvas').join(',')!=='17,17,17,255')throw Error('map pixel');
                  q('#compact-cell-frame').value='1';change(q('#compact-cell-frame'));
                  q('#compact-cell-graphics').value='2';change(q('#compact-cell-graphics'));
                  if(pixel('#compact-cell-canvas').join(',')!=='51,51,51,255')throw Error('raw animation pixel');
                  q('#compact-cell-loop').checked=true;q('#compact-cell-rate').value='20';
                  q('#compact-cell-record').value='1';change(q('#compact-cell-record'));
                  if(!q('#compact-cell-loop').checked||q('#compact-cell-rate').value!=='20'||q('#compact-cell-graphics').value!=='2')throw Error('settings');
                  q('#compact-cell-record').value='0';change(q('#compact-cell-record'));
                  q('#compact-cell-frame').value='1';change(q('#compact-cell-frame'));
                  const link=q('#compact-cell-bank>details>p>a');link.dispatchEvent(new Event('focus'));
                  return link.href;
                })()''')
                browser.command('Page.navigate', {'url': bookmark})
                wait_for(browser, "document.readyState==='complete' && document.querySelector('#compact-cell-graphics').options.length===4 && document.querySelector('#compact-cell-graphics').value==='2'")
                self.assertTrue(browser.evaluate("document.querySelector('#compact-cell-bank>details').open && document.querySelectorAll('script[data-transfer-graphics]').length===1"))
            finally: browser.close()

    def test_missing_and_wrong_asset_are_reported_and_retryable(self):
        with tempfile.TemporaryDirectory(prefix='viewer-lazy-transfer-missing-') as directory:
            work = Path(directory); index, _ = fixture(work); parts = index.parent / 'index.html.parts'
            asset = parts / payload(parts / 'interface-maps.html')['wiki']['transfer_graphics']['href']; saved = asset.read_bytes(); asset.unlink()
            browser = Chrome(work, parts / 'interface-maps.html')
            try:
                browser.evaluate("document.querySelector('#tilemap-load-transfer').click()")
                wait_for(browser, "document.querySelector('#tilemap-transfer-load-status').textContent.includes('could not be opened')")
                self.assertTrue(browser.evaluate("!document.querySelector('#tilemap-load-transfer').disabled && document.querySelector('#tilemap-graphics').options.length===2"))
                asset.write_text('globalThis.__FFTA_TYPE3_PAYLOADS__={key:"wrong",payload:{streams:[],raw_streams:[]}};')
                browser.evaluate("document.querySelector('#tilemap-load-transfer').click()")
                wait_for(browser, "document.querySelector('#tilemap-transfer-load-status').textContent.includes('does not match')")
                self.assertEqual(browser.evaluate("document.querySelector('#tilemap-graphics').options.length"), 2)
                asset.write_bytes(saved)
                browser.evaluate("document.querySelector('#tilemap-load-transfer').click()")
                wait_for(browser, "document.querySelector('#tilemap-graphics').options.length===4")
            finally: browser.close()

    def test_single_page_and_missing_resource(self):
        with tempfile.TemporaryDirectory(prefix='viewer-lazy-transfer-single-') as directory:
            work = Path(directory); index, _ = fixture(work, split=False); browser = Chrome(work, index)
            try:
                browser.evaluate("document.querySelector('#compact-cell-load-transfer').click()")
                wait_for(browser, "document.querySelector('#compact-cell-graphics').options.length===4")
                self.assertEqual(browser.evaluate("document.querySelectorAll('script[data-transfer-graphics]').length"), 0)
            finally: browser.close()
        with tempfile.TemporaryDirectory(prefix='viewer-lazy-transfer-none-') as directory:
            work = Path(directory); index, _ = fixture(work, include_type3=False); browser = Chrome(work, index.parent / 'index.html.parts/interface-maps.html')
            try: self.assertTrue(browser.evaluate("document.querySelector('#compact-cell-load-transfer').hidden && document.querySelector('#tilemap-load-transfer').hidden"))
            finally: browser.close()
