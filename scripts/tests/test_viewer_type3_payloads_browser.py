"""Check isolated type-3 compressed and raw transfer tile previews."""
import copy
import hashlib
import json
import shutil
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from view_rom_data import generate, project_type3_payloads
from viewer_browser_harness import Chrome


def literal_lz(data):
    return bytes([0x10]) + len(data).to_bytes(3, 'little') + b''.join(
        bytes([0]) + data[index:index + 8] for index in range(0, len(data), 8))


def segments():
    return [(0x1000, 'transfer-row', bytes(12)),
            (0x2000, 'lz77-input', literal_lz(bytes([0x21]) * 32)),
            (0x3000, 'lz77-input', bytes.fromhex('10040000800000')),
            (0x4000, 'raw-input', bytes([0x43]) + bytes([0x65]) * 64)]


def fixture(work, *, bad_hash=False, missing=False):
    dataset = work / 'dataset'; dataset.mkdir(); parts = segments()
    content = b''.join(chunk for _, _, chunk in parts)
    if not missing: (dataset / 'type3.bin').write_bytes(content)
    manifest = ('profile="europe"\nextractor_revision=1\ndecoder_revision=1\n'
                '[[resources]]\nid="graphics.tilemap-type3-transfers"\npath="type3.bin"\n'
                f'format="tilemap-type3-transfers-v1-bin"\nsha256="{hashlib.sha256(content).hexdigest()}"\n')
    for index, (offset, role, chunk) in enumerate(parts):
        digest = '0' * 64 if bad_hash and index == 0 else hashlib.sha256(chunk).hexdigest()
        manifest += (f'[[resources.sources]]\noffset={offset}\nlength={len(chunk)}\n'
                     f'role="{role}"\nsha256="{digest}"\n')
    (dataset / 'manifest.toml').write_text(manifest)
    return generate(dataset, work / 'view', split=True)


def payload(path):
    return json.JSONDecoder().raw_decode(path.read_text().split('const data = ', 1)[1])[0]


class Type3PayloadProjectionTest(unittest.TestCase):
    def test_valid_outputs_and_invalid_stream_isolation(self):
        parts = segments(); content = b''.join(chunk for _, _, chunk in parts)
        sources = [{'offset': offset, 'role': role, 'length': len(chunk), 'sha256': hashlib.sha256(chunk).hexdigest()}
                   for offset, role, chunk in parts]
        result = project_type3_payloads(content, sources)
        self.assertEqual(len(result['streams']), 1); self.assertEqual(result['streams'][0]['words'], [0x21212121] * 8)
        self.assertEqual(len(result['raw_streams']), 1); self.assertEqual(result['raw_streams'][0]['decoded_length'], 65)
        self.assertEqual(result['errors'][0]['source_offset'], 0x3000)
        bad = copy.deepcopy(sources); bad[0]['sha256'] = '0' * 64
        with self.assertRaises(ValueError): project_type3_payloads(content, bad)
        bad = copy.deepcopy(sources); bad[1]['offset'] = 0x1001
        with self.assertRaises(ValueError): project_type3_payloads(content, bad)
        with self.assertRaises(ValueError): project_type3_payloads(content + b'\x00', sources)

    def test_scope_missing_and_bad_source_hash(self):
        for options in ({}, {'bad_hash': True}, {'missing': True}):
            with self.subTest(options=options), tempfile.TemporaryDirectory(prefix='viewer-type3-projection-') as directory:
                index = fixture(Path(directory), **options); parts = index.parent / 'index.html.parts'
                home, world, page = payload(index), payload(parts / 'world.html'), payload(parts / 'transfer-graphics.html')
                self.assertNotIn('type3_payloads', home); self.assertNotIn('type3_payloads', world)
                self.assertEqual(home['resources'][0]['viewer_scope'], 'transfer-graphics')
                self.assertEqual(bool(page['type3_payloads']), not options)
                if not options:
                    self.assertIn('1 invalid LZ77', page['resources'][0]['status'])


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class Type3PayloadBrowserTest(unittest.TestCase):
    def test_preview_depth_offsets_and_invalid_notice(self):
        with tempfile.TemporaryDirectory(prefix='viewer-type3-browser-') as directory:
            work = Path(directory); index = fixture(work); browser = Chrome(work, index.parent / 'index.html.parts/transfer-graphics.html')
            try:
                self.assertEqual(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change')),
                    input=e=>e.dispatchEvent(new Event('input')),
                    pixel=()=>[...q('#obj-sheet').getContext('2d').getImageData(0,0,1,1).data];
                  q('[data-resource-id="graphics.tilemap-type3-transfers"] a[href="#obj-tiles"]').click();
                  if(!q('#obj-tiles>details').open||q('#obj-catalog').value!=='lz'||q('#obj-bank').options.length!==1||
                     pixel().join(',')!=='17,17,17,255'||!q('#obj-input-errors').textContent.includes('0x3000'))throw Error('LZ preview');
                  q('#obj-catalog').value='raw';change(q('#obj-catalog'));
                  if(q('#obj-bank').options.length!==1||pixel().join(',')!=='51,51,51,255'||
                     !q('#obj-detail').textContent.includes('65 raw bytes'))throw Error('raw preview');
                  q('#obj-depth').value='8';change(q('#obj-depth'));
                  if(pixel().join(',')!=='67,67,67,255')throw Error('8-bit');
                  q('#obj-lz-offset').value='1';input(q('#obj-lz-offset'));
                  if(pixel().join(',')!=='101,101,101,255')throw Error('offset');
                  q('#obj-lz-offset').value='66';input(q('#obj-lz-offset'));
                  if(!q('#obj-detail').textContent.includes('Choose a raw byte offset'))throw Error('invalid offset');
                  return 'PASS';
                })()'''), 'PASS')
            finally: browser.close()

    def test_missing_or_invalid_resource_leaves_preview_hidden(self):
        for options in ({'bad_hash': True}, {'missing': True}):
            with self.subTest(options=options), tempfile.TemporaryDirectory(prefix='viewer-type3-missing-') as directory:
                work = Path(directory); index = fixture(work, **options); browser = Chrome(work, index.parent / 'index.html.parts/transfer-graphics.html')
                try: self.assertTrue(browser.evaluate("document.querySelector('#obj-tiles').hidden"))
                finally: browser.close()
