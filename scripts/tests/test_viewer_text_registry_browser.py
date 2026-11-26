"""Exercise text reader callback projection and scoped wiki navigation."""
import hashlib
import json
import shutil
import tempfile
import time
import unittest
from pathlib import Path

import view_rom_data
from test_viewer_scoped_browser import parse_page
from viewer_browser_harness import Chrome


def fixture(work, *, callbacks=None, readers=(0x08001000, 0x08002000), registry_first=True,
            malformed=False, with_text=True, split=True):
    dataset = work / 'dataset'
    dataset.mkdir()
    callbacks = [0, 0x08002001, 0x08002001, 0x08001000, 0x08003001] if callbacks is None else callbacks
    registry = {'table_offset': 0x100, 'callbacks': callbacks}
    if malformed:
        registry['table_offset'] = 3
    program = {'source_offset': 1, 'header': 0, 'units': [{'kind': 'glyph', 'index': 176}]}
    text = {'glyphs': [{'index': 176, 'width': 8, 'tile_words': [1] + [0] * 31}],
            'strings': [{'source_offset': 1, 'encoding': 'compact', 'units': [{'kind': 'glyph', 'index': 176}]}],
            'pointer_catalogs': [{'reader': 0x08001000, 'languages': [{'source_offset': 1, 'entries': [0]}]}],
            'archive_catalogs': [{'reader': reader, 'languages': [
                {'source_offset': 1, 'offsets': [0, 65535], 'programs': [program]}]} for reader in readers]}
    resources = [('engine.text-reader-registry', 'registry.json', 'text-reader-registry-v1-json', registry)]
    if with_text:
        resources.append(('text.catalogs', 'text.json', 'text-catalogs-v1-json', text))
    if not registry_first:
        resources.reverse()
    manifest = 'profile="europe"\nextractor_revision=1\ndecoder_revision=1\n'
    for identity, name, fmt, value in resources:
        content = json.dumps(value).encode()
        (dataset / name).write_bytes(content)
        digest = hashlib.sha256(content).hexdigest()
        manifest += f'[[resources]]\nid="{identity}"\npath="{name}"\nformat="{fmt}"\nsha256="{digest}"\n'
    (dataset / 'manifest.toml').write_text(manifest)
    return view_rom_data.generate(dataset, work / 'view', split=split)


