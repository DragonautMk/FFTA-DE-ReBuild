"""Check type-2 rectangle and copy-window joins across extracted resources."""
import hashlib
import json
import shutil
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from view_rom_data import generate, project_type2_rectangles
from viewer_browser_harness import Chrome


def entry(root, first, count, a=6, b=7):
    return (0x08000000 + root).to_bytes(4, 'little') + bytes([a, b, first, count])


def segments():
    result = [(0x1000, 'rectangle-list-entry', entry(0x2000, 0, 2)),
              (0x1008, 'rectangle-list-entry', entry(0xae6db8, 1, 2)),
              (0x1010, 'rectangle-list-entry', entry(0xae6db0, 0, 1)),
              (0x1018, 'rectangle-list-entry', entry(0x3000, 0, 1)),
              (0x1020, 'rectangle-list-entry', entry(0x5000, 0, 1)),
              (0x1100, 'rectangle-list-header', bytes([5, 0, 0, 0]) + (0x08001000).to_bytes(4, 'little')),
              (0x1200, 'transfer-row', bytes([2, 0, 0, 0]) + (0x08001100).to_bytes(4, 'little') + bytes(4)),
              (0x2000, 'rectangle-header', bytes([2, 2])),
              (0x2002, 'rectangle-cells', bytes.fromhex('0100341202000300')),
              (0x2100, 'rectangle-header', bytes([1, 2])),
              (0x2102, 'rectangle-cells', bytes([1])),
              (0xae6dc0, 'rectangle-cells', bytes.fromhex('010002000300040005000600'))]
    return result


def fixture(work, *, prior=True, palette=True, bad_hash=False, missing=False):
    dataset = work / 'dataset'; dataset.mkdir(); parts = segments(); content = b''.join(chunk for _, _, chunk in parts)
    if not missing: (dataset / 'type2.bin').write_bytes(content)
    manifest = ('profile="europe"\nextractor_revision=1\ndecoder_revision=1\n'
                '[[resources]]\nid="graphics.tilemap-type2-transfers"\npath="type2.bin"\n'
                f'format="tilemap-type2-transfers-v1-bin"\nsha256="{hashlib.sha256(content).hexdigest()}"\n')
    for index, (offset, role, chunk) in enumerate(parts):
        digest = '0' * 64 if bad_hash and index == 0 else hashlib.sha256(chunk).hexdigest()
        manifest += (f'[[resources.sources]]\noffset={offset}\nlength={len(chunk)}\n'
                     f'role="{role}"\nsha256="{digest}"\n')
    resources = []
    if prior:
        resources.append(('observed-tilemap', {'source_offset': 0x4000, 'width': 1, 'height': 1, 'mode': 4,
                         'runs': [{'control': 0x4001, 'value': 0}],
                         'rectangles': [{'source_offset': 0x3000, 'width': 1, 'height': 1, 'cells': [7]}]}))
    if palette:
        resources.append(('firaga-palette-rows', {'rows': [{'declared_rows': 1, 'width': 3, 'colors': [9, 10, 11]},
                                                           {'declared_rows': 3, 'width': 3, 'colors': [701, 10111, 30719]}]}))
    for name, value in resources:
        data = json.dumps(value).encode(); (dataset / (name + '.json')).write_bytes(data)
        manifest += (f'[[resources]]\nid="graphics.{name}"\npath="{name}.json"\n'
                     f'format="{name}-v1-json"\nsha256="{hashlib.sha256(data).hexdigest()}"\n')
    (dataset / 'manifest.toml').write_text(manifest)
    return generate(dataset, work / 'view', split=True)


def payload(path):
    return json.JSONDecoder().raw_decode(path.read_text().split('const data = ', 1)[1])[0]


