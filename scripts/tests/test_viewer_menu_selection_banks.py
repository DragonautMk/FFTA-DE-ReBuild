"""Check the menu-bank layout, decoded rows, and source-linked navigation."""
import hashlib
import json
from pathlib import Path
import shutil
import struct
import tempfile
import unittest

from test_viewer_direct_rectangle_bank import pack_segments, payload
from view_rom_data import generate, project_menu_selection_banks, viewer_section_updates
from viewer_browser_harness import Chrome


def chunks():
    literal_offsets = (0x3bd94, 0x3bddc, 0x3be30, 0x3be70, 0x3beb8, 0x3bef8, 0x3bf40, 0x3bfec,
                       0x3c2f4, 0x3c32c, 0x3c354, 0x3c37c, 0x3c3b4, 0x3c3dc, 0x3c414,
                       0x3e040, 0x3e068, 0x3e090, 0x3e0b8, 0x3e0f8, 0x3e2f8, 0x3ea10, 0x3eb58, 0x497a0, 0x498dc)
    literal_targets = (0x3958e8, 0x395978, 0x3958d0, 0x395978, 0x3958bc, 0x395978, 0x395908, 0x395978,
                       0x395908, 0x3958bc, 0x3958d0, 0x3958e8, 0x3958d0, 0x3958e8, 0x3958e8,
                       0x39576c, 0x395848, 0x395850, 0x395858, 0x395820, 0x395874,
                       (0x395a1c, 0x395820, 0x39576c, 0x395848, 0x395850, 0x395858), 0x395858, 0x395860, 0x395860)
    result = []
    for offset, targets in zip(literal_offsets, literal_targets, strict=True):
        words = targets if isinstance(targets, tuple) else (targets,)
        result.append((offset, 'menu-selection-literal', b''.join(struct.pack('<I', 0x08000000 + target) for target in words)))
    result.append((0x39576c, 'menu-selection-tables', bytes(range(256)) * 2 + bytes(range(56))))
    result.append((0x395a1c, 'menu-row-selectors', bytes((99, 0, 4, 255, 1, 1, 12, 13)) * 5))
    return result


def fixture(work, *, bad=False, legacy=False):
    dataset = work / 'dataset'; dataset.mkdir()
    content, sources = pack_segments(chunks())
    path = dataset / 'menu-selection-banks.bin'; path.write_bytes(content)
    manifest = ('profile="europe"\nextractor_revision=380\ndecoder_revision=379\n[[resources]]\n'
                'id="gameplay.menu-selection-banks"\nformat="menu-selection-banks-v1-bin"\n'
                f'path="{path.name}"\nsha256="{hashlib.sha256(content).hexdigest()}"\n')
    if bad:
        sources[-2]['sha256'] = '0' * 64
    for source in sources:
        manifest += '[[resources.sources]]\n' + ''.join(f'{key}={json.dumps(value)}\n' for key, value in source.items())
    (dataset / 'manifest.toml').write_text(manifest)
    return generate(dataset, work / 'view', split=not legacy)


