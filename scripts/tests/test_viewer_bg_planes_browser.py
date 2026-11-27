"""Check inherited background-plane previews and their source-data guards."""
import copy
import hashlib
import json
import shutil
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from view_rom_data import generate, project_bg_plane_programs
from viewer_browser_harness import Chrome


def catalog():
    def program(source, parent, packets):
        return {'relative_source': source, 'parent_descriptor': parent,
                'writes': {'kind': 'paired', 'header': 0x80000020, 'dictionary_offset': 32,
                           'payload': {'packets': packets}}}
    return {'paired': {'descriptor_programs': [0, 1, 1], 'programs': [
        program(0x1000, None, [{'byte_offset': 0, 'values': [0x00010000, 0x03ff0002, 0x0000f000]},
                               {'byte_offset': 0x2000, 'values': [0x00000001]}]),
        program(0x2000, 0, [{'byte_offset': 0, 'values': [0x00010401]}])]},
        'auxiliary': {'descriptor_programs': [0, 0, 0], 'programs': [
            {'relative_source': 0x3000, 'parent_descriptor': None,
             'writes': {'kind': 'halfwords', 'packets': [{'byte_offset': 0, 'values': [1]}]}}]}}


def fixture(work, *, graphic=True, invalid=None):
    dataset = work / 'dataset'; dataset.mkdir(); source = catalog()
    if invalid == 'paired': source['paired']['programs'][0]['writes']['dictionary_offset'] = None
    if invalid == 'terrain': source['auxiliary']['programs'][0]['writes']['packets'][0]['byte_offset'] = 1
    items = [('graphics.bg-packet-programs', 'bg-packet-programs', 'bg-packet-programs-v1-json', source)]
    if graphic:
        items += [('graphics.bg-main-graphics', 'bg-main-graphics', 'bg-main-graphics-v1-json',
                   {'descriptor_graphics': [0, 0, 0], 'graphics': [{'relative_source': 300,
                    'destination_offset': 0, 'words': [0x11111111] * 8 + [0x22222222] * 8 + [0] * 8}]}),
                  ('graphics.bg-base-palettes', 'bg-base-palettes', 'bg-base-palettes-v2-json',
                   {'entries': [{'banks': [[0, 0x001f, 0x03e0] + [0] * 13]} for _ in range(3)]})]
    manifest = 'profile="test"\nextractor_revision=1\ndecoder_revision=1\n'
    for rid, name, fmt, value in items:
        content = json.dumps(value).encode(); (dataset / (name + '.json')).write_bytes(content)
        manifest += (f'[[resources]]\nid="{rid}"\npath="{name}.json"\nformat="{fmt}"\n'
                     f'sha256="{hashlib.sha256(content).hexdigest()}"\n')
    (dataset / 'manifest.toml').write_text(manifest)
    index = generate(dataset, work / 'view', split=True)
    return index, index.parent / (index.name + '.parts/interface.html')


def payload(path):
    return json.JSONDecoder().raw_decode(path.read_text().split('const data = ', 1)[1])[0]


