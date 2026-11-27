"""Exercise links between separate physical and witnessed compact pages."""
import copy
import json
import shutil
import tempfile
import time
import unittest
from pathlib import Path

from test_viewer_scoped_browser import REPO, parse_page, prepared
from view_rom_data import viewer_section_updates, write_split_viewer
from viewer_browser_harness import Chrome


def fixture(work):
    data, index = prepared(work)
    obj = {'x': 0, 'y': 0, 'affine_mode': 0, 'object_mode': 0, 'mosaic': False, 'color_256': False,
           'shape': 0, 'transform': 0, 'size': 0, 'tile': 0, 'priority': 0, 'palette': 0}
    def item(offset, index):
        return {'source_offset': offset, 'source_index': index, 'relative_offset': offset - 0x1004, 'objects': [copy.deepcopy(obj)]}
    def seq(selector, index):
        return {'selector': selector, 'source_offset': 0x1100 + selector * 4, 'relative_offset': selector * 4,
                'frames': [{'object_list': index, 'list_id': 2, 'duration': 3}]}
    old = {'object_root': 0x1004, 'sequence_root': 0x1100, 'flags': 12,
           'lists': [item(0x1030, 2)], 'sequences': [seq(0, 0), seq(6, 0)]}
    new = {'object_root': 0x1004, 'sequence_root': 0x1100, 'flags': 12,
           'lists': [item(0x1010, 0), item(0x1020, 1), item(0x1030, 2)],
           'sequences': [seq(0, 2), seq(6, 2)]}
    other = copy.deepcopy(new); other['object_root'] = 0x2004; other['sequence_root'] = 0x2100
    data['obj_animation_compact'] = {'compact_catalog': {'path': 'old.json', 'banks': [old], 'selector_tables': [],
        'auxiliary': {'groups': [{'id': 'schedule', 'source_offset': 0x3000, 'rows': [{'selector': 6}],
                                  'links': [[{'field': 'selector', 'bank': 0, 'sequence': 1, 'selector': 6}]]}]}}}
    data['physical_obj_animations'] = {'banks': [other, new], 'selector_tables': [], 'path': 'physical.bin'}
    data['section_updates'] = viewer_section_updates()
    data['resources'].append({'id': 'graphics.physical-compact-obj-banks', 'path': 'physical.bin', 'format': 'binary',
                              'description': 'Physical compact banks', 'status': 'Available'})
    return write_split_viewer(data, index.parent, index.name, (REPO / 'view_rom_data.html').read_text())


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class WitnessedCompactLinksBrowserTest(unittest.TestCase):
    def test_split_navigation_restores_bank_and_selector_and_source_references(self):
        with tempfile.TemporaryDirectory(prefix='viewer-compact-crosslinks-') as directory:
            work = Path(directory); index = fixture(work); parts = index.parent / (index.name + '.parts')
            old, physical, home = (parse_page(path) for path in (parts / 'objects.html', parts / 'objects-physical.html', index))
            self.assertEqual(len(old['compact_witnessed_links']), 2)
            self.assertEqual(old['compact_witnessed_links'], physical['compact_witnessed_links'])
            self.assertNotIn('compact_witnessed_links', home)
            self.assertNotIn('physical_obj_animations', old)
            self.assertNotIn('obj_animation_compact', physical)
            browser = Chrome(work, parts / 'objects.html')
            def wait_for(scope):
                deadline = time.monotonic() + 30
                while time.monotonic() < deadline:
                    try:
                        if browser.evaluate('document.readyState === "complete" && data.wiki.current === ' + json.dumps(scope)):
                            return
                    except (RuntimeError, OSError):
                        pass
                    time.sleep(.05)
                self.fail('Page did not load: ' + scope)
            try:
                self.assertTrue(browser.evaluate('''(()=>{const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change'));
                  q('#obj-animation-catalog').value='compact_catalog';change(q('#obj-animation-catalog'));
                  q('#obj-animation-sequence').value='1';change(q('#obj-animation-sequence'));
                  const a=q('#obj-animation-witnessed-link a');if(!a || !a.textContent.includes('physical bank'))return false;a.click();return true;})()'''))
                wait_for('objects-physical')
                self.assertTrue(browser.evaluate('''(()=>{const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change'));
                  if(q('#obj-animation-catalog').value!=='physical_catalog'||q('#obj-animation-bank').value!=='1'||
                    q('#obj-animation-sequence').value!=='1'||!q('#obj-animations>details').open)return false;
                  q('#obj-animation-bank').value='0';change(q('#obj-animation-bank'));
                  if(!q('#obj-animation-witnessed-link').hidden)return false;
                  q('#obj-animation-bank').value='1';change(q('#obj-animation-bank'));
                  q('#obj-animation-sequence').value='1';change(q('#obj-animation-sequence'));
                  const a=q('#obj-animation-witnessed-link a');if(!a || !a.textContent.includes('source references'))return false;a.click();return true;})()'''))
                wait_for('objects')
                self.assertTrue(browser.evaluate('''(()=>{const q=s=>document.querySelector(s);return q('#obj-animation-catalog').value==='compact_catalog'&&
                  q('#obj-animation-bank').value==='0'&&q('#obj-animation-sequence').value==='1'&&
                  q('#obj-animation-catalog-references').textContent.includes('schedule #0')&&
                  q('#obj-animations>details').open;})()'''))
            finally:
                browser.close()


if __name__ == '__main__':
    unittest.main()
