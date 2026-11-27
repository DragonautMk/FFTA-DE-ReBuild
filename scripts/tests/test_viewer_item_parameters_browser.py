"""Check the bounded item parameter view and source-gated item references."""
import copy
import hashlib
import json
import shutil
import sys
import tempfile
import unittest
from contextlib import ExitStack
from pathlib import Path
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import view_rom_data
from test_viewer_item_names_browser import fixture as item_fixture
from test_viewer_scoped_browser import parse_page
from viewer_browser_harness import Chrome


def fixture(work, *, wrong_items=False, wrong_parameters=False, with_text=True, profile='europe', split=True):
    item_fixture(work, profile=profile, with_text=with_text)
    dataset = work / 'dataset'; manifest = dataset / 'manifest.toml'; text = manifest.read_text()
    item_path = dataset / 'items.json'; old = item_path.read_bytes(); items = json.loads(old)
    for index, record in enumerate(items['records']):
        record['bytes_18_1f'][0] = 1 if index < 27 else 2
    raw = json.dumps(items).encode(); item_path.write_bytes(raw); item_sha = hashlib.sha256(raw).hexdigest()
    text = text.replace(hashlib.sha256(old).hexdigest(), item_sha)
    parameters = {'source_offset': 0x6e84e0, 'records': [
        {'byte00': index, 'byte01': index + 1, 'value02': 1000 + index} for index in range(31)]}
    raw = json.dumps(parameters).encode(); digest = hashlib.sha256(raw).hexdigest(); (dataset / 'parameters.json').write_bytes(raw)
    text += (f'[[resources]]\nid="gameplay.conditional-item-parameters"\npath="parameters.json"\n'
             f'format="conditional-item-parameters-v1-json"\nsha256="{digest}"\n')
    manifest.write_text(text)
    text_sha = hashlib.sha256((dataset / 'text.json').read_bytes()).hexdigest() if with_text else 'missing'
    with ExitStack() as stack:
        stack.enter_context(patch.object(view_rom_data, 'EUROPE_ITEMS_SHA256', 'wrong' if wrong_items else item_sha))
        stack.enter_context(patch.object(view_rom_data, 'EUROPE_ITEM_PARAMETERS_SHA256', 'wrong' if wrong_parameters else digest))
        stack.enter_context(patch.object(view_rom_data, 'EUROPE_TEXT_CATALOG_SHA256', text_sha))
        page = view_rom_data.generate(dataset, work / 'parameters-view', split=split)
    return page, parameters


