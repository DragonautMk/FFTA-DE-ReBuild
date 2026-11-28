"""Validate the full-screen source, optional graphics binding and lazy preview."""
import copy
import hashlib
import json
import shutil
import struct
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import view_rom_data as viewer
from test_viewer_navbar_split_browser import wait_for
from test_viewer_scoped_browser import parse_page
from viewer_browser_harness import Chrome


def source(offset, role, raw):
    return {'offset': offset, 'role': role, 'length': len(raw), 'sha256': hashlib.sha256(raw).hexdigest()}


def contents():
    literals = struct.pack('<3I', 0x083b53fc, 0x03000e60, 0x03000940)
    cells = struct.pack('<600H', *(index | ((index % 2) << 12) for index in range(600)))
    pieces = [(0x04fdd4, 'reader_literals', literals), (0x3b53fc, 'tilemap_cells', cells)]
    graphics = {'banks': [{'source_offset': 0x3b1108, 'words': [0x11111110] + [0x11111111] * 4799}], 'selectors': [],
                'direct_pointers': [{'source_offset': 0x04fdd0, 'bank_offset': 0x3b1108}]}
    binding = [source(0x04fdd0, 'ui-custom-literal/0004fdd0', struct.pack('<I', 0x083b1108))]
    palette = {'entries': [{'banks': [[0, 31] + [0] * 14, [0, 992] + [0] * 14]}]}
    ui_palettes = {'banks': [{'source_offset': offset, 'colors': ([0x7fff, color] + [0] * 14) * 5}
                             for offset, color in ((0x40fcb0, 31), (0x411310, 992), (0x410830, 31744))]}
    palette_binding = [source(bank['source_offset'], f"ui-raw-palette/{bank['source_offset']:08x}",
                              struct.pack('<80H', *bank['colors'])) for bank in ui_palettes['banks']]
    return pieces, graphics, binding, palette, ui_palettes, palette_binding


def fixture(work, *, omit_graphics=False, invalid_map=False, invalid_binding=False, trusted=True, split=True,
            omit_palettes=False, invalid_palette=False, trusted_palettes=True):
    pieces, graphics, binding, palette, ui_palettes, palette_binding = contents(); dataset = work/'dataset'; dataset.mkdir()
    packed = b''.join(raw for _, _, raw in pieces); graphic_bytes = json.dumps(graphics).encode()
    palette_bytes = json.dumps(ui_palettes).encode()
    src = [source(*item) for item in pieces]
    if invalid_map: src[1]['sha256'] = '0' * 64
    if invalid_binding: binding[0]['sha256'] = '0' * 64
    if invalid_palette: palette_binding[0]['sha256'] = '0' * 64
    rows = [('graphics.full-screen-map', 'map.bin', 'full-screen-map-v1-bin', packed, src),
            ('graphics.ui-custom-banks', 'graphics.json', 'ui-custom-banks-v1-json', graphic_bytes, binding),
            ('graphics.bg-base-palettes', 'palette.json', 'bg-base-palettes-v2-json', json.dumps(palette).encode(), []),
            ('graphics.ui-raw-palettes', 'ui-palettes.json', 'ui-raw-palettes-v1-json', palette_bytes, palette_binding)]
    manifest = 'profile="europe"\nextractor_revision=374\ndecoder_revision=373\n'
    for identity, name, fmt, raw, sources in rows:
        if omit_graphics and identity == 'graphics.ui-custom-banks': continue
        if omit_palettes and identity == 'graphics.ui-raw-palettes': continue
        (dataset/name).write_bytes(raw)
        manifest += f'[[resources]]\nid="{identity}"\npath="{name}"\nformat="{fmt}"\nsha256="{hashlib.sha256(raw).hexdigest()}"\n'
        for item in sources:
            manifest += (f'[[resources.sources]]\noffset={item["offset"]}\nlength={item["length"]}\n'
                         f'role="{item["role"]}"\nsha256="{item["sha256"]}"\n')
    (dataset/'manifest.toml').write_text(manifest)
    with patch.object(viewer, 'EUROPE_FULL_SCREEN_MAP_SHA256', hashlib.sha256(packed).hexdigest() if trusted else '0' * 64), \
         patch.object(viewer, 'EUROPE_UI_CUSTOM_BANKS_SHA256', hashlib.sha256(graphic_bytes).hexdigest()), \
         patch.object(viewer, 'EUROPE_UI_RAW_PALETTES_SHA256', hashlib.sha256(palette_bytes).hexdigest() if trusted_palettes else '0' * 64):
        return viewer.generate(dataset, work/'view', split=split)


