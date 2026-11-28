"""Verify source-gated screen palettes and offline color inspection."""
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
from test_viewer_bg_planes_browser import fixture as background_fixture
from test_viewer_navbar_split_browser import wait_for
from test_viewer_scoped_browser import parse_page
from viewer_browser_harness import Chrome


def catalog(*, bad_literal=False):
    first = [0x001f, 0x03e0, 0x7c00, 0x801f] + list(range(4, 480))
    second = list(range(256)); second[-1] = 0x8000
    pieces = [(0x143ce0, 'reader_literals', struct.pack('<I', 0x08b8be58 if not bad_literal else 0)),
              (0x144374, 'reader_literals', struct.pack('<I', 0x08b8c038)),
              (0x144b24, 'reader_literals', struct.pack('<I', 0x08b8c378)),
              (0xb8be58, 'palette_colors', struct.pack('<480H', *first)),
              (0xb8c378, 'palette_colors', struct.pack('<256H', *second))]
    content = b''.join(data for _, _, data in pieces)
    sources = [{'offset': offset, 'role': role, 'length': len(data), 'sha256': hashlib.sha256(data).hexdigest()}
               for offset, role, data in pieces]
    return content, sources


def fixture(work, *, profile='europe', trusted=True, bad_source=False, bad_literal=False, split=True):
    background_fixture(work)
    dataset = work / 'dataset'; content, sources = catalog(bad_literal=bad_literal)
    if bad_source:
        sources[0]['sha256'] = '0' * 64
    (dataset / 'screen.bin').write_bytes(content)
    manifest = (dataset / 'manifest.toml').read_text().replace('profile="test"', f'profile="{profile}"')
    manifest += ('[[resources]]\nid="graphics.screen-palette-bank"\npath="screen.bin"\n'
                 'format="screen-palette-bank-v1-bin"\nsha256="' + hashlib.sha256(content).hexdigest() + '"\n')
    for item in sources:
        manifest += (f'[[resources.sources]]\noffset={item["offset"]}\nlength={item["length"]}\n'
                     f'role="{item["role"]}"\nsha256="{item["sha256"]}"\n')
    (dataset / 'manifest.toml').write_text(manifest)
    with patch.object(viewer, 'EUROPE_SCREEN_PALETTE_SHA256', hashlib.sha256(content).hexdigest() if trusted else '0' * 64):
        return viewer.generate(dataset, work / 'screen-view', split=split)


