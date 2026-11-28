"""Check source-bound linked records and their focused offline page."""
import copy
import hashlib
import json
import shutil
import struct
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import view_rom_data as viewer
from test_viewer_navbar_split_browser import wait_for
from test_viewer_scoped_browser import parse_page
from viewer_browser_harness import Chrome


def catalog():
    records = [{'alias_u16': index + 100, 'bytes_02_0f': [(index + field) % 256 for field in range(14)]}
               for index in range(128)]
    raw = b''.join(struct.pack('<H14B', item['alias_u16'], *item['bytes_02_0f']) for item in records)
    source = {'offset': 0x6e4c64, 'role': 'metadata16/eur-fixed', 'length': len(raw), 'sha256': hashlib.sha256(raw).hexdigest()}
    return {'source_offset': 0x6e4c64, 'records': records}, [source]


def fixture(work, *, profile='europe', invalid=False, trusted=True, split=True):
    dataset = work / 'dataset'; dataset.mkdir(); value, sources = catalog()
    content = json.dumps(value).encode(); digest = hashlib.sha256(content).hexdigest()
    (dataset / 'metadata.json').write_bytes(content)
    if invalid: sources[0]['sha256'] = '0' * 64
    item = sources[0]
    manifest = (f'profile="{profile}"\nextractor_revision=1\ndecoder_revision=1\n'
                f'[[resources]]\nid="gameplay.linked-metadata16"\npath="metadata.json"\n'
                f'format="linked-metadata16-v1-json"\nsha256="{digest}"\n'
                f'[[resources.sources]]\noffset={item["offset"]}\nlength={item["length"]}\n'
                f'role="{item["role"]}"\nsha256="{item["sha256"]}"\n')
    (dataset / 'manifest.toml').write_text(manifest)
    with patch.object(viewer, 'EUROPE_METADATA16_SHA256', digest if trusted else '0' * 64):
        return viewer.generate(dataset, work / 'view', split=split)


class LinkedMetadata16ProjectionTest(unittest.TestCase):
    def test_source_validation_and_profile(self):
        value, sources = catalog(); content = json.dumps(value).encode(); digest = hashlib.sha256(content).hexdigest()
        with patch.object(viewer, 'EUROPE_METADATA16_SHA256', digest):
            project = viewer.project_metadata16
            result = project('europe', digest, value, sources)
            self.assertEqual(len(result['records']), 128); self.assertEqual(result['records'][127]['alias'], 227)
            self.assertEqual(result['records'][4]['bytes'], value['records'][4]['bytes_02_0f'])
            self.assertIsNone(project('usa', digest, value, sources)); self.assertIsNone(project('europe', 'wrong', value, sources))
            mutations = [lambda c: c['records'].pop(), lambda c: c.update(source_offset=0),
                         lambda c: c['records'][0].update(alias_u16=True),
                         lambda c: c['records'][0]['bytes_02_0f'].__setitem__(0, 256),
                         lambda c: c['records'][0]['bytes_02_0f'].__setitem__(0, 11)]
            for mutate in mutations:
                bad = copy.deepcopy(value); mutate(bad)
                with self.assertRaises(ValueError): project('europe', digest, bad, sources)
            for bad in (None, [True], [], [{**sources[0], 'offset': 0}], [{**sources[0], 'sha256': '0' * 64}]):
                with self.assertRaises(ValueError): project('europe', digest, value, bad)

    def test_page_scoping_and_invalid_source(self):
        for options in ({}, {'invalid': True}, {'profile': 'usa'}, {'trusted': False}):
            with self.subTest(options=options), tempfile.TemporaryDirectory(prefix='viewer-metadata16-') as directory:
                page = fixture(Path(directory), **options); parts = page.parent / 'index.html.parts'
                home, world, focused = map(parse_page, (page, parts/'world.html', parts/'world-metadata16.html'))
                self.assertNotIn('metadata16', home); self.assertNotIn('metadata16', world)
                row = next(item for item in home['resources'] if item['id'] == 'gameplay.linked-metadata16')
                self.assertEqual(row['viewer_scope'], 'world-metadata16')
                if options:
                    self.assertIsNone(focused['metadata16'])
                    self.assertTrue(row['status'].startswith(('Invalid linked metadata', 'Preview unavailable')))
                else:
                    self.assertEqual(len(focused['metadata16']['records']), 128)
                    self.assertEqual(row['status'], 'Available: 128 linked metadata records')


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class LinkedMetadata16BrowserTest(unittest.TestCase):
    def test_table_record_navigation_and_bookmark(self):
        with tempfile.TemporaryDirectory(prefix='viewer-metadata16-browser-') as directory:
            work = Path(directory); page = fixture(work); browser = Chrome(work, page)
            try:
                target = browser.evaluate('''(()=>{const q=s=>document.querySelector(s);
                  const link=q('[data-resource-id="gameplay.linked-metadata16"] a');
                  if(!link||!link.href.includes('world-metadata16.html'))throw Error('resource route');return link.href;})()''')
                browser.command('Page.navigate', {'url': target})
                wait_for(browser, "data.wiki.current==='world-metadata16' && !document.querySelector('#metadata16').hidden")
                self.assertTrue(browser.evaluate('''(()=>{const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change',{bubbles:true}));
                  if(q('#metadata16>details>summary time').dateTime!=='2026-11-27T22:55:45Z'||
                     q('#metadata16-all>summary time').dateTime!=='2026-11-27T22:55:45Z'||q('#metadata16-list').rows.length!==128||
                     q('#metadata16-fields').rows.length!==14||!q('#metadata16-display').textContent.includes('index: 0'))throw Error('initial');
                  q('#metadata16-record').value='127';change(q('#metadata16-record'));
                  if(!q('#metadata16-alias').textContent.includes('227')||!q('#metadata16-source').textContent.includes('0x6E5454')||
                     !q('#metadata16-display').textContent.includes('index: 521')||!q('#metadata16-fields').rows[5].textContent.includes('Bit test'))throw Error('last record');
                  q('#metadata16-list').rows[4].querySelector('button').click();
                  return q('#metadata16-record').value==='4'&&q('#metadata16-display').textContent.includes('index: 398');})()'''))
                saved = browser.evaluate("document.querySelector('#metadata16>details a').href")
                browser.command('Page.navigate', {'url': saved})
                wait_for(browser, "data.wiki.current==='world-metadata16' && document.querySelector('#metadata16-record').value==='4'")
                self.assertTrue(browser.evaluate("document.querySelector('#metadata16>details').open"))
            finally:
                browser.close()

    def test_legacy_single_page(self):
        with tempfile.TemporaryDirectory(prefix='viewer-metadata16-single-') as directory:
            work = Path(directory); page = fixture(work, split=False); browser = Chrome(work, page)
            try:
                self.assertTrue(browser.evaluate("!document.querySelector('#metadata16').hidden && document.querySelector('#metadata16-list').rows.length===128"))
            finally:
                browser.close()


if __name__ == '__main__':
    unittest.main()
