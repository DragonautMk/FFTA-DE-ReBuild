"""Exercise Firaga transfer relationships and partial palette rows."""
import copy
import hashlib
import json
import shutil
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from view_rom_data import generate, project_firaga_palette_rows, project_firaga_transfer
from test_viewer_observed_lz_browser import block
from viewer_browser_harness import Chrome


def transfer():
    return {'caller_literals': [0x0839cbd8, 0x0839cbcc], 'program_pointer': 0x0839cb9c, 'program_count': 4,
      'records': [{'kind': 2, 'flags': 0, 'parameter': 0, 'payload': 0x0839cafc, 'destination': 0},
                  {'kind': 3, 'flags': 0, 'parameter': 0x60, 'payload': 0x0839cb04, 'destination': 0x06017f80},
                  {'kind': 3, 'flags': 0, 'parameter': 0x2820, 'payload': 0x0839cb2c, 'destination': 0x0600d7c0},
                  {'kind': 3, 'flags': 0, 'parameter': 0x0c40, 'payload': 0x0839cb34, 'destination': 0x06017340}],
      'palette_count': 2, 'palette_reserved': [0, 0, 0], 'palette_entries_pointer': 0x0839caec,
      'palette_entries': [{'source': 0x08ae6db8, 'parameters': [7, 1, 0, 1]},
                          {'source': 0x08ae6db0, 'parameters': [6, 1, 0, 1]}],
      'lz_wrappers': [{'header': 0x08af565c, 'argument': 0}, {'header': 0x08af4994, 'argument': 0},
                      {'header': 0x08af5450, 'argument': 3}],
      'format_headers': [{'kind': 2, 'flags': 2, 'reserved': 0x7ffd},
                         {'kind': 2, 'flags': 2, 'reserved': 0x7ffd},
                         {'kind': 1, 'flags': 2, 'reserved': 0x7ffd}]}


def palettes():
    return {'rows': [{'declared_rows': 1, 'width': 3, 'colors': [0x001f, 0x03e0, 0x7c00]},
                     {'declared_rows': 3, 'width': 3, 'colors': [0x801f, 0x1234, 0xffff]}]}


def fixture(work, *, include_transfer=True, include_palette=True, include_lz=True, invalid=None):
    dataset = work / 'dataset'; dataset.mkdir(); program, colors = transfer(), palettes()
    if invalid == 'transfer': program['records'][1]['payload'] = 0
    if invalid == 'palette': colors['rows'][0]['width'] = 4
    data = block(0xaf5660, [0x12] * 64)
    data['additional_blocks'] = [block(0xaf4998, [0x23] * 64), block(0xaf5454, [0x34] * 64)]
    items = []
    if include_transfer: items.append(('graphics.firaga-transfer-program', 'firaga-transfer-program', 1, program))
    if include_palette: items.append(('graphics.firaga-palette-rows', 'firaga-palette-rows', 1, colors))
    if include_lz: items.append(('graphics.observed-lz-block', 'observed-lz-block', 2, data))
    manifest = 'profile="test"\nextractor_revision=1\ndecoder_revision=1\n'
    for rid, name, version, value in items:
        content = json.dumps(value).encode(); (dataset / (name + '.json')).write_bytes(content)
        manifest += (f'[[resources]]\nid="{rid}"\npath="{name}.json"\nformat="{name}-v{version}-json"\n'
                     f'sha256="{hashlib.sha256(content).hexdigest()}"\n')
    (dataset / 'manifest.toml').write_text(manifest)
    return generate(dataset, work / 'view', split=True)


def payload(path): return json.JSONDecoder().raw_decode(path.read_text().split('const data = ', 1)[1])[0]


