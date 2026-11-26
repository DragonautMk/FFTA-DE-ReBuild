"""Verify guarded links between counted item pairs and metadata references."""
import hashlib
import json
import shutil
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import view_rom_data
from test_viewer_job_names_browser import fixture as job_fixture, payload
from viewer_browser_harness import Chrome


def fixture(work, *, with_text=True, with_links=True, with_metadata=True, wrong_identity=False, split=True):
    dataset, _, _ = job_fixture(work)
    slots = [{'selector': selector, 'value': index + 1} for index, selector in enumerate([0, 1, 255, 2, 1, 0, 0, 0, 0])]
    links = {'source_offset': 0x1234, 'records': [
        {'count': 4, 'opaque': 0, 'slots': slots},
        {'count': 2, 'opaque': 0, 'slots': [{'selector': 1, 'value': 8}, {'selector': 1, 'value': 9}] + slots[2:]},
    ]}
    content = json.dumps(links).encode(); (dataset / 'item-links.json').write_bytes(content)
    link_sha = hashlib.sha256(content).hexdigest()
    manifest = dataset / 'manifest.toml'; text = manifest.read_text()
    if not with_text or not with_metadata:
        chunks = text.split('[[resources]]\n')
        excluded = ({'text.catalogs'} if not with_text else set()) | ({'gameplay.linked-metadata52'} if not with_metadata else set())
        text = chunks[0] + ''.join('[[resources]]\n' + chunk for chunk in chunks[1:]
                                   if not any(chunk.startswith('id="' + identity + '"\n') for identity in excluded))
    if with_links:
        text += (f'[[resources]]\nid="gameplay.item-links"\npath="item-links.json"\n'
                 f'format="item-links-v1-json"\nsha256="{link_sha}"\n')
    manifest.write_text(text)
    metadata_sha = hashlib.sha256((dataset / 'linked-metadata52.json').read_bytes()).hexdigest()
    text_sha = hashlib.sha256((dataset / 'text-catalogs.json').read_bytes()).hexdigest()
    with patch.object(view_rom_data, 'EUROPE_ITEM_LINKS_SHA256', 'wrong' if wrong_identity else link_sha), \
         patch.object(view_rom_data, 'EUROPE_METADATA52_SHA256', metadata_sha), \
         patch.object(view_rom_data, 'EUROPE_TEXT_CATALOG_SHA256', text_sha):
        page = view_rom_data.generate(dataset, work / 'view', split=split)
    return page, (link_sha, metadata_sha, text_sha)


