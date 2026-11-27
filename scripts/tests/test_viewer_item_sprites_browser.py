"""Check authenticated item-to-equipment sprite links and reverse navigation."""
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
from test_viewer_job_names_browser import payload
from test_viewer_special_units_browser import battle_catalog, wait_for
from viewer_browser_harness import Chrome


def fixture(work, *, wrong_items=False, wrong_equipment=False, with_equipment=True, with_text=True, split=True):
    item_fixture(work, with_text=with_text)
    dataset = work / 'dataset'; manifest = dataset / 'manifest.toml'; text = manifest.read_text()
    item_path = dataset / 'items.json'; original = item_path.read_bytes(); items = json.loads(original)
    for index, record in enumerate(items['records']):
        style = 1 if index <= 25 or index >= 28 else 0 if index == 26 else 20
        sprite = 10 if index <= 27 else 11 if index == 28 else 12
        record['bytes_08_0f'][0] = style
        record['bytes_08_0f'][6:8] = [sprite, 0]
    content = json.dumps(items).encode(); item_path.write_bytes(content); item_sha = hashlib.sha256(content).hexdigest()
    text = text.replace(hashlib.sha256(original).hexdigest(), item_sha)
    equipment = battle_catalog('normal')
    equipment['bindings'].extend([
        {'sprite_id': 11, 'animation': 16, 'part': 0, 'index_source': 108, 'sequence_source': 500},
        {'sprite_id': 12, 'animation': 0, 'part': 0, 'index_source': 112, 'sequence_source': None},
    ])
    content = json.dumps(equipment).encode(); equipment_sha = hashlib.sha256(content).hexdigest()
    if with_equipment:
        (dataset / 'equipment.json').write_bytes(content)
        text += (f'[[resources]]\nid="graphics.unit-obj-equipment"\npath="equipment.json"\n'
                 f'format="unit-obj-equipment-v1-json"\nsha256="{equipment_sha}"\n')
    manifest.write_text(text)
    text_sha = hashlib.sha256((dataset / 'text.json').read_bytes()).hexdigest() if with_text else 'missing'
    with ExitStack() as stack:
        stack.enter_context(patch.object(view_rom_data, 'EUROPE_ITEMS_SHA256', 'wrong' if wrong_items else item_sha))
        stack.enter_context(patch.object(view_rom_data, 'EUROPE_EQUIPMENT_SPRITES_SHA256', 'wrong' if wrong_equipment else equipment_sha))
        stack.enter_context(patch.object(view_rom_data, 'EUROPE_TEXT_CATALOG_SHA256', text_sha))
        page = view_rom_data.generate(dataset, work / 'linked-view', split=split)
    return page


