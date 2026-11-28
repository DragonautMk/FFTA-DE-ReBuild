"""Check the source table and local CRC32 calculator without save files."""
import copy
import hashlib
import json
import shutil
import struct
import tempfile
import unittest
import zlib
from pathlib import Path
from unittest.mock import patch

import view_rom_data as viewer
from test_viewer_navbar_split_browser import wait_for
from test_viewer_scoped_browser import parse_page
from viewer_browser_harness import Chrome


def source(offset, role, raw):
    return {'offset': offset, 'role': role, 'length': len(raw), 'sha256': hashlib.sha256(raw).hexdigest()}


def contents():
    words = []
    for index in range(256):
        value = index
        for _ in range(8): value = (value >> 1) ^ (0xedb88320 if value & 1 else 0)
        words.append(value)
    return [(0x13ef1c, 'reader_literals', struct.pack('<2I', 0x00ffffff, 0x08397180)),
            (0x397180, 'checksum_table', struct.pack('<256I', *words))]


def fixture(work, *, invalid=False, trusted=True, profile='europe', split=True):
    pieces = contents(); packed = b''.join(item[2] for item in pieces); dataset = work/'dataset'; dataset.mkdir()
    (dataset/'crc.bin').write_bytes(packed); bindings = [source(*item) for item in pieces]
    if invalid: bindings[1]['sha256'] = '0' * 64
    manifest = (f'profile="{profile}"\nextractor_revision=375\ndecoder_revision=374\n'
                f'[[resources]]\nid="engine.save-crc32-table"\npath="crc.bin"\nformat="save-crc32-table-v1-bin"\n'
                f'sha256="{hashlib.sha256(packed).hexdigest()}"\n')
    for item in bindings:
        manifest += (f'[[resources.sources]]\noffset={item["offset"]}\nlength={item["length"]}\n'
                     f'role="{item["role"]}"\nsha256="{item["sha256"]}"\n')
    (dataset/'manifest.toml').write_text(manifest)
    with patch.object(viewer, 'EUROPE_SAVE_CRC32_SHA256', hashlib.sha256(packed).hexdigest() if trusted else '0' * 64):
        return viewer.generate(dataset, work/'view', split=split)