class TextReaderRegistryProjectionTest(unittest.TestCase):
    def test_validation_and_unique_address_join(self):
        project = view_rom_data.project_text_reader_registry
        link = view_rom_data.link_text_reader_registry
        registry = {**project({'table_offset': 0x100, 'callbacks': [0, 0x08002001, 0x08002001,
                                                                      0x08001000, 0x08003001, 0xffffffff]}),
                    'path': 'registry.json'}
        relative = {'catalogs': [{'reader': 0x08002000}, {'reader': 0x08003000}, {'reader': 0x08003000}]}
        rows = link(registry, relative)['rows']
        self.assertEqual([row['reader_index'] for row in rows], [None, 0, 0, None, None, None])
        self.assertEqual([row['reader'] for row in rows], [None, 0x08002000, 0x08002000, None, 0x08003000, None])
        self.assertTrue(all(row['reader_index'] is None for row in link(registry, None)['rows']))
        for value in ({'table_offset': 1, 'callbacks': [0]}, {'table_offset': 0, 'callbacks': []},
                      {'table_offset': 0, 'callbacks': [True]}, {'table_offset': 0, 'callbacks': [-1]},
                      {'table_offset': 0, 'callbacks': [0x100000000]},
                      {'table_offset': 0x1fffffc, 'callbacks': [0, 0]},
                      {'table_offset': 0, 'callbacks': [0] * 1025}):
            with self.subTest(value=value), self.assertRaises(ValueError):
                project(value)

    def test_generator_scopes_and_resource_order(self):
        for registry_first in (True, False):
            with self.subTest(registry_first=registry_first), tempfile.TemporaryDirectory() as directory:
                page = fixture(Path(directory), registry_first=registry_first)
                parts = page.parent / (page.name + '.parts')
                index = parse_page(page)
                projection = parse_page(parts / 'text-relative.html')['text_reader_registry']
                self.assertEqual([row['reader_index'] for row in projection['rows']], [None, 1, 1, None, None])
                for scope in ('other', 'text', 'text-relative'):
                    data = parse_page(parts / (scope + '.html'))
                    self.assertEqual(data['text_reader_registry'], projection)
                    self.assertIn('engine.text-reader-registry', [row['id'] for row in data['resources']])
                self.assertNotIn('relative_texts', parse_page(parts / 'other.html'))
                self.assertNotIn('text_reader_registry', parse_page(parts / 'audio.html'))
                row = next(row for row in index['resources'] if row['id'] == 'engine.text-reader-registry')
                self.assertEqual(row['viewer_scope'], 'other')
                self.assertTrue(row['status'].startswith('Available'))
        for kwargs in ({'malformed': True}, {'with_text': False}):
            with self.subTest(kwargs=kwargs), tempfile.TemporaryDirectory() as directory:
                page = fixture(Path(directory), split=False, **kwargs)
                data = parse_page(page)
                if kwargs.get('malformed'):
                    self.assertIsNone(data['text_reader_registry'])
                    self.assertTrue(data['resources'][0]['status'].startswith('Invalid'))
                else:
                    self.assertTrue(all(row['reader_index'] is None for row in data['text_reader_registry']['rows']))


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class TextReaderRegistryBrowserTest(unittest.TestCase):
    def test_single_page_filter_and_bidirectional_links(self):
        with tempfile.TemporaryDirectory() as directory:
            work = Path(directory)
            browser = Chrome(work, fixture(work, split=False))
            try:
                self.assertEqual(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s), input=e=>e.dispatchEvent(new Event('input'));
                  if(q('#text-reader-registry').hidden || q('#text-reader-registry>details').open ||
                     q('#text-registry-rows').rows.length!==5 || q('#text-registry-rows tr[data-slot="0"] a') ||
                     q('#text-registry-rows tr[data-slot="3"] a') || q('#text-registry-rows tr[data-slot="4"] a') ||
                     !q('#relative-registry-navigation').hidden) throw Error('initial rows');
                  q('#relative-pin').checked=true;q('#relative-pin').dispatchEvent(new Event('change'));
                  q('#text-registry-rows tr[data-slot="1"] a').click();
                  if(q('#relative-reader').value!=='1' || !q('#relative-texts>details').open ||
                     q('#relative-registry-navigation').hidden || !q('#relative-pin').checked) throw Error('open archive');
                  q('#relative-registry-open').click();
                  if(!q('#text-reader-registry>details').open || q('#text-registry-rows').rows.length!==2 ||
                     q('#text-registry-filter').value!=='0x08002000') throw Error('reverse navigation');
                  q('#text-registry-filter').value='0x08001000';input(q('#text-registry-filter'));
                  if(q('#text-registry-rows').rows.length!==1 || q('#text-registry-rows tr').dataset.slot!=='3') throw Error('filter');
                  q('#relative-reader').value='0';q('#relative-reader').dispatchEvent(new Event('change'));
                  if(!q('#relative-registry-navigation').hidden) throw Error('unmatched reader');
                  return 'PASS';
                })()'''), 'PASS')
            finally:
                browser.close()

    def test_cross_page_link_selects_reader_and_duplicate_slots(self):
        with tempfile.TemporaryDirectory() as directory:
            work = Path(directory); page = fixture(work)
            browser = Chrome(work, page.parent / (page.name + '.parts/other.html'))
            try:
                self.assertTrue(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s);
                  return !q('#text-reader-registry').hidden && typeof data.relative_texts==='undefined' &&
                    q('#text-registry-rows tr[data-slot="1"] a').href.includes('select=relative-reader%3A1');
                })()'''))
                browser.evaluate("document.querySelector('#text-reader-registry>details').open=true")
                browser.click('#text-registry-rows tr[data-slot="1"] a')
                end = time.monotonic() + 30
                while time.monotonic() < end:
                    try:
                        if browser.evaluate("document.readyState==='complete' && data.wiki.current==='text-relative'"):
                            break
                    except (RuntimeError, OSError):
                        pass
                    time.sleep(.05)
                else:
                    self.fail('Relative archive page did not load')
                self.assertTrue(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s);
                  if(q('#relative-reader').value!=='1' || !q('#relative-texts>details').open ||
                     q('#relative-registry-navigation').hidden) return false;
                  q('#relative-registry-open').click();
                  return q('#text-registry-rows').rows.length===2 && q('#text-reader-registry>details').open;
                })()'''))
            finally:
                browser.close()
