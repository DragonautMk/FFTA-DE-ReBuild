"""Exercise inherited terrain grids and encounter overlays in Chrome."""
import hashlib
import json
import shutil
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from view_rom_data import generate
from viewer_browser_harness import Chrome


def fixture(work, *, terrain_only=False):
    dataset = work / 'dataset'
    dataset.mkdir()
    terrain = {'auxiliary': {'descriptor_programs': [0, 1, 1], 'programs': [
        {'relative_source': 100, 'parent_descriptor': None, 'writes': {'kind': 'halfwords', 'packets': [
            {'byte_offset': 0, 'values': [0x0102, 0x0205]}, {'byte_offset': 2, 'values': [0x0308]}]}},
        {'relative_source': 200, 'parent_descriptor': 0, 'writes': {'kind': 'halfwords', 'packets': [
            {'byte_offset': 0, 'values': [0x0506]}, {'byte_offset': 510, 'values': [0x0a0b]}]}},
    ]}}
    background = {'descriptor_graphics': [0, 0, 0], 'graphics': [
        {'relative_source': 1000, 'destination_offset': 0, 'words': [0x21] + [0] * 7}]}
    def header(index, map_argument, pointer):
        return {'index': index, 'unit_count': 0, 'map_argument': map_argument, 'unit_templates_pointer': 0,
                'placement_parameter0': 0, 'placement_parameter2': 0, 'placement_count0': 2 if pointer else 0,
                'random_position_range': 0, 'placement_count2': 0, 'position_pointers': [pointer, 0, 0, 0],
                'unit_count_comparison': 0, 'nonzero_gate': 0, 'switch_selector': 0, 'zero_gate': 0,
                'script_default': 0, 'opaque': [0] * 7}
    headers = {'source_offset': 3000, 'headers': [header(1, 2, 0x08000000 + 5000),
                                                  header(2, 0, 0), header(3, 9, 0)]}
    coords = {'source_offset': 5000, 'lists': [{'index': 0, 'header_index': 1, 'slot': 0,
              'source_offset': 5000, 'points': [{'x': 2, 'y': 3}, {'x': 20, 'y': 40}]}]}
    resources = [('graphics.bg-packet-programs', 'bg-packet-programs', terrain)]
    if not terrain_only:
        resources += [('graphics.bg-main-graphics', 'bg-main-graphics', background),
                      ('gameplay.encounter-headers', 'encounter-headers', headers),
                      ('gameplay.encounter-coordinates', 'encounter-coordinates', coords)]
    manifest = 'profile="test"\nextractor_revision=1\ndecoder_revision=1\n'
    for resource_id, name, value in resources:
        content = json.dumps(value).encode()
        (dataset / (name + '.json')).write_bytes(content)
        manifest += (f'[[resources]]\nid="{resource_id}"\npath="{name}.json"\n'
                     f'format="{name}-v1-json"\nsha256="{hashlib.sha256(content).hexdigest()}"\n')
    (dataset / 'manifest.toml').write_text(manifest)
    return generate(dataset, work / 'view')


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class ViewerTerrainTest(unittest.TestCase):
    def test_terrain_inheritance_colors_and_conditional_overlay(self):
        with tempfile.TemporaryDirectory(prefix='viewer-terrain-') as directory:
            work = Path(directory)
            browser = Chrome(work, fixture(work))
            try:
                self.assertEqual(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change'));
                  const pixel=(x,y)=>[...q('#terrain-canvas').getContext('2d').getImageData(x,y,1,1).data].join(',');
                  if(q('#terrain-seeds').hidden || q('#terrain-seeds>details').open)throw Error('terrain disclosure');
                  const row=[...document.querySelectorAll('#rows tr')].find(r=>r.cells[0].textContent.includes('graphics.bg-packet-programs'));
                  if(!row || !row.querySelector('a'))throw Error('resource');row.querySelector('a').click();
                  q('#terrain-descriptor').value='1';change(q('#terrain-descriptor'));
                  if(!q('#terrain-source').textContent.includes('inheritance 0 → 1') || q('#terrain-cells').rows.length!==16 ||
                     q('#terrain-cells').rows[0].cells.length!==16)throw Error('inherited map');
                  q('#terrain-numbers').checked=false;change(q('#terrain-numbers'));
                  if(pixel(36,12)!=='114,114,114,255')throw Error('height pixel');
                  q('#terrain-cell').value='1';change(q('#terrain-cell'));
                  if(!q('#terrain-cell-detail').textContent.includes('0x0308') || !q('#terrain-cell-detail').textContent.includes('height 8') ||
                     !q('#terrain-cell-detail').textContent.includes('flags 0x03'))throw Error('cell detail');
                  q('#terrain-color-mode').value='flags';change(q('#terrain-color-mode'));
                  if(pixel(36,12)!=='107,205,151,255')throw Error('flag pixel');
                  q('#terrain-encounter').value='1';change(q('#terrain-encounter'));q('#terrain-overlay-list').value='0';change(q('#terrain-overlay-list'));
                  if(!q('#terrain-overlay-status').textContent.includes('1 of 2') ||
                     !q('#terrain-overlay-status').textContent.includes('pointer and count match') || pixel(64,84)!=='224,86,43,255')throw Error('coordinate overlay');
                  q('#terrain-encounter-open').click();
                  if(q('#encounter-id').value!=='1' || !q('#encounter-header-details').open)throw Error('terrain to encounter');
                  q('#encounter-terrain-open').click();
                  if(q('#terrain-descriptor').value!=='1' || q('#terrain-encounter').value!=='1' || q('#terrain-overlay-list').value!=='0')throw Error('encounter to terrain');
                  q('#bg-descriptor').value='2';change(q('#bg-descriptor'));q('#bg-terrain-open').click();
                  if(q('#terrain-descriptor').value!=='2' || !q('#terrain-source').textContent.includes('inheritance 0 → 2'))throw Error('background to terrain');
                  q('#terrain-bg-open').click();if(q('#bg-descriptor').value!=='2' || !q('#bg-tiles>details').open)throw Error('terrain to background');
                  q('#encounter-id').value='2';change(q('#encounter-id'));if(!q('#encounter-terrain-navigation').hidden)throw Error('zero map sentinel');
                  q('#encounter-id').value='3';change(q('#encounter-id'));if(!q('#encounter-terrain-navigation').hidden)throw Error('missing descriptor');
                  return 'PASS';
                })()'''), 'PASS')
            finally:
                browser.close()

    def test_terrain_without_other_catalogs(self):
        with tempfile.TemporaryDirectory(prefix='viewer-terrain-only-') as directory:
            work = Path(directory)
            browser = Chrome(work, fixture(work, terrain_only=True))
            try:
                self.assertEqual(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s);
                  if(q('#terrain-seeds').hidden || !q('#terrain-encounter').disabled || !q('#terrain-overlay-list').disabled ||
                     !q('#terrain-bg-navigation').hidden || q('#terrain-cells').rows.length!==16)throw Error('standalone terrain');
                  return 'PASS';
                })()'''), 'PASS')
            finally:
                browser.close()
