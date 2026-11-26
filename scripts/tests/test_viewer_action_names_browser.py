"""Verify action-linked records and source-gated localized names."""
import copy
import hashlib
import json
import shutil
import sys
import tempfile
import time
import unittest
from pathlib import Path
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import view_rom_data
from test_viewer_scoped_browser import parse_page
from viewer_browser_harness import Chrome


def record(index):
    return {'index': index, 'field00': index % 3, 'field02': 255, 'opaque03': 7,
            'fields04_0b': list(range(8)), 'inline0c_0f': [11, 12, 13, 14],
            'flags10': 0x80000801, 'field14': 0x1234, 'field16': 0xffff,
            'field18': 18, 'field19': 19, 'field1a': 20, 'opaque1b': 21}


def fixture(work, *, profile='europe', with_text=True, with_records=True, invalid=False, duplicate=False, split=True):
    dataset = work / 'dataset'; dataset.mkdir()
    records = {'source_offset': 0x3000, 'records': [record(i) for i in range(30)]}
    if invalid:
        records['records'][0]['inline0c_0f'].pop()
    def text(units): return {'source_offset': 1, 'encoding': 'compact', 'units': units}
    def glyph(index): return {'kind': 'glyph', 'index': index}
    strings = [text([glyph(176), {'kind': 'control_byte', 'value': 64}, glyph(203)]),
               text([glyph(177)]), text([glyph(204)])]
    reader = {'reader': 0x08016588, 'languages': [{'source_offset': 10, 'entries': [0, 1, 2]},
                                                  {'source_offset': 20, 'entries': [1, 0]}]}
    catalog = {'glyphs': [{'index': index, 'width': 8, 'tile_words': [1] + [0] * 31}
                          for index in (176, 177, 203, 204)],
               'strings': strings, 'pointer_catalogs': [reader, copy.deepcopy(reader)] if duplicate else [reader],
               'archive_catalogs': []}
    resources = []
    if with_records:
        resources.append(('gameplay.linked-metadata28', 'records.json', 'linked-metadata28-v1-json', records))
    if with_text:
        resources.append(('text.catalogs', 'text.json', 'text-catalogs-v1-json', catalog))
    manifest = f'profile="{profile}"\nextractor_revision=1\ndecoder_revision=1\n'; digests = {}
    for identity, name, fmt, value in resources:
        content = json.dumps(value).encode(); (dataset / name).write_bytes(content)
        digests[identity] = hashlib.sha256(content).hexdigest()
        manifest += f'[[resources]]\nid="{identity}"\npath="{name}"\nformat="{fmt}"\nsha256="{digests[identity]}"\n'
    (dataset / 'manifest.toml').write_text(manifest)
    with patch.object(view_rom_data, 'EUROPE_ACTION_RECORDS_SHA256', digests.get('gameplay.linked-metadata28')), \
         patch.object(view_rom_data, 'EUROPE_TEXT_CATALOG_SHA256', digests.get('text.catalogs')):
        page = view_rom_data.generate(dataset, work / 'view', split=split)
    return page, digests


