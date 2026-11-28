"""Verify bounded selection source data and the independent scan explorer."""
import copy
import hashlib
import json
import shutil
import tempfile
import unittest
from pathlib import Path

from view_rom_data import generate, project_selection_byte_tables, viewer_section_updates
from viewer_browser_harness import Chrome


def sample():
    catalog = {'threshold_source_offset': 0x71c73c, 'weight_source_offset': 0x71c740,
               'thresholds': [10, 20, 30, 40], 'weights': [0, 10, 1] + [2] * 27}
    source = [{'offset': offset, 'length': len(values), 'role': role,
               'sha256': hashlib.sha256(bytes(values)).hexdigest()}
              for offset, role, values in ((0x71c73c, 'count-thresholds', catalog['thresholds']),
                                           (0x71c740, 'ordinal-weights', catalog['weights']))]
    return catalog, source


def fixture(work, invalid=False):
    dataset = work / 'dataset'; dataset.mkdir(); catalog, source = sample()
    content = json.dumps(catalog).encode(); (dataset / 'selection-byte-tables.json').write_bytes(content)
    manifest = ('profile="europe"\nextractor_revision=362\ndecoder_revision=361\n'
                '[[resources]]\nid="gameplay.selection-byte-tables"\npath="selection-byte-tables.json"\n'
                f'format="selection-byte-tables-v1-json"\nsha256="{hashlib.sha256(content).hexdigest()}"\n')
    for index, item in enumerate(source):
        digest = '0' * 64 if invalid and index == 1 else item['sha256']
        manifest += (f'[[resources.sources]]\noffset={item["offset"]}\nlength={item["length"]}\n'
                     f'role="{item["role"]}"\nsha256="{digest}"\n')
    (dataset / 'manifest.toml').write_text(manifest)
    return generate(dataset, work / 'view', split=True)


def payload(path):
    return json.JSONDecoder().raw_decode(path.read_text().split('const data = ', 1)[1])[0]


class SelectionByteProjectionTest(unittest.TestCase):
    def test_source_binding_and_rejection(self):
        catalog, source = sample(); self.assertEqual(project_selection_byte_tables(catalog, source)['weights'][:3], [0, 10, 1])
        cases = [lambda c, s: c['weights'].pop(), lambda c, s: c['thresholds'].__setitem__(0, -1),
                 lambda c, s: c['weights'].__setitem__(0, True), lambda c, s: s[0].update(offset=0),
                 lambda c, s: s[1].update(sha256='0' * 64), lambda c, s: s.__setitem__(0, None)]
        for mutate in cases:
            c, s = copy.deepcopy(catalog), copy.deepcopy(source); mutate(c, s)
            with self.assertRaises(ValueError): project_selection_byte_tables(c, s)

    def test_scoped_and_invalid_sources(self):
        for invalid in (False, True):
            with self.subTest(invalid=invalid), tempfile.TemporaryDirectory(prefix='viewer-selection-scope-') as directory:
                index = fixture(Path(directory), invalid); folder = index.parent / (index.name + '.parts')
                home, page, world = map(payload, (index, folder / 'selection-tables.html', folder / 'world.html'))
                self.assertNotIn('selection_byte_tables', home); self.assertNotIn('selection_byte_tables', world)
                self.assertEqual(page['selection_byte_tables'] is None, invalid)
                self.assertEqual(any(s['id'] == 'selection-byte-tables' for p in home['wiki']['pages']
                                     if p['id'] == 'selection-tables' for s in p.get('sections', [])), not invalid)


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class SelectionByteBrowserTest(unittest.TestCase):
    def test_threshold_scans_signed_wrap_and_independent_live_inputs(self):
        with tempfile.TemporaryDirectory(prefix='viewer-selection-browser-') as directory:
            work = Path(directory); index = fixture(work); page = index.parent / (index.name + '.parts') / 'selection-tables.html'
            browser = Chrome(work, page)
            try:
                expected = viewer_section_updates()['selection-tables']['selection-byte-tables']
                self.assertTrue(browser.evaluate('''(()=>{const expected=EXPECTED,q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change')),
                  input=e=>e.dispatchEvent(new Event('input')),first=i=>q('[aria-label="Sum scan ordinal '+i+'"]'),second=i=>q('[aria-label="Choice scan ordinal '+i+'"]');
                  const badge=q('#selection-byte-tables>details>summary time');
                  if(!badge||badge.dateTime!==expected.at||!badge.textContent.startsWith('Added ')||q('#selection-byte-rows').rows.length!==30||
                    !q('[data-resource-id="gameplay.selection-byte-tables"] a[href="#selection-byte-tables"]'))throw Error('section');
                  q('#selection-byte-threshold-input').value='10';input(q('#selection-byte-threshold-input'));
                  if(!q('#selection-byte-threshold-result').textContent.includes('continues'))throw Error('equal');
                  q('#selection-byte-threshold-input').value='11';input(q('#selection-byte-threshold-input'));
                  if(!q('#selection-byte-threshold-result').textContent.includes('exits'))throw Error('threshold');
                  q('#selection-byte-count').value='4';change(q('#selection-byte-count'));
                  if(!q('#selection-byte-threshold-result').textContent.includes('before the threshold'))throw Error('count');
                  if(!q('#selection-byte-choice').textContent.includes('ordinal: 2'))throw Error('zero weight');
                  q('#selection-byte-rank').value='-1';input(q('#selection-byte-rank'));
                  if(!q('#selection-byte-choice').textContent.includes('ordinal: 1'))throw Error('negative rank');
                  second(1).checked=false;change(second(1));q('#selection-byte-rank').value='-2147483648';input(q('#selection-byte-rank'));
                  if(q('#selection-byte-rows').rows[1].cells[5].textContent!=='2147483638'||!q('#selection-byte-choice').textContent.includes('No ordinal'))throw Error('signed wrap');
                  q('#selection-byte-clear').click();if(!q('#selection-byte-choice').textContent.includes('not reached'))throw Error('zero total');
                  first(2).checked=true;change(first(2));second(3).checked=true;change(second(3));q('#selection-byte-rank').value='0';input(q('#selection-byte-rank'));
                  if(!q('#selection-byte-choice').textContent.includes('ordinal: 3'))throw Error('independent scans');
                  q('#selection-byte-copy').click();if(!q('#selection-byte-choice').textContent.includes('ordinal: 2'))throw Error('copy eligibility');
                  q('#selection-byte-rank').value='4294967296';input(q('#selection-byte-rank'));
                  return q('#selection-byte-choice').textContent.includes('32-bit');})()'''.replace('EXPECTED', json.dumps(expected))))
            finally:
                browser.close()


if __name__ == '__main__':
    unittest.main()
