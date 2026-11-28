"""Verify grouped selection pool and weighted-row previews."""
import copy
import hashlib
import json
import shutil
import struct
import tempfile
import unittest
from pathlib import Path

from view_rom_data import (generate, project_grouped_selection_tables,
                           project_white_flowers_first_row, viewer_section_updates)
from viewer_browser_harness import Chrome


def fixture_data():
    literals = (0x03b0a8, 0x0d3a5c, 0x0d3aa4, 0x0d3af0, 0x0d3b34, 0x0d5db0, 0x0d5fbc, 0x0d6ac4)
    segments = [(offset, 'reader_literals', bytes(4)) for offset in literals]
    segments += [(0x6e89bc, 'halfword_pools', b''.join(struct.pack('<H', n) for n in range(120))),
                 (0x6e900c, 'halfword_pools', b''.join(struct.pack('<H', n + 1000) for n in range(360))),
                 (0x6e92dc, 'weighted_rows', b''.join(struct.pack('<4B2HI', n % 101, n % 256, 2, 3,
                                                                     n + 200, n + 400, n + 600) for n in range(160)))]
    first = list(range(9000, 9020))
    catalog = {'row_offset': 0x6e8994, 'row': first, 'byte_offset': 0x3c9360, 'byte': 0}
    return segments, catalog


def sources(segments):
    return [{'offset': offset, 'role': role, 'length': len(raw), 'sha256': hashlib.sha256(raw).hexdigest()}
            for offset, role, raw in segments]


def write_fixture(work, *, invalid_group=False, invalid_first=False, missing_first=False, reverse=False):
    dataset = work / 'dataset'; dataset.mkdir(); chunks, catalog = fixture_data(); raw = b''.join(chunk for _, _, chunk in chunks)
    white = json.dumps(catalog).encode(); (dataset / 'white.json').write_bytes(white); (dataset / 'group.bin').write_bytes(raw)
    group_source = sources(chunks)
    if invalid_group: group_source[-1]['sha256'] = '0' * 64
    white_source = sources([(0x6e8994, 'white-flowers-first-halfword-row', struct.pack('<20H', *catalog['row']))])
    if invalid_first: white_source[0]['sha256'] = '0' * 64
    def resource(identity, path, fmt, content, entries):
        block = (f'[[resources]]\nid="{identity}"\npath="{path}"\nformat="{fmt}"\n'
                 f'sha256="{hashlib.sha256(content).hexdigest()}"\n')
        for source in entries:
            block += (f'[[resources.sources]]\noffset={source["offset"]}\nlength={source["length"]}\n'
                      f'role="{source["role"]}"\nsha256="{source["sha256"]}"\n')
        return block
    groups = resource('gameplay.grouped-selection-tables', 'group.bin', 'grouped-selection-tables-v1-bin', raw, group_source)
    prior = resource('engine.white-flowers-isolated-values', 'white.json', 'white-flowers-isolated-values-v1-json', white, white_source)
    blocks = [groups, prior] if reverse else [prior, groups]
    if missing_first: blocks = [groups]
    (dataset / 'manifest.toml').write_text('profile="europe"\nextractor_revision=363\ndecoder_revision=362\n' + ''.join(blocks))
    return generate(dataset, work / 'view', split=True)


def payload(path):
    return json.JSONDecoder().raw_decode(path.read_text().split('const data = ', 1)[1])[0]


