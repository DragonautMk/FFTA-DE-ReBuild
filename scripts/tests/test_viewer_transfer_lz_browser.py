"""Check bounded binary LZ inputs and their manual graphics previews."""
import copy
import hashlib
import json
import shutil
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from view_rom_data import generate, project_transfer_lz
from test_viewer_observed_lz_browser import fixture as observed_fixture, literal_stream, payload
from viewer_browser_harness import Chrome


def inputs(offset=0x5000):
    blocks = [bytes(literal_stream([0x21] * 64)), bytes(literal_stream([0x45] * 32 + [1, 2, 3]))]
    sources = [{'offset': offset + index * 0x1000, 'length': len(block),
                'sha256': hashlib.sha256(block).hexdigest(), 'role': 'lz77-input'}
               for index, block in enumerate(blocks)]
    return b''.join(blocks), sources


def fixture(work, *, observed=True, bad_source=False, overlap=False):
    observed_fixture(work)
    dataset = work / 'dataset'; manifest = dataset / 'manifest.toml'
    text = manifest.read_text()
    if not observed:
        chunks = text.split('[[resources]]\n')
        text = chunks[0] + ''.join('[[resources]]\n' + chunk for chunk in chunks[1:]
                                  if not chunk.startswith('id="graphics.observed-lz-block"\n'))
    content, sources = inputs(0xaf4998 if overlap else 0x5000)
    (dataset / 'tilemap-transfer-lz77.bin').write_bytes(content)
    text += ('[[resources]]\nid="graphics.tilemap-transfer-lz77"\npath="tilemap-transfer-lz77.bin"\n'
             f'format="tilemap-transfer-lz77-v1-bin"\nsha256="{hashlib.sha256(content).hexdigest()}"\n')
    for index, source in enumerate(sources):
        sha = '0' * 64 if bad_source and index == 0 else source['sha256']
        text += (f'[[resources.sources]]\noffset={source["offset"]}\nlength={source["length"]}\n'
                 f'sha256="{sha}"\nrole="lz77-input"\n')
    manifest.write_text(text)
    return generate(dataset, work / 'combined', split=True), content, sources