class ScreenPaletteProjectionTest(unittest.TestCase):
    def test_all_words_and_source_guards(self):
        content, sources = catalog(); digest = hashlib.sha256(content).hexdigest()
        with patch.object(viewer, 'EUROPE_SCREEN_PALETTE_SHA256', digest):
            project = viewer.project_screen_palette_bank
            value = project('europe', digest, content, sources)
            self.assertEqual([len(table['colors']) for table in value['tables']], [240, 240, 256])
            self.assertEqual(value['tables'][0]['colors'][:4], [0x001f, 0x03e0, 0x7c00, 0x801f])
            self.assertEqual(value['tables'][1]['source_offset'], 0xb8c038)
            self.assertEqual(value['tables'][2]['colors'][-1], 0x8000)
            self.assertEqual(value['tables'][2]['staged_slot'], 256)
            self.assertIsNone(project('usa', digest, content, sources))
            self.assertIsNone(project('europe', 'wrong', content, sources))
            for mutate in (lambda s: s.pop(), lambda s: s[0].update(sha256='0'*64),
                           lambda s: s[0].update(offset=0), lambda s: s[0].update(role='palette_colors')):
                bad = copy.deepcopy(sources); mutate(bad)
                with self.assertRaises(ValueError):
                    project('europe', digest, content, bad)
            invalid, invalid_sources = catalog(bad_literal=True)
            with patch.object(viewer, 'EUROPE_SCREEN_PALETTE_SHA256', hashlib.sha256(invalid).hexdigest()):
                with self.assertRaisesRegex(ValueError, 'reader literal'):
                    project('europe', hashlib.sha256(invalid).hexdigest(), invalid, invalid_sources)

    def test_scoped_page_and_invalid_input_isolation(self):
        for options in ({}, {'trusted': False}, {'profile': 'usa'}, {'bad_source': True}, {'bad_literal': True}):
            with self.subTest(options=options), tempfile.TemporaryDirectory(prefix='viewer-screen-palettes-') as directory:
                page = fixture(Path(directory), **options); parts = page.parent / 'index.html.parts'
                home, screen, interface = map(parse_page, (page, parts / 'interface-screen-palettes.html', parts / 'interface.html'))
                self.assertNotIn('screen_palette_bank', home)
                self.assertNotIn('screen_palette_bank', interface)
                self.assertTrue(interface['palettes'])
                row = next(row for row in home['resources'] if row['id'] == 'graphics.screen-palette-bank')
                self.assertEqual(row['viewer_scope'], 'interface-screen-palettes')
                if options:
                    self.assertIsNone(screen['screen_palette_bank'])
                    self.assertTrue(row['status'].startswith(('Preview unavailable', 'Invalid screen palette')))
                else:
                    self.assertEqual(len(screen['screen_palette_bank']['tables']), 3)
                    self.assertEqual(row['status'], 'Available: three screen color tables')
                    info = next(page for page in home['wiki']['pages'] if page['id'] == 'interface-screen-palettes')
                    self.assertEqual(info['sections'][0]['change'], {'kind': 'added', 'at': '2026-11-27T22:25:47Z'})


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class ScreenPaletteBrowserTest(unittest.TestCase):
    def test_navigation_swatches_values_and_bookmarks(self):
        with tempfile.TemporaryDirectory(prefix='viewer-screen-palettes-browser-') as directory:
            work = Path(directory); page = fixture(work); browser = Chrome(work, page)
            try:
                target = browser.evaluate('''(()=>{const q=s=>document.querySelector(s),row=q('#rows [data-resource-id="graphics.screen-palette-bank"]');
                  const link=row.querySelector('a');if(!link||!link.href.includes('/interface-screen-palettes.html'))throw Error('resource route');
                  const nav=q('[data-category="Interface"]');if(!nav.querySelector('a[href$="/interface-screen-palettes.html#screen-palettes"]'))throw Error('navbar');
                  return link.href;})()''')
                browser.command('Page.navigate', {'url': target})
                wait_for(browser, "data.wiki.current==='interface-screen-palettes' && !document.querySelector('#screen-palettes').hidden")
                self.assertTrue(browser.evaluate('''(()=>{const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change',{bubbles:true}));
                  if(q('#screen-palettes>details>summary time').dateTime!=='2026-11-27T22:25:47Z'||q('#screen-palette-colors').children.length!==16||
                     q('#screen-palette-colors .color').style.backgroundColor!=='rgb(255, 0, 0)')throw Error('initial');
                  q('#screen-palette-colors').children[3].click();
                  if(!q('#screen-palette-selection').textContent.includes('0x801F')||!q('#screen-palette-selection').textContent.includes('bit 15 1'))throw Error('word');
                  q('#screen-palette-bank').value='1';change(q('#screen-palette-bank'));
                  q('#screen-palette-table').value='2';change(q('#screen-palette-table'));
                  if(q('#screen-palette-bank').value!=='1')throw Error('retained bank');
                  q('#screen-palette-bank').value='15';change(q('#screen-palette-bank'));q('#screen-palette-colors').children[15].click();
                  return q('#screen-palette-selection').textContent.includes('source 0xB8C576')&&
                    q('#screen-palette-selection').textContent.includes('staged palette slot 511')&&
                    q('#screen-palette-selection').textContent.includes('0x8000');})()'''))
                saved = browser.evaluate("document.querySelector('#screen-palettes a').href")
                browser.command('Page.navigate', {'url': saved})
                wait_for(browser, "data.wiki.current==='interface-screen-palettes' && document.querySelector('#screen-palette-table').value==='2' && document.querySelector('#screen-palette-bank').value==='15'")
                self.assertTrue(browser.evaluate("document.querySelector('#screen-palettes>details').open"))
            finally:
                browser.close()

    def test_legacy_single_page(self):
        with tempfile.TemporaryDirectory(prefix='viewer-screen-palettes-single-') as directory:
            work = Path(directory); page = fixture(work, split=False); browser = Chrome(work, page)
            try:
                self.assertTrue(browser.evaluate("!document.querySelector('#screen-palettes').hidden && document.querySelector('#screen-palette-colors').children.length===16"))
            finally:
                browser.close()


if __name__ == '__main__':
    unittest.main()
