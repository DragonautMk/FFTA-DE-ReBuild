"""Check special-unit source records, verified palette choices and sprite links."""
import copy
import hashlib
import json
import shutil
import sys
import tempfile
import time
import unittest
from contextlib import ExitStack
from pathlib import Path
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import view_rom_data
from test_viewer_job_names_browser import fixture as job_fixture, payload
from viewer_browser_harness import Chrome


def special_catalog():
    records = []
    for index in range(93):
        records.append({'id': index + 2, 'source_offset': 0x6e6f6c + index * 14, 'byte00': index,
                        'selectors01_02': [99, index % 2], 'byte03': 0, 'halfwords04_06': [10, 10],
                        'packed08_0a': [0x21, 0, 1], 'byte0b': 1, 'bytes0c_0d': [2, 3]})
    records[1]['packed08_0a'] = [0, 0, 0]
    return {'address_bias': 0x6e6f50, 'source_offset': 0x6e6f6c, 'first_id': 2, 'records': records}


def palette_catalog():
    spans = []
    for variant, root in [(0, 0x40f290), (1, 0x4108f0), (2, 0x40fe10)]:
        spans.append({'id': 'unit' + str(variant), 'book': 'unit-base', 'variant': variant,
                      'source_offset': root, 'colors': [0] * 256})
    for variant, root in [(0, 0x40fa90), (1, 0x4110f0), (2, 0x410610)]:
        colors = [0] * 144
        colors[1] = 0x001f
        colors[17] = 0x7c00
        spans.append({'id': 'special' + str(variant), 'book': 'special-unit', 'variant': variant,
                      'source_offset': root, 'colors': colors})
    return {'spans': spans, 'slice_offsets': [0, 1, 2, 3]}


def battle_catalog(mode):
    tile = {'indices': [[1] * 8 for _ in range(8)]}
    obj = {'x': 0, 'y': 0, 'shape': 0, 'size': 0, 'tile': 0, 'transform': 0,
           'affine_mode': 0, 'object_mode': 0, 'mosaic': False, 'color_256': False}
    def frame(operation):
        return {'word00': 0, 'word04': 0, 'duration': 4, 'operation': operation, 'bytes_0a_13': [0] * 10}
    value = {'pixels': [{'source_offset': 0x898a40, 'tiles': [tile]}],
             'objects': [{'source_offset': 0xa50020, 'objects': [obj]}],
             'sequences': [{'source_offset': 500, 'opaque_header': 2, 'frames': [frame(2), frame(1)]}]}
    if mode == 'normal':
        value['bindings'] = [{'sprite_id': 10, 'animation': 0, 'part': 0, 'index_source': 100, 'sequence_source': None},
                             {'sprite_id': 10, 'animation': 1, 'part': 0, 'index_source': 104, 'sequence_source': 500}]
    else:
        value['bindings'] = [{'sprite_id': 10, 'animation': 0, 'part': 0, 'index_source': 200,
                              'target': {'kind': 'sequence', 'source_offset': 500}}]
    return value