class TransferLzProjectionTest(unittest.TestCase):
    def test_segments_decode_and_guard_metadata(self):
        content, sources = inputs(); result = project_transfer_lz(content, sources)
        self.assertEqual(len(result['streams']), 2)
        self.assertEqual(result['streams'][0]['words'][0], 0x21212121)
        self.assertEqual(result['streams'][1]['tail'], [1, 2, 3])
        for mutate in [lambda a: a[0].update(offset=True), lambda a: a[0].update(length=0),
                       lambda a: a[0].update(role='unknown'), lambda a: a[0].update(sha256='0' * 64),
                       lambda a: a[1].update(offset=a[0]['offset'] + 1)]:
            bad = copy.deepcopy(sources); mutate(bad)
            with self.subTest(bad=bad), self.assertRaises(ValueError): project_transfer_lz(content, bad)
        with self.assertRaises(ValueError): project_transfer_lz(content + b'\x00', sources)
        with self.assertRaises(ValueError): project_transfer_lz(content[:-1], sources)
        broken = b'\x10\x01\x00\x00\x80\x00\x00'
        with self.assertRaises(ValueError):
            project_transfer_lz(broken, [{'offset': 1, 'length': len(broken), 'role': 'lz77-input',
                                          'sha256': hashlib.sha256(broken).hexdigest()}])

    def test_merging_scopes_and_invalid_or_overlapping_resources(self):
        for options in ({}, {'observed': False}, {'bad_source': True}, {'overlap': True}):
            with self.subTest(options=options), tempfile.TemporaryDirectory(prefix='viewer-transfer-lz-projection-') as directory:
                work = Path(directory); index, _, _ = fixture(work, **options)
                parts = index.parent / 'index.html.parts'; world = payload(parts / 'interface.html'); objects = payload(parts / 'objects.html')
                self.assertNotIn('lz_graphics', payload(index))
                self.assertEqual(world['lz_graphics'], objects['lz_graphics'])
                valid = not (options.get('bad_source') or options.get('overlap'))
                count = (5 if options.get('observed', True) else 0) + (2 if valid else 0)
                self.assertEqual(len(world['lz_graphics']['streams']), count)
                row = next(row for row in world['resources'] if row['id'] == 'graphics.tilemap-transfer-lz77')
                self.assertEqual(row['status'].startswith('Available'), valid)
                if valid:
                    self.assertEqual(world['lz_graphics']['streams'][-1]['path'], 'tilemap-transfer-lz77.bin')
                    self.assertEqual(world['lz_graphics']['streams'][-1]['resource_id'], row['id'])


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class TransferLzBrowserTest(unittest.TestCase):
    def test_tiles_tilemap_and_object_animation_sources(self):
        with tempfile.TemporaryDirectory(prefix='viewer-transfer-lz-browser-') as directory:
            work = Path(directory); index, _, _ = fixture(work); parts = index.parent / 'index.html.parts'
            browser = Chrome(work, parts / 'interface.html')
            try:
                self.assertEqual(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change',{bubbles:true})),
                    pixel=(c,x,y)=>[...c.getContext('2d').getImageData(x,y,1,1).data].join(',');
                  const row=q('[data-resource-id="graphics.tilemap-transfer-lz77"]'),link=row.querySelector('a[href="#obj-tiles"]');
                  if(!link)throw Error('resource link');link.click();
                  if(!q('#obj-tiles>details').open || q('#obj-catalog').value!=='lz' || q('#obj-bank').value!=='5' ||
                     q('#obj-bank').options.length!==7 || !q('#obj-detail').textContent.includes('tilemap-transfer-lz77.bin') ||
                     !q('#obj-bank').selectedOptions[0].textContent.includes('Task 345'))throw Error('source selection');
                  if(pixel(q('#obj-sheet'),0,0)!=='17,17,17,255' || pixel(q('#obj-sheet'),1,0)!=='34,34,34,255')throw Error('4-bit pixels');
                  q('#obj-depth').value='8';change(q('#obj-depth'));if(pixel(q('#obj-sheet'),0,0)!=='33,33,33,255')throw Error('8-bit pixels');
                  q('#tilemap-graphics').value='5';change(q('#tilemap-graphics'));
                  if(pixel(q('#tilemap-canvas'),0,0)!=='17,17,17,255')throw Error('tilemap graphics');
                  q('#obj-bank').value='6';change(q('#obj-bank'));q('#obj-depth').value='4';change(q('#obj-depth'));
                  if(!q('#obj-detail').textContent.includes('3 trailing bytes'))throw Error('tail');
                  return 'PASS';
                })()'''), 'PASS')
            finally: browser.close()
            other = work / 'objects-browser'; other.mkdir(); browser = Chrome(other, parts / 'objects.html')
            try:
                self.assertTrue(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change'));
                  q('#obj-animation-graphics').value='5';change(q('#obj-animation-graphics'));
                  q('#obj-animation-outlines').checked=false;change(q('#obj-animation-outlines'));
                  return [...q('#obj-animation-canvas').getContext('2d').getImageData(8,8,1,1).data].join(',')==='17,17,17,255';
                })()'''))
            finally: browser.close()

    def test_transfer_only_and_rejected_source_links(self):
        for options in ({'observed': False}, {'bad_source': True}, {'overlap': True}):
            with self.subTest(options=options), tempfile.TemporaryDirectory(prefix='viewer-transfer-lz-partial-') as directory:
                work = Path(directory); index, _, _ = fixture(work, **options)
                browser = Chrome(work, index.parent / 'index.html.parts/interface.html')
                try:
                    valid = not (options.get('bad_source') or options.get('overlap'))
                    self.assertEqual(browser.evaluate('''(()=>{
                      const q=s=>document.querySelector(s), row=q('[data-resource-id="graphics.tilemap-transfer-lz77"]');
                      const a=row.querySelector('a[href="#obj-tiles"]');if(a)a.click();
                      return {link:!!a,visible:!q('#obj-tiles').hidden,
                        selected:a?q('#obj-detail').textContent.includes('tilemap-transfer-lz77.bin'):true};
                    })()'''), {'link': valid, 'visible': True, 'selected': True})
                finally: browser.close()