class GroupedSelectionProjectionTest(unittest.TestCase):
    def test_pools_and_weighted_field_widths(self):
        chunks, catalog = fixture_data(); first = project_white_flowers_first_row(
            catalog, sources([(0x6e8994, 'white-flowers-first-halfword-row', struct.pack('<20H', *catalog['row']))]))
        result = project_grouped_selection_tables(b''.join(chunk for _, _, chunk in chunks), sources(chunks), first)
        self.assertEqual([len(bank['groups']) for bank in result['banks']], [7, 7, 1, 10])
        self.assertEqual([result['banks'][0]['groups'][0][0], result['banks'][0]['groups'][1][0],
                          result['banks'][1]['groups'][0][0], result['banks'][2]['groups'][0][0],
                          result['banks'][3]['groups'][9][19]], [9000, 0, 1000, 1140, 1359])
        self.assertEqual(result['weighted_rows'][159], [58, 159, 2, 3, 359, 559, 759])
        self.assertEqual(result['observed_group_starts'], [])

    def test_bad_source_shapes_and_first_row_are_rejected(self):
        chunks, catalog = fixture_data(); raw = b''.join(chunk for _, _, chunk in chunks)
        bad = sources(chunks); bad[-1]['sha256'] = '0' * 64
        with self.assertRaises(ValueError): project_grouped_selection_tables(raw, bad)
        bad = sources(chunks); bad[-2]['offset'] += 2
        with self.assertRaises(ValueError): project_grouped_selection_tables(raw, bad)
        with self.assertRaises(ValueError): project_grouped_selection_tables(raw[:-1], sources(chunks))
        source = sources([(0x6e8994, 'white-flowers-first-halfword-row', struct.pack('<20H', *catalog['row']))])
        for mutate in (lambda c, s: c['row'].pop(), lambda c, s: c['row'].__setitem__(0, True),
                       lambda c, s: s[0].update(offset=0), lambda c, s: s[0].update(sha256='0' * 64)):
            c, s = copy.deepcopy(catalog), copy.deepcopy(source); mutate(c, s)
            with self.assertRaises(ValueError): project_white_flowers_first_row(c, s)

    def test_scoping_order_missing_dependency_and_invalid_group(self):
        for options in ({}, {'reverse': True}, {'invalid_first': True}, {'missing_first': True}, {'invalid_group': True}):
            with self.subTest(options=options), tempfile.TemporaryDirectory(prefix='viewer-grouped-scope-') as directory:
                index = write_fixture(Path(directory), **options); folder = index.parent / (index.name + '.parts')
                home, page, world = map(payload, (index, folder / 'selection-groups.html', folder / 'world.html'))
                self.assertNotIn('grouped_selection_tables', home); self.assertNotIn('grouped_selection_tables', world)
                data = page['grouped_selection_tables']
                if options.get('invalid_group'):
                    self.assertIsNone(data)
                else:
                    self.assertEqual(data['banks'][0]['groups'][0] is None, bool(options.get('missing_first') or options.get('invalid_first')))
                    self.assertEqual(len(data['weighted_rows']), 160)
                sections = [s['id'] for p in home['wiki']['pages'] if p['id'] == 'selection-groups' for s in p.get('sections', [])]
                self.assertEqual(len(sections), 0 if options.get('invalid_group') else 2)


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class GroupedSelectionBrowserTest(unittest.TestCase):
    def test_pool_search_weight_rows_navigation_and_badges(self):
        with tempfile.TemporaryDirectory(prefix='viewer-grouped-browser-') as directory:
            work = Path(directory); index = write_fixture(work); page = index.parent / (index.name + '.parts') / 'selection-groups.html'
            browser = Chrome(work, page)
            try:
                expected = viewer_section_updates()['selection-groups']
                self.assertTrue(browser.evaluate('''(()=>{const expected=EXPECTED,q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change'));
                  for(const name of ['grouped-selection-pools','grouped-selection-weights']){
                    const badge=q('#'+name+'>details>summary time');if(!badge||badge.dateTime!==expected[name].at||!badge.textContent.startsWith('Added '))throw Error('badge '+name);
                    if(!q('[data-resource-id="gameplay.grouped-selection-tables"] a[href="#'+name+'"]'))throw Error('resource link '+name);
                  }
                  if(q('#grouped-pool-rows').rows.length!==20||q('#grouped-pool-rows').rows[0].cells[2].textContent!=='9000')throw Error('first pool');
                  q('#grouped-pool-bank').value='3';change(q('#grouped-pool-bank'));q('#grouped-pool-group').value='9';change(q('#grouped-pool-group'));
                  if(q('#grouped-pool-rows').rows[19].cells[2].textContent!=='1359')throw Error('last pool');
                  q('#grouped-pool-search').value='0x2328';q('#grouped-pool-find').click();
                  if(!q('#grouped-pool-search-status').textContent.startsWith('1 match'))throw Error('search');
                  q('#grouped-pool-matches button').click();if(q('#grouped-pool-bank').value!=='0'||q('#grouped-pool-group').value!=='0')throw Error('search navigation');
                  q('#grouped-pool-search').value='65536';q('#grouped-pool-find').click();if(!q('#grouped-pool-search-status').textContent.includes('Enter'))throw Error('bad search');
                  q('#grouped-weight-range').value='140:160';change(q('#grouped-weight-range'));
                  const row=q('#grouped-weight-rows').rows[19];
                  if(row.cells[0].textContent!=='159'||row.cells[2].textContent.trim()!=='58'||row.cells[4].textContent!=='0x0167'||row.cells[6].textContent!=='0x000002F7')throw Error('weight row');
                  return true;})()'''.replace('EXPECTED', json.dumps(expected))))
            finally:
                browser.close()

    def test_missing_first_row_leaves_other_banks_browsable(self):
        with tempfile.TemporaryDirectory(prefix='viewer-grouped-missing-') as directory:
            work = Path(directory); index = write_fixture(work, missing_first=True); page = index.parent / (index.name + '.parts') / 'selection-groups.html'
            browser = Chrome(work, page)
            try:
                self.assertTrue(browser.evaluate('''(()=>{const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change'));
                  if(!q('#grouped-pool-status').textContent.includes('unavailable')||q('#grouped-pool-rows').rows.length)throw Error('missing');
                  q('#grouped-pool-group').value='1';change(q('#grouped-pool-group'));
                  if(q('#grouped-pool-rows').rows.length!==20)throw Error('remaining');
                  q('#grouped-pool-search').value='0';q('#grouped-pool-find').click();
                  return q('#grouped-pool-search-status').textContent.includes('one unavailable group');})()'''))
            finally:
                browser.close()


if __name__ == '__main__':
    unittest.main()