def fixture(work, *, wrong_identity=False, with_palette=True, with_metadata=True, split=True):
    dataset, _, _ = job_fixture(work); manifest = dataset / 'manifest.toml'; text = manifest.read_text()
    if not with_metadata:
        chunks = text.split('[[resources]]\n')
        text = chunks[0] + ''.join('[[resources]]\n' + chunk for chunk in chunks[1:]
                                   if not chunk.startswith('id="gameplay.linked-metadata52"\n'))
    items = [('gameplay.linked-metadata14', 'linked-metadata14', special_catalog())]
    if with_palette:
        items.append(('graphics.color-mode-palettes', 'color-mode-palettes', palette_catalog()))
    items += [('graphics.unit-obj-' + mode + '-battle', 'unit-obj-' + mode + '-battle', battle_catalog(mode))
              for mode in ('normal', 'alternate')]
    hashes = {}
    for identity, name, value in items:
        content = json.dumps(value).encode(); (dataset / (name + '.json')).write_bytes(content)
        digest = hashlib.sha256(content).hexdigest(); hashes[identity] = digest
        text += (f'[[resources]]\nid="{identity}"\npath="{name}.json"\n'
                 f'format="{name}-v1-json"\nsha256="{digest}"\n')
    manifest.write_text(text)
    with ExitStack() as stack:
        stack.enter_context(patch.object(view_rom_data, 'EUROPE_METADATA14_SHA256',
                            'wrong' if wrong_identity else hashes['gameplay.linked-metadata14']))
        stack.enter_context(patch.object(view_rom_data, 'EUROPE_METADATA52_SHA256',
                            hashlib.sha256((dataset / 'linked-metadata52.json').read_bytes()).hexdigest()))
        stack.enter_context(patch.object(view_rom_data, 'EUROPE_TEXT_CATALOG_SHA256',
                            hashlib.sha256((dataset / 'text-catalogs.json').read_bytes()).hexdigest()))
        stack.enter_context(patch.object(view_rom_data, 'EUROPE_COLOR_MODE_SHA256', hashes.get('graphics.color-mode-palettes', 'missing')))
        stack.enter_context(patch.object(view_rom_data, 'EUROPE_BATTLE_SPRITE_SHA256',
                            {mode: hashes['graphics.unit-obj-' + mode + '-battle'] for mode in ('normal', 'alternate')}))
        page = view_rom_data.generate(dataset, work / 'view', split=split)
    return page


def wait_for(browser, condition):
    deadline = time.monotonic() + 30
    while time.monotonic() < deadline:
        try:
            if browser.evaluate('document.readyState === "complete" && ' + condition):
                return
        except (RuntimeError, OSError):
            pass
        time.sleep(.05)
    raise AssertionError('Page did not reach expected state: ' + condition)


