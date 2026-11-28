"""Check the focused rectangle bank, source rows and malformed inputs."""
import hashlib
import json
import shutil
import struct
import tempfile
import unittest
from pathlib import Path

from view_rom_data import generate, project_direct_rectangle_bank, viewer_section_updates
from viewer_browser_harness import Chrome


def rectangle(width, height):
    return bytes((height, width)) + struct.pack(f'<{width * height}H', *range(width * height))


def fixture_chunks():
    # Synthetic values use the documented record dimensions and directory layout.
    literals = [(0x0dc8a4 + 8 * i, 'reader_literals', struct.pack('<I', 0x08ae6f60) * (1 + (i < 11)))
                for i in range(49)]
    directories = [
        (0x39ae3c, 'rectangle_directories', b''.join(struct.pack('<IB3s', 0x08ae7994, row, b'\x11\x22\x33') for row in range(6))),
        (0x39bcb4, 'rectangle_directories', struct.pack('<2I', 0x08ae6f60, 0x08ae6f7c)),
        (0x39fa20, 'rectangle_directories', struct.pack('<8I', *([0x08ae8c64] * 8))),
        (0x39fa40, 'rectangle_row_selectors', bytes((0, 1, 2, 3, 4, 5, 6, 6))),
        (0x3a4a3c, 'rectangle_directories', struct.pack('<5I', 0x08ae7674, 0x08ae7b94, 0x08ae7ba4, 0x08ae7bb4, 0x08ae7bc4)),
    ]
    grids = [(0xae6f60, 12, 1), (0xae6f7c, 12, 1), (0xae7478, 15, 2), (0xae7684, 6, 1),
             (0xae787c, 3, 5), (0xae7b94, 6, 1), (0xae7ba4, 6, 1), (0xae7bb4, 6, 1),
             (0xae7bc4, 6, 1), (0xae8648, 15, 6), (0xae8754, 15, 2), (0xae8c64, 3, 7)]
    return literals + directories + [(offset, 'rectangle_cells', rectangle(w, h)) for offset, w, h in grids]


def pack_segments(chunks):
    content = b''.join(raw for _, _, raw in chunks)
    sources = [{'offset': offset, 'role': role, 'length': len(raw), 'sha256': hashlib.sha256(raw).hexdigest()}
               for offset, role, raw in chunks]
    return content, sources


def generate_fixture(work, *, split=True, invalid=False, bank=True, previous=True):
    dataset = work / 'dataset'; dataset.mkdir()
    resources = []
    if bank:
        resources.append(('graphics.direct-rectangle-bank', 'direct-rectangle-bank', fixture_chunks()))
    if previous:
        chunks = []
        for offset, w, h in [(0xae7674, 6, 1), (0xae7994, 3, 7)]:
            raw = rectangle(w, h)
            chunks.extend(((offset, 'rectangle-header', raw[:2]), (offset + 2, 'rectangle-cells', raw[2:])))
        resources.append(('graphics.tilemap-type2-transfers', 'tilemap-type2-transfers', chunks))
    manifest = 'profile="europe"\nextractor_revision=378\ndecoder_revision=377\n'
    for resource, name, chunks in resources:
        content, sources = pack_segments(chunks)
        filename = name + '.bin'
        (dataset / filename).write_bytes(content)
        manifest += (f'[[resources]]\nid="{resource}"\npath="{filename}"\nformat="{name}-v1-bin"\n'
                     f'sha256="{hashlib.sha256(content).hexdigest()}"\n')
        if invalid and name == 'direct-rectangle-bank':
            sources[-1]['sha256'] = '0' * 64
        for source in sources:
            manifest += '[[resources.sources]]\n' + ''.join(f'{key}={json.dumps(value)}\n' for key, value in source.items())
    (dataset / 'manifest.toml').write_text(manifest)
    return generate(dataset, work / 'view', split=split)


def payload(path):
    return json.JSONDecoder().raw_decode(path.read_text().split('const data = ', 1)[1])[0]