class FullScreenProjectionTest(unittest.TestCase):
    def test_source_binding_geometry_and_optional_graphics(self):
        pieces, graphics, binding, _, ui_palettes, palette_binding = contents(); raw = b''.join(item[2] for item in pieces); digest = hashlib.sha256(raw).hexdigest()
        sources = [source(*item) for item in pieces]
        with patch.object(viewer, 'EUROPE_FULL_SCREEN_MAP_SHA256', digest):
            result = viewer.project_full_screen_map('europe', digest, raw, sources)
            self.assertEqual((result['maps'][0]['width'], result['maps'][0]['height']), (30, 20))
            self.assertEqual(len(result['maps'][0]['cells']), 600); self.assertEqual(result['maps'][0]['cells'][-1], 599 | 0x1000)
            self.assertIsNone(viewer.project_full_screen_map('usa', digest, raw, sources))
            for mutate in (lambda s: s.pop(), lambda s: s[0].update(offset=0), lambda s: s[1].update(sha256='0' * 64)):
                bad = copy.deepcopy(sources); mutate(bad)
                with self.assertRaises(ValueError): viewer.project_full_screen_map('europe', digest, raw, bad)
            bad = copy.deepcopy(pieces); bad[0] = (bad[0][0], bad[0][1], bytes(12)); invalid = b''.join(x[2] for x in bad)
            with patch.object(viewer, 'EUROPE_FULL_SCREEN_MAP_SHA256', hashlib.sha256(invalid).hexdigest()):
                with self.assertRaises(ValueError): viewer.project_full_screen_map('europe', hashlib.sha256(invalid).hexdigest(), invalid, [source(*item) for item in bad])
        self.assertEqual(viewer.project_full_screen_graphics(graphics, binding, graphics)['source_offset'], 0x3b1108)
        for mutate in (lambda g, s: g['direct_pointers'][0].update(bank_offset=0),
                       lambda g, s: g['banks'][0]['words'].pop(), lambda g, s: s[0].update(sha256='0' * 64)):
            g, s = copy.deepcopy(graphics), copy.deepcopy(binding); mutate(g, s)
            with self.assertRaises(ValueError): viewer.project_full_screen_graphics(g, s, g)
        digest = hashlib.sha256(json.dumps(ui_palettes).encode()).hexdigest()
        with patch.object(viewer, 'EUROPE_UI_RAW_PALETTES_SHA256', digest):
            self.assertEqual(viewer.project_full_screen_palettes(digest, ui_palettes, palette_binding),
                             {'mode_sources': [0x40fcb0, 0x411310, 0x410830, 0x40fcb0], 'clear_first_color': True})
            for mutate in (lambda p, s: p['banks'].pop(), lambda p, s: p['banks'][0]['colors'].pop(),
                           lambda p, s: p['banks'].append(copy.deepcopy(p['banks'][0])),
                           lambda p, s: s[0].update(sha256='0' * 64), lambda p, s: s[0].update(role='wrong')):
                p, s = copy.deepcopy(ui_palettes), copy.deepcopy(palette_binding); mutate(p, s)
                with self.assertRaises(ValueError): viewer.project_full_screen_palettes(digest, p, s)
            with self.assertRaises(ValueError): viewer.project_full_screen_palettes('0' * 64, ui_palettes, palette_binding)

    def test_scope_and_missing_source_behavior(self):
        for options in ({}, {'omit_graphics': True}, {'invalid_binding': True}, {'invalid_map': True}, {'trusted': False},
                        {'omit_palettes': True}, {'invalid_palette': True}, {'trusted_palettes': False}):
            with self.subTest(options=options), tempfile.TemporaryDirectory(prefix='viewer-fullscreen-') as directory:
                page = fixture(Path(directory), **options); parts = page.parent/'index.html.parts'
                home, interface, focused = map(parse_page, (page, parts/'interface.html', parts/'interface-full-screen.html'))
                self.assertNotIn('full_screen_map', home); self.assertNotIn('full_screen_map', interface)
                row = next(item for item in home['resources'] if item['id'] == 'graphics.full-screen-map')
                self.assertEqual(row['viewer_scope'], 'interface-full-screen')
                if options.get('invalid_map') or options.get('trusted') is False: self.assertIsNone(focused['full_screen_map'])
                else:
                    item = focused['full_screen_map']; self.assertEqual(len(item['maps'][0]['cells']), 600)
                    self.assertEqual('graphics' in item, not (options.get('omit_graphics') or options.get('invalid_binding')))
                    self.assertEqual('palettes' in item, not (options.get('omit_palettes') or options.get('invalid_palette') or options.get('trusted_palettes') is False))
                    if not options: self.assertIn('fixed_map_graphics_asset', focused)


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class FullScreenBrowserTest(unittest.TestCase):
    def test_scoped_lazy_graphics_pixels_and_cell_inspection(self):
        with tempfile.TemporaryDirectory(prefix='viewer-fullscreen-browser-') as directory:
            work = Path(directory); page = fixture(work); browser = Chrome(work, page)
            try:
                target = browser.evaluate("document.querySelector('[data-resource-id=\"graphics.full-screen-map\"] a').href")
                browser.command('Page.navigate', {'url': target})
                wait_for(browser, "data.wiki.current==='interface-full-screen' && !document.querySelector('#tilemaps').hidden")
                self.assertTrue(browser.evaluate('''(async()=>{const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change',{bubbles:true}));
                  const wait=async f=>{const end=Date.now()+5000;while(!f()){if(Date.now()>end)throw Error('timeout');await new Promise(r=>setTimeout(r,10))}};
                  if(q('#tilemaps>details>summary time').dateTime!=='2026-11-27T23:50:05Z'||q('#tilemap-cells').rows.length!==20||
                     q('#tilemap-cells').rows[0].cells.length!==31||q('#full-screen-use-graphics').hidden||
                     q('#full-screen-color-control').hidden||
                     typeof globalThis.__FFTA_MAP_GRAPHICS__!=='undefined')throw Error('initial scope');
                  q('#tilemaps>details').open=true;q('#full-screen-use-graphics').click();await wait(()=>q('#tilemap-load-map-graphics').hidden);
                  if(!q('#tilemap-graphics').selectedOptions[0].textContent.includes('0x3B1108')||q('#tilemap-base').value!=='0'||
                     !q('#tilemap-map-graphics-status').textContent.includes('initializer colors selected'))throw Error('graphics link');
                  const pixel=x=>[...q('#tilemap-canvas').getContext('2d').getImageData(x,0,1,1).data].join(',');
                  if(pixel(0)!=='0,0,0,255'||pixel(1)!=='255,0,0,255'||!q('#tilemap-cell-palettes').checked||
                     !q('#tilemap-description').textContent.includes('0 cells outside'))throw Error('initial colors');
                  for(const [mode,color] of [['1','0,255,0,255'],['2','0,0,255,255'],['3','255,0,0,255'],['0','255,0,0,255']]){
                    q('#full-screen-color-mode').value=mode;change(q('#full-screen-color-mode'));if(pixel(1)!==color)throw Error('mode '+mode);
                  }
                  q('#tilemap-palette-source').value='0';change(q('#tilemap-palette-source'));
                  if(pixel(1)!=='255,0,0,255')throw Error('manual palette pixel');
                  q('#tilemap-cells').rows[19].cells[30].querySelector('button').click();
                  return q('#tilemap-cell-detail').textContent.includes('0x3B58AA')&&q('#tilemap-cell-detail').textContent.includes('599');})()'''))
            finally:
                browser.close()

    def test_single_page_immediate_graphics(self):
        with tempfile.TemporaryDirectory(prefix='viewer-fullscreen-single-') as directory:
            work = Path(directory); page = fixture(work, split=False); browser = Chrome(work, page)
            try:
                self.assertTrue(browser.evaluate('''(()=>{const q=s=>document.querySelector(s);q('#full-screen-use-graphics').click();
                  return !q('#tilemaps').hidden&&q('#tilemap-load-map-graphics').hidden&&q('#tilemap-graphics').selectedOptions[0].textContent.includes('0x3B1108');})()'''))
            finally:
                browser.close()

    def test_unverified_palette_keeps_manual_preview_available(self):
        with tempfile.TemporaryDirectory(prefix='viewer-fullscreen-manual-') as directory:
            work = Path(directory); index = fixture(work, invalid_palette=True)
            browser = Chrome(work, index.parent/'index.html.parts/interface-full-screen.html')
            try:
                self.assertTrue(browser.evaluate('''(async()=>{const q=s=>document.querySelector(s);
                  if(!q('#full-screen-color-control').hidden)throw Error('unverified palette offered');
                  q('#full-screen-use-graphics').click();
                  for(let i=0;i<100&&!q('#tilemap-load-map-graphics').hidden;i++)await new Promise(r=>setTimeout(r,20));
                  return q('#tilemap-load-map-graphics').hidden&&q('#tilemap-palette-source').value==='gray'&&
                    q('#tilemap-map-graphics-status').textContent.includes('choose a palette');})()'''))
            finally:
                browser.close()

    def test_loader_graphics_retry_after_missing_asset(self):
        with tempfile.TemporaryDirectory(prefix='viewer-fullscreen-retry-') as directory:
            work = Path(directory); index = fixture(work); parts = index.parent/'index.html.parts'
            page = parts/'interface-full-screen.html'; asset = parts/parse_page(page)['fixed_map_graphics_asset']['href']
            saved = asset.with_suffix('.saved'); asset.rename(saved); browser = Chrome(work, page)
            try:
                self.assertTrue(browser.evaluate('''(async()=>{const q=s=>document.querySelector(s);q('#full-screen-use-graphics').click();
                  for(let i=0;i<100;i++){if(!q('#tilemap-load-map-graphics').disabled&&q('#tilemap-map-graphics-status').textContent.includes('Unable'))return true;
                    await new Promise(r=>setTimeout(r,20));}return false;})()'''))
                saved.rename(asset)
                self.assertTrue(browser.evaluate('''(async()=>{const q=s=>document.querySelector(s);q('#full-screen-use-graphics').click();
                  for(let i=0;i<100&&!q('#tilemap-load-map-graphics').hidden;i++)await new Promise(r=>setTimeout(r,20));
                  return q('#tilemap-load-map-graphics').hidden&&q('#tilemap-graphics').selectedOptions[0].textContent.includes('0x3B1108');})()'''))
            finally:
                browser.close()


if __name__ == '__main__':
    unittest.main()