class ActionRecordProjectionTest(unittest.TestCase):
    def test_record_fields_and_validation(self):
        catalog = {'source_offset': 0x3000, 'records': [record(i) for i in range(30)]}
        self.assertEqual(view_rom_data.project_action_records(catalog), catalog)
        for change in ('source', 'extent', 'empty', 'index', 'halfword', 'byte', 'flags', 'array', 'array_value'):
            value = copy.deepcopy(catalog)
            if change == 'source': value['source_offset'] = True
            elif change == 'extent': value['source_offset'] = 0x1fffffc
            elif change == 'empty': value['records'] = []
            elif change == 'index': value['records'][2]['index'] = 1
            elif change == 'halfword': value['records'][0]['field00'] = 65536
            elif change == 'byte': value['records'][0]['opaque1b'] = -1
            elif change == 'flags': value['records'][0]['flags10'] = 0x100000000
            elif change == 'array': value['records'][0]['fields04_0b'].pop()
            elif change == 'array_value': value['records'][0]['inline0c_0f'][0] = True
            with self.subTest(change=change), self.assertRaises(ValueError):
                view_rom_data.project_action_records(value)

    def test_identity_guards_scopes_and_missing_resources(self):
        with tempfile.TemporaryDirectory() as directory:
            page, digests = fixture(Path(directory)); parts = page.parent / (page.name + '.parts')
            world = parse_page(parts / 'world.html'); names = world['action_names']
            self.assertEqual(names['rows'][0]['names'], [[176, [64], 203], [177]])
            self.assertEqual(names['rows'][2]['names'], [[204], None])
            self.assertNotIn('pointer_texts', world)
            self.assertNotIn('action_records', parse_page(parts / 'audio.html'))
            row = next(row for row in parse_page(page)['resources'] if row['id'] == 'gameplay.linked-metadata28')
            self.assertEqual(row['viewer_scope'], 'world'); self.assertTrue(row['status'].startswith('Available'))
            pointer = parse_page(parts / 'text-pointers.html')['pointer_texts']
            record_sha, text_sha = digests['gameplay.linked-metadata28'], digests['text.catalogs']
            with patch.object(view_rom_data, 'EUROPE_ACTION_RECORDS_SHA256', record_sha), \
                 patch.object(view_rom_data, 'EUROPE_TEXT_CATALOG_SHA256', text_sha):
                project = view_rom_data.project_action_names
                self.assertEqual(project('europe', record_sha, text_sha, world['action_records'], pointer), names)
                self.assertIsNone(project('usa', record_sha, text_sha, world['action_records'], pointer))
                self.assertIsNone(project('europe', 'wrong', text_sha, world['action_records'], pointer))
                self.assertIsNone(project('europe', record_sha, 'wrong', world['action_records'], pointer))
                self.assertIsNone(project('europe', record_sha, text_sha, None, pointer))
                self.assertIsNone(project('europe', record_sha, text_sha, world['action_records'], None))
                altered = copy.deepcopy(pointer); altered['catalogs'].append(copy.deepcopy(altered['catalogs'][0]))
                self.assertIsNone(project('europe', record_sha, text_sha, world['action_records'], altered))
        for kwargs in ({'with_text': False}, {'with_records': False}, {'invalid': True},
                       {'duplicate': True}, {'profile': 'usa'}):
            with self.subTest(kwargs=kwargs), tempfile.TemporaryDirectory() as directory:
                page, _ = fixture(Path(directory), split=False, **kwargs); data = parse_page(page)
                self.assertIsNone(data['action_names'])
                if kwargs.get('with_records', True) and not kwargs.get('invalid'):
                    self.assertEqual(len(data['action_records']['records']), 30)
                else:
                    self.assertIsNone(data['action_records'])
                if kwargs.get('invalid'):
                    row = next(row for row in data['resources'] if row['id'] == 'gameplay.linked-metadata28')
                    self.assertTrue(row['status'].startswith('Invalid action record'))


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class ActionRecordBrowserTest(unittest.TestCase):
    def test_fields_flags_language_filter_and_source_link(self):
        with tempfile.TemporaryDirectory() as directory:
            work = Path(directory); page, _ = fixture(work)
            browser = Chrome(work, page.parent / (page.name + '.parts/world.html'))
            try:
                self.assertEqual(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change'));
                  if(q('#action-records').hidden || q('#action-records>details').open || q('#action-names').hidden ||
                     q('#action-record-id').options.length!==30 || q('#action-name-rows').rows.length!==25 ||
                     !q('#action-record-id').options[0].text.includes('A b'))throw Error('initial records');
                  if(!q('#action-record-fields').textContent.includes('4660 (0x1234)') ||
                     !q('#action-record-fields').textContent.includes('0x0B 0x0C 0x0D 0x0E') ||
                     q('#action-record-flags').rows.length!==32 ||
                     q('#action-record-flags').rows[0].cells[1].textContent!=='1' ||
                     q('#action-record-flags').rows[11].cells[1].textContent!=='1' ||
                     q('#action-record-flags').rows[31].cells[1].textContent!=='1' ||
                     q('#action-record-flags').rows[20].cells[2].textContent!=='31' ||
                     q('#action-record-flags').rows[21].cells[2].textContent!=='—')throw Error('fields or flags');
                  q('#action-name-next').click();if(q('#action-name-rows').rows.length!==5)throw Error('page');
                  q('#action-name-filter').value='A b';q('#action-name-filter').dispatchEvent(new Event('input'));
                  if(q('#action-name-rows').rows.length!==10 || q('#action-name-rows a').textContent!=='0')throw Error('filter');
                  q('#action-name-filter').value='2';q('#action-name-filter').dispatchEvent(new Event('input'));
                  q('#action-name-rows a').click();
                  if(q('#action-record-id').value!=='2' || !q('#action-record-source').textContent.includes('0x3038'))throw Error('select');
                  q('#action-name-language').value='1';change(q('#action-name-language'));
                  if(q('#action-record-id').value!=='2' || q('#action-name-preview').textContent!=='Name unavailable' ||
                     q('#action-name-source a'))throw Error('missing language');
                  q('#action-record-id').value='1';change(q('#action-record-id'));
                  if(!q('#action-name-preview').textContent.includes('A b') || !q('#action-name-source a') ||
                     !q('#action-record-id').options[1].text.includes('A b'))throw Error('language');
                  return 'PASS';
                })()'''), 'PASS')
                browser.evaluate("document.querySelector('#action-records>details').open=true")
                browser.click('#action-name-source a')
                end = time.monotonic() + 30
                while time.monotonic() < end:
                    try:
                        if browser.evaluate("document.readyState==='complete' && data.wiki.current==='text-pointers'"):
                            break
                    except (RuntimeError, OSError):
                        pass
                    time.sleep(.05)
                else:
                    self.fail('Pointer text page did not load')
                self.assertTrue(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s);
                  return q('#pointer-texts>details').open && q('#pointer-reader').value==='0' &&
                    q('#pointer-language').value==='1' && q('#pointer-entry').value==='1';
                })()'''))
            finally:
                browser.close()

    def test_partial_and_legacy_data(self):
        for kwargs in ({'split': False}, {'split': False, 'with_text': False}, {'split': False, 'with_records': False},
                       {'split': False, 'invalid': True}, {'split': False, 'profile': 'usa'}):
            with self.subTest(kwargs=kwargs), tempfile.TemporaryDirectory() as directory:
                work = Path(directory); page, _ = fixture(work, **kwargs); browser = Chrome(work, page)
                try:
                    if len(kwargs) == 1:
                        self.assertTrue(browser.evaluate('''(()=>{
                          const q=s=>document.querySelector(s);q('#action-record-id').value='1';
                          q('#action-record-id').dispatchEvent(new Event('change'));q('#action-name-source a').click();
                          return q('#pointer-texts>details').open && q('#pointer-entry').value==='1';
                        })()'''))
                    elif not kwargs.get('with_records', True) or kwargs.get('invalid'):
                        self.assertTrue(browser.evaluate("document.querySelector('#action-records').hidden"))
                    else:
                        self.assertTrue(browser.evaluate("!document.querySelector('#action-records').hidden && document.querySelector('#action-names').hidden"))
                finally:
                    browser.close()
