"""Check source-backed opcode-39 pose joins and animation controls."""
import copy
import hashlib
import json
import shutil
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from view_rom_data import generate, project_script39_poses, validate_script39_poses
from viewer_browser_harness import Chrome


def fixture(work, *, include_registry=True, extra_null_cells=0):
    dataset = work / 'dataset'; dataset.mkdir()
    obj = {'x': 0, 'y': 0, 'affine_mode': 0, 'object_mode': 0, 'mosaic': False, 'color_256': False,
           'shape': 0, 'transform': 0, 'size': 0, 'tile': 0, 'priority': 0, 'palette': 0}
    objects = [{'source_offset': 0xa50020, 'objects': [obj]}]
    pixels = [{'source_offset': 0x898a40, 'tiles': [{'indices': [[index] * 8 for _ in range(8)]}
                                                  for index in (1, 2)]}]
    def frame(operation, offset=0):
        return {'word00': offset, 'word04': 0, 'duration': 2, 'operation': operation, 'bytes_0a_13': [0] * 10}
    registry = {'cells': [{'source_offset': 0x1000, 'target': {'kind': 'sequence', 'source_offset': 0x2000}},
                          {'source_offset': 0x1004, 'target': {'kind': 'null'}}],
                'sequences': [{'source_offset': 0x2000, 'opaque_header': 0,
                               'frames': [frame(1), frame(2), frame(6)]}],
                'pixels': pixels, 'producer_pairs': []}
    registry['cells'].extend({'source_offset': 0x1100 + 4 * index, 'target': {'kind': 'null'}}
                             for index in range(extra_null_cells))
    special = {'cells': [{'source_offset': 0x1010, 'target': {'kind': 'sequence', 'source_offset': 0x3000}},
                         {'source_offset': 0x1014, 'target': {'kind': 'sequence', 'source_offset': 0x4000}},
                         {'source_offset': 0x1018, 'target': {'kind': 'sequence', 'source_offset': 0x4100}}],
               'sequences': [{'source_offset': 0x3000, 'opaque_header': 7, 'frames': [frame(1, 32), frame(6)]}],
               'objects': objects, 'pixels': [], 'producer_pairs': []}
    resources = [('script39-special-poses', special)]
    if include_registry:
        resources.append(('script39-registry-poses', registry))
        equipment = {'pixels': pixels, 'objects': objects,
                     'sequences': [{'source_offset': 0x4000, 'opaque_header': 5, 'frames': [frame(1), frame(6)]}],
                     'bindings': [{'sprite_id': 1, 'animation': 0, 'part': 0,
                                   'index_source': 0x5000, 'sequence_source': 0x4000}]}
        resources.append(('unit-obj-equipment', equipment))
    manifest = 'profile="europe"\nextractor_revision=1\ndecoder_revision=1\n'
    for name, value in resources:
        content = json.dumps(value).encode(); (dataset / (name + '.json')).write_bytes(content)
        manifest += (f'[[resources]]\nid="graphics.{name}"\npath="{name}.json"\n'
                     f'format="{name}-v1-json"\nsha256="{hashlib.sha256(content).hexdigest()}"\n')
    (dataset / 'manifest.toml').write_text(manifest)
    return dataset, registry, special


def payload(page):
    return json.JSONDecoder().raw_decode(page.read_text().split('const data = ', 1)[1])[0]