class ItemSpriteProjectionTest(unittest.TestCase):
    def test_guards_item_zero_styles_and_normal_animations(self):
        for options in ({}, {'wrong_items':True}, {'wrong_equipment':True}, {'with_equipment':False}, {'with_text':False}):
            with self.subTest(options=options), tempfile.TemporaryDirectory(prefix='viewer-item-sprite-project-') as directory:
                page = fixture(Path(directory), **options); parts = page.parent / 'index.html.parts'
                world, actors, battle = map(payload, (parts / 'world.html', parts / 'actors.html', parts / 'battle.html'))
                links = world['item_sprite_links']; self.assertEqual(links, actors['item_sprite_links'])
                self.assertNotIn('item_sprite_links', battle)
                if options.get('wrong_items') or options.get('wrong_equipment') or options.get('with_equipment') is False:
                    self.assertIsNone(links); continue
                self.assertEqual(links['sprite_ids'], [None] + [10] * 25 + [None] * 4)
                self.assertEqual(links['names_verified'], options.get('with_text', True))
                self.assertEqual(bool(actors['item_names']), options.get('with_text', True))
                with patch.object(view_rom_data, 'EUROPE_ITEMS_SHA256', 'items'), \
                     patch.object(view_rom_data, 'EUROPE_EQUIPMENT_SPRITES_SHA256', 'equipment'):
                    self.assertIsNone(view_rom_data.link_item_sprites('usa', 'items', 'equipment', world['items'], actors['equipment_sprites'], None))
                    graph = copy.deepcopy(actors['equipment_sprites'])
                    graph['bindings'] = [row for row in graph['bindings'] if row['animation'] == 16]
                    result = view_rom_data.link_item_sprites('europe', 'items', 'equipment', world['items'], graph, None)
                    self.assertTrue(all(row is None for row in result['sprite_ids']))


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class ItemSpriteBrowserTest(unittest.TestCase):
    def test_item_to_sprite_preview_reverse_names_and_pagination(self):
        with tempfile.TemporaryDirectory(prefix='viewer-item-sprite-browser-') as directory:
            work = Path(directory); page = fixture(work); parts = page.parent / 'index.html.parts'
            browser = Chrome(work, parts / 'world.html')
            try:
                link = browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change',{bubbles:true}));
                  if(!q('#item-sprite-navigation').hidden)throw Error('item zero');
                  q('#item-id').value='1';change(q('#item-id'));
                  const a=q('#item-sprite-open');
                  if(q('#item-sprite-navigation').hidden || a.dataset.spriteId!=='10' || !a.href.includes('sprite_catalog=equipment'))throw Error('item link');
                  const target=a.href;
                  for(const id of ['26','27','28','29']){q('#item-id').value=id;change(q('#item-id'));if(!q('#item-sprite-navigation').hidden)throw Error('invalid '+id);}
                  return target;
                })()''')
                browser.command('Page.navigate', {'url': link})
                wait_for(browser, "typeof data !== 'undefined' && data.wiki.current==='actors' && document.querySelector('#equipment-sprite').value==='10'")
                self.assertTrue(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change',{bubbles:true}));
                  const pixel=[...q('#equipment-pose').getContext('2d').getImageData(0,0,1,1).data].join(',');
                  if(q('#equipment-catalog').value!=='equipment' || q('#equipment-frame').value!=='1' || pixel!=='17,17,17,255' ||
                     q('#equipment-item-references').hidden || q('#equipment-item-rows').rows.length!==20 ||
                     !q('#equipment-item-page').textContent.includes('25 items'))throw Error('preview or reverse');
                  q('#equipment-item-language').value='1';change(q('#equipment-item-language'));
                  if(!q('#equipment-item-rows tr[data-item-id="1"]').cells[1].textContent.includes('A b'))throw Error('localized name');
                  q('#equipment-item-next').click();
                  if(q('#equipment-item-rows').rows.length!==5 || !q('#equipment-item-page').textContent.includes('page 2'))throw Error('pagination');
                  const a=q('#equipment-item-rows tr[data-item-id="21"] a');
                  return !!a && a.href.includes('/world.html?') && a.href.includes('item-id%3A21');
                })()'''))
                browser.click('#equipment-item-rows tr[data-item-id="21"] a')
                wait_for(browser, "typeof data !== 'undefined' && data.wiki.current==='world' && document.querySelector('#item-id').value==='21'")
                self.assertTrue(browser.evaluate("document.querySelector('#items>details').open && !document.querySelector('#item-sprite-navigation').hidden"))
            finally: browser.close()

    def test_no_unverified_links_and_numeric_fallback(self):
        for options in ({'wrong_items':True}, {'wrong_equipment':True}, {'with_text':False}):
            with self.subTest(options=options), tempfile.TemporaryDirectory(prefix='viewer-item-sprite-partial-') as directory:
                work = Path(directory); page = fixture(work, split=False, **options); browser = Chrome(work, page)
                try:
                    state = browser.evaluate('''(()=>{const q=s=>document.querySelector(s);q('#item-id').value='1';q('#item-id').dispatchEvent(new Event('change'));
                      return {hidden:q('#item-sprite-navigation').hidden, itemPanel:q('#equipment-item-references').hidden,
                        label:q('#equipment-item-rows tr[data-item-id="1"] td:nth-child(2)')?.textContent};})()''')
                    self.assertEqual(state['hidden'], options.get('with_text', True))
                    self.assertEqual(state['itemPanel'], options.get('with_text', True))
                    if options.get('with_text') is False: self.assertEqual(state['label'], '1')
                finally: browser.close()
