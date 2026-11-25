"""Check tilemap decoding and manual pixel interpretation in Chrome."""
import hashlib
import json
import shutil
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from view_rom_data import generate
from viewer_browser_harness import Chrome


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class ViewerTilemapsTest(unittest.TestCase):
    def test_tilemap_pixels_cells_and_missing_tiles(self):
        with tempfile.TemporaryDirectory(prefix='viewer-tilemap-') as directory:
            work = Path(directory)
            dataset = work / 'dataset'
            dataset.mkdir()
            maps = {
                'source_offset': 100, 'width': 2, 'height': 2, 'mode': 4,
                'runs': [{'control': 0x4001, 'value': value} for value in (5, 0x1405, 0x1805, 0xfc06)],
                'additional_streams': [{'source_offset': 200, 'width': 2, 'height': 2, 'mode': 4,
                                        'tokens': [{'control': 2, 'values': [7, 8]},
                                                   {'control': 0x8002, 'values': [9]}]}],
                'rectangles': [{'source_offset': 300, 'width': 2, 'height': 1, 'cells': [0x1234, 0xabcd]}],
            }
            graphics = {'descriptor_graphics': [0], 'graphics': [
                {'relative_source': 400, 'destination_offset': 5 * 32,
                 'words': [1] + [0] * 6 + [2] + [0x33333333] * 8},
            ]}
            palettes = {'entries': [{'banks': [[0, 31, 992, 31744] + [0] * 12,
                                              [0, 992, 31744, 31] + [0] * 12]}]}
            manifest = 'profile="test"\nextractor_revision=1\ndecoder_revision=1\n'
            for resource_id, name, fmt, value in [
                ('graphics.observed-tilemap', 'maps.json', 'observed-tilemap-v4-json', maps),
                ('graphics.bg-main-graphics', 'graphics.json', 'bg-main-graphics-v1-json', graphics),
                ('graphics.bg-base-palettes', 'palette.json', 'bg-base-palettes-v2-json', palettes),
            ]:
                content = json.dumps(value).encode()
                (dataset / name).write_bytes(content)
                manifest += (f'[[resources]]\nid="{resource_id}"\npath="{name}"\n'
                             f'format="{fmt}"\nsha256="{hashlib.sha256(content).hexdigest()}"\n')
            (dataset / 'manifest.toml').write_text(manifest)
            page = generate(dataset, work / 'view')
            browser = Chrome(work, page)
            try:
                result = browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change'));
                  const canvas=q('#tilemap-canvas'), pixel=(x,y)=>[...canvas.getContext('2d').getImageData(x,y,1,1).data].join(',');
                  if(q('#tilemaps').hidden || q('#tilemap-id').options.length!==3)throw Error('map catalog');
                  if(![...q('#contents').querySelectorAll('a')].some(a=>a.getAttribute('href')==='#tilemaps'))throw Error('contents');
                  if(![...q('#rows').querySelectorAll('a')].some(a=>a.getAttribute('href')==='#tilemaps'))throw Error('resource link');
                  if(!q('#tilemap-description').textContent.includes('shades show'))throw Error('index display');
                  q('#tilemap-graphics').value='0';change(q('#tilemap-graphics'));
                  if(q('#tilemap-base').value!=='5')throw Error('background destination base');
                  q('#tilemap-palette-source').value='0';change(q('#tilemap-palette-source'));
                  if(pixel(0,0)!=='255,0,0,255' || pixel(15,0)!=='255,0,0,255' || pixel(8,0)!=='0,0,0,255')throw Error('horizontal flip');
                  if(pixel(0,8)!=='0,255,0,255' || pixel(8,8)!=='0,0,255,255')throw Error('vertical flip or palette');
                  const cellPalettes=q('#tilemap-cell-palettes');cellPalettes.checked=true;change(cellPalettes);
                  if(!q('#tilemap-palette-bank').disabled || pixel(0,0)!=='255,0,0,255' || pixel(15,0)!=='0,255,0,255' || pixel(0,8)!=='0,0,255,255')throw Error('cell palette banks');
                  if(pixel(8,8)!=='0,192,255,255' || !q('#tilemap-description').textContent.includes('1 cells have no selected palette bank'))throw Error('missing palette bank');
                  cellPalettes.checked=false;change(cellPalettes);q('#tilemap-palette-bank').value='1';change(q('#tilemap-palette-bank'));
                  if(q('#tilemap-palette-bank').disabled || pixel(0,0)!=='0,255,0,255')throw Error('manual palette bank');
                  const buttons=q('#tilemap-cells').querySelectorAll('button');buttons[3].click();
                  if(!q('#tilemap-cell-detail').textContent.includes('0xFC06') || !q('#tilemap-cell-detail').textContent.includes('palette ID 15') || buttons[3].getAttribute('aria-pressed')!=='true')throw Error('cell details');
                  q('#tilemap-base').value='6';q('#tilemap-base').dispatchEvent(new Event('input'));
                  if(pixel(0,0)!=='255,0,255,255' || !q('#tilemap-description').textContent.includes('3 cells outside'))throw Error('missing tiles');
                  q('#tilemap-base').value='';q('#tilemap-base').dispatchEvent(new Event('input'));
                  if(!q('#tilemap-description').textContent.includes('Choose a first tile ID'))throw Error('invalid base');
                  q('#tilemap-graphics').value='none';change(q('#tilemap-graphics'));
                  q('#tilemap-id').value='2';change(q('#tilemap-id'));
                  if(canvas.width!==16 || canvas.height!==8 || !q('#tilemap-cell-detail').textContent.includes('0x1234'))throw Error('rectangle');
                  q('#tilemaps > details').open=true;q('#collapse-all').click();if(q('#tilemaps > details').open)throw Error('collapse');
                  return 'PASS';
                })()''')
                self.assertEqual(result, 'PASS')
            finally:
                browser.close()
