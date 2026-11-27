"""Check source-row associations and the controlled position preview."""
import copy
import json
import shutil
import tempfile
import time
import unittest
from pathlib import Path

from test_viewer_screen_update_curves import parts, sources
from test_viewer_scoped_browser import REPO, parse_page, prepared
from view_rom_data import project_screen_update_curves, project_screen_update_grids, viewer_section_updates, write_split_viewer
from viewer_browser_harness import Chrome


def catalogs():
    chunks = parts(); curves = project_screen_update_curves(b''.join(c for _, _, c in chunks), sources(chunks))
    groups = [{'id': 'other', 'source_offset': 0x3000, 'rows': [{'value': 1}], 'links': [[]]}]
    for bank, (name, root) in enumerate((('grid_first', 0x3a9abc), ('grid_second', 0x3aae44))):
        rows = [{'x': index - 12, 'y': index * 2 - 24,
                 'value': 0x08000000 + curves['banks'][bank]['curves'][4 + index % 13]['source_offset']}
                for index in range(24)]
        groups.append({'id': name, 'source_offset': root, 'rows': rows, 'links': [[] for _ in rows]})
    obj = {'x': 0, 'y': 0, 'affine_mode': 0, 'object_mode': 0, 'mosaic': False, 'color_256': False,
           'shape': 0, 'transform': 0, 'size': 0, 'tile': 0, 'priority': 0, 'palette': 0}
    bank = {'object_root': 0x1004, 'sequence_root': 0x1100, 'flags': 12,
            'lists': [{'source_offset': 0x1030, 'source_index': 0, 'relative_offset': 44, 'objects': [obj]}],
            'sequences': [{'selector': 0, 'source_offset': 0x1104, 'relative_offset': 4,
                           'frames': [{'object_list': 0, 'list_id': 0, 'duration': 1}]}]}
    catalog = {'path': 'compact.json', 'banks': [bank], 'selector_tables': [], 'auxiliary': {'groups': groups}}
    curves['path'] = 'screen-update-tables.bin'
    return curves, catalog


class ScreenUpdateGridProjectionTest(unittest.TestCase):
    def test_joins_both_tables_by_source_pointer(self):
        curves, catalog = catalogs(); joined = project_screen_update_grids(curves, catalog)
        self.assertEqual([len(bank['rows']) for bank in joined['banks']], [24, 24])
        self.assertEqual([bank['group'] for bank in joined['banks']], [1, 2])
        self.assertEqual(joined['banks'][1]['rows'][14]['curve'], 5)
        self.assertIsNone(project_screen_update_grids(curves, None))

    def test_rejects_unknown_pointers_wrong_extents_and_ambiguous_groups(self):
        mutations = [lambda c: c['auxiliary']['groups'][1]['rows'][0].update(value=123),
                     lambda c: c['auxiliary']['groups'][1].update(source_offset=0),
                     lambda c: c['auxiliary']['groups'][1]['rows'].pop(),
                     lambda c: c['auxiliary']['groups'].append(copy.deepcopy(c['auxiliary']['groups'][1]))]
        for mutate in mutations:
            curves, catalog = catalogs(); mutate(catalog)
            with self.assertRaises(ValueError): project_screen_update_grids(curves, catalog)


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class ScreenUpdateGridBrowserTest(unittest.TestCase):
    def test_controlled_positions_playback_curve_selection_and_source_link(self):
        with tempfile.TemporaryDirectory(prefix='viewer-grid-browser-') as directory:
            work = Path(directory); data, index = prepared(work); curves, catalog = catalogs()
            curves['grids'] = project_screen_update_grids(curves, catalog)
            data['screen_update_curves'] = curves; data['obj_animation_compact'] = {'compact_catalog': catalog}
            data['section_updates'] = viewer_section_updates()
            data['resources'].append({'id': 'graphics.screen-update-tables', 'path': 'screen-update-tables.bin',
                                      'format': 'binary', 'description': 'Screen curves', 'status': 'Available'})
            index = write_split_viewer(data, index.parent, index.name, (REPO / 'view_rom_data.html').read_text())
            folder = index.parent / (index.name + '.parts'); page = parse_page(folder / 'interface-curves.html')
            self.assertEqual(len(page['screen_update_curves']['grids']['banks'][0]['rows']), 24)
            self.assertNotIn('obj_animation_compact', page)
            browser = Chrome(work, folder / 'interface-curves.html')
            try:
                expected = viewer_section_updates()['interface-curves']['screen-update-curves']
                self.assertTrue(browser.evaluate('''(()=>{const e=EXPECTED,q=s=>document.querySelector(s),change=x=>x.dispatchEvent(new Event('change'));
                  window.pending=new Map();let serial=0;window.setTimeout=(f,ms)=>{let id=++serial;pending.set(id,{f,ms});return id};
                  window.clearTimeout=id=>pending.delete(id);window.fire=()=>{let [id,t]=pending.entries().next().value;pending.delete(id);t.f()};
                  if(q('#screen-grid').hidden||q('#screen-update-curves>details>summary time').dateTime!==e.at)return false;
                  q('#screen-update-curves>details').open=true;q('#screen-grid').open=true;q('#screen-grid').dispatchEvent(new Event('toggle'));
                  if(q('#screen-grid-values').rows.length!==24||q('#screen-grid-values').rows[0].cells[5].textContent!=='56')return false;
                  const firstRow=q('#screen-grid-values').rows[0];
                  q('#screen-curve-loop').checked=true;q('#screen-curve-rate').value='10';q('#screen-curve-play').click();fire();
                  if(q('#screen-grid-values').rows[0]!==firstRow||firstRow.cells[5].textContent!=='57')return false;
                  q('#screen-grid-row').value='14';change(q('#screen-grid-row'));q('#screen-grid-show-curve').click();
                  if(q('#screen-curve-id').value!=='5'||!q('#screen-curve-loop').checked||q('#screen-curve-rate').value!=='10')return false;
                  q('#screen-curve-bank').value='1';change(q('#screen-curve-bank'));
                  if(!q('#screen-grid-current').textContent.includes('Row 14'))return false;
                  q('#screen-grid-source-link').click();return true;})()'''.replace('EXPECTED', json.dumps(expected))))
                deadline = time.monotonic() + 30
                while time.monotonic() < deadline:
                    try:
                        if browser.evaluate('document.readyState === "complete" && data.wiki.current === "objects"'):
                            break
                    except (RuntimeError, OSError):
                        pass
                    time.sleep(.05)
                else:
                    self.fail('Source table page did not load')
                self.assertTrue(browser.evaluate('''(()=>{const q=s=>document.querySelector(s);return q('#compact-catalog-tables>details').open&&
                  q('#compact-table-group').value==='2'&&q('#compact-table-rows').rows.length===24;})()'''))
            finally:
                browser.close()


if __name__ == '__main__':
    unittest.main()