class MenuSelectionProjectionTest(unittest.TestCase):
    def test_decoded_tables_round_trip_and_keep_selector_values(self):
        spans = chunks()
        catalog = project_menu_selection_banks(*pack_segments(spans))
        tables = catalog['tables']
        self.assertEqual(len(tables), 13)
        self.assertEqual(sum(len(table['rows']) for table in tables), 101)
        self.assertEqual(b''.join(bytes(row) for table in tables[:-1] for row in table['rows']), spans[-2][2])
        self.assertEqual(bytes(tables[-1]['rows'][0]), spans[-1][2][:8])
        self.assertEqual(catalog['selector_targets'], [0x395820, 0x39576c, 0x395848, 0x395850, 0x395858])
        self.assertEqual(len(catalog['literals']), 30)
        self.assertEqual(catalog['literals'][0], {'source_offset': 0x3bd94, 'target': 0x3958e8})

    def test_rejects_swapped_source_word_and_truncated_owner(self):
        parts = chunks()
        parts[0] = (*parts[0][:2], struct.pack('<I', 0x08395908))
        with self.assertRaisesRegex(ValueError, 'reader words'):
            project_menu_selection_banks(*pack_segments(parts))
        parts = chunks()
        parts[-2] = (*parts[-2][:2], parts[-2][2][:-1])
        with self.assertRaisesRegex(ValueError, 'menu selection sources'):
            project_menu_selection_banks(*pack_segments(parts))

    def test_optional_scope_and_source_hash_isolation(self):
        for bad in (False, True):
            with self.subTest(bad=bad), tempfile.TemporaryDirectory() as directory:
                index = fixture(Path(directory), bad=bad)
                parts = index.parent / (index.name + '.parts')
                root = payload(index)
                self.assertNotIn('menu_selection_banks', root)
                self.assertNotIn('menu_selection_banks', payload(parts / 'world.html'))
                data = payload(parts / 'interface-menu-selection.html')
                self.assertEqual(data['menu_selection_banks'] is None, bad)
                self.assertEqual(data['resources'][0]['status'].startswith('Invalid menu selection banks'), bad)
                topic = next(page for page in root['wiki']['pages'] if page['id'] == 'interface-menu-selection')
                self.assertEqual(len(topic['sections']), 0 if bad else 3)


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class MenuSelectionBrowserTest(unittest.TestCase):
    def test_selector_links_highlight_rows_and_show_decoded_fields(self):
        for legacy in (False, True):
            with self.subTest(legacy=legacy), tempfile.TemporaryDirectory(prefix='viewer-menu-banks-') as directory:
                work = Path(directory); index = fixture(work, legacy=legacy)
                page = index if legacy else index.parent / (index.name + '.parts/interface-menu-selection.html')
                expected = viewer_section_updates()['interface-menu-selection']['menu-selection-rows']['at']
                browser = Chrome(work, page)
                try:
                    result = browser.evaluate('''(()=>{
                      const q=s=>document.querySelector(s), bank=q('#menu-selection-bank'), raw=q('#menu-selection-raw');
                      const rows=q('#menu-selection-values').rows, links=q('#menu-selection-link-values').rows;
                      const readers=q('#menu-selection-reader-values').rows, row=q('#menu-selection-row');
                      if(bank.options.length!==13||links.length!==5||readers.length!==30)throw Error('scope');
                      if(links[0].cells[1].textContent!=='0'||links[0].cells[1].firstChild.nodeName==='A'||
                         links[0].cells[3].textContent!=='255'||links[0].cells[3].firstChild.nodeName==='A')throw Error('link bounds');
                      links[0].cells[2].querySelector('a').click();
                      if(bank.value!=='0'||row.value!=='3'||rows[3].getAttribute('aria-current')!=='true'||
                         !q('#menu-selection-rows>details').open||rows[3].cells[2].textContent!=='0x6F6E'||
                         !raw.textContent.startsWith('0x6C 0x6D 0x6E'))throw Error('linked physical row');
                      readers[0].querySelector('a').click();
                      if(!bank.selectedOptions[0].textContent.includes('0x3958E8')||row.value!=='0'||rows.length!==8)
                        throw Error('reader link');
                      bank.value='5';bank.dispatchEvent(new Event('change'));
                      if(!q('#menu-selection-fields').textContent.includes('Scale term')||
                         Number(rows[0].cells[2].textContent)+100!==Number(rows[0].cells[3].textContent))throw Error('scale');
                      for(const id of ['menu-selection-rows','menu-selection-links','menu-selection-readers']){
                        if(q('#'+id+'>details>summary time').dateTime!==EXPECTED)throw Error('badge '+id);
                      }
                      return 'PASS';
                    })()'''.replace('EXPECTED', json.dumps(expected)))
                    self.assertEqual(result, 'PASS')
                finally:
                    browser.close()
