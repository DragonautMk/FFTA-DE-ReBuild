"""Check background descriptor fields and navigation to related previews."""
import copy
import hashlib
import json
import shutil
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from view_rom_data import generate, project_bg_descriptors
from test_viewer_bg_transfers_browser import catalog as transfer_catalog
from viewer_browser_harness import Chrome


def catalog():
    def entry(index):
        absent = {'control_source': 0, 'data_source': 0, 'destination': 0}
        present = {'control_source': 500 + index, 'data_source': 600 + index, 'destination': 0x06000000}
        return {'graphics_source': 300 + index, 'paired_planes_source': 310 + index,
                'halfword_planes_source': 320 + index, 'palette_source': 330 + index,
                'auxiliary_patches_source': 340 + index, 'transfer': present if index == 0 else absent,
                'extra_transfers': [absent.copy() for _ in range(4)], 'variant_source': 0 if index == 0 else 350 + index,
                'selectors': [4 + index, 255 if index == 0 else 20 + index],
                'alternate_palette_key': 255 if index == 0 else index - 1, 'trailing_byte': 12 + index}
    return {'entries': [entry(index) for index in range(3)]}


def fixture(work, *, extra=True, invalid=False):
    dataset = work / 'dataset'; dataset.mkdir()
    descriptors = catalog()
    if invalid: descriptors['entries'][0]['selectors'] = [True, 1]
    items = [('graphics.bg-descriptors', 'bg-descriptors', 'bg-descriptors-json-v2', descriptors)]
    if extra:
        items += [('graphics.bg-main-graphics', 'bg-main-graphics', 'bg-main-graphics-v1-json',
                   {'descriptor_graphics': [0, 0, 0], 'graphics': [{'relative_source': 80, 'destination_offset': 0,
                                                                     'words': [0x11111111] * 8}]}),
                  ('graphics.bg-base-palettes', 'bg-base-palettes', 'bg-base-palettes-v2-json',
                   {'entries': [{'banks': [[0, 31] + [0] * 14]} for _ in range(3)]}),
                  ('graphics.bg-transfer-programs', 'bg-transfer-programs', 'bg-transfer-programs-v1-json', transfer_catalog())]
    manifest = 'profile="test"\nextractor_revision=1\ndecoder_revision=1\n'
    for rid, name, fmt, contents in items:
        content = json.dumps(contents).encode(); (dataset / (name + '.json')).write_bytes(content)
        manifest += (f'[[resources]]\nid="{rid}"\npath="{name}.json"\nformat="{fmt}"\n'
                     f'sha256="{hashlib.sha256(content).hexdigest()}"\n')
        if rid == 'graphics.bg-descriptors':
            manifest += '[[resources.sources]]\noffset=1000\nlength=264\nrole="descriptor"\nsha256="' + '0' * 64 + '"\n'
    (dataset / 'manifest.toml').write_text(manifest)
    index = generate(dataset, work / 'view', split=True)
    return index, index.parent / (index.name + '.parts/world.html')


def payload(path):
    return json.JSONDecoder().raw_decode(path.read_text().split('const data = ', 1)[1])[0]


