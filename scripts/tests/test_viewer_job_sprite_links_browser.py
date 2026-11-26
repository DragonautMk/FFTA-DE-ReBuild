"""Check verified job-to-battle sprite links and explicit palette previews."""
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
from test_viewer_job_names_browser import fixture as job_fixture, payload
from test_viewer_special_units_browser import battle_catalog, palette_catalog, wait_for
from viewer_browser_harness import Chrome


def fixture(work, *, wrong_identity=False, inconsistent=False, with_palette=True):
    dataset, _, metadata = job_fixture(work)
    manifest = dataset / 'manifest.toml'
    text = manifest.read_text()
    old = (dataset / 'linked-metadata52.json').read_bytes()
    records = metadata['records']
    records[0]['bytes_06_1f'][1:6] = [10, 0, 10, 0, 0x53]
    records[1]['routing'] = 255
    records[2]['bytes_06_1f'][1:5] = [255, 255, 10, 0]
    content = json.dumps(metadata).encode()
    (dataset / 'linked-metadata52.json').write_bytes(content)
    text = text.replace(hashlib.sha256(old).hexdigest(), hashlib.sha256(content).hexdigest())
    producers = []
    for identity, record in enumerate(records):
        if record['routing']:
            continue
        for mode, offset in (('normal', 1), ('alternate', 3)):
            raw = record['bytes_06_1f']
            sprite = raw[offset] | raw[offset + 1] << 8
            if sprite != 65535:
                producers.append({'kind': 'job', 'mode': mode, 'logical_id': identity,
                                  'metadata_source': metadata['source_offset'] + identity * 52 + 6 + offset,
                                  'sprite_id': sprite})
    if inconsistent:
        producers[0]['sprite_id'] = 11
    items = [('graphics.unit-obj-property-words', 'unit-obj-property-words', {'producers': producers})]
    if with_palette:
        colors = palette_catalog()
        colors['spans'][2]['colors'][3 * 16 + 1] = 0x03e0
        items.append(('graphics.color-mode-palettes', 'color-mode-palettes', colors))
    items += [('graphics.unit-obj-' + mode + '-battle', 'unit-obj-' + mode + '-battle', battle_catalog(mode))
              for mode in ('normal', 'alternate')]
    hashes = {}
    for identity, name, value in items:
        content = json.dumps(value).encode()
        (dataset / (name + '.json')).write_bytes(content)
        digest = hashlib.sha256(content).hexdigest()
        hashes[identity] = digest
        text += (f'[[resources]]\nid="{identity}"\npath="{name}.json"\n'
                 f'format="{name}-v1-json"\nsha256="{digest}"\n')
    manifest.write_text(text)
    with ExitStack() as stack:
        for name, value in {
            'EUROPE_METADATA52_SHA256': hashlib.sha256((dataset / 'linked-metadata52.json').read_bytes()).hexdigest(),
            'EUROPE_TEXT_CATALOG_SHA256': hashlib.sha256((dataset / 'text-catalogs.json').read_bytes()).hexdigest(),
            'EUROPE_OBJ_PROPERTY_WORDS_SHA256': 'wrong' if wrong_identity else hashes['graphics.unit-obj-property-words'],
            'EUROPE_COLOR_MODE_SHA256': hashes.get('graphics.color-mode-palettes', 'missing'),
            'EUROPE_BATTLE_SPRITE_SHA256': {mode: hashes['graphics.unit-obj-' + mode + '-battle']
                                             for mode in ('normal', 'alternate')},
        }.items():
            stack.enter_context(patch.object(view_rom_data, name, value))
        page = view_rom_data.generate(dataset, work / 'view', split=True)
    return page, producers


