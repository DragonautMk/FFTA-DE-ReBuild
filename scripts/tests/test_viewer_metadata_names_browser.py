"""Check metadata name references without asserting live job identities."""
import copy
import hashlib
import json
import shutil
import tempfile
import unittest
from pathlib import Path

from test_viewer_job_names_browser import fixture, payload
from view_rom_data import generate, project_job_metadata, project_metadata_name_references
from viewer_browser_harness import Chrome


def revise_metadata(dataset, edit):
    path = dataset / 'linked-metadata52.json'
    old = path.read_bytes()
    value = json.loads(old)
    edit(value['records'])
    new = json.dumps(value).encode()
    path.write_bytes(new)
    manifest = dataset / 'manifest.toml'
    manifest.write_text(manifest.read_text().replace(hashlib.sha256(old).hexdigest(), hashlib.sha256(new).hexdigest()))


def remove_resources(dataset, identifiers):
    manifest = dataset / 'manifest.toml'
    chunks = manifest.read_text().split('[[resources]]\n')
    manifest.write_text(chunks[0] + ''.join('[[resources]]\n' + part for part in chunks[1:]
                                           if not any(part.startswith('id="' + item + '"\n') for item in identifiers)))


class MetadataNameProjectionTest(unittest.TestCase):
    def test_direct_names_and_unresolved_sources(self):
        with tempfile.TemporaryDirectory(prefix='viewer-metadata-projection-') as directory:
            work = Path(directory)
            dataset, _, metadata = fixture(work)
            result = generate(dataset, work / 'view', split=True)
            pointer = payload(result.parent / 'index.html.parts/text.html')['pointer_texts']
            metadata = project_job_metadata(metadata)
            metadata['path'] = 'metadata.json'
            metadata['records'][1]['routing'] = 255
            metadata['records'][2]['bytes_00_04'][:2] = [255, 255]
            projected = project_metadata_name_references(metadata, pointer)
            self.assertEqual(len(projected['rows']), 116)
            self.assertEqual(projected['rows'][0]['names'], [[176, 203, [64], 205, 1000], [205]])
            self.assertIsNone(projected['rows'][1]['name_index'])
            self.assertEqual(projected['rows'][1]['names'], [None, None])
            self.assertEqual(projected['rows'][2]['name_index'], 65535)
            self.assertEqual(projected['rows'][2]['names'], [None, None])
            self.assertEqual({glyph['index'] for glyph in projected['glyphs']}, {176, 203, 205, 1000})
            ambiguous = copy.deepcopy(pointer)
            ambiguous['catalogs'].append(copy.deepcopy(ambiguous['catalogs'][0]))
            for candidate in (None, ambiguous):
                unavailable = project_metadata_name_references(metadata, candidate)
                self.assertEqual(unavailable['languages'], [])
                self.assertEqual(unavailable['rows'][0]['name_index'], 0)
                self.assertEqual(unavailable['rows'][0]['names'], [])
                self.assertIsNone(unavailable['reader_index'])

    def test_metadata_without_slot_or_text_catalog(self):
        with tempfile.TemporaryDirectory(prefix='viewer-metadata-only-') as directory:
            work = Path(directory)
            dataset, _, _ = fixture(work)
            remove_resources(dataset, {'gameplay.slot-job-lookup', 'text.catalogs'})
            result = generate(dataset, work / 'view', split=True)
            world = payload(result.parent / 'index.html.parts/world.html')
            self.assertIsNone(world['job_names'])
            self.assertEqual(len(world['metadata_names']['rows']), 116)
            self.assertEqual(world['metadata_names']['languages'], [])
            self.assertTrue(next(row for row in world['resources'] if row['id'] == 'gameplay.linked-metadata52')
                            ['status'].startswith('Available'))


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class MetadataNameBrowserTest(unittest.TestCase):
    def test_filters_pagination_language_and_links(self):
        with tempfile.TemporaryDirectory(prefix='viewer-metadata-browser-') as directory:
            work = Path(directory)
            dataset, _, _ = fixture(work)
            revise_metadata(dataset, lambda records: (records[1].update(routing=255), records[26].update(routing=1)))
            result = generate(dataset, work / 'view', split=True)
            browser = Chrome(work, result.parent / 'index.html.parts/world.html')
            try:
                self.assertTrue(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s), rows=q('#metadata-name-rows');
                  if(q('#metadata-names').hidden || q('#metadata-names>details').open || rows.rows.length!==25 ||
                     rows.rows[0].cells[2].textContent!=='Ab d' || !rows.rows[0].querySelector('.text-gap') ||
                     rows.rows[0].querySelectorAll('canvas').length!==1 ||
                     rows.rows[1].cells[1].textContent!=='Redirect 255 (not evaluated)' || rows.rows[1].querySelector('a') ||
                     q('#metadata-name-page').textContent!=='1–25 of 116' || !q('#metadata-name-prev').disabled) return false;
                  q('#metadata-name-next').click();
                  if(rows.rows[0].cells[0].textContent!=='25' || rows.rows[1].cells[1].textContent!=='Redirect 1 (not evaluated)') return false;
                  q('#metadata-name-prev').click();
                  q('#metadata-name-filter').value='26';q('#metadata-name-filter').dispatchEvent(new Event('input'));
                  if(rows.rows.length!==1 || rows.rows[0].cells[0].textContent!=='26' || !q('#metadata-name-next').disabled) return false;
                  q('#metadata-name-filter').value='AB D';q('#metadata-name-filter').dispatchEvent(new Event('input'));
                  if(rows.rows[0].cells[0].textContent!=='0') return false;
                  q('#metadata-name-filter').value='';q('#metadata-name-filter').dispatchEvent(new Event('input'));
                  q('#metadata-name-language').value='1';q('#metadata-name-language').dispatchEvent(new Event('change'));
                  const a=rows.rows[0].querySelector('a'), u=new URL(a.href);
                  return rows.rows[0].cells[2].textContent==='d' && u.searchParams.get('job_reader')==='0' &&
                    u.searchParams.get('job_language')==='1' && u.searchParams.get('job_entry')==='0' &&
                    u.pathname.endsWith('/text-pointers.html') && typeof data.pointer_texts==='undefined';
                })()'''))
                self.assertTrue(browser.evaluate('''(()=>{
                  const row=[...document.querySelectorAll('#rows tr')].find(r=>r.dataset.resourceId==='gameplay.linked-metadata52');
                  return [...row.querySelectorAll('a')].some(a=>a.textContent==='Open direct metadata names');
                })()'''))
            finally:
                browser.close()

    def test_single_page_text_link_and_missing_catalog(self):
        with tempfile.TemporaryDirectory(prefix='viewer-metadata-single-') as directory:
            work = Path(directory)
            dataset, _, _ = fixture(work)
            result = generate(dataset, work / 'view')
            browser = Chrome(work, result)
            try:
                self.assertTrue(browser.evaluate('''(()=>{
                  document.querySelector('#metadata-name-language').value='1';
                  document.querySelector('#metadata-name-language').dispatchEvent(new Event('change'));
                  document.querySelector('#metadata-name-rows tr:first-child a').click();
                  return document.querySelector('#pointer-texts>details').open &&
                    document.querySelector('#pointer-reader').value==='0' &&
                    document.querySelector('#pointer-language').value==='1' && document.querySelector('#pointer-entry').value==='0';
                })()'''))
            finally:
                browser.close()
            remove_resources(dataset, {'gameplay.slot-job-lookup', 'text.catalogs'})
            result = generate(dataset, work / 'missing', split=True)
            browser_work = work / 'missing-browser'
            browser_work.mkdir()
            browser = Chrome(browser_work, result.parent / 'index.html.parts/world.html')
            try:
                self.assertTrue(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s);
                  return !q('#metadata-names').hidden && q('#metadata-name-language-control').hidden &&
                    q('#metadata-name-rows tr:first-child').cells[2].textContent==='Name unavailable' &&
                    !q('#metadata-name-rows tr:first-child a');
                })()'''))
            finally:
                browser.close()
