"""Check byte-pair lookup semantics, source guards and metadata navigation."""
import copy
import hashlib
import json
import shutil
import sys
import tempfile
import unittest
from contextlib import ExitStack
from pathlib import Path
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import view_rom_data
from test_viewer_job_names_browser import fixture as job_fixture, payload
from viewer_browser_harness import Chrome


def pair_catalog():
    records = [{'key': index, 'value': index + 10} for index in range(44)]
    records[5]['value'] = records[6]['value'] = 200
    records[43]['value'] = 250
    return {'source_offset': 0x6e6c44, 'records': records}


def fixture(work, *, with_metadata=True, with_text=True, wrong_identity=False, split=True):
    dataset, _, metadata = job_fixture(work); manifest = dataset / 'manifest.toml'; text = manifest.read_text()
    if not with_metadata or not with_text:
        excluded = ({'gameplay.linked-metadata52'} if not with_metadata else set()) | ({'text.catalogs'} if not with_text else set())
        chunks = text.split('[[resources]]\n')
        text = chunks[0] + ''.join('[[resources]]\n' + chunk for chunk in chunks[1:]
                                  if not any(chunk.startswith('id="' + identity + '"\n') for identity in excluded))
    content = json.dumps(pair_catalog()).encode(); (dataset / 'metadata2.json').write_bytes(content)
    pair_sha = hashlib.sha256(content).hexdigest()
    metadata_sha = hashlib.sha256((dataset / 'linked-metadata52.json').read_bytes()).hexdigest()
    text_sha = hashlib.sha256((dataset / 'text-catalogs.json').read_bytes()).hexdigest()
    text += (f'[[resources]]\nid="gameplay.linked-metadata2"\npath="metadata2.json"\n'
             f'format="linked-metadata2-v1-json"\nsha256="{pair_sha}"\n')
    manifest.write_text(text)
    with ExitStack() as stack:
        stack.enter_context(patch.object(view_rom_data, 'EUROPE_METADATA2_SHA256', 'wrong' if wrong_identity else pair_sha))
        stack.enter_context(patch.object(view_rom_data, 'EUROPE_METADATA52_SHA256', metadata_sha))
        stack.enter_context(patch.object(view_rom_data, 'EUROPE_TEXT_CATALOG_SHA256', text_sha))
        page = view_rom_data.generate(dataset, work / 'view', split=split)
    return page, (pair_sha, metadata_sha, text_sha), metadata


