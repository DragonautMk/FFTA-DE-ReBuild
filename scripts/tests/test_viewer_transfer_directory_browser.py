"""Check transfer selector links and their decoded tile previews."""
import copy
import hashlib
import json
import shutil
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from view_rom_data import generate, project_transfer_directory
from test_viewer_observed_lz_browser import payload
from test_viewer_transfer_lz_browser import fixture as transfer_fixture
from test_viewer_ui_context_browser import wait_for
from viewer_browser_harness import Chrome


def catalog():
    return {
        'slots': [{'row_pointer': 0x08001000, 'row_count': 3},
                  {'row_pointer': 0x08001000, 'row_count': 1},
                  {'row_pointer': 0, 'row_count': 0},
                  {'row_pointer': 0x08002000, 'row_count': 1}],
        'rows': [{'source_offset': 0x1000, 'field0': 3, 'field1': 1, 'field2': 32,
                  'descriptor_pointer': 0x08003000, 'field8': 0x06010000},
                 {'source_offset': 0x100c, 'field0': 4, 'field1': 2, 'field2': 64,
                  'descriptor_pointer': 0x08003008, 'field8': 0x06011000},
                 {'source_offset': 0x1018, 'field0': 3, 'field1': 0, 'field2': 32,
                  'descriptor_pointer': 0x08003010, 'field8': 0x06012000}],
        'descriptors': [{'source_offset': 0x3000, 'asset_pointer': 0x08004ffc,
                         'parameter16': 4, 'parameter8': 2, 'opaque_byte': 7,
                         'wrapper': {'source_offset': 0x4ffc, 'kind': 2, 'flags': 2, 'opaque_halfword': 65533}},
                        {'source_offset': 0x3008, 'asset_pointer': 0, 'parameter16': 0,
                         'parameter8': 0, 'opaque_byte': 0, 'wrapper': None},
                        {'source_offset': 0x3010, 'asset_pointer': 0x08af4994, 'parameter16': 0,
                         'parameter8': 0, 'opaque_byte': 0,
                         'wrapper': {'source_offset': 0xaf4994, 'kind': 2, 'flags': 2, 'opaque_halfword': 0}}],
    }


def fixture(work, *, invalid=False, missing_lz=False):
    transfer_fixture(work, bad_source=missing_lz)
    dataset = work / 'dataset'; value = catalog()
    if invalid:
        value['rows'][0]['field0'] = True
    content = json.dumps(value).encode(); (dataset / 'transfer-directory.json').write_bytes(content)
    manifest = dataset / 'manifest.toml'
    with manifest.open('a') as stream:
        stream.write('[[resources]]\nid="graphics.tilemap-transfer-directory"\npath="transfer-directory.json"\n'
                     'format="tilemap-transfer-directory-v1-json"\n'
                     f'sha256="{hashlib.sha256(content).hexdigest()}"\n')
    return generate(dataset, work / 'directory-view', split=True)