class JobSpriteProjectionTest(unittest.TestCase):
    def test_verified_sources_redirects_and_missing_palette(self):
        for options in ({}, {'wrong_identity': True}, {'inconsistent': True}, {'with_palette': False}):
            with self.subTest(options=options), tempfile.TemporaryDirectory(prefix='viewer-job-sprite-project-') as directory:
                page, producers = fixture(Path(directory), **options)
                parts = page.parent / 'index.html.parts'
                world, battle, interface = map(payload, (parts / 'world.html', parts / 'battle.html', parts / 'interface.html'))
                links = world['job_sprite_links']
                if options.get('wrong_identity') or options.get('inconsistent'):
                    self.assertIsNone(links)
                else:
                    self.assertTrue(links['names_verified'])
                    self.assertEqual(links['palette_preview'], options.get('with_palette', True))
                    self.assertEqual(links['rows'][0], {'id': 0, 'routing': 0, 'palette_nibbles': [3, 5],
                                                        'normal': 10, 'alternate': 10})
                    self.assertIsNone(links['rows'][1]['normal'])
                    self.assertIsNone(links['rows'][1]['palette_nibbles'])
                    self.assertIsNone(links['rows'][2]['normal'])
                    self.assertEqual(links['rows'][2]['alternate'], 10)
                self.assertNotIn('job_sprite_links', battle)
                self.assertNotIn('job_sprite_links', interface)
                duplicate = copy.deepcopy(producers)
                duplicate.append(copy.deepcopy(duplicate[0]))
                with self.assertRaises(ValueError):
                    view_rom_data.project_job_sprite_producers({'producers': duplicate})


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class JobSpriteBrowserTest(unittest.TestCase):
    def test_navigation_and_explicit_preview_palette(self):
        with tempfile.TemporaryDirectory(prefix='viewer-job-sprite-browser-') as directory:
            work = Path(directory)
            page, _ = fixture(work)
            parts = page.parent / 'index.html.parts'
            browser = Chrome(work, parts / 'world.html')
            try:
                link = browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change',{bubbles:true}));
                  if(q('#metadata-sprite-controls').hidden)throw Error('missing controls');
                  q('#job-name-rows button').click();
                  if(q('#metadata-name-rows tr').dataset.metadataId!=='0' || !q('#metadata-names>details').open)
                    throw Error('metadata navigation');
                  const find=()=>q('#metadata-name-rows [data-job-id="0"][data-sprite-mode="normal"]');
                  if(!find() || !q('#metadata-name-rows [data-job-id="0"][data-sprite-mode="alternate"]'))throw Error('links');
                  if(new URL(find().href).searchParams.has('unit_bank'))throw Error('grayscale');
                  q('#metadata-sprite-palette').value='high';change(q('#metadata-sprite-palette'));
                  if(new URL(find().href).searchParams.get('unit_bank')!=='5')throw Error('high nibble');
                  q('#metadata-sprite-palette').value='low';change(q('#metadata-sprite-palette'));
                  q('#metadata-sprite-variant').value='2';change(q('#metadata-sprite-variant'));
                  const target=find().href, url=new URL(target);
                  if(url.searchParams.get('unit_bank')!=='3' || url.searchParams.get('unit_variant')!=='2')throw Error('low nibble');
                  openMetadataReference(1);
                  if(q('#metadata-name-rows [data-job-id]') || !q('#metadata-name-rows').textContent.includes('Redirect not evaluated'))
                    throw Error('redirect');
                  openMetadataReference(2);
                  if(q('#metadata-name-rows [data-sprite-mode="normal"]') || !q('#metadata-name-rows [data-sprite-mode="alternate"]'))
                    throw Error('sentinel');
                  return target;
                })()''')
                browser.command('Page.navigate', {'url': link})
                wait_for(browser, "typeof data !== 'undefined' && data.wiki.current==='battle' && document.querySelector('#equipment-sprite').value==='10'")
                self.assertTrue(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s),pixel=[...q('#equipment-pose').getContext('2d').getImageData(0,0,1,1).data].join(',');
                  const palette=data.unit_palettes[Number(q('#equipment-palette').value)];
                  return q('#equipment-catalog').value==='normal' && q('#equipment-frame').value==='1' &&
                    palette.variant===2 && palette.bank===3 && pixel==='0,255,0,255' && q('#equipment-sprites>details').open;
                })()'''))
                browser.command('Page.navigate', {'url': link.replace('unit_bank=3', 'unit_bank=16')})
                wait_for(browser, "typeof data !== 'undefined' && data.wiki.current==='battle' && location.search.includes('unit_bank=16')")
                self.assertEqual(browser.evaluate("document.querySelector('#equipment-palette').value"), 'gray')
                browser.command('Page.navigate', {'url': (parts / 'actors.html').as_uri()})
                wait_for(browser, "typeof data !== 'undefined' && data.wiki.current==='actors'")
                self.assertTrue(browser.evaluate('''(()=>{
                  const a=document.querySelector('[data-resource-id="graphics.unit-obj-property-words"] a[href$="world.html#metadata-names"]');
                  return !!a && a.textContent.includes('job sprite');
                })()'''))
            finally:
                browser.close()