class BackgroundDescriptorProjectionTest(unittest.TestCase):
    def test_projection_preserves_complete_records_and_rejects_invalid_data(self):
        value = catalog()
        self.assertEqual(project_bg_descriptors(value), value)
        mutations = [lambda c: c['entries'].clear(),
                     lambda c: c['entries'][0].__setitem__('graphics_source', 1),
                     lambda c: c['entries'][0].__setitem__('palette_source', True),
                     lambda c: c['entries'][0].__setitem__('variant_source', 1),
                     lambda c: c['entries'][0]['selectors'].__setitem__(0, 256),
                     lambda c: c['entries'][0]['extra_transfers'].pop(),
                     lambda c: c['entries'][0]['transfer'].__setitem__('destination', 0x06000001),
                     lambda c: c['entries'][0]['transfer'].__setitem__('data_source', 0)]
        for mutate in mutations:
            bad = copy.deepcopy(value); mutate(bad)
            with self.subTest(mutation=mutate), self.assertRaises(ValueError):
                project_bg_descriptors(bad)

    def test_scope_and_invalid_resource_isolation(self):
        for invalid in (False, True):
            with self.subTest(invalid=invalid), tempfile.TemporaryDirectory(prefix='viewer-bg-descriptor-scope-') as temp:
                index, world = fixture(Path(temp), invalid=invalid)
                self.assertNotIn('bg_descriptors', payload(index))
                data = payload(world)
                self.assertTrue(data['bg_graphics'])
                self.assertEqual(bool(data['bg_descriptors']), not invalid)
                row = next(r for r in data['resources'] if r['id'] == 'graphics.bg-descriptors')
                self.assertTrue(row['status'].startswith('Invalid' if invalid else 'Available'))
                if not invalid: self.assertEqual(data['bg_descriptors']['source_offset'], 1000)


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class BackgroundDescriptorBrowserTest(unittest.TestCase):
    def test_fields_selector_fallback_and_cross_navigation(self):
        with tempfile.TemporaryDirectory(prefix='viewer-bg-descriptor-browser-') as temp:
            work = Path(temp); _, world = fixture(work); browser = Chrome(work, world)
            try:
                self.assertEqual(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change'));
                  const row=q('[data-resource-id="graphics.bg-descriptors"]');row.querySelector('a').click();
                  if(!q('#bg-descriptors>details').open||q('#bg-descriptor-id').options.length!==3||
                     q('#bg-descriptor-sources').rows.length!==6||q('#bg-descriptor-transfers').rows.length!==5||
                     !q('#bg-descriptor-sources').rows[0].textContent.includes('0x514')||
                     !q('#bg-descriptor-sources').rows[5].textContent.includes('Absent'))throw Error('initial fields');
                  q('#bg-descriptor-environment').value='1';change(q('#bg-descriptor-environment'));
                  if(!q('#bg-descriptor-selectors').textContent.includes('selected for low bits 1: 0x4')||
                     !q('#bg-descriptor-selectors').textContent.includes('no verified alternate'))throw Error('fallback');
                  q('#bg-descriptor-id').value='1';change(q('#bg-descriptor-id'));
                  if(!q('#bg-descriptor-selectors').textContent.includes('selected for low bits 1: 0x15')||
                     !q('#bg-descriptor-sources').rows[5].textContent.includes('0x15F')||
                     q('#bg-descriptor-transfers').rows[0].cells[1].textContent!=='Absent')throw Error('other descriptor');
                  const link=label=>[...q('#bg-descriptor-related').querySelectorAll('a')].find(a=>a.textContent===label);
                  if(!link('Tiles')||!link('Base palette')||!link('Animated transfers')||link('Terrain seed'))throw Error('related links');
                  link('Tiles').click();if(q('#bg-descriptor').value!=='1'||!q('#bg-tiles>details').open||q('#bg-descriptor-navigation').hidden)throw Error('tiles link');
                  q('#bg-descriptor-open').click();if(q('#bg-descriptor-id').value!=='1'||!q('#bg-descriptors>details').open)throw Error('return link');
                  link('Animated transfers').click();if(q('#bg-transfer-descriptor').value!=='1'||!q('#bg-transfers>details').open)throw Error('transfer link');
                  q('#bg-descriptor-id').value='2';change(q('#bg-descriptor-id'));
                  if(link('Animated transfers'))throw Error('empty transfer target');
                  return 'PASS';
                })()'''), 'PASS')
            finally: browser.close()

    def test_standalone_descriptor(self):
        with tempfile.TemporaryDirectory(prefix='viewer-bg-descriptor-alone-') as temp:
            work = Path(temp); _, world = fixture(work, extra=False); browser = Chrome(work, world)
            try:
                self.assertTrue(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s);return !q('#bg-descriptors').hidden &&
                    q('#bg-descriptor-related').textContent.includes('No related previews') && q('#bg-tiles').hidden;
                })()'''))
            finally: browser.close()
