"""Check descriptor-linked animated background transfer tile previews."""
import copy
import hashlib
import json
import shutil
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from view_rom_data import generate, project_bg_transfers
from viewer_browser_harness import Chrome


def catalog():
    def link(control, tiles, destination=0x06000000):
        return {'control': control, 'tiles': tiles, 'destination': destination}
    return {'descriptor_links': [[link(0, 0), None, link(1, 0, 0x06000400), None, None],
                                 [None, link(2, 1), None, None, None], [None] * 5],
            'variant_sources': [None, None, None],
            'controls': [
                {'relative_source': 100, 'transfer_bytes': 32, 'header_byte3': 7,
                 'frames': [{'duration_ticks': 2, 'source_tile': 0}, {'duration_ticks': 3, 'source_tile': 1}]},
                {'relative_source': 200, 'transfer_bytes': 32, 'header_byte3': 0,
                 'frames': [{'duration_ticks': 0, 'source_tile': 1}]},
                {'relative_source': 300, 'transfer_bytes': 129 * 32, 'header_byte3': 1,
                 'frames': [{'duration_ticks': 1, 'source_tile': 0}, {'duration_ticks': 4, 'source_tile': 1}]}],
            'tiles': [{'relative_source': 1000, 'words': [0x11111111] * 8 + [0x22222222] * 8},
                      {'relative_source': 2000, 'words': [0x11111111] * (129 * 8) + [0x22222222] * 8}]}


def fixture(work, *, background=True, palette=True, value=None, split=False):
    dataset = work / 'dataset'; dataset.mkdir()
    data = [('graphics.bg-transfer-programs', 'bg-transfer-programs', 1, catalog() if value is None else value)]
    if background:
        graphics = {'descriptor_graphics': [0, 0, 0], 'graphics': [
            {'relative_source': 80, 'destination_offset': 0, 'words': [0x11111111] * 8}]}
        data.append(('graphics.bg-main-graphics', 'bg-main-graphics', 1, graphics))
    if palette:
        entries = [{'banks': [[0, 0x001f, 0x03e0] + [0] * 13]} for _ in range(3)]
        data.append(('graphics.bg-base-palettes', 'bg-base-palettes', 2, {'entries': entries}))
    manifest = 'profile="test"\nextractor_revision=1\ndecoder_revision=1\n'
    for rid, name, version, contents in data:
        content = json.dumps(contents).encode(); (dataset / (name + '.json')).write_bytes(content)
        manifest += (f'[[resources]]\nid="{rid}"\npath="{name}.json"\nformat="{name}-v{version}-json"\n'
                     f'sha256="{hashlib.sha256(content).hexdigest()}"\n')
    (dataset / 'manifest.toml').write_text(manifest)
    return generate(dataset, work / 'view', split=split)