class Metadata2ProjectionTest(unittest.TestCase):
    def test_source_shapes_identity_and_missing_dependencies(self):
        projected = view_rom_data.project_metadata2(pair_catalog())
        self.assertEqual(projected['records'][43], {'key': 43, 'value': 250})
        for mutate in (lambda d: d['records'].pop(), lambda d: d.__setitem__('source_offset', True),
                       lambda d: d['records'][0].__setitem__('key', True),
                       lambda d: d['records'][0].__setitem__('value', 256)):
            bad = copy.deepcopy(pair_catalog()); mutate(bad)
            with self.subTest(bad=bad), self.assertRaises(ValueError): view_rom_data.project_metadata2(bad)
        for options in ({}, {'with_text':False}, {'with_metadata':False}, {'wrong_identity':True}):
            with self.subTest(options=options), tempfile.TemporaryDirectory() as directory:
                page, hashes, metadata = fixture(Path(directory), **options); parts = page.parent / 'index.html.parts'
                world, battle = map(payload, (parts / 'world.html', parts / 'battle.html'))
                self.assertEqual(len(world['metadata2']['records']), 44)
                self.assertEqual(world['metadata2']['reader_verified'], not options.get('wrong_identity', False))
                self.assertNotIn('metadata2', battle)
                links = world['metadata2_links']
                if options.get('with_metadata') is False or options.get('wrong_identity'):
                    self.assertIsNone(links)
                else:
                    self.assertEqual(links['names_verified'], options.get('with_text', True))
                meta = view_rom_data.project_job_metadata(metadata)
                with patch.object(view_rom_data, 'EUROPE_METADATA2_SHA256', hashes[0]), \
                     patch.object(view_rom_data, 'EUROPE_METADATA52_SHA256', hashes[1]):
                    pairs = copy.deepcopy(world['metadata2']); pairs['records'][0]['key'] = 116
                    self.assertIsNone(view_rom_data.link_metadata2('europe', hashes[0], hashes[1], hashes[2], pairs, meta, None))
                    self.assertIsNone(view_rom_data.link_metadata2('usa', hashes[0], hashes[1], hashes[2], world['metadata2'], meta, None))


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class Metadata2BrowserTest(unittest.TestCase):
    def test_lookup_duplicates_final_row_and_bidirectional_navigation(self):
        with tempfile.TemporaryDirectory(prefix='viewer-metadata2-browser-') as directory:
            work = Path(directory); page, _, _ = fixture(work); browser = Chrome(work, page.parent / 'index.html.parts/world.html')
            try:
                self.assertTrue(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s),input=e=>e.dispatchEvent(new Event('input',{bubbles:true}));
                  if(q('#metadata2').hidden || q('#metadata2-rows').rows.length!==44 || q('#metadata2-calculator').hidden ||
                     !q('[data-resource-id="gameplay.linked-metadata2"] a[href="#metadata2"]'))throw Error('view');
                  const last=q('#metadata2-rows tr[data-pair-row="43"]');
                  if(last.cells[3].textContent!=='250' || last.cells[4].textContent!=='Excluded')throw Error('last row');
                  q('#metadata2-forward').value='43';input(q('#metadata2-forward'));
                  q('#metadata2-inverse').value='250';input(q('#metadata2-inverse'));
                  if(!q('#metadata2-forward-result').textContent.includes('Value 250') ||
                     !q('#metadata2-inverse-result').textContent.includes('No match') ||
                     !q('#metadata2-inverse-result').textContent.includes('key 0'))throw Error('asymmetric lookup');
                  q('#metadata2-inverse').value='200';input(q('#metadata2-inverse'));
                  if(!q('#metadata2-inverse-result').textContent.includes('row 5') ||
                     !q('#metadata2-inverse-result').textContent.includes('key 5'))throw Error('first duplicate');
                  q('#metadata2-inverse').value='';input(q('#metadata2-inverse'));
                  if(!q('#metadata2-inverse-result').textContent.includes('0 to 255'))throw Error('empty guard');
                  q('#metadata2-rows tr[data-pair-row="1"] a').click();
                  if(q('#metadata-name-rows tr').dataset.metadataId!=='1' || !q('#metadata-names>details').open)throw Error('metadata link');
                  const reverse=q('#metadata-name-rows [data-metadata2-key="1"]');if(!reverse)throw Error('reverse missing');reverse.click();
                  return q('#metadata2>details').open && q('#metadata2-filter-mode').value==='exact-key' &&
                    q('#metadata2-rows').rows.length===1 && q('#metadata2-rows tr').dataset.pairRow==='1';
                })()'''))
            finally: browser.close()

    def test_raw_view_without_verified_links(self):
        for options in ({'with_text':False}, {'with_metadata':False}, {'wrong_identity':True}):
            with self.subTest(options=options), tempfile.TemporaryDirectory(prefix='viewer-metadata2-raw-') as directory:
                work = Path(directory); page, _, _ = fixture(work, split=False, **options); browser = Chrome(work, page)
                try:
                    state = browser.evaluate('''(()=>{const q=s=>document.querySelector(s);return {
                      hidden:q('#metadata2').hidden, rows:q('#metadata2-rows').rows.length,
                      link:!!q('#metadata2-rows a'), label:q('#metadata2-rows tr:nth-child(2) td:nth-child(3)').textContent,
                      calculator:q('#metadata2-calculator').hidden};})()''')
                    self.assertFalse(state['hidden']); self.assertEqual(state['rows'], 44)
                    self.assertEqual(state['link'], options.get('with_text') is False)
                    self.assertEqual(state['label'], '1')
                    self.assertEqual(state['calculator'], options.get('wrong_identity', False))
                finally: browser.close()