class DirectRectangleBankTest(unittest.TestCase):
    def test_preserves_geometry_cells_directory_values_and_literals(self):
        parts = fixture_chunks()
        result = project_direct_rectangle_bank(*pack_segments(parts), None)
        for item, (offset, _, raw) in zip(result['maps'], parts[-12:], strict=True):
            self.assertEqual(item['source_offset'], offset)
            self.assertEqual(item['cell_offset'], offset + 2)
            self.assertEqual((item['height'], item['width']), tuple(raw[:2]))
            self.assertEqual(item['cells'], list(range(raw[0] * raw[1])))
        self.assertEqual(sum(len(item['cells']) for item in result['maps']), 240)
        slots = [directory['slots'] for directory in result['directories']]
        self.assertEqual(list(map(len, slots)), [6, 2, 8, 5])
        self.assertEqual(slots[0][5], {'source_offset': 0x39ae64, 'pointer': 0x08ae7994, 'target': 0xae7994,
                                      'row': 5, 'row_source': 0x39ae68, 'extra_bytes': [17, 34, 51]})
        self.assertEqual(slots[2][7]['row'], 6)
        self.assertEqual(slots[2][7]['row_source'], 0x39fa47)
        self.assertIsNone(slots[3][1]['row_source'])
        self.assertEqual(len(result['literals']), 60)
        self.assertEqual(result['literals'][1], {'source_offset': 0x0dc8a8, 'value': 0x08ae6f60})

    def test_rejects_source_hash_invalid_target_invalid_row_and_geometry(self):
        content, sources = pack_segments(fixture_chunks())
        sources[-1]['sha256'] = '0' * 64
        with self.assertRaisesRegex(ValueError, 'SHA-256'):
            project_direct_rectangle_bank(content, sources, None)
        for source, index, value, message in [(0x39bcb4, 0, 0, 'target or row'), (0x39fa40, 5, 7, 'target or row'),
                                               (0xae6f60, 0, 2, 'dimensions')]:
            parts = [(offset, role, raw[:index] + bytes((value,)) + raw[index+1:] if offset == source else raw)
                     for offset, role, raw in fixture_chunks()]
            with self.subTest(source=source), self.assertRaisesRegex(ValueError, message):
                project_direct_rectangle_bank(*pack_segments(parts), None)

    def test_scope_requires_valid_bank_and_uses_only_selected_prior_grids(self):
        for mode in ('valid', 'no-type2', 'invalid', 'old'):
            with self.subTest(mode=mode), tempfile.TemporaryDirectory() as directory:
                index = generate_fixture(Path(directory), invalid=mode == 'invalid', bank=mode != 'old', previous=mode != 'no-type2')
                parts = index.parent / (index.name + '.parts')
                self.assertNotIn('direct_rectangle_bank', payload(index))
                self.assertNotIn('direct_rectangle_bank', payload(parts / 'interface.html'))
                page = parts / 'interface-rectangle-bank.html'
                if mode == 'old':
                    self.assertFalse(page.exists())
                    continue
                scope = payload(page)
                bank = scope['direct_rectangle_bank']
                self.assertNotIn('extra_tilemaps', scope)
                self.assertNotIn('bg_graphics', scope)
                if mode == 'invalid':
                    self.assertIsNone(bank)
                    self.assertTrue(scope['resources'][0]['status'].startswith('Invalid direct rectangle bank'))
                else:
                    self.assertEqual(len(bank['maps']), 14 if mode == 'valid' else 12)


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class DirectRectangleBankBrowserTest(unittest.TestCase):
    def test_slots_open_correct_rows_literals_badges_and_legacy_reuse(self):
        for split in (True, False):
            with self.subTest(split=split), tempfile.TemporaryDirectory() as directory:
                work = Path(directory)
                index = generate_fixture(work, split=split)
                page = index.parent / (index.name + '.parts/interface-rectangle-bank.html') if split else index
                expected = viewer_section_updates()['interface-rectangle-bank']['tilemaps']['at']
                browser = Chrome(work, page)
                try:
                    self.assertEqual(browser.evaluate('''(()=>{
                      const q=s=>document.querySelector(s), map=q('#tilemap-id'), canvas=q('#tilemap-canvas');
                      const picker=q('#direct-rectangle-bank-directory'), rows=q('#direct-rectangle-bank-slot-rows').rows;
                      if(map.options.length!==14||rows.length!==6||q('#direct-rectangle-bank-word-rows').rows.length!==60)
                        throw Error('scope size or duplicate prior grids');
                      rows[5].querySelector('a').click();
                      const detail=q('#tilemap-cell-detail'), grid=q('#tilemap-cells');
                      if(!map.selectedOptions[0].textContent.includes('0xAE7994')||!q('#tilemaps>details').open||
                         !detail.textContent.includes('Cell (0, 5)')||grid.rows[5].getAttribute('aria-current')!=='true')
                        throw Error('prior rectangle row');
                      picker.value='2';picker.dispatchEvent(new Event('change',{bubbles:true}));
                      if(rows.length!==8||rows[6].cells[4].textContent!=='0x39FA46')throw Error('separate selector');
                      rows[6].querySelector('a').click();
                      if(!detail.textContent.includes('source 0xAE8C8A')||grid.rows[6].getAttribute('aria-current')!=='true'||
                         canvas.width!==24||canvas.height!==56)throw Error('copied row grid');
                      q('#direct-rectangle-bank-word-rows a').click();
                      if(!map.selectedOptions[0].textContent.includes('0xAE6F60')||grid.querySelector('[aria-current]'))
                        throw Error('reader word link');
                      if(document.getElementById('wiki-pages').hidden===false){
                        const group=q('[data-category="Interface"]');
                        const nav=group.querySelector('a[href$="interface-rectangle-bank.html#tilemaps"] time');
                        if(!nav||nav.dateTime!==EXPECTED)throw Error('navbar badge');
                      }
                      for(const id of ['tilemaps','direct-rectangle-bank-slots','direct-rectangle-bank-literals']){
                        const badge=q('#'+id+'>details>summary time');
                        if(!badge||badge.dateTime!==EXPECTED)throw Error('section badge '+id);
                      }
                      q('#collapse-all').click();
                      return q('#tilemaps>details').open?'open':'PASS';})()'''.replace('EXPECTED', json.dumps(expected))), 'PASS')
                finally:
                    browser.close()