class CRC32ProjectionTest(unittest.TestCase):
    def test_literal_extent_hash_and_polynomial(self):
        pieces = contents(); raw = b''.join(item[2] for item in pieces); digest = hashlib.sha256(raw).hexdigest()
        sources = [source(*item) for item in pieces]
        with patch.object(viewer, 'EUROPE_SAVE_CRC32_SHA256', digest):
            value = viewer.project_save_crc32('europe', digest, raw, sources)
            self.assertEqual(len(value['values']), 256); self.assertEqual(value['values'][1], 0x77073096)
            self.assertIsNone(viewer.project_save_crc32('usa', digest, raw, sources))
            for mutate in (lambda s: s.pop(), lambda s: s[0].update(offset=0), lambda s: s[1].update(sha256='0' * 64)):
                bad = copy.deepcopy(sources); mutate(bad)
                with self.assertRaises(ValueError): viewer.project_save_crc32('europe', digest, raw, bad)
            for index in (0, 1):
                bad = copy.deepcopy(pieces); payload = bytearray(bad[index][2]); payload[0] ^= 1
                bad[index] = (bad[index][0], bad[index][1], bytes(payload)); altered = b''.join(item[2] for item in bad)
                with patch.object(viewer, 'EUROPE_SAVE_CRC32_SHA256', hashlib.sha256(altered).hexdigest()):
                    with self.assertRaises(ValueError): viewer.project_save_crc32('europe', hashlib.sha256(altered).hexdigest(), altered, [source(*item) for item in bad])

    def test_scoped_and_invalid_data(self):
        for options in ({}, {'invalid': True}, {'trusted': False}, {'profile': 'usa'}):
            with self.subTest(options=options), tempfile.TemporaryDirectory(prefix='viewer-crc32-') as directory:
                page = fixture(Path(directory), **options); parts = page.parent/'index.html.parts'
                home, world, scope = map(parse_page, (page, parts/'world.html', parts/'checksums.html'))
                self.assertNotIn('save_crc32_table', home); self.assertNotIn('save_crc32_table', world)
                row = next(item for item in home['resources'] if item['id'] == 'engine.save-crc32-table')
                self.assertEqual(row['viewer_scope'], 'checksums')
                if options: self.assertIsNone(scope['save_crc32_table'])
                else: self.assertEqual(len(scope['save_crc32_table']['values']), 256)


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class CRC32BrowserTest(unittest.TestCase):
    def test_text_hex_seed_file_chunks_cancel_and_bookmark(self):
        with tempfile.TemporaryDirectory(prefix='viewer-crc32-browser-') as directory:
            work = Path(directory); page = fixture(work); browser = Chrome(work, page)
            data = bytes((i * 73 + 19) & 255 for i in range(200000)); expected = zlib.crc32(data, 0x12345678)
            try:
                target = browser.evaluate("document.querySelector('[data-resource-id=\"engine.save-crc32-table\"] a').href")
                browser.command('Page.navigate', {'url': target})
                wait_for(browser, "data.wiki.current==='checksums' && !document.querySelector('#crc32-table').hidden")
                js = '''(async()=>{const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change',{bubbles:true}));
                  const wait=async f=>{const end=Date.now()+5000;while(!f()){if(Date.now()>end)throw Error('timeout');await new Promise(r=>setTimeout(r,10))}};
                  if(q('#crc32-table>details>summary time').dateTime!=='2026-11-27T23:37:22Z'||q('#crc32-rows').rows.length!==16||
                     q('#crc32-calculator>summary time').dateTime!=='2026-11-27T23:37:22Z')throw Error('source or badges');
                  q('#crc32-page').value='15';change(q('#crc32-page'));
                  if(!q('#crc32-rows').rows[15].textContent.includes('0x39757C'))throw Error('last word');
                  q('#crc32-input').value='123456789';q('#crc32-calculate').click();
                  if(!q('#crc32-result').textContent.includes('0xCBF43926'))throw Error('text CRC');
                  q('#crc32-mode').value='hex';q('#crc32-input').value='31 32 33 34 35 36 37 38 39';q('#crc32-calculate').click();
                  if(!q('#crc32-result').textContent.includes('0xCBF43926'))throw Error('hex CRC');
                  q('#crc32-input').value='F';q('#crc32-calculate').click();if(!q('#crc32-result').textContent.includes('byte pairs'))throw Error('hex guard');
                  q('#crc32-seed').value='0x100000000';q('#crc32-calculate').click();if(!q('#crc32-result').textContent.includes('initial CRC'))throw Error('seed guard');
                  q('#crc32-seed').value='0x12345678';const bytes=new Uint8Array(200000);for(let i=0;i<bytes.length;i++)bytes[i]=(i*73+19)&255;
                  const transfer=new DataTransfer();transfer.items.add(new File([bytes],'synthetic.bin'));q('#crc32-file').files=transfer.files;
                  q('#crc32-file-calculate').click();await wait(()=>q('#crc32-cancel').disabled);
                  if(!q('#crc32-result').textContent.includes('0xEXPECTED')||!q('#crc32-result').textContent.includes('200000 bytes'))throw Error('chunked file CRC');
                  q('#crc32-file-calculate').click();q('#crc32-cancel').click();await new Promise(r=>setTimeout(r,30));
                  if(!q('#crc32-result').textContent.includes('cancelled')||q('#crc32-file-calculate').disabled)throw Error('cancel');
                  return true;})()'''.replace('EXPECTED', f'{expected:08X}')
                self.assertTrue(browser.evaluate(js))
                saved = browser.evaluate("document.querySelector('#crc32-table>details a').href")
                browser.command('Page.navigate', {'url': saved})
                wait_for(browser, "data.wiki.current==='checksums' && document.querySelector('#crc32-page').value==='15'")
                self.assertTrue(browser.evaluate("document.querySelector('#crc32-table>details').open"))
            finally:
                browser.close()

    def test_single_page_local_calculation(self):
        with tempfile.TemporaryDirectory(prefix='viewer-crc32-single-') as directory:
            work = Path(directory); browser = Chrome(work, fixture(work, split=False))
            try:
                self.assertTrue(browser.evaluate('''(()=>{const q=s=>document.querySelector(s);q('#crc32-input').value='123456789';
                  q('#crc32-calculate').click();return !q('#crc32-table').hidden&&q('#crc32-result').textContent.includes('0xCBF43926');})()'''))
            finally:
                browser.close()


if __name__ == '__main__':
    unittest.main()
