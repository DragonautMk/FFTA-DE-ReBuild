"""Check the focused physical-bank page and its playback controls."""
import json
import shutil
import tempfile
import unittest
from pathlib import Path

from test_viewer_scoped_browser import REPO, parse_page, prepared
from view_rom_data import viewer_section_updates, write_split_viewer
from viewer_browser_harness import Chrome


def fixture(work, *, descriptors=False, contexts=False):
    data, index = prepared(work)
    obj = {'x': 0, 'y': 0, 'affine_mode': 0, 'object_mode': 0, 'mosaic': False, 'color_256': False,
           'shape': 0, 'transform': 0, 'size': 0, 'tile': 0, 'priority': 0, 'palette': 0}
    banks = []
    for n in range(2):
        root = 0x6b3428 + n * 0x100
        banks.append({'object_root': root + 4, 'sequence_root': root + 48, 'flags': 12,
                      'lists': [{'source_offset': root + 8, 'source_index': 0, 'relative_offset': 4, 'objects': [obj]},
                                {'source_offset': root + 16, 'source_index': 1, 'relative_offset': 12, 'objects': [dict(obj, x=8)]}],
                      'sequences': [{'selector': 0, 'source_offset': root + 50, 'relative_offset': 2,
                                     'frames': [{'object_list': 0, 'list_id': 0, 'duration': 1},
                                                {'object_list': 1, 'list_id': 1, 'duration': 2}]}]})
    data['physical_obj_animations'] = {'banks': banks, 'selector_tables': [], 'path': 'physical.bin · prior source owners'}
    if contexts:
        banks[0]['observed_contexts'] = [{'name': 'Example context', 'selector': 0}]
    if descriptors:
        data['physical_obj_descriptors'] = {'path': 'physical.bin · prior source owners', 'records': [
            {'source_offset': 0x390000 + n * 24, 'bank_root': banks[n]['object_root'] - 4,
             'bank': n, 'selector': 0, 'sequence': 0, 'fields': [0] + list(range(1, 10))}
            for n in range(2)]}
    data['section_updates'] = viewer_section_updates()
    data['resources'].append({'id': 'graphics.physical-compact-obj-banks', 'path': 'physical.bin', 'format': 'binary',
                              'description': 'Physical compact banks', 'status': 'Available'})
    index = write_split_viewer(data, index.parent, index.name, (REPO / 'view_rom_data.html').read_text())
    return index


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class PhysicalCompactBrowserTest(unittest.TestCase):
    def test_focused_page_badge_playback_and_persistent_settings(self):
        with tempfile.TemporaryDirectory(prefix='viewer-physical-compact-browser-') as directory:
            work = Path(directory); index = fixture(work); parts = index.parent / (index.name + '.parts')
            home, old, physical = map(parse_page, (index, parts / 'objects.html', parts / 'objects-physical.html'))
            self.assertNotIn('physical_obj_animations', home)
            self.assertNotIn('physical_obj_animations', old)
            self.assertEqual(len(physical['physical_obj_animations']['banks']), 2)
            browser = Chrome(work, parts / 'objects-physical.html')
            try:
                expected = viewer_section_updates()['objects-physical']['obj-animations']
                self.assertTrue(browser.evaluate('''(()=>{
                  const expected=EXPECTED_BADGE;
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change'));
                  window.pending=new Map();let serial=0;window.setTimeout=(f,ms)=>{const id=++serial;pending.set(id,{f,ms});return id};
                  window.clearTimeout=id=>pending.delete(id);window.fire=()=>{const [id,t]=pending.entries().next().value;pending.delete(id);t.f();return t.ms};
                  const badge=q('#obj-animations > details > summary time');
                  if(!badge || badge.dateTime!==expected.at || !badge.textContent.startsWith(expected.kind==='added'?'Added ':'Updated '))throw Error('section badge');
                  if(q('#obj-animations').hidden || q('#obj-animation-catalog').value!=='physical_catalog' ||
                     q('#obj-animation-bank').options.length!==2 || q('#obj-animation-frame').options.length!==2)throw Error('physical page');
                  q('#obj-animations>details').open=true;q('#obj-animation-loop').checked=true;
                  q('#obj-animation-rate').value='10';q('#obj-animation-frame').value='1';change(q('#obj-animation-frame'));
                  q('#obj-animation-play').click();const delay=fire();
                  if(delay!==200 || q('#obj-animation-frame').value!=='0')throw Error('play from frame: '+delay+' / '+q('#obj-animation-frame').value);
                  q('#obj-animation-bank').value='1';change(q('#obj-animation-bank'));
                  if(!q('#obj-animation-loop').checked || q('#obj-animation-rate').value!=='10' || q('#obj-animation-stop').disabled)throw Error('bank settings');
                  q('#obj-animation-stop').click();return q('#obj-animation-stop').disabled;
                })()'''.replace('EXPECTED_BADGE', json.dumps(expected))))
            finally:
                browser.close()


if __name__ == '__main__':
    unittest.main()