class BackgroundPlanesProjectionTest(unittest.TestCase):
    def test_order_inheritance_and_rejected_packets(self):
        value = catalog(); projected = project_bg_plane_programs(value)
        self.assertEqual(projected['descriptor_programs'], [0, 1, 1])
        self.assertEqual(projected['programs'][0]['packets'], value['paired']['programs'][0]['writes']['payload']['packets'])
        mutations = [lambda c: c['paired']['descriptor_programs'].__setitem__(0, 1),
                     lambda c: c['paired']['programs'][0].__setitem__('parent_descriptor', 1),
                     lambda c: c['paired']['programs'][0]['writes'].__setitem__('dictionary_offset', None),
                     lambda c: c['paired']['programs'][0]['writes']['payload']['packets'][0].__setitem__('byte_offset', 2),
                     lambda c: c['paired']['programs'][0]['writes']['payload']['packets'][0]['values'].__setitem__(0, True),
                     lambda c: c['paired']['programs'][0]['writes']['payload']['packets'][0].__setitem__('byte_offset', 0x4000)]
        for mutate in mutations:
            bad = copy.deepcopy(value); mutate(bad)
            with self.subTest(mutation=mutate), self.assertRaises(ValueError):
                project_bg_plane_programs(bad)

    def test_scope_and_partial_resource_isolation(self):
        for invalid in (None, 'paired', 'terrain'):
            with self.subTest(invalid=invalid), tempfile.TemporaryDirectory(prefix='viewer-bg-plane-scope-') as temp:
                index, interface = fixture(Path(temp), invalid=invalid); data = payload(interface)
                world = payload(interface.with_name('world-map.html'))
                self.assertNotIn('bg_plane_programs', payload(index))
                self.assertEqual(bool(data['bg_plane_programs']), invalid != 'paired')
                self.assertEqual(bool(world['terrain_seeds']), invalid != 'terrain')
                self.assertNotIn('terrain_seeds', data)
                self.assertNotIn('bg_plane_programs', world)
                self.assertTrue(next(r for r in data['resources'] if r['id'] == 'graphics.bg-packet-programs')['status'].startswith('Available'))


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class BackgroundPlanesBrowserTest(unittest.TestCase):
    def test_writes_layers_pixels_and_navigation(self):
        with tempfile.TemporaryDirectory(prefix='viewer-bg-plane-browser-') as temp:
            work = Path(temp); _, world = fixture(work); browser = Chrome(work, world)
            try:
                self.assertEqual(browser.evaluate('''(async()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change')),
                    input=e=>e.dispatchEvent(new Event('input'));
                  const pixel=(x,y)=>[...q('#bg-plane-canvas').getContext('2d').getImageData(x,y,1,1).data].join(',');
                  const composed=initialBackgroundPlanes(data.bg_plane_programs,1,0x1234);
                  if(composed.chain.join(',')!=='0,1'||composed.cells[0]!==0x0401||composed.cells[1]!==1||
                     composed.cells[4096]!==1||composed.cells[100]!==0x1234||!composed.written[0]||composed.written[100])throw Error('ordered writes');
                  if(!q('#bg-planes>details')||q('#bg-planes>details').open||q('#bg-plane-source').textContent)throw Error('lazy preview');
                  const row=q('[data-resource-id="graphics.bg-packet-programs"]');
                  [...row.querySelectorAll('a')].find(a=>a.textContent==='Open background planes').click();
                  await new Promise(r=>setTimeout(r,0));
                  if(!q('#bg-planes>details').open||pixel(0,0)!=='255,0,0,255'||pixel(8,0)!=='0,255,0,255'||
                     pixel(24,0)!=='255,0,255,255'||pixel(32,0)!=='0,192,255,255')throw Error('base pixels');
                  q('#bg-plane-missing').checked=false;change(q('#bg-plane-missing'));
                  if(pixel(24,0)!=='0,0,0,0'||pixel(32,0)!=='0,0,0,0'||!q('#bg-plane-source').textContent.includes('missing tile cells'))throw Error('hide missing');
                  q('#bg-plane-missing').checked=true;change(q('#bg-plane-missing'));if(pixel(24,0)!=='255,0,255,255')throw Error('show missing');
                  q('#bg-plane-layer').value='1';change(q('#bg-plane-layer'));if(pixel(0,0)!=='0,255,0,255')throw Error('second plane');
                  q('#bg-plane-layer').value='0-on-1';change(q('#bg-plane-layer'));if(pixel(0,0)!=='255,0,0,255')throw Error('plane order');
                  q('#bg-plane-layer').value='1-on-0';change(q('#bg-plane-layer'));if(pixel(0,0)!=='0,255,0,255')throw Error('reverse plane order');
                  const canvas=q('#bg-plane-canvas'),rect=canvas.getBoundingClientRect();
                  canvas.dispatchEvent(new MouseEvent('click',{clientX:rect.left+rect.width/128,clientY:rect.top+rect.height/128}));
                  if(!q('#bg-plane-cell').textContent.includes('Cell 0, 0')||!q('#bg-plane-cell').textContent.includes('stored write'))throw Error('cell details');
                  q('#bg-plane-fill').value='0x1234';input(q('#bg-plane-fill'));q('#bg-plane-descriptor').value='1';change(q('#bg-plane-descriptor'));
                  if(!q('#bg-plane-source').textContent.includes('inheritance 0 → 1')||q('#bg-plane-fill').value!=='0x1234')throw Error('inheritance or settings');
                  q('#bg-plane-fill').value='-1';input(q('#bg-plane-fill'));if(!q('#bg-plane-source').textContent.includes('Choose an initial word'))throw Error('fill guard');
                  q('#bg-plane-fill').value='0';input(q('#bg-plane-fill'));q('#bg-plane-tile-open').click();
                  if(q('#bg-descriptor').value!=='1'||!q('#bg-tiles>details').open)throw Error('tile link');
                  q('#bg-plane-open').click();if(q('#bg-plane-descriptor').value!=='1'||!q('#bg-planes>details').open)throw Error('return link');
                  return 'PASS';
                })()'''), 'PASS')
            finally: browser.close()

    def test_standalone_shades(self):
        with tempfile.TemporaryDirectory(prefix='viewer-bg-plane-alone-') as temp:
            work = Path(temp); _, world = fixture(work, graphic=False); browser = Chrome(work, world)
            try:
                self.assertTrue(browser.evaluate('''(async()=>{
                  const q=s=>document.querySelector(s);q('#bg-planes>details').open=true;await new Promise(r=>setTimeout(r,0));
                  return q('#bg-plane-graphics').value==='none'&&q('#bg-plane-palette').value==='gray'&&
                    q('#bg-plane-source').textContent.includes('tile IDs shown as shades')&&q('#bg-plane-tile-navigation').hidden;
                })()'''))
            finally: browser.close()