class TransferProjectionTest(unittest.TestCase):
    def test_validation_rejects_invalid_links_and_frame_extents(self):
        expected = catalog(); expected.pop('variant_sources')
        self.assertEqual(project_bg_transfers(catalog()), expected)
        mutations = [
            lambda c: c['variant_sources'].__setitem__(0, 1),
            lambda c: c['descriptor_links'][0][0].__setitem__('control', 3),
            lambda c: c['descriptor_links'][0][0].__setitem__('tiles', -1),
            lambda c: c['descriptor_links'][0][0].__setitem__('destination', 0x06018000),
            lambda c: c['controls'][0].__setitem__('transfer_bytes', 31),
            lambda c: c['controls'][0]['frames'][0].__setitem__('source_tile', 2),
            lambda c: c['controls'][0]['frames'][0].__setitem__('duration_ticks', True),
            lambda c: c['tiles'][0]['words'].__setitem__(0, 0x100000000),
            lambda c: c['descriptor_links'][1].pop(),
        ]
        for mutate in mutations:
            value = copy.deepcopy(catalog()); mutate(value)
            with self.subTest(mutation=mutate), self.assertRaises(ValueError):
                project_bg_transfers(value)

    def test_split_scope_and_invalid_resource(self):
        def payload(path):
            return json.JSONDecoder().raw_decode(path.read_text().split('const data = ', 1)[1])[0]
        with tempfile.TemporaryDirectory(prefix='viewer-transfer-scope-') as directory:
            index = fixture(Path(directory), split=True); parts = index.parent / (index.name + '.parts')
            self.assertNotIn('bg_transfers', payload(index))
            self.assertEqual(len(payload(parts / 'interface.html')['bg_transfers']['controls']), 3)
            self.assertNotIn('bg_transfers', payload(parts / 'audio.html'))
        with tempfile.TemporaryDirectory(prefix='viewer-transfer-invalid-') as directory:
            bad = catalog(); bad['controls'][0]['frames'][0]['source_tile'] = 20
            data = payload(fixture(Path(directory), value=bad))
            self.assertIsNone(data['bg_transfers'])
            self.assertTrue(data['resources'][0]['status'].startswith('Invalid background transfer data:'))


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class ViewerBgTransfersTest(unittest.TestCase):
    def test_frames_palette_loop_paging_links_and_focus(self):
        with tempfile.TemporaryDirectory(prefix='viewer-bg-transfers-') as directory:
            work = Path(directory); browser = Chrome(work, fixture(work))
            try:
                self.assertEqual(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change'));
                  const pixel=()=>[...q('#bg-transfer-sheet').getContext('2d').getImageData(0,0,1,1).data].join(',');
                  window.pending=new Map();let serial=0;
                  window.setTimeout=(fn,delay)=>{const id=++serial;pending.set(id,{fn,delay});return id};
                  window.clearTimeout=id=>pending.delete(id);
                  window.fireTransferTimer=()=>{const [id,item]=pending.entries().next().value;pending.delete(id);item.fn();return item.delay};
                  if(q('#bg-transfers').hidden||q('#bg-transfers>details').open||q('#bg-transfer-descriptor').options.length!==2)throw Error('disclosure/descriptors');
                  const row=[...q('#rows').rows].find(r=>r.dataset.resourceId==='graphics.bg-transfer-programs');
                  if(!row||!row.querySelector('a'))throw Error('resource link');row.querySelector('a').click();
                  if(!q('#bg-transfers>details').open||q('#bg-transfer-slot').options.length!==2||pixel()!=='255,0,0,255' ||
                     !q('#bg-transfer-detail').textContent.includes('header byte 0x7'))throw Error('initial frame');
                  q('#bg-transfer-frame').value='1';change(q('#bg-transfer-frame'));
                  if(pixel()!=='0,255,0,255'||q('#bg-transfer-progress').value!==2||!q('#bg-transfer-position').textContent.includes('duration 3'))throw Error('frame');
                  q('#bg-transfer-rate').value='10';q('#bg-transfer-loop').checked=true;q('#bg-transfer-play').click();
                  if(fireTransferTimer()!==300||q('#bg-transfer-frame').value!=='0'||pixel()!=='255,0,0,255')throw Error('timing/loop');
                  q('#bg-transfer-slot').value='2';change(q('#bg-transfer-slot'));
                  if(q('#bg-transfer-stop').disabled||!q('#bg-transfer-loop').checked||q('#bg-transfer-rate').value!=='10'||
                     !q('#bg-transfer-position').textContent.includes('65536-tick countdown'))throw Error('preserved state/zero duration');
                  if(fireTransferTimer()!==6553600||q('#bg-transfer-frame').value!=='0')throw Error('zero countdown');
                  q('#bg-transfer-descriptor').value='1';change(q('#bg-transfer-descriptor'));
                  q('#bg-transfer-page').value='1';change(q('#bg-transfer-page'));
                  if(q('#bg-transfer-stop').disabled||pixel()!=='255,0,0,255'||!q('#bg-transfer-detail').textContent.includes('tiles 128–128 of 129'))throw Error('large transfer page');
                  if(fireTransferTimer()!==100||pixel()!=='0,255,0,255')throw Error('paged frame');
                  q('#bg-transfer-loop').checked=false;fireTransferTimer();
                  if(!q('#bg-transfer-stop').disabled||!q('#bg-transfer-status').textContent.includes('finished'))throw Error('finish');
                  q('#bg-transfer-bg-open').click();
                  if(!q('#bg-tiles>details').open||q('#bg-descriptor').value!=='1'||q('#bg-palette').value!=='0')throw Error('background link');
                  q('#bg-descriptor').value='2';change(q('#bg-descriptor'));if(!q('#bg-transfer-navigation').hidden)throw Error('no transfers');
                  q('#bg-descriptor').value='0';change(q('#bg-descriptor'));q('#bg-transfer-open').click();
                  if(q('#bg-transfer-descriptor').value!=='0'||!q('#bg-transfers>details').open)throw Error('return link');
                  return 'PASS';
                })()'''), 'PASS')
                self.assertTrue(browser.evaluate('''(()=>{const s=document.querySelector('#bg-transfer-frame');s.focus();return document.activeElement===s})()'''))
                browser.click('#bg-transfer-play')
                self.assertEqual(browser.evaluate('document.activeElement.id'), 'bg-transfer-frame')
            finally:
                browser.close()

    def test_independent_grayscale_view(self):
        with tempfile.TemporaryDirectory(prefix='viewer-transfer-alone-') as directory:
            work = Path(directory); browser = Chrome(work, fixture(work, background=False, palette=False))
            try:
                self.assertEqual(browser.evaluate('''(()=>{const q=s=>document.querySelector(s);
                  const p=[...q('#bg-transfer-sheet').getContext('2d').getImageData(0,0,1,1).data].join(',');
                  return !q('#bg-transfers').hidden&&q('#bg-transfer-bg-navigation').hidden&&q('#bg-transfer-palette').value==='gray'&&p==='17,17,17,255';})()'''), True)
            finally:
                browser.close()
