"""Check lazy cross-scope search of verified localized name references."""
import json
import shutil
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import view_rom_data
from test_viewer_action_names_browser import fixture as action_fixture
from test_viewer_item_names_browser import fixture as item_fixture
from test_viewer_job_sprite_links_browser import fixture as job_fixture
from test_viewer_scoped_browser import parse_page
from test_viewer_special_units_browser import wait_for
from test_viewer_world_names_browser import generate_fixture as region_fixture
from viewer_browser_harness import Chrome


class NameIndexProjectionTest(unittest.TestCase):
    def test_source_gates_glyphs_and_language_gaps(self):
        source = {'languages': [0, 1], 'glyphs': [{'index': n} for n in (2, 176, 203, 228)],
                  'rows': [{'id': 0, 'names': [[2, [64], 176, 203, 1000], None]},
                           {'id': 1, 'names': [[1000], [176, 228]]}]}
        metadata = {'languages': source['languages'], 'glyphs': source['glyphs'],
                    'rows': [dict(row, routing=(0 if row['id']==0 else 255)) for row in source['rows']]}
        data = {'profile': 'europe', 'items': {'records': [1, 2]}, 'item_names': source,
                'metadata_names': metadata, 'job_sprite_links': {'names_verified': True}}
        entries = view_rom_data.project_wiki_name_index(data)
        self.assertEqual([(row['kind'], row['id']) for row in entries], [('item', 0), ('item', 1), ('job', 0)])
        self.assertEqual(entries[0]['names']['0']['text'], 'À Ab�')
        self.assertNotIn('1', entries[0]['names']); self.assertNotIn('0', entries[1]['names'])
        data['job_sprite_links']['names_verified'] = False
        self.assertFalse(any(row['kind']=='job' for row in view_rom_data.project_wiki_name_index(data)))
        data['profile'] = 'usa'; self.assertEqual(view_rom_data.project_wiki_name_index(data), [])

    def test_generated_name_asset_is_not_embedded_in_index(self):
        with tempfile.TemporaryDirectory(prefix='viewer-name-index-') as directory:
            page, _ = item_fixture(Path(directory)); home = parse_page(page); asset = home['wiki']['name_search']
            self.assertNotIn('item_names', home)
            path = page.parent / asset['href']; self.assertTrue(path.is_file())
            raw = path.read_text(); payload = json.loads(raw.split(' = ', 1)[1].removesuffix(';\n'))
            self.assertEqual(payload['key'], asset['key']); self.assertEqual(len(payload['entries']), 30)
            self.assertEqual(payload['entries'][0]['names']['0']['text'], 'A b')
            self.assertNotIn('A b', page.read_text())
            self.assertNotIn('pointer_texts', home)

    def test_observed_nodes_require_a_present_map_and_source_names(self):
        source = {'languages': [0, 1], 'glyphs': [{'index': n} for n in (176, 203)],
                  'locations': [{'node_index': 19, 'names': [[176, 203], [203]]},
                                {'node_index': 99, 'names': [[176], None]}]}
        data = {'profile': 'europe', 'world_location_names': source, 'world_nodes': {'nodes': [{'index': 19}]}}
        entries = view_rom_data.project_wiki_name_index(data)
        self.assertEqual([(row['kind'], row['id']) for row in entries], [('node', 19)])
        self.assertEqual(entries[0]['names']['0']['text'], 'Ab')
        self.assertEqual(entries[0]['names']['1']['text'], 'b')
        data['world_nodes'] = None; self.assertEqual(view_rom_data.project_wiki_name_index(data), [])
        data['profile'] = 'usa'; self.assertEqual(view_rom_data.project_wiki_name_index(data), [])


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class NameSearchBrowserTest(unittest.TestCase):
    def open_search(self, browser):
        self.assertTrue(browser.evaluate("typeof globalThis.__FFTA_WIKI_NAMES__==='undefined' && !document.querySelector('#wiki-name-search').hidden"))
        browser.evaluate("document.querySelector('#wiki-name-search>details').open=true")
        wait_for(browser, "Array.isArray(globalThis.__FFTA_WIKI_NAMES__?.entries) && document.querySelector('#wiki-name-rows').rows.length>0")

    def test_item_search_paging_language_and_direct_record(self):
        with tempfile.TemporaryDirectory(prefix='viewer-name-search-item-') as directory:
            work=Path(directory); page,_=item_fixture(work); browser=Chrome(work,page)
            try:
                self.open_search(browser)
                target=browser.evaluate('''(()=>{const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change')),
                  input=e=>e.dispatchEvent(new Event('input'));
                  if(q('#wiki-name-rows').rows.length!==25 || !q('#wiki-name-status').textContent.includes('30 results'))throw Error('paging');
                  q('#wiki-name-next').click();if(q('#wiki-name-rows').rows.length!==5)throw Error('last page');
                  q('#wiki-name-query').value='A b';input(q('#wiki-name-query'));
                  if(q('#wiki-name-rows').rows.length!==10)throw Error('name query');
                  q('#wiki-name-language').value='1';change(q('#wiki-name-language'));
                  if(q('#wiki-name-rows').rows.length!==10 || q('#wiki-name-rows tr').dataset.recordId!=='1')throw Error('language');
                  return q('#wiki-name-rows tr a').href;})()''')
                browser.command('Page.navigate',{'url':target})
                wait_for(browser,"data.wiki.current==='world' && document.querySelector('#item-id').value==='1'")
                self.assertTrue(browser.evaluate("document.querySelector('#item-name-language').value==='1' && document.querySelector('#items>details').open"))
                browser.click('#wiki-page-links a[href$="#wiki-name-search"]')
                wait_for(browser,"data.wiki.current==='index' && document.querySelector('#wiki-name-search>details').open && !!globalThis.__FFTA_WIKI_NAMES__")
            finally:browser.close()

    def test_actions_jobs_and_regions_open_exact_records(self):
        for kind,build in [('action',action_fixture),('job',job_fixture),('region',region_fixture)]:
            with self.subTest(kind=kind),tempfile.TemporaryDirectory(prefix='viewer-name-search-cross-') as directory:
                work=Path(directory);page,_=build(work);browser=Chrome(work,page)
                try:
                    self.open_search(browser)
                    target=browser.evaluate('''(()=>{const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change')),
                      input=e=>e.dispatchEvent(new Event('input'));
                      q('#wiki-name-kind').value=KIND;change(q('#wiki-name-kind'));q('#wiki-name-query').value=ID;input(q('#wiki-name-query'));
                      const row=q('#wiki-name-rows tr[data-record-id="'+ID+'"]');if(!row || !row.querySelector('a'))throw Error('missing record');
                      return row.querySelector('a').href;})()'''.replace('KIND',json.dumps(kind)).replace('ID',json.dumps('16' if kind=='region' else '0')))
                    browser.command('Page.navigate',{'url':target})
                    if kind=='action':
                        wait_for(browser,"data.wiki.current==='world' && document.querySelector('#action-record-id').value==='0'")
                        self.assertTrue(browser.evaluate("document.querySelector('#action-records>details').open"))
                    elif kind=='job':
                        wait_for(browser,"data.wiki.current==='world' && document.querySelector('#metadata-name-filter').value==='0'")
                        self.assertTrue(browser.evaluate("document.querySelector('#metadata-name-filter-mode').value==='exact-id' && document.querySelector('#metadata-name-rows').rows.length===1"))
                    else:
                        wait_for(browser,"data.wiki.current==='world-map' && document.querySelector('#world-region-filter').value==='16'")
                        self.assertTrue(browser.evaluate("document.querySelector('#world-region-filter-mode').value==='exact-id' && document.querySelector('#world-region-rows').rows.length===1 && document.querySelector('#world-region-rows tr').dataset.regionId==='16'"))
                finally:browser.close()

    def test_no_unverified_index(self):
        with tempfile.TemporaryDirectory(prefix='viewer-name-search-missing-') as directory:
            work=Path(directory);page,_=item_fixture(work,with_text=False);browser=Chrome(work,page)
            try:self.assertTrue(browser.evaluate("!data.wiki.name_search && document.querySelector('#wiki-name-search').hidden && typeof globalThis.__FFTA_WIKI_NAMES__==='undefined'"))
            finally:browser.close()

    def test_observed_node_search_selects_map_and_language(self):
        with tempfile.TemporaryDirectory(prefix='viewer-name-search-node-') as directory:
            work = Path(directory); page, _ = region_fixture(work); browser = Chrome(work, page)
            try:
                self.open_search(browser)
                target = browser.evaluate('''(()=>{const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change'));
                  q('#wiki-name-kind').value='node';change(q('#wiki-name-kind'));q('#wiki-name-language').value='1';change(q('#wiki-name-language'));
                  const row=q('#wiki-name-rows tr[data-record-id="19"]');
                  if(!row || row.cells[0].textContent!=='Observed world nodes' || row.cells[2].textContent!=='d')throw Error('observed node');
                  return row.querySelector('a').href;})()''')
                browser.command('Page.navigate', {'url': target})
                wait_for(browser, "data.wiki.current==='world-map' && document.querySelector('#world-detail').textContent.includes('node 19')")
                self.assertTrue(browser.evaluate('''(()=>{const q=s=>document.querySelector(s);return q('#world>details').open&&
                  q('#world-name-language').value==='1'&&q('#world-selected-name').textContent==='Observed name: d';})()'''))
                invalid = (page.parent / (page.name + '.parts') / 'world-map.html').as_uri() + '?view=world&node_id=99999#world'
                browser.command('Page.navigate', {'url': invalid})
                wait_for(browser, "data.wiki.current==='world-map' && document.querySelector('#world-detail').textContent.includes('node 0')")
                self.assertTrue(browser.evaluate("document.querySelector('#world>details').open"))
            finally:
                browser.close()
