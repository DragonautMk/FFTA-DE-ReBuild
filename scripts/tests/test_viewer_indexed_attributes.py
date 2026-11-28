"""Exercise authenticated packed attributes and links to their graphics archives."""
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
import viewer_indexed_attributes as attributes
from test_viewer_indexed_graphics_browser import archive
from test_viewer_navbar_split_browser import wait_for
from test_viewer_scoped_browser import parse_page
from viewer_browser_harness import Chrome


def encoded_table(count, tiles, pattern):
    return struct.pack('<HH', 4 + count * tiles // 2, count * tiles) + bytes(pattern) * count


def source(offset, role, raw):
    return {'offset': offset, 'role': role, 'length': len(raw), 'sha256': hashlib.sha256(raw).hexdigest()}


def synthetic():
    first = encoded_table(522, 4, [0xab, 0xcd])
    second = bytearray(encoded_table(62, 4, [0x12, 0x34])); second[36] = 0xe1
    pieces = [(0x0cf3dc, 'reader_literals', struct.pack('<I', 0x083bd70c)),
              (0x0cf494, 'reader_literals', struct.pack('<II', 0x02000000, 0x1f6e)),
              (0x3bd70c, 'tile_attributes', first),
              (0x3c943c, 'tile_attributes', second[:36]),
              (0x3c9461, 'tile_attributes', second[37:])]
    locations = [0x43d990, 0x44efb8, 0x460b20, 0x471b58, 0x483634]
    prefixes = [{'source_offset': offset, 'bytes': list(encoded_table(44, 8, [0x10, 0x32, 0x54, 0x76]))}
                for offset in locations]
    # Keep test programs synthetic and within the 32-bit word limit.
    def graphics(offset, count, tiles):
        value = archive(offset, 1, tiles=tiles); program = value['programs'][0]
        value['programs'] = [{**program, 'index': i, 'relative_offset': i * 4} for i in range(count)]
        value['entry_offsets'] = [i * 4 for i in range(count)]
        return value
    catalog = {'archives': [graphics(0x3bdb24, 522, 4)], 'white_flowers_direct': {'archive': graphics(0x3c94bc, 62, 4)},
               'additional': {'groups': [{'source_offset': 0x397d94, 'pointers': locations}],
                              'embedded_prefixes': prefixes,
                              'archives': [graphics(offset + 180, 44, 8) for offset in locations]}}
    archive_sources = [source(0x397d94, 'additional-indexed-selector/397d94',
                              struct.pack('<5I', *(0x08000000 + item for item in locations)))]
    archive_sources.extend(source(item['source_offset'], f"additional-indexed-prefix/{item['source_offset']:x}", bytes(item['bytes']))
                           for item in prefixes)
    shared = {'byte_offset': 0x3c9460, 'byte': second[36]}
    shared_sources = [source(0x3c9460, 'white-flowers-observed-nibble-byte', bytes([second[36]]))]
    return pieces, catalog, archive_sources, shared, shared_sources


def fixture(work, *, omit=(), bad_source=False, profile='europe', split=True):
    pieces, catalog, archive_sources, shared, shared_sources = synthetic()
    dataset = work / 'dataset'; dataset.mkdir()
    content = b''.join(raw for _, _, raw in pieces)
    attr_sources = [source(offset, role, raw) for offset, role, raw in pieces]
    if bad_source:
        attr_sources[2]['sha256'] = '0' * 64
    archive_content = json.dumps(catalog).encode(); shared_content = json.dumps(shared).encode()
    items = [('graphics.indexed-tile-attributes', 'attributes.bin', 'indexed-tile-attributes-v1-bin', content, attr_sources),
             ('graphics.indexed-archives', 'archives.json', 'indexed-graphics-v2-json', archive_content, archive_sources),
             ('engine.white-flowers-isolated-values', 'shared.json', 'white-flowers-isolated-values-v1-json', shared_content, shared_sources)]
    manifest = f'profile="{profile}"\nextractor_revision=1\ndecoder_revision=1\n'
    for identity, name, fmt, raw, sources in items:
        if identity in omit:
            continue
        (dataset / name).write_bytes(raw)
        manifest += f'[[resources]]\nid="{identity}"\npath="{name}"\nformat="{fmt}"\nsha256="{hashlib.sha256(raw).hexdigest()}"\n'
        for item in sources:
            manifest += (f'[[resources.sources]]\noffset={item["offset"]}\nlength={item["length"]}\n'
                         f'role="{item["role"]}"\nsha256="{item["sha256"]}"\n')
    (dataset / 'manifest.toml').write_text(manifest)
    with patch.object(attributes, 'ATTRIBUTES_SHA256', hashlib.sha256(content).hexdigest()), \
         patch.object(attributes, 'ARCHIVES_SHA256', hashlib.sha256(archive_content).hexdigest()), \
         patch.object(attributes, 'SHARED_BYTE_SHA256', hashlib.sha256(shared_content).hexdigest()):
        return viewer.generate(dataset, work / 'view', split=split)


class IndexedAttributesProjectionTest(unittest.TestCase):
    def test_bound_sources_and_partial_owners(self):
        pieces, catalog, sources, shared, shared_sources = synthetic()
        segments = viewer.project_binary_segments(b''.join(raw for _, _, raw in pieces),
                                                  [source(offset, role, raw) for offset, role, raw in pieces])
        localized = attributes.project_localized(catalog, sources)
        value = attributes.project_tables(segments, attributes.project_shared_byte(shared, shared_sources), localized)
        self.assertEqual([len(table['entries']) for table in value['tables']], [522, 62, 44, 44, 44, 44, 44])
        self.assertEqual(value['tables'][0]['entries'][0], [0xab, 0xcd])
        self.assertEqual(value['tables'][1]['entries'][16][0], 0xe1)
        self.assertEqual(value['tables'][2]['entries'][0], [0x10, 0x32, 0x54, 0x76])
        partial = attributes.project_tables(segments)
        self.assertEqual(len(partial['tables']), 1); self.assertEqual(len(partial['missing']), 2)
        for mutate in (lambda c: c['additional']['groups'][0]['pointers'].__setitem__(0, 0),
                       lambda c: c['additional']['embedded_prefixes'][0]['bytes'].__setitem__(4, 99)):
            bad = copy.deepcopy(catalog); mutate(bad)
            with self.assertRaises(ValueError): attributes.project_localized(bad, sources)
        bad_sources = copy.deepcopy(shared_sources); bad_sources[0]['sha256'] = '0' * 64
        with self.assertRaises(ValueError): attributes.project_shared_byte(shared, bad_sources)
        with self.assertRaises(ValueError): attributes.project_shared_byte(shared, [True])
        bad_segments = list(segments); bad_segments[0] = (segments[0][0], segments[0][1], bytes(4))
        with self.assertRaises(ValueError): attributes.project_tables(bad_segments)

    def test_scopes_and_missing_sources(self):
        cases = [({}, 7, 0), ({'omit': ('engine.white-flowers-isolated-values',)}, 6, 1),
                 ({'omit': ('graphics.indexed-archives',)}, 2, 1),
                 ({'bad_source': True}, 0, 0), ({'profile': 'usa'}, 0, 0)]
        for options, count, missing in cases:
            with self.subTest(options=options), tempfile.TemporaryDirectory(prefix='indexed-attributes-') as directory:
                page = fixture(Path(directory), **options); parts = page.parent / 'index.html.parts'
                home = parse_page(page); index = parse_page(parts / 'indexed-graphics.html')
                self.assertNotIn('indexed_tile_attributes', home); self.assertNotIn('indexed_tile_attributes', index)
                row = next(item for item in home['resources'] if item['id'] == 'graphics.indexed-tile-attributes')
                self.assertEqual(row['viewer_scope'], 'indexed-attributes')
                if count:
                    data = parse_page(parts / 'indexed-attributes.html')['indexed_tile_attributes']
                    self.assertEqual(len(data['tables']), count); self.assertEqual(len(data['missing']), missing)
                    self.assertEqual(sum('archive_link' in item for item in data['tables']), count if 'omit' not in options else
                                     (0 if 'graphics.indexed-archives' in options['omit'] else count))
                else:
                    self.assertTrue(row['status'].startswith(('Invalid indexed attribute', 'Preview unavailable')))


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class IndexedAttributesBrowserTest(unittest.TestCase):
    def test_navigation_stored_nibbles_and_graphics_crosslinks(self):
        with tempfile.TemporaryDirectory(prefix='indexed-attributes-browser-') as directory:
            work = Path(directory); page = fixture(work); browser = Chrome(work, page)
            try:
                target = browser.evaluate('''(()=>{const q=s=>document.querySelector(s);
                  const link=q('[data-resource-id="graphics.indexed-tile-attributes"] a');
                  if(!link||!link.href.includes('indexed-attributes.html'))throw Error('resource route');return link.href;})()''')
                browser.command('Page.navigate', {'url': target})
                wait_for(browser, "data.wiki.current==='indexed-attributes' && !document.querySelector('#indexed-attributes').hidden")
                href = browser.evaluate('''(()=>{const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change',{bubbles:true}));
                  if(q('#indexed-attributes>details>summary time').dateTime!=='2026-11-27T22:45:22Z'||
                     q('#indexed-attribute-table').options.length!==7||q('#indexed-attribute-rows').rows.length!==4||
                     !q('#indexed-attribute-getter').textContent.includes('0xA'))throw Error('direct table');
                  q('#indexed-attribute-table').value='4';change(q('#indexed-attribute-table'));
                  q('#indexed-attribute-program').value='43';change(q('#indexed-attribute-program'));
                  const rows=q('#indexed-attribute-rows').rows;
                  if(rows.length!==8||!rows[0].textContent.includes('High')||!rows[0].textContent.includes('0x1')||
                     !rows[1].textContent.includes('Low')||!q('#indexed-attribute-source').textContent.includes('0x460B20'))throw Error('localized table');
                  return q('#indexed-attribute-graphics').href;})()''')
                browser.command('Page.navigate', {'url': href})
                wait_for(browser, "data.wiki.current==='indexed-graphics' && document.querySelector('#indexed-program').value==='43'")
                back = browser.evaluate('''(()=>{const q=s=>document.querySelector(s);
                  if(!q('#indexed-detail').textContent.includes('0x460BD4')||q('#indexed-attribute-link-container').hidden)throw Error('graphics match');
                  return q('#indexed-attribute-link').href;})()''')
                browser.command('Page.navigate', {'url': back})
                wait_for(browser, "data.wiki.current==='indexed-attributes' && document.querySelector('#indexed-attribute-table').value==='4' && document.querySelector('#indexed-attribute-program').value==='43'")
                self.assertTrue(browser.evaluate("document.querySelector('#indexed-attributes>details').open"))
            finally:
                browser.close()


if __name__ == '__main__':
    unittest.main()
