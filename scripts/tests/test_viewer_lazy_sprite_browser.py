"""Check bounded offline sprite-tile loading and animation state."""
import copy
import hashlib
import json
import shutil
import tempfile
import time
import unittest
from pathlib import Path

from test_viewer_scoped_browser import REPO, parse_page, prepared
from viewer_browser_harness import Chrome
from view_rom_data import write_split_viewer


def fixture(work):
    data, index = prepared(work)
    blocks = [{'source_offset': 0x1000 + i * 32,
               'tiles': [{'indices': [[i % 15 + 1] * 8 for _ in range(8)]}]} for i in range(130)]
    obj = {'x': 0, 'y': 0, 'shape': 0, 'size': 0, 'tile': 0, 'transform': 0,
           'affine_mode': 0, 'object_mode': 0, 'mosaic': False, 'color_256': False}
    def frame(block):
        return {'word00': block * 32, 'word04': 0, 'duration': 2, 'operation': 1,
                'bytes_0a_13': [0] * 10, 'block': block, 'objects': 0}
    catalog = {'path': 'sprites.json', 'blocks': blocks, 'objects': [{'source_offset': 0x2000, 'objects': [obj]}],
        'sequences': [{'source_offset': 0x3000, 'opaque_header': 0, 'frames': [frame(0), frame(64)]},
                      {'source_offset': 0x4000, 'opaque_header': 0, 'frames': [frame(129)]}],
        'bindings': [{'sprite_id': 10, 'animation': 0, 'part': 0, 'index_source': 0x5000, 'sequence': 0},
                     {'sprite_id': 11, 'animation': 0, 'part': 0, 'index_source': 0x5004, 'sequence': 1}]}
    data['battle_sprites'] = {'normal': catalog}
    data['script_sprites'] = {'registry-generic': {**copy.deepcopy(catalog), 'producers': []}}
    write_split_viewer(data, index.parent, index.name, (REPO / 'view_rom_data.html').read_text())
    return index, data


def wait_for(browser, expression, timeout=10):
    end = time.monotonic() + timeout
    while time.monotonic() < end:
        try:
            if browser.evaluate(expression):
                return
        except (RuntimeError, OSError):
            pass
        time.sleep(.03)
    raise AssertionError('Timed out waiting for: ' + expression)