class Script39ProjectionTest(unittest.TestCase):
    def test_cross_catalog_tiles_and_partial_sources(self):
        with tempfile.TemporaryDirectory(prefix='viewer-script39-projection-') as directory:
            work = Path(directory); dataset, registry, special = fixture(work)
            index = generate(dataset, work / 'view', split=True)
            actors = payload(index.parent / 'index.html.parts/script-actors.html')
            self.assertNotIn('script39_poses', payload(index))
            self.assertNotIn('script39_poses', payload(index.parent / 'index.html.parts/world.html'))
            family = actors['script39_poses']['graphics.script39-special-poses']
            first = family['sequences'][0]['frames'][0]
            self.assertEqual(first['pixel_source'], 0x898a60)
            self.assertEqual(family['blocks'][first['block']]['tiles'][0]['indices'][0][0], 2)
            self.assertEqual(family['objects'][first['objects']]['source_offset'], 0xa50020)
            self.assertEqual(family['bindings'][0]['index_source'], 0x1010)
            self.assertEqual(family['cells'], [{'source_offset': 0x1010, 'target': 0x3000},
                                               {'source_offset': 0x1014, 'target': 0x4000},
                                               {'source_offset': 0x1018, 'target': 0x4100}])
            self.assertEqual(family['bindings'][1]['index_source'], 0x1014)
            self.assertEqual(family['sequences'][family['bindings'][1]['sequence']]['path'], 'unit-obj-equipment.json')
            self.assertIsNone(family['bindings'][2]['sequence'])
            self.assertEqual(family['bindings'][2]['target']['kind'], 'unavailable-sequence')
            pools = [(special['objects'][0], 'objects.json')]
            pixels = [(registry['pixels'][0], 'pixels.json')]
            partial = project_script39_poses(special, 'special.json', [], pixels)
            self.assertIsNone(partial['sequences'][0]['frames'][0]['block'])
            conflict = copy.deepcopy(registry['pixels'][0]); conflict['tiles'][1]['indices'][0][0] = 3
            partial = project_script39_poses(special, 'special.json', pools, pixels + [(conflict, 'conflict.json')])
            self.assertIsNone(partial['sequences'][0]['frames'][0]['block'])
            wide = copy.deepcopy(special['objects'][0]); wide['objects'][0]['shape'] = 1
            spanning = copy.deepcopy(special); spanning['sequences'][0]['frames'][0]['word00'] = 0
            partial_block = {'source_offset': 0x898a60, 'tiles': [conflict['tiles'][1]]}
            partial = project_script39_poses(spanning, 'special.json', [(wide, 'objects.json')],
                                             pixels + [(partial_block, 'conflict.json')])
            self.assertIsNone(partial['sequences'][0]['frames'][0]['block'])
            unsupported = copy.deepcopy(special['objects'][0]); unsupported['objects'][0]['affine_mode'] = 1
            partial = project_script39_poses(special, 'special.json', [(unsupported, 'objects.json')], pixels)
            self.assertIsNone(partial['sequences'][0]['frames'][0]['objects'])
            unbound = copy.deepcopy(special); unbound['cells'] = []
            partial = project_script39_poses(unbound, 'special.json', pools, pixels)
            self.assertIsNone(partial['bindings'][0]['index_source'])
            bad = copy.deepcopy(registry); bad['cells'][1]['source_offset'] = bad['cells'][0]['source_offset']
            with self.assertRaises(ValueError): validate_script39_poses(bad)
            bad = copy.deepcopy(registry); bad['sequences'][0]['frames'][0]['duration'] = True
            with self.assertRaises(ValueError): validate_script39_poses(bad)
            bad = copy.deepcopy(registry); bad['pixels'][0]['tiles'][0]['indices'][0][0] = 16
            with self.assertRaises(ValueError): validate_script39_poses(bad)


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class Script39BrowserTest(unittest.TestCase):
    def test_frame_pixels_source_links_and_playback(self):
        with tempfile.TemporaryDirectory(prefix='viewer-script39-browser-') as directory:
            work = Path(directory); dataset, _, _ = fixture(work)
            index = generate(dataset, work / 'view', split=True)
            browser = Chrome(work, index.parent / 'index.html.parts/script-actors.html')
            try:
                self.assertEqual(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change'));
                  const row=q('[data-resource-id="graphics.script39-special-poses"]');
                  row.querySelector('a[href="#equipment-sprites"]').click();
                  if(q('#equipment-catalog').value!=='graphics.script39-special-poses' ||
                     q('#equipment-sprite-label').textContent!=='Sequence source' || q('#equipment-source-note').hidden ||
                     q('#equipment-sprite').selectedOptions[0].textContent!=='0x3000' ||
                     !q('#equipment-binding-detail').textContent.includes('0x1010'))throw Error('source selection');
                  const pixel=[...q('#equipment-pose').getContext('2d').getImageData(0,0,1,1).data];
                  if(pixel.join(',')!=='34,34,34,255' || !q('#equipment-pose-detail').textContent.includes('script39-registry-poses.json'))throw Error('joined pixel');
                  q('#equipment-sprite').value=String(0x4000);change(q('#equipment-sprite'));
                  if(!q('#equipment-binding-detail').textContent.includes('unit-obj-equipment.json') || q('#equipment-frame').options.length!==2)throw Error('earlier sequence');
                  q('#equipment-sprite').value=String(0x4100);change(q('#equipment-sprite'));
                  if(!q('#equipment-binding-detail').textContent.includes('unavailable') || !q('#equipment-play').disabled)throw Error('missing sequence');
                  q('#equipment-sprite').value=String(0x3000);change(q('#equipment-sprite'));
                  window.poseTimers=new Map();let serial=0;
                  window.setTimeout=(fn,delay)=>{const id=++serial;poseTimers.set(id,{fn,delay});return id};
                  window.clearTimeout=id=>poseTimers.delete(id);
                  window.firePoseTimer=()=>{const [id,item]=poseTimers.entries().next().value;poseTimers.delete(id);item.fn();return item.delay};
                  q('#equipment-rate').value='20';q('#equipment-loop').checked=true;q('#equipment-play').click();
                  if(firePoseTimer()!==100 || q('#equipment-frame').value!=='1')throw Error('frame advance');
                  firePoseTimer();if(q('#equipment-frame').value!=='0')throw Error('loop');
                  q('#equipment-catalog').value='graphics.script39-registry-poses';change(q('#equipment-catalog'));
                  if(!q('#equipment-loop').checked || q('#equipment-rate').value!=='20' || q('#equipment-stop').disabled ||
                     q('#equipment-frame').value!=='0' || !q('#equipment-binding-detail').textContent.includes('0x1000'))throw Error('persistent controls');
                  q('#equipment-stop').click();if(poseTimers.size)throw Error('stop');
                  return 'PASS';
                })()'''), 'PASS')
            finally:
                browser.close()

    def test_unresolved_graphics_keep_sequence_browsable(self):
        with tempfile.TemporaryDirectory(prefix='viewer-script39-partial-') as directory:
            work = Path(directory); dataset, _, _ = fixture(work, include_registry=False)
            browser = Chrome(work, generate(dataset, work / 'view'))
            try:
                self.assertTrue(browser.evaluate("!document.querySelector('#equipment-sprites').hidden && document.querySelector('#equipment-pose-detail').textContent.includes('missing') && document.querySelector('#equipment-frame').options.length===2"))
            finally:
                browser.close()

    def test_index_cells_pagination_filters_and_navigation(self):
        with tempfile.TemporaryDirectory(prefix='viewer-script39-cells-') as directory:
            work = Path(directory); dataset, _, _ = fixture(work, extra_null_cells=123)
            index = generate(dataset, work / 'view', split=True)
            browser = Chrome(work, index.parent / 'index.html.parts/script-actors.html')
            try:
                self.assertEqual(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change')),
                    input=e=>e.dispatchEvent(new Event('input'));
                  q('[data-resource-id="graphics.script39-registry-poses"] a[href="#equipment-sprites"]').click();
                  if(q('#equipment-index-cells').hidden || q('#equipment-index-cells').open ||
                     q('#equipment-cell-rows').rows.length!==100 || !q('#equipment-cell-page').textContent.includes('125 cells') ||
                     q('#equipment-cell-next').disabled || !q('#equipment-cell-prev').disabled)throw Error('first page');
                  q('#equipment-cell-next').click();
                  if(q('#equipment-cell-rows').rows.length!==25 || !q('#equipment-cell-next').disabled ||
                     !q('#equipment-cell-page').textContent.includes('page 2 of 2'))throw Error('next page');
                  q('#equipment-cell-kind').value='sequence';change(q('#equipment-cell-kind'));
                  if(q('#equipment-cell-rows').rows.length!==1 || !q('#equipment-cell-page').textContent.includes('1 cells'))throw Error('sequence filter');
                  q('#equipment-cell-rows a').click();
                  if(q('#equipment-sprite').value!==String(0x2000) || !q('#equipment-binding-detail').textContent.includes('0x1000'))throw Error('sequence link');
                  q('#equipment-cell-kind').value='null';change(q('#equipment-cell-kind'));
                  q('#equipment-cell-filter').value='0x1004';input(q('#equipment-cell-filter'));
                  if(q('#equipment-cell-rows').rows.length!==1 || q('#equipment-cell-rows').rows[0].cells[1].textContent!=='Null')throw Error('null filter');
                  q('#equipment-catalog').value='graphics.script39-special-poses';change(q('#equipment-catalog'));
                  if(q('#equipment-cell-kind').value!=='null' || q('#equipment-cell-filter').value!=='0x1004' || q('#equipment-cell-rows').rows.length)throw Error('persistent filter');
                  q('#equipment-cell-kind').value='sequence';change(q('#equipment-cell-kind'));
                  q('#equipment-cell-filter').value='0x4100';input(q('#equipment-cell-filter'));
                  q('#equipment-cell-rows a').click();
                  if(q('#equipment-sprite').value!==String(0x4100) || !q('#equipment-binding-detail').textContent.includes('unavailable'))throw Error('unavailable link');
                  return 'PASS';
                })()'''), 'PASS')
            finally:
                browser.close()
            single_work = work / 'single'; single_work.mkdir()
            single = Chrome(single_work, generate(dataset, single_work / 'view'))
            try:
                self.assertTrue(single.evaluate('''(()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change'));
                  if(!q('#equipment-index-cells').hidden)return false;
                  q('#equipment-catalog').value='graphics.script39-special-poses';change(q('#equipment-catalog'));
                  if(q('#equipment-index-cells').hidden)return false;
                  q('#equipment-catalog').value='equipment';change(q('#equipment-catalog'));
                  return q('#equipment-index-cells').hidden;
                })()'''))
            finally:
                single.close()