class SpecialUnitProjectionTest(unittest.TestCase):
    def test_fields_and_identity_guards(self):
        projected = view_rom_data.project_metadata14(special_catalog())
        self.assertEqual(projected['records'][0]['packed08_0a'], [0x21, 0, 1])
        mutations = [lambda d: d['records'].pop(), lambda d: d['records'][0].update(id=True),
                     lambda d: d['records'][0].update(source_offset=1),
                     lambda d: d['records'][0]['halfwords04_06'].__setitem__(0, 65536),
                     lambda d: d['records'][0]['packed08_0a'].pop(),
                     lambda d: d['records'][0]['selectors01_02'].__setitem__(0, True)]
        for mutate in mutations:
            bad = copy.deepcopy(special_catalog()); mutate(bad)
            with self.subTest(bad=bad), self.assertRaises(ValueError):
                view_rom_data.project_metadata14(bad)
        for options in ({}, {'wrong_identity': True}, {'with_palette': False}, {'with_metadata': False}):
            with self.subTest(options=options), tempfile.TemporaryDirectory() as directory:
                page = fixture(Path(directory), **options); parts = page.parent / 'index.html.parts'
                world, battle, interface = map(payload, (parts / 'world.html', parts / 'battle.html', parts / 'interface.html'))
                self.assertEqual(len(world['metadata14']['records']), 93)
                self.assertNotIn('metadata14', battle); self.assertNotIn('metadata14', interface)
                self.assertEqual(bool(world['metadata14_links']), not options.get('wrong_identity', False))
                self.assertEqual(bool(world['special_unit_palettes']), options.get('with_palette', True))
                self.assertEqual(world['special_unit_palettes'], battle['special_unit_palettes'])
                if world['metadata14_links']:
                    self.assertEqual(world['metadata14_links']['metadata_verified'], options.get('with_metadata', True))
                    self.assertEqual(world['metadata14_links']['sprite_ids']['normal'], [10])


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class SpecialUnitBrowserTest(unittest.TestCase):
    def test_names_palette_sprite_links_and_bookmark(self):
        with tempfile.TemporaryDirectory(prefix='viewer-special-units-') as directory:
            work = Path(directory); page = fixture(work); parts = page.parent / 'index.html.parts'
            browser = Chrome(work, parts / 'world.html')
            try:
                link = browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change',{bubbles:true}));
                  q('[data-resource-id="gameplay.linked-metadata14"] a[href="#metadata14"]').click();
                  const select=q('#metadata14-record');
                  if(q('#metadata14').hidden || !q('#metadata14>details').open || select.options.length!==93 ||
                     q('#metadata14-fields').rows.length!==7 || q('#metadata14-colors').children.length!==16 ||
                     !q('#metadata14-palette-detail').textContent.includes('source index 0'))throw Error('source record');
                  q('#metadata14-job a').click();
                  if(q('#metadata-name-rows tr').dataset.metadataId!=='0' || !q('#metadata-name-rows [data-special-unit="2"]'))throw Error('job reference');
                  q('#metadata-name-rows [data-special-unit="2"]').click();if(select.value!=='2')throw Error('reverse reference');
                  q('#metadata14-branch').value='one';change(q('#metadata14-branch'));
                  q('#metadata14-variant').value='2';change(q('#metadata14-variant'));
                  if(!q('#metadata14-palette-detail').textContent.includes('source index 1') ||
                     !q('#metadata14-palette-detail').textContent.includes('destination bank 1'))throw Error('palette decoding');
                  const target=q('#metadata14-sprites [data-sprite-mode="normal"]'), alternate=q('#metadata14-sprites [data-sprite-mode="alternate"]');
                  if(!target || !target.href.includes('special_variant=2') || !target.href.includes('special_index=1'))throw Error('sprite link');
                  if(!alternate || !alternate.href.includes('/battle-alternate.html?'))throw Error('alternate sprite scope');
                  return {normal:target.href,alternate:alternate.href};
                })()''')
                browser.command('Page.navigate', {'url': link['normal']})
                wait_for(browser, "typeof data !== 'undefined' && data.wiki.current==='battle' && document.querySelector('#equipment-sprite').value==='10'")
                self.assertTrue(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s),pixel=[...q('#equipment-pose').getContext('2d').getImageData(0,0,1,1).data].join(',');
                  return q('#equipment-catalog').value==='normal' && q('#equipment-frame').value==='1' &&
                    q('#equipment-palette').value==='special:2:1' && pixel==='0,0,255,255' && q('#equipment-sprites>details').open;
                })()'''))
                browser.command('Page.navigate', {'url': link['alternate']})
                wait_for(browser, "typeof data !== 'undefined' && data.wiki.current==='battle-alternate' && document.querySelector('#equipment-sprite').value==='10'")
                self.assertEqual(browser.evaluate("document.querySelector('#equipment-palette').value"), 'special:2:1')
            finally:
                browser.close()

    def test_zero_and_unverified_sources(self):
        for options in ({'wrong_identity': True}, {'with_palette': False}, {'with_metadata': False}):
            with self.subTest(options=options), tempfile.TemporaryDirectory(prefix='viewer-special-partial-') as directory:
                page = fixture(Path(directory), split=False, **options); browser = Chrome(Path(directory), page)
                try:
                    result = browser.evaluate('''(()=>{
                      const q=s=>document.querySelector(s),select=q('#metadata14-record');select.value='3';select.dispatchEvent(new Event('change'));
                      return {hidden:q('#metadata14').hidden,job:!!q('#metadata14-job a'),sprite:!!q('#metadata14-sprites a'),
                        palette:q('#metadata14-palette-detail').textContent,colors:q('#metadata14-colors').children.length};})()''')
                    self.assertFalse(result['hidden'])
                    self.assertEqual(result['job'], not (options.get('wrong_identity') or options.get('with_metadata') is False))
                    self.assertEqual(result['sprite'], not options.get('wrong_identity', False))
                    self.assertEqual(result['colors'], 0)
                    self.assertIn('authenticated' if options.get('wrong_identity') else 'selector 0', result['palette'])
                finally:
                    browser.close()