class FiragaTransferProjectionTest(unittest.TestCase):
    def test_preserves_fields_and_exact_targets(self):
        program, colors = project_firaga_transfer(transfer()), project_firaga_palette_rows(palettes())
        self.assertEqual(program['lz_targets'], [0xaf5660, 0xaf4998, 0xaf5454])
        self.assertEqual(program['palette_entries'], transfer()['palette_entries'])
        self.assertEqual([r['source_offset'] for r in colors['rows']], [0xae6db0, 0xae6db8])
        self.assertEqual([r['colors'] for r in colors['rows']], [r['colors'] for r in palettes()['rows']])

    def test_rejects_changed_bindings_or_invalid_colors(self):
        for mutate in (lambda c: c['records'][1].__setitem__('payload', 0),
                       lambda c: c['lz_wrappers'][0].__setitem__('header', 0),
                       lambda c: c['palette_entries'][0]['parameters'].__setitem__(1, True)):
            value = copy.deepcopy(transfer()); mutate(value)
            with self.subTest(mutation=mutate), self.assertRaises(ValueError): project_firaga_transfer(value)
        for mutate in (lambda c: c['rows'][0]['colors'].pop(), lambda c: c['rows'][1].__setitem__('declared_rows', 1),
                       lambda c: c['rows'][0]['colors'].__setitem__(0, 65536)):
            value = copy.deepcopy(palettes()); mutate(value)
            with self.subTest(mutation=mutate), self.assertRaises(ValueError): project_firaga_palette_rows(value)

    def test_world_scope_and_independent_resources(self):
        for invalid in (None, 'transfer', 'palette'):
            with self.subTest(invalid=invalid), tempfile.TemporaryDirectory(prefix='viewer-firaga-transfer-scope-') as directory:
                index = fixture(Path(directory), invalid=invalid); parts = index.parent / (index.name + '.parts')
                home, world, objects = payload(index), payload(parts / 'world.html'), payload(parts / 'objects.html')
                self.assertNotIn('firaga_transfer', home); self.assertNotIn('firaga_palette_rows', home)
                self.assertNotIn('firaga_transfer', objects); self.assertNotIn('firaga_palette_rows', objects)
                self.assertEqual(bool(world['firaga_transfer']), invalid != 'transfer')
                self.assertEqual(bool(world['firaga_palette_rows']), invalid != 'palette')


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class FiragaTransferBrowserTest(unittest.TestCase):
    def test_transfer_lz_palette_and_reverse_links(self):
        with tempfile.TemporaryDirectory(prefix='viewer-firaga-transfer-') as directory:
            work = Path(directory); index = fixture(work); browser = Chrome(work, index.parent / (index.name + '.parts/world.html'))
            try:
                self.assertEqual(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s),row=id=>[...q('#rows').rows].find(r=>r.dataset.resourceId===id);
                  row('graphics.firaga-transfer-program').querySelector('a').click();
                  if(!q('#firaga-transfers>details').open||q('#firaga-transfer-records').rows.length!==4||
                     q('#firaga-transfer-palette-records').rows.length!==2)throw Error('program');
                  for(let i=1;i<4;i++){
                    const link=q('#firaga-transfer-records').rows[i].querySelector('a');if(!link)throw Error('LZ link');link.click();
                    const source=data.lz_graphics.streams[Number(q('#obj-bank').value)].source_offset;
                    if(q('#obj-catalog').value!=='lz'||source!==data.firaga_transfer.lz_targets[i-1]||
                       q('#obj-lz-transfer-reference').hidden)throw Error('LZ target');
                    q('#obj-lz-transfer-reference a').click();if(!q('#firaga-transfers>details').open)throw Error('LZ reverse');
                  }
                  q('#firaga-transfer-records').rows[0].querySelector('button').click();
                  if(!q('#firaga-transfer-palette-entries').open)throw Error('entry disclosure');
                  q('#firaga-transfer-palette-records').rows[0].querySelector('a').click();
                  if(!q('#firaga-palette-rows>details').open||q('#firaga-palette-row').value!=='1'||
                     !q('#firaga-palette-source').textContent.includes('declared rows 3')||
                     !q('#firaga-palette-source').textContent.includes('one row available')||
                     q('#firaga-palette-colors').children.length!==3||
                     !q('#firaga-palette-colors').textContent.includes('0x801F')||
                     !q('#firaga-palette-colors').textContent.includes('bit 15 1'))throw Error('palette row');
                  q('#firaga-palette-references a').click();if(!q('#firaga-transfer-palette-entries').open)throw Error('palette reverse');
                  q('#firaga-transfer-palette-records').rows[1].querySelector('a').click();
                  const colors=[...document.querySelectorAll('#firaga-palette-colors .color')].map(x=>x.style.backgroundColor);
                  if(colors.join('|')!=='rgb(255, 0, 0)|rgb(0, 255, 0)|rgb(0, 0, 255)')throw Error('RGB');
                  return 'PASS';
                })()'''), 'PASS')
            finally: browser.close()

    def test_missing_sources_do_not_create_links(self):
        for mode in ('no_lz', 'no_palette', 'no_transfer', 'bad_transfer', 'bad_palette'):
            with self.subTest(mode=mode), tempfile.TemporaryDirectory(prefix='viewer-firaga-transfer-missing-') as directory:
                work = Path(directory)
                index = fixture(work, include_lz=mode != 'no_lz', include_palette=mode != 'no_palette',
                                include_transfer=mode != 'no_transfer', invalid=mode.removeprefix('bad_') if mode.startswith('bad_') else None)
                browser = Chrome(work, index.parent / (index.name + '.parts/world.html'))
                try:
                    self.assertTrue(browser.evaluate('''(mode=>{
                      const q=s=>document.querySelector(s),has=(selector)=>!!q(selector);
                      if(mode==='no_lz'&&has('#firaga-transfer-records a'))return false;
                      if((mode==='no_palette'||mode==='bad_palette')&&has('#firaga-transfer-palette-records a'))return false;
                      if((mode==='no_transfer'||mode==='bad_transfer')&&has('#firaga-palette-references a'))return false;
                      return true;
                    })('''+json.dumps(mode)+''')'''))
                finally: browser.close()