class ItemParameterProjectionTest(unittest.TestCase):
    def test_records_scope_and_authentication_guards(self):
        for options in ({}, {'wrong_items': True}, {'wrong_parameters': True}, {'with_text': False}, {'profile': 'usa'}):
            with self.subTest(options=options), tempfile.TemporaryDirectory(prefix='viewer-item-parameters-project-') as directory:
                page, parameters = fixture(Path(directory), **options); parts = page.parent / 'index.html.parts'
                home, world, audio, maps = map(parse_page, (page, parts/'world.html', parts/'audio.html', parts/'world-map.html'))
                self.assertNotIn('item_parameters', home); self.assertNotIn('item_parameters', audio)
                self.assertNotIn('item_parameters', maps)
                if options.get('profile') == 'usa':
                    self.assertIsNone(world['item_parameters']); self.assertIsNone(world['item_parameter_links']); continue
                self.assertEqual(world['item_parameters']['records'], parameters['records'])
                links = world['item_parameter_links']
                if options.get('wrong_items') or options.get('wrong_parameters'):
                    self.assertIsNone(links); continue
                self.assertEqual(links['selectors'], [1] * 27 + [2] * 3)
                self.assertEqual(links['names_verified'], options.get('with_text', True))
                self.assertIn('item-parameters', [section['id'] for page in home['wiki']['pages']
                                                  if page['id'] == 'world' for section in page['sections']])
                with patch.object(view_rom_data, 'EUROPE_ITEMS_SHA256', 'items'), \
                     patch.object(view_rom_data, 'EUROPE_ITEM_PARAMETERS_SHA256', 'parameters'):
                    altered = copy.deepcopy(world['items']); altered['records'][0]['bytes_18_1f'][0] = 31
                    self.assertIsNone(view_rom_data.link_item_parameters('europe', 'items', 'parameters', altered, parameters, None))
                    self.assertIsNone(view_rom_data.link_item_parameters('usa', 'items', 'parameters', world['items'], parameters, None))
        bad = {'source_offset': 0x6e84e0, 'records': [{'byte00': 0, 'byte01': 0, 'value02': 0} for _ in range(31)]}
        for mutate in (lambda c: c.update(source_offset=0), lambda c: c['records'].pop(),
                       lambda c: c['records'][0].update(byte00=True), lambda c: c['records'][0].update(byte01=256),
                       lambda c: c['records'][0].update(value02=65536)):
            altered = copy.deepcopy(bad); mutate(altered)
            with self.assertRaises(ValueError): view_rom_data.project_item_parameters(altered)


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class ItemParameterBrowserTest(unittest.TestCase):
    def test_bidirectional_links_names_filter_and_bookmark(self):
        with tempfile.TemporaryDirectory(prefix='viewer-item-parameters-browser-') as directory:
            work = Path(directory); page, _ = fixture(work); browser = Chrome(work, page.parent / 'index.html.parts/world.html')
            try:
                bookmark = browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change',{bubbles:true}));
                  if(q('#item-parameters').hidden || q('#item-parameters>details').open || q('#item-parameter-id').options.length!==31)throw Error('section');
                  q('#item-id').value='1';change(q('#item-id'));q('#item-parameter-open').click();
                  if(q('#item-parameter-id').value!=='1' || !q('#item-parameters>details').open ||
                     !q('#item-parameter-source').textContent.includes('6E84E4') ||
                     q('#item-parameter-fields').rows[2].cells[2].textContent!=='1001' ||
                     !q('#item-parameter-page').textContent.includes('27 items') || q('#item-parameter-rows').rows.length!==25)throw Error('item to parameters');
                  q('#item-parameter-next').click();if(q('#item-parameter-rows').rows.length!==2)throw Error('pagination');
                  q('#item-parameter-filter').value='A b';q('#item-parameter-filter').dispatchEvent(new Event('input'));
                  if(q('#item-parameter-rows').rows.length!==9)throw Error('name filter');
                  q('#item-parameter-filter').value='26';q('#item-parameter-filter').dispatchEvent(new Event('input'));
                  q('#item-parameter-rows a').click();if(q('#item-id').value!=='26')throw Error('parameters to item');
                  q('#item-name-language').value='1';change(q('#item-name-language'));
                  q('#item-parameter-filter').value='';q('#item-parameter-filter').dispatchEvent(new Event('input'));
                  if(!q('#item-parameter-rows tr[data-item-id="1"] td:nth-child(2)').textContent.includes('A b'))throw Error('language');
                  const link=q('#item-parameters>details>p>a');link.dispatchEvent(new Event('focus'));return link.href;
                })()''')
                browser.command('Page.navigate', {'url': bookmark})
                from test_viewer_navbar_split_browser import wait_for
                wait_for(browser, "data.wiki.current==='world' && document.querySelector('#item-parameter-id').value==='1'")
                self.assertTrue(browser.evaluate("document.querySelector('#item-parameters>details').open"))
            finally:
                browser.close()

    def test_unverified_sources_have_no_links_and_missing_names_use_ids(self):
        for options in ({'wrong_items': True}, {'wrong_parameters': True}, {'with_text': False}):
            with self.subTest(options=options), tempfile.TemporaryDirectory(prefix='viewer-item-parameters-guard-') as directory:
                work = Path(directory); page, _ = fixture(work, split=False, **options); browser = Chrome(work, page)
                try:
                    result = browser.evaluate('''(()=>{const q=s=>document.querySelector(s);
                      q('#item-id').value='1';q('#item-id').dispatchEvent(new Event('change'));
                      q('#item-parameter-id').value='1';q('#item-parameter-id').dispatchEvent(new Event('change'));
                      return {hidden:q('#item-parameter-navigation').hidden,references:q('#item-parameter-references').hidden,
                        unavailable:q('#item-parameter-unavailable').hidden,label:q('#item-parameter-rows tr[data-item-id="1"] td:nth-child(2)')?.textContent};})()''')
                    verified = options.get('with_text') is False
                    self.assertEqual(result['hidden'], not verified)
                    self.assertEqual(result['references'], not verified)
                    self.assertEqual(result['unavailable'], verified)
                    if verified: self.assertEqual(result['label'], '1')
                finally:
                    browser.close()
