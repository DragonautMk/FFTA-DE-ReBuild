"""Verify source-gated item names, language selection and text navigation."""
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


def fixture(work, *, profile='europe', with_text=True, duplicate=False, split=True):
    dataset = work / 'dataset'; dataset.mkdir()
    items = {'records': [{'halfwords_00_06': [index % 3, 0, 0, 0], 'bytes_08_0f': [0] * 8,
                          'signed_10_17': [0] * 8, 'bytes_18_1f': [0] * 8} for index in range(30)]}
    def text(units):
        return {'source_offset': 1, 'encoding': 'compact', 'units': units}
    glyph = lambda index: {'kind': 'glyph', 'index': index}
    strings = [text([glyph(176), {'kind': 'control_byte', 'value': 64}, glyph(203)]),
               text([glyph(177)]), text([glyph(204)])]
    reader = {'reader': 0x080165e8, 'languages': [{'source_offset': 10, 'entries': [0, 1, 2]},
                                                  {'source_offset': 20, 'entries': [1, 0]}]}
    catalog = {'glyphs': [{'index': index, 'width': 8, 'tile_words': [1] + [0] * 31}
                          for index in (176, 177, 203, 204)],
               'strings': strings, 'pointer_catalogs': [reader, copy.deepcopy(reader)] if duplicate else [reader],
               'archive_catalogs': []}
    resources = [('gameplay.item-records', 'items.json', 'item-records-v1-json', items)]
    if with_text:
        resources.append(('text.catalogs', 'text.json', 'text-catalogs-v1-json', catalog))
    manifest = f'profile="{profile}"\nextractor_revision=1\ndecoder_revision=1\n'; digests = []
    for identity, name, fmt, value in resources:
        content = json.dumps(value).encode(); (dataset / name).write_bytes(content)
        digest = hashlib.sha256(content).hexdigest(); digests.append(digest)
        manifest += f'[[resources]]\nid="{identity}"\npath="{name}"\nformat="{fmt}"\nsha256="{digest}"\n'
    (dataset / 'manifest.toml').write_text(manifest)
    with patch.object(view_rom_data, 'EUROPE_ITEMS_SHA256', digests[0]), \
         patch.object(view_rom_data, 'EUROPE_TEXT_CATALOG_SHA256', digests[-1]):
        page = view_rom_data.generate(dataset, work / 'view', split=split)
    return page, digests


class ItemNameProjectionTest(unittest.TestCase):
    def test_generation_scope_and_identity_guards(self):
        with tempfile.TemporaryDirectory() as directory:
            page, digests = fixture(Path(directory))
            parts = page.parent / (page.name + '.parts'); world = parse_page(parts / 'world.html')
            names = world['item_names']
            self.assertEqual(len(names['rows']), 30)
            self.assertEqual(names['rows'][0]['names'], [[176, [64], 203], [177]])
            self.assertEqual(names['rows'][2]['names'], [[204], None])
            self.assertNotIn('pointer_texts', world)
            self.assertNotIn('item_names', parse_page(parts / 'audio.html'))
            pointer = parse_page(parts / 'text-pointers.html')['pointer_texts']
            with patch.object(view_rom_data, 'EUROPE_ITEMS_SHA256', digests[0]), \
                 patch.object(view_rom_data, 'EUROPE_TEXT_CATALOG_SHA256', digests[1]):
                project = view_rom_data.project_item_names
                self.assertEqual(project('europe', *digests, world['items'], pointer), names)
                self.assertIsNone(project('usa', *digests, world['items'], pointer))
                self.assertIsNone(project('europe', 'wrong', digests[1], world['items'], pointer))
                self.assertIsNone(project('europe', digests[0], 'wrong', world['items'], pointer))
                self.assertIsNone(project('europe', *digests, None, pointer))
                self.assertIsNone(project('europe', *digests, world['items'], None))
                altered = copy.deepcopy(pointer); altered['catalogs'].append(copy.deepcopy(altered['catalogs'][0]))
                self.assertIsNone(project('europe', *digests, world['items'], altered))
        for kwargs in ({'with_text': False}, {'duplicate': True}, {'profile': 'usa'}):
            with self.subTest(kwargs=kwargs), tempfile.TemporaryDirectory() as directory:
                page, _ = fixture(Path(directory), split=False, **kwargs)
                data = parse_page(page)
                self.assertIsNone(data['item_names'])
                self.assertEqual(len(data['items']['records']), 30)


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class ItemNameBrowserTest(unittest.TestCase):
    def test_filter_languages_pagination_and_partial_name(self):
        with tempfile.TemporaryDirectory() as directory:
            work = Path(directory); page, _ = fixture(work)
            browser = Chrome(work, page.parent / (page.name + '.parts/world.html'))
            try:
                self.assertEqual(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change'));
                  if(q('#item-names').hidden || q('#item-id').options.length!==30 ||
                     q('#item-name-rows').rows.length!==25 || !q('#item-id').options[0].text.includes('A b'))throw Error('initial names');
                  q('#item-name-next').click();if(q('#item-name-rows').rows.length!==5)throw Error('page');
                  q('#item-name-filter').value='A b';q('#item-name-filter').dispatchEvent(new Event('input'));
                  if(q('#item-name-rows').rows.length!==10 || q('#item-name-rows a').textContent!=='0')throw Error('name filter');
                  q('#item-name-filter').value='2';q('#item-name-filter').dispatchEvent(new Event('input'));
                  q('#item-name-rows a').click();if(q('#item-id').value!=='2')throw Error('select item');
                  q('#item-name-language').value='1';change(q('#item-name-language'));
                  if(q('#item-id').value!=='2' || q('#item-name-preview').textContent!=='Name unavailable' ||
                     q('#item-name-source a'))throw Error('missing language string');
                  q('#item-id').value='1';change(q('#item-id'));
                  if(!q('#item-name-preview').textContent.includes('A b') || !q('#item-name-source a') ||
                     !q('#item-id').options[1].text.includes('A b'))throw Error('selected language');
                  return 'PASS';
                })()'''), 'PASS')
                browser.evaluate("document.querySelector('#items>details').open=true")
                browser.click('#item-name-source a')
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

    def test_legacy_and_missing_sources(self):
        for kwargs in ({'split': False}, {'split': False, 'with_text': False}, {'split': False, 'duplicate': True}):
            with self.subTest(kwargs=kwargs), tempfile.TemporaryDirectory() as directory:
                work = Path(directory); page, _ = fixture(work, **kwargs); browser = Chrome(work, page)
                try:
                    if kwargs.get('with_text', True) and not kwargs.get('duplicate'):
                        self.assertTrue(browser.evaluate('''(()=>{
                          const q=s=>document.querySelector(s);q('#item-id').value='1';
                          q('#item-id').dispatchEvent(new Event('change'));q('#item-name-source a').click();
                          return q('#pointer-texts>details').open && q('#pointer-entry').value==='1';
                        })()'''))
                    else:
                        self.assertTrue(browser.evaluate("document.querySelector('#item-names').hidden && !document.querySelector('#items').hidden"))
                finally:
                    browser.close()
