"""Check indexed control records and their bidirectional map links."""
import copy
import hashlib
import json
import shutil
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from view_rom_data import generate, project_indexed_tilemap_controls
from test_viewer_tilemap_stream_bank_browser import fixture as bank_fixture, payload
from viewer_browser_harness import Chrome


def catalog():
    def slot(index, pointer, target=None):
        descriptor = 0x3000 + index * 8
        return {'pointer_offset': 0x6000 + index * 4, 'pointer': pointer,
                'row': {'source_offset': 0x4000 + index * 12, 'field0': 4, 'field1': 1, 'field2': 2048,
                        'descriptor_pointer': 0x08000000 + descriptor, 'field8': 0x06000000},
                'descriptor': {'source_offset': descriptor, 'tilemap_pointer': pointer if target is None else target,
                               'parameter16': 1019, 'parameter8': 6, 'opaque_byte': 0}}
    return {'slots': [slot(0, 0x08001000), slot(1, 0x080007d0),
                      slot(2, 0x08001000, 0x08001234), slot(3, 0x08009999), slot(4, 0x08001000)]}


def fixture(work, *, invalid=False, invalid_bank=False):
    bank_fixture(work, invalid_bank=invalid_bank)
    dataset = work / 'dataset'; value = catalog()
    if invalid:
        value['slots'][0]['row']['field1'] = True
    observed = {'source_offset': 2000, 'width': 1, 'height': 1, 'mode': 4,
                'runs': [{'control': 0x4001, 'value': 0}]}
    entries = [('indexed-tilemap-controls', value), ('observed-tilemap', observed)]
    with (dataset / 'manifest.toml').open('a') as stream:
        for name, body in entries:
            content = json.dumps(body).encode(); (dataset / (name + '.json')).write_bytes(content)
            stream.write(f'[[resources]]\nid="graphics.{name}"\npath="{name}.json"\n'
                         f'format="{name}-v1-json"\nsha256="{hashlib.sha256(content).hexdigest()}"\n')
    return generate(dataset, work / 'indexed-view', split=True)


class IndexedTilemapProjectionTest(unittest.TestCase):
    def test_validation_preserves_unresolved_pointers(self):
        self.assertEqual(project_indexed_tilemap_controls(catalog()), catalog())
        mutations = [lambda d: d['slots'][0].update(pointer_offset=True),
                     lambda d: d['slots'][0]['row'].update(field2=65536),
                     lambda d: d['slots'][0]['row'].update(source_offset=-1),
                     lambda d: d['slots'][0]['descriptor'].update(opaque_byte=256),
                     lambda d: d.update(slots=d['slots'] * 103)]
        for mutate in mutations:
            value = catalog(); mutate(value)
            with self.subTest(value=value), self.assertRaises(ValueError):
                project_indexed_tilemap_controls(value)

    def test_scope_and_invalid_resource(self):
        for invalid in (False, True):
            with self.subTest(invalid=invalid), tempfile.TemporaryDirectory() as directory:
                page = fixture(Path(directory), invalid=invalid); parts = page.parent / 'index.html.parts'
                home, interface, world = map(payload, (page, parts / 'interface.html', parts / 'world.html'))
                row = next(item for item in home['resources'] if item['id'] == 'graphics.indexed-tilemap-controls')
                self.assertEqual(row['viewer_scope'], 'interface')
                self.assertEqual(row['status'].startswith('Available'), not invalid)
                self.assertEqual(bool(interface['indexed_tilemap_controls']), not invalid)
                self.assertNotIn('indexed_tilemap_controls', home)
                self.assertNotIn('indexed_tilemap_controls', world)
                menu = next(item for item in home['wiki']['pages'] if item['id'] == 'interface')
                self.assertEqual(any(item['id'] == 'indexed-tilemap-controls' for item in menu['sections']), not invalid)


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class IndexedTilemapBrowserTest(unittest.TestCase):
    def test_forward_reverse_and_partial_links(self):
        with tempfile.TemporaryDirectory(prefix='viewer-indexed-maps-') as directory:
            work = Path(directory); page = fixture(work); parts = page.parent / 'index.html.parts'
            browser = Chrome(work, parts / 'interface.html')
            try:
                self.assertEqual(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s);
                  q('[data-resource-id="graphics.indexed-tilemap-controls"] a[href="#indexed-tilemap-controls"]').click();
                  const section=q('#indexed-tilemap-controls'),rows=q('#indexed-tilemap-rows').rows;
                  if(section.hidden || !section.querySelector(':scope > details').open || rows.length!==5 ||
                     !rows[0].cells[1].textContent.includes('halfword 0x800'))throw Error('catalog');
                  if(rows[2].querySelector('a') || !rows[2].textContent.includes('do not identify') ||
                     rows[3].querySelector('a') || !rows[3].textContent.includes('unavailable'))throw Error('unresolved');
                  rows[0].querySelector('a').click();
                  if(!q('#tilemaps>details').open || !q('#tilemap-id').selectedOptions[0].textContent.includes('0x1000') ||
                     q('#tilemap-indexed-references').hidden || q('#tilemap-indexed-references').textContent!=='Indexed controls: slot 0, slot 4')throw Error('forward stream');
                  q('#tilemap-indexed-references a:last-child').click();
                  if(!section.closest('.section-group').open || !section.querySelector('details').open)throw Error('reverse');
                  rows[1].querySelector('a').click();
                  if(!q('#tilemap-id').selectedOptions[0].textContent.includes('0x7D0') ||
                     q('#tilemap-indexed-references').textContent!=='Indexed controls: slot 1')throw Error('observed source');
                  return 'PASS';
                })()'''), 'PASS')
            finally:
                browser.close()

    def test_missing_and_ambiguous_maps_do_not_link(self):
        with tempfile.TemporaryDirectory(prefix='viewer-indexed-missing-') as directory:
            work = Path(directory); page = fixture(work, invalid_bank=True)
            browser = Chrome(work, page.parent / 'index.html.parts/interface.html')
            try:
                self.assertTrue(browser.evaluate("!document.querySelector('#indexed-tilemap-slot-0 a') && !!document.querySelector('#indexed-tilemap-slot-1 a')"))
            finally:
                browser.close()
        with tempfile.TemporaryDirectory(prefix='viewer-indexed-ambiguous-') as directory:
            work = Path(directory); page = fixture(work); target = page.parent / 'index.html.parts/interface.html'
            text = target.read_text(); start = text.index('const data = ') + len('const data = ')
            data, length = json.JSONDecoder().raw_decode(text[start:])
            data['extra_tilemaps']['stream_bank']['maps'].append(copy.deepcopy(data['extra_tilemaps']['stream_bank']['maps'][0]))
            target.write_text(text[:start] + json.dumps(data) + text[start + length:])
            browser = Chrome(work, target)
            try:
                self.assertTrue(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s),select=q('#tilemap-id');
                  if(q('#indexed-tilemap-slot-0 a') || !q('#indexed-tilemap-slot-0').textContent.includes('ambiguous'))return false;
                  const i=[...select.options].findIndex(o=>o.textContent.includes('0x1000'));
                  select.selectedIndex=i;select.dispatchEvent(new Event('change'));
                  return q('#tilemap-indexed-references').hidden;
                })()'''))
            finally:
                browser.close()