class LazySpriteAssetTest(unittest.TestCase):
    def test_chunk_identity_reconstruction_and_output_guard(self):
        with tempfile.TemporaryDirectory(prefix='viewer-lazy-sprite-assets-') as directory:
            work = Path(directory)
            index, data = fixture(work)
            parts = index.parent / (index.name + '.parts')
            battle = parse_page(parts / 'battle.html')['battle_sprites']['normal']
            script = parse_page(parts / 'script-actors.html')['script_sprites']['registry-generic']
            self.assertEqual(battle['blocks'], script['blocks'])
            self.assertEqual(len(list(parts.glob('sprite-tiles-*.js'))), 3)
            for original, stored in zip(data['battle_sprites']['normal']['blocks'], battle['blocks']):
                self.assertNotIn('tiles', stored)
                asset = stored['tiles_asset']
                content = json.JSONDecoder().raw_decode((parts / asset['href']).read_text().split(' = ', 2)[-1])[0]
                encoded = json.dumps(content['blocks'], separators=(',', ':')).encode()
                self.assertEqual(hashlib.sha256(encoded).hexdigest(), asset['key'])
                self.assertEqual(content['blocks'][asset['index']], original['tiles'])
                self.assertEqual(stored['tile_count'], len(original['tiles']))
            path = parts / battle['blocks'][0]['tiles_asset']['href']
            previous = (parts / 'battle.html').read_bytes(); path.write_text('changed by another owner')
            with self.assertRaisesRegex(ValueError, 'sprite tile asset'):
                write_split_viewer(data, index.parent, index.name, (REPO / 'view_rom_data.html').read_text())
            self.assertEqual((parts / 'battle.html').read_bytes(), previous)
            self.assertEqual(path.read_text(), 'changed by another owner')


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class LazySpriteBrowserTest(unittest.TestCase):
    def test_selected_blocks_animation_cancellation_and_script_page(self):
        with tempfile.TemporaryDirectory(prefix='viewer-lazy-sprite-browser-') as directory:
            work = Path(directory)
            index, _ = fixture(work)
            parts = index.parent / (index.name + '.parts')
            browser = Chrome(work, parts / 'battle.html')
            try:
                wait_for(browser, "Array.isArray(data.battle_sprites.normal.blocks[0].tiles)")
                self.assertTrue(browser.evaluate('''(()=>{const q=s=>document.querySelector(s),blocks=data.battle_sprites.normal.blocks;
                  const pixel=[...q('#equipment-pose').getContext('2d').getImageData(0,0,1,1).data].join(',');
                  return Object.keys(globalThis.__FFTA_SPRITE_TILES__).length===1 && !('tiles' in blocks[64]) &&
                    !('tiles' in blocks[129]) && pixel==='17,17,17,255';})()'''))
                self.assertTrue(browser.evaluate('''(async()=>{
                  const q=s=>document.querySelector(s),wait=async f=>{const end=Date.now()+5000;while(!f()){
                    if(Date.now()>end)throw Error('timeout');await new Promise(r=>setTimeout(r,10))}};
                  const append=document.head.append.bind(document.head);window.queued=[];
                  document.head.append=function(node){if(node.src&&node.src.includes('sprite-tiles-'))queued.push(node);else append(node)};
                  window.release=()=>queued.splice(0).forEach(node=>append(node));
                  q('#equipment-loop').checked=true;q('#equipment-rate').value='120';q('#equipment-play').click();
                  await wait(()=>queued.length===1);if(q('#equipment-stop').disabled)throw Error('cannot stop pending animation');
                  q('#equipment-stop').click();release();await wait(()=>Array.isArray(data.battle_sprites.normal.blocks[64].tiles));
                  await new Promise(r=>setTimeout(r,60));
                  if(q('#equipment-frame').value!=='0'||!q('#equipment-stop').disabled||!q('#equipment-loop').checked)
                    throw Error('cancelled animation resumed');
                  q('#equipment-play').click();await wait(()=>q('#equipment-frame').value==='1');
                  if(q('#equipment-stop').disabled)throw Error('animation did not resume');q('#equipment-stop').click();
                  q('#equipment-sprite').value='11';q('#equipment-sprite').dispatchEvent(new Event('change'));
                  await wait(()=>queued.length===1);q('#equipment-sprite').value='10';q('#equipment-sprite').dispatchEvent(new Event('change'));
                  release();await wait(()=>Array.isArray(data.battle_sprites.normal.blocks[129].tiles));
                  await new Promise(r=>setTimeout(r,40));
                  if(q('#equipment-sprite').value!=='10'||!q('#equipment-loop').checked)throw Error('stale selection');
                  return true;})()'''))
                browser.command('Page.navigate', {'url': (parts / 'script-actors.html').as_uri()})
                wait_for(browser, "document.readyState==='complete' && typeof data!=='undefined' && data.wiki.current==='script-actors' && Array.isArray(data.script_sprites['registry-generic'].blocks[0].tiles)")
                self.assertTrue(browser.evaluate('''(()=>{const q=s=>document.querySelector(s),blocks=data.script_sprites['registry-generic'].blocks;
                  return q('#equipment-catalog').value==='registry-generic' && !('tiles' in blocks[64]) &&
                    [...q('#equipment-pose').getContext('2d').getImageData(0,0,1,1).data].join(',')==='17,17,17,255';})()'''))
            finally:
                browser.close()

    def test_missing_block_can_be_retried(self):
        with tempfile.TemporaryDirectory(prefix='viewer-lazy-sprite-retry-') as directory:
            work = Path(directory)
            index, _ = fixture(work)
            parts = index.parent / (index.name + '.parts')
            block = parse_page(parts / 'battle.html')['battle_sprites']['normal']['blocks'][0]
            asset = parts / block['tiles_asset']['href']; saved = asset.with_suffix('.saved'); asset.rename(saved)
            browser = Chrome(work, parts / 'battle.html')
            try:
                wait_for(browser, "document.querySelector('#equipment-pose-detail').textContent.startsWith('Cannot load sprite graphics:')")
                self.assertFalse(browser.evaluate("document.querySelector('#equipment-retry-graphics').hidden"))
                saved.rename(asset); browser.evaluate("document.querySelector('#equipment-retry-graphics').click()")
                wait_for(browser, "Array.isArray(data.battle_sprites.normal.blocks[0].tiles)")
                self.assertTrue(browser.evaluate('''(()=>{const q=s=>document.querySelector(s);
                  return q('#equipment-retry-graphics').hidden &&
                    [...q('#equipment-pose').getContext('2d').getImageData(0,0,1,1).data].join(',')==='17,17,17,255';})()'''))
            finally:
                if saved.exists(): saved.rename(asset)
                browser.close()