class TransferDirectoryProjectionTest(unittest.TestCase):
    def test_field_validation_and_bounded_input(self):
        self.assertEqual(project_transfer_directory(catalog()), catalog())
        mutations = [lambda d: d['slots'][0].update(row_count=True),
                     lambda d: d['slots'][0].update(row_pointer=-1),
                     lambda d: d['rows'][0].update(field2=65536),
                     lambda d: d['rows'].append(copy.deepcopy(d['rows'][0])),
                     lambda d: d['descriptors'].append(copy.deepcopy(d['descriptors'][0])),
                     lambda d: d['descriptors'][0]['wrapper'].update(source_offset=1),
                     lambda d: d['descriptors'][0]['wrapper'].update(flags=256),
                     lambda d: d.update(slots=[{'row_pointer': 0, 'row_count': 255}] * 17)]
        for mutate in mutations:
            value = catalog(); mutate(value)
            with self.subTest(value=value), self.assertRaises(ValueError):
                project_transfer_directory(value)

    def test_scope_and_invalid_resource(self):
        for invalid in (False, True):
            with self.subTest(invalid=invalid), tempfile.TemporaryDirectory() as directory:
                page = fixture(Path(directory), invalid=invalid); parts = page.parent / 'index.html.parts'
                home, interface, world = map(payload, (page, parts / 'interface.html', parts / 'world.html'))
                row = next(item for item in home['resources'] if item['id'] == 'graphics.tilemap-transfer-directory')
                self.assertEqual(row['viewer_scope'], 'interface')
                self.assertEqual(row['status'].startswith('Available'), not invalid)
                self.assertEqual(bool(interface['transfer_directory']), not invalid)
                self.assertNotIn('transfer_directory', home)
                self.assertNotIn('transfer_directory', world)
                menu = next(item for item in home['wiki']['pages'] if item['id'] == 'interface')
                self.assertEqual(any(item['id'] == 'transfer-directory' for item in menu['sections']), not invalid)


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class TransferDirectoryBrowserTest(unittest.TestCase):
    def test_aliases_empty_rows_and_decoded_tiles(self):
        with tempfile.TemporaryDirectory(prefix='viewer-transfer-directory-') as directory:
            work = Path(directory); page = fixture(work); parts = page.parent / 'index.html.parts'
            browser = Chrome(work, parts / 'interface.html')
            try:
                self.assertEqual(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change',{bubbles:true}));
                  q('[data-resource-id="graphics.tilemap-transfer-directory"] a[href="#transfer-directory"]').click();
                  if(q('#transfer-directory').hidden || !q('#transfer-directory>details').open ||
                     q('#transfer-directory-slot').options.length!==4 || q('#transfer-directory-rows').rows.length!==3)
                    throw Error('directory view');
                  const rows=q('#transfer-directory-rows').rows;
                  if(rows[0].cells[3].textContent!=='1' || !rows[0].cells[2].textContent.includes('wrapper kind 0x2') ||
                     rows[1].querySelector('a') || !rows[1].cells[4].textContent.includes('unavailable'))throw Error('stored links');
                  rows[0].querySelector('a').click();
                  const pixel=[...q('#obj-sheet').getContext('2d').getImageData(0,0,1,1).data].join(',');
                  if(!q('#obj-tiles>details').open || q('#obj-catalog').value!=='lz' || q('#obj-bank').value!=='5' ||
                     q('#obj-lz-offset').value!=='0' || pixel!=='17,17,17,255')throw Error('decoded preview');
                  rows[2].querySelector('a').click();
                  if(q('#obj-bank').value!=='0' || !q('#obj-detail').textContent.includes('observed-lz-block.json'))throw Error('previously observed stream');
                  q('#transfer-directory-slot').value='2';change(q('#transfer-directory-slot'));
                  if(q('#transfer-directory-rows').rows.length || !q('#transfer-directory-source').textContent.includes('no rows'))throw Error('empty slot');
                  q('#transfer-directory-slot').value='3';change(q('#transfer-directory-slot'));
                  if(!q('#transfer-directory-rows').textContent.includes('not present'))throw Error('missing row');
                  q('#transfer-directory-slot').value='1';change(q('#transfer-directory-slot'));
                  if(q('#transfer-directory-rows').rows[0].cells[3].textContent!=='0')throw Error('reverse alias');
                  return 'PASS';
                })()'''), 'PASS')
                bookmark = browser.evaluate("document.querySelector('#transfer-directory>details>p>a').href")
                browser.command('Page.navigate', {'url': bookmark})
                wait_for(browser, "typeof data !== 'undefined' && document.querySelector('#transfer-directory-slot').value==='1'")
                self.assertTrue(browser.evaluate("document.querySelector('#transfer-directory-slot').value==='1' && document.querySelector('#transfer-directory>details').open"))
            finally:
                browser.close()

    def test_unavailable_lz_has_no_preview_link(self):
        with tempfile.TemporaryDirectory(prefix='viewer-transfer-directory-partial-') as directory:
            work = Path(directory); page = fixture(work, missing_lz=True)
            browser = Chrome(work, page.parent / 'index.html.parts/interface.html')
            try:
                self.assertTrue(browser.evaluate("!document.querySelector('#transfer-directory').hidden && !document.querySelector('#transfer-directory-rows').rows[0].querySelector('a') && document.querySelector('#transfer-directory-rows').rows[2].querySelector('a') && document.querySelector('#transfer-directory-rows').textContent.includes('unavailable')"))
            finally:
                browser.close()