class ItemMetadataProjectionTest(unittest.TestCase):
    def test_identity_and_partial_inputs(self):
        with tempfile.TemporaryDirectory() as directory:
            page, hashes = fixture(Path(directory)); world = payload(page.parent / 'index.html.parts/world.html')
            self.assertEqual(world['item_metadata_links'], {'names_verified': True})
            self.assertNotIn('item_metadata_links', payload(page.parent / 'index.html.parts/audio.html'))
            with patch.object(view_rom_data, 'EUROPE_ITEM_LINKS_SHA256', hashes[0]), \
                 patch.object(view_rom_data, 'EUROPE_METADATA52_SHA256', hashes[1]), \
                 patch.object(view_rom_data, 'EUROPE_TEXT_CATALOG_SHA256', hashes[2]):
                project = view_rom_data.project_item_metadata_links
                args = ('europe', *hashes, world['item_links'], world['metadata_names'])
                self.assertEqual(project(*args), {'names_verified': True})
                self.assertEqual(project('europe', hashes[0], hashes[1], None, world['item_links'],
                                         {'reader_index': None}), {'names_verified': False})
                for bad in (('usa', hashes[0], hashes[1], hashes[2], world['item_links'], world['metadata_names']),
                            ('europe', 'wrong', hashes[1], hashes[2], world['item_links'], world['metadata_names']),
                            ('europe', hashes[0], 'wrong', hashes[2], world['item_links'], world['metadata_names']),
                            ('europe', hashes[0], hashes[1], 'wrong', world['item_links'], world['metadata_names']),
                            ('europe', hashes[0], hashes[1], hashes[2], None, world['metadata_names']),
                            ('europe', hashes[0], hashes[1], hashes[2], world['item_links'], None)):
                    self.assertIsNone(project(*bad))
        for kwargs in ({'with_text': False}, {'with_links': False}, {'with_metadata': False}, {'wrong_identity': True}):
            with self.subTest(kwargs=kwargs), tempfile.TemporaryDirectory() as directory:
                page, _ = fixture(Path(directory), split=False, **kwargs); data = payload(page)
                if not kwargs.get('with_text', True):
                    self.assertEqual(data['item_metadata_links'], {'names_verified': False})
                else:
                    self.assertIsNone(data['item_metadata_links'])


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class ItemMetadataBrowserTest(unittest.TestCase):
    def test_active_references_reverse_links_and_sentinels(self):
        with tempfile.TemporaryDirectory() as directory:
            work = Path(directory); page, _ = fixture(work)
            browser = Chrome(work, page.parent / 'index.html.parts/world.html')
            try:
                self.assertEqual(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change'));
                  let rows=q('#item-link-slots').rows;
                  if(rows.length!==9 || rows[0].cells[2].querySelector('a') || !rows[1].cells[2].querySelector('a') ||
                     rows[2].cells[2].querySelector('a') || !rows[3].cells[2].querySelector('a') ||
                     rows[4].cells[2].querySelector('a'))throw Error('active and special slots');
                  if(!rows[1].cells[2].textContent.includes('d'))throw Error('localized label');
                  q('#metadata-name-language').value='1';change(q('#metadata-name-language'));
                  rows=q('#item-link-slots').rows;
                  if(!rows[1].cells[2].textContent.includes('Ab d'))throw Error('language update');
                  rows[1].cells[2].querySelector('a').click();
                  if(!q('#metadata-names>details').open || q('#metadata-name-filter-mode').value!=='exact-id' ||
                     q('#metadata-name-rows').rows.length!==1 || q('#metadata-name-rows tr').dataset.metadataId!=='1')throw Error('forward');
                  const details=q('#metadata-name-rows tr details');
                  if(!details || !details.querySelector('summary').textContent.includes('3 matching slots') ||
                     details.querySelectorAll('a').length!==3)throw Error('reverse list');
                  details.querySelectorAll('a')[1].click();
                  if(q('#item-link-record').value!=='1' || !q('#item-links>details').open)throw Error('reverse navigation');
                  q('#metadata-name-filter-mode').value='contains';change(q('#metadata-name-filter-mode'));
                  q('#metadata-name-filter').value='1';q('#metadata-name-filter').dispatchEvent(new Event('input'));
                  if(q('#metadata-name-rows').rows.length<=1)throw Error('ordinary filtering');
                  return 'PASS';
                })()'''), 'PASS')
            finally:
                browser.close()

    def test_missing_text_and_unsupported_sources(self):
        for kwargs in ({'with_text': False}, {'with_metadata': False}, {'wrong_identity': True}, {'with_links': False}):
            with self.subTest(kwargs=kwargs), tempfile.TemporaryDirectory() as directory:
                work = Path(directory); page, _ = fixture(work, split=False, **kwargs); browser = Chrome(work, page)
                try:
                    result = browser.evaluate('''(()=>{
                      const q=s=>document.querySelector(s),link=q('#item-link-slots tr:nth-child(2) td:nth-child(3) a');
                      return {itemHidden:q('#item-links').hidden, metadataHidden:q('#metadata-names').hidden,
                        link:link ? link.textContent : null};
                    })()''')
                    if not kwargs.get('with_text', True):
                        self.assertEqual(result['link'], '1')
                    elif not kwargs.get('with_links', True):
                        self.assertTrue(result['itemHidden']); self.assertFalse(result['metadataHidden'])
                    else:
                        self.assertIsNone(result['link'])
                finally:
                    browser.close()
