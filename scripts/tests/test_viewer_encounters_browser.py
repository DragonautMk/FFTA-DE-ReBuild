"""Exercise encounter records, coordinate plots and item links in Chrome."""
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


def fixture(work, *, coordinates_only=False):
    dataset = work / 'dataset'
    dataset.mkdir()
    def header(index, count, pointer, position):
        return {'index': index, 'unit_count': count, 'map_argument': 5, 'unit_templates_pointer': pointer,
                'placement_parameter0': 2, 'placement_parameter2': 0, 'placement_count0': 2 if position else 0,
                'random_position_range': 0, 'placement_count2': 0,
                'position_pointers': [0x08000000 + 5000 if position else 0, 0, 0, 0],
                'unit_count_comparison': count, 'nonzero_gate': 2, 'switch_selector': 0,
                'zero_gate': 0, 'script_default': 11, 'opaque': [1, 2, 3, 4, 5, 6, 7]}
    headers = {'source_offset': 1000, 'headers': [header(1, 1, 0x08000000 + 2000, True),
                                                  header(2, 23, 0x08000000 + 2048, False)]}
    units = {'source_offset': 2000, 'records': []}
    for index in range(24):
        units['records'].append({'index': index, 'header_index': 1 if index == 0 else 2,
            'unit_index': 0 if index == 0 else index - 1, 'source_offset': 2000 + index * 48,
            'byte00': 3, 'byte01': 4, 'byte02': 5, 'byte03': 6, 'byte04': 7,
            'arguments05_07': [8, 9, 10], 'equipment08_10': [1, 0, 0, 0, 0],
            'halfword12': 0x1234, 'ability_bits14_27': list(range(20)), 'byte28': 11,
            'byte29': 12, 'flags2a': 0x8000, 'byte2c': 13, 'byte2d': 14, 'opaque2e_2f': [15, 16]})
    coords = {'source_offset': 5000, 'lists': [{'index': 0, 'header_index': 1, 'slot': 0,
               'source_offset': 5000, 'points': [{'x': 4, 'y': 9}, {'x': 200, 'y': 255}]}]}
    items = {'records': [{'halfwords_00_06': [0] * 4, 'bytes_08_0f': [0] * 8,
                          'signed_10_17': [0] * 8, 'bytes_18_1f': [0] * 8} for _ in range(2)]}
    resources = [('encounter-coordinates', coords)]
    if not coordinates_only:
        resources += [('encounter-headers', headers), ('encounter-unit-templates', units), ('item-records', items)]
    manifest = 'profile="test"\nextractor_revision=1\ndecoder_revision=1\n'
    for name, value in resources:
        content = json.dumps(value).encode()
        (dataset / (name + '.json')).write_bytes(content)
        manifest += (f'[[resources]]\nid="gameplay.{name}"\npath="{name}.json"\n'
                     f'format="{name}-v1-json"\nsha256="{hashlib.sha256(content).hexdigest()}"\n')
    (dataset / 'manifest.toml').write_text(manifest)
    return generate(dataset, work / 'view')


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class ViewerEncountersTest(unittest.TestCase):
    def test_headers_units_coordinates_and_item_navigation(self):
        with tempfile.TemporaryDirectory(prefix='viewer-encounters-') as directory:
            work = Path(directory)
            browser = Chrome(work, fixture(work))
            try:
                self.assertEqual(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change'));
                  const pixel=(x,y)=>[...q('#encounter-plot').getContext('2d').getImageData(x,y,1,1).data].join(',');
                  if(q('#encounters').hidden || q('#encounters>details').open)throw Error('encounter disclosure');
                  for(const name of ['headers','unit-templates','coordinates']){
                    const row=[...document.querySelectorAll('#rows tr')].find(r=>r.cells[0].textContent.includes('gameplay.encounter-'+name+')'));
                    if(!row || !row.querySelector('a'))throw Error('resource '+name);row.querySelector('a').click();
                  }
                  if(!q('#encounter-unit-status').textContent.includes('match the header') ||
                     !q('#encounter-list-source').textContent.includes('pointer and count match'))throw Error('verified links');
                  const fields=q('#encounter-header-fields').textContent;
                  if(!fields.includes('0x080007D0') || !fields.includes('0x01 0x02 0x03 0x04 0x05 0x06 0x07'))throw Error('header fields');
                  if(q('#encounter-equipment').rows.length!==5 || !q('#encounter-unit-fields').textContent.includes('0x1234'))throw Error('unit fields');
                  const raw=[...q('#encounter-points').rows[1].cells].map(x=>x.textContent);
                  if(raw.join('|')!=='1|200|255|-56|-1')throw Error('signed values');
                  q('#encounter-point').value='1';change(q('#encounter-point'));
                  q('#encounter-coordinate-mode').value='signed';change(q('#encounter-coordinate-mode'));
                  if(pixel(35,35)!=='224,86,43,255')throw Error('signed plot');
                  q('#encounter-equipment a').click();
                  if(q('#item-id').value!=='1' || !q('#items>details').open)throw Error('unit to item');
                  if(!q('#item-encounter-page').textContent.includes('24 references') || q('#item-encounter-rows').rows.length!==20)throw Error('reverse page');
                  q('#item-encounter-next').click();
                  if(q('#item-encounter-rows').rows.length!==4 || !q('#item-encounter-page').textContent.includes('page 2'))throw Error('next page');
                  q('#item-encounter-rows a').click();
                  if(q('#encounter-id').value!=='2' || q('#encounter-unit').value!=='20' || !q('#encounter-unit-details').open ||
                     !q('#encounter-coordinate-status').textContent.includes('No extracted'))throw Error('item to unit');
                  data.encounters.headers.headers[0].unit_templates_pointer=1;
                  data.encounters.headers.headers[0].position_pointers[0]=1;
                  q('#encounter-id').value='1';change(q('#encounter-id'));
                  if(!q('#encounter-unit-status').textContent.includes('differ') ||
                     !q('#encounter-list-source').textContent.includes('differs'))throw Error('inconsistent associations');
                  return 'PASS';
                })()'''), 'PASS')
            finally:
                browser.close()

    def test_coordinate_catalog_without_headers_or_units(self):
        with tempfile.TemporaryDirectory(prefix='viewer-coordinates-only-') as directory:
            work = Path(directory)
            browser = Chrome(work, fixture(work, coordinates_only=True))
            try:
                self.assertEqual(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s);
                  if(q('#encounters').hidden || q('#encounter-id').value!=='1' ||
                     !q('#encounter-header-status').textContent.includes('No extracted header') ||
                     !q('#encounter-unit-status').textContent.includes('unavailable') ||
                     !q('#encounter-list-source').textContent.includes('header unavailable') ||
                     q('#encounter-points').rows.length!==2)throw Error('partial dataset');
                  return 'PASS';
                })()'''), 'PASS')
            finally:
                browser.close()