class Type2RectangleProjectionTest(unittest.TestCase):
    def test_complete_partial_and_prior_sources(self):
        parts = segments(); content = b''.join(chunk for _, _, chunk in parts)
        sources = [{'offset': off, 'role': role, 'length': len(chunk), 'sha256': hashlib.sha256(chunk).hexdigest()}
                   for off, role, chunk in parts]
        prior = {'maps': [{'source_offset': 0x3000, 'width': 1, 'height': 1, 'cells': [7]}]}
        palette = {'rows': [{'source_offset': 0xae6db0, 'declared_rows': 1, 'width': 3, 'colors': [9, 10, 11]},
                            {'source_offset': 0xae6db8, 'declared_rows': 3, 'width': 3, 'colors': [701, 10111, 30719]}]}
        value = project_type2_rectangles(content, sources, prior, palette)
        self.assertEqual([x['source_offset'] for x in value['maps']], [0x2000, 0xae6db0, 0xae6db8])
        self.assertEqual(value['maps'][2]['cells'], [701, 10111, 30719, 1, 2, 3, 4, 5, 6])
        self.assertEqual(value['maps'][0]['type2_references'][0]['list_headers'], [0x1100])
        self.assertEqual(value['maps'][0]['type2_references'][0]['transfer_rows'], [0x1200])
        self.assertEqual(value['prior_references'][0x3000][0]['first_row'], 0)
        self.assertEqual(len(value['errors']), 1); self.assertEqual(len(value['unresolved']), 1)
        without = project_type2_rectangles(content, sources)
        self.assertEqual(len(without['maps']), 1); self.assertEqual(len(without['unresolved']), 4)
        bad = [dict(item) for item in sources]; bad[0]['sha256'] = '0' * 64
        with self.assertRaises(ValueError): project_type2_rectangles(content, bad, prior, palette)

    def test_scope_and_optional_dependencies(self):
        for options in ({}, {'prior': False}, {'palette': False}, {'bad_hash': True}, {'missing': True}):
            with self.subTest(options=options), tempfile.TemporaryDirectory(prefix='viewer-type2-projection-') as directory:
                index = fixture(Path(directory), **options); parts = index.parent / 'index.html.parts'
                home, world, audio = payload(index), payload(parts / 'world.html'), payload(parts / 'audio.html')
                self.assertNotIn('extra_tilemaps', home); self.assertNotIn('extra_tilemaps', audio)
                valid = not (options.get('bad_hash') or options.get('missing'))
                self.assertEqual('type2_rectangles' in world['extra_tilemaps'], valid)
                if valid:
                    group = world['extra_tilemaps']['type2_rectangles']
                    self.assertEqual(len(group['maps']), 3 if options.get('palette', True) else 1)
                    self.assertEqual(len(group['unresolved']), 1 + (not options.get('prior', True)) + 2 * (not options.get('palette', True)))


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class Type2RectangleBrowserTest(unittest.TestCase):
    def test_windows_palette_fragments_and_existing_map_links(self):
        with tempfile.TemporaryDirectory(prefix='viewer-type2-browser-') as directory:
            work = Path(directory); index = fixture(work); browser = Chrome(work, index.parent / 'index.html.parts/world.html')
            try:
                self.assertEqual(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change'));
                  q('[data-resource-id="graphics.tilemap-type2-transfers"] a[href="#tilemaps"]').click();
                  if(!q('#tilemaps>details').open||!q('#tilemap-description').textContent.includes('0x2000')||
                     q('#type2-windows').hidden||q('#type2-window-rows').rows.length!==1)throw Error('rectangle');
                  const row=q('#type2-window-rows').rows[0];
                  if(row.cells[0].textContent!=='0x1000'||row.cells[4].textContent!=='0x1100'||row.cells[5].textContent!=='0x1200')throw Error('provenance');
                  if(q('#type2-unresolved').hidden||q('#type2-unresolved-rows').rows.length!==1||
                     !q('#type2-unresolved-detail').textContent.includes('1 incomplete'))throw Error('partial sources');
                  const map=q('#tilemap-id'),select=text=>{const option=[...map.options].find(item=>item.textContent.includes(text));
                    if(!option)throw Error('missing map '+text);map.value=option.value;change(map)};
                  select('Type-2 palette-row rectangle · 0xAE6DB8');
                  if([...q('#tilemap-cells').querySelectorAll('button')].map(b=>b.textContent).join(',')!==
                     '0x02BD,0x277F,0x77FF,0x0001,0x0002,0x0003,0x0004,0x0005,0x0006'||
                     q('#type2-window-rows').rows[0].cells[2].textContent!=='1')throw Error('split rectangle');
                  select('Rectangle · 0x3000');
                  if(q('#type2-windows').hidden||q('#type2-window-rows').rows[0].cells[0].textContent!=='0x1018')throw Error('prior map');
                  return 'PASS';
                })()'''), 'PASS')
            finally: browser.close()
