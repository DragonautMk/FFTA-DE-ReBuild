"""Check scoped browsing and playback of decoded indexed-archive programs."""
import copy
import hashlib
import json
import shutil
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from view_rom_data import generate, project_indexed_graphics
from viewer_browser_harness import Chrome


def archive(source, count, *, tiles=129):
    programs = []
    for index in range(count):
        word = (index + 1) * 0x11111111
        programs.append({'index': index, 'relative_offset': 16 + index * 20,
                         'packets': [{'kind': 'literal', 'bytes': [0]}, {'kind': 'dictionary', 'offset': 0, 'length': 3}],
                         'decoded_words': [word] * (tiles * 8)})
    return {'source_offset': source, 'opaque_header': [1, 2],
            'entry_offsets': [item['relative_offset'] for item in programs],
            'dictionary_relative_offset': 1000, 'dictionary_mode': 0, 'dictionary_length_bits': 5,
            'fixed_decoded_bytes': tiles * 32, 'dictionary_bytes': [1, 2, 3], 'programs': programs}


def catalog():
    return {'archives': [archive(0x1000, 2), archive(0x2000, 1, tiles=2)],
            'color_archives': [archive(0x3000, 2, tiles=2)],
            'white_flowers': None, 'white_flowers_direct': None,
            'additional': {'archives': [archive(0x4000, 2, tiles=2)]}, 'reader_banks': None}


def fixture(work, value=None, *, palettes=True, version=1):
    dataset = work / 'dataset'; dataset.mkdir()
    items = [('graphics.indexed-archives', 'indexed-graphics', version, catalog() if value is None else value)]
    if palettes:
        items.append(('graphics.bg-base-palettes', 'bg-base-palettes', 2,
                      {'entries': [{'banks': [[0, 0x001f, 0x03e0, 0x7c00] + [0] * 12]}]}))
    manifest = 'profile="test"\nextractor_revision=1\ndecoder_revision=1\n'
    for rid, name, version, contents in items:
        content = json.dumps(contents).encode(); (dataset / (name + '.json')).write_bytes(content)
        manifest += (f'[[resources]]\nid="{rid}"\npath="{name}.json"\nformat="{name}-v{version}-json"\n'
                     f'sha256="{hashlib.sha256(content).hexdigest()}"\n')
    (dataset / 'manifest.toml').write_text(manifest)
    index = generate(dataset, work / 'view', split=True)
    return index, index.parent / (index.name + '.parts/indexed-graphics.html')


def payload(path):
    return json.JSONDecoder().raw_decode(path.read_text().split('const data = ', 1)[1])[0]


class IndexedGraphicsProjectionTest(unittest.TestCase):
    def test_projected_programs_preserve_words_and_packet_counts(self):
        source = catalog(); result = project_indexed_graphics(source)
        self.assertEqual([item['id'] for item in result['families']], ['base', 'color', 'additional'])
        program = result['families'][0]['archives'][0]['programs'][1]
        self.assertEqual(program['words'], source['archives'][0]['programs'][1]['decoded_words'])
        self.assertEqual((program['packet_count'], program['dictionary_packets']), (2, 1))
        self.assertNotIn('packets', program)
        self.assertEqual(source, catalog())

    def test_invalid_sources_are_rejected(self):
        mutations = [lambda c: c['archives'][0].__setitem__('dictionary_mode', 3),
                     lambda c: c['archives'][0]['entry_offsets'].__setitem__(0, 99),
                     lambda c: c['archives'][0]['programs'][0]['decoded_words'].__setitem__(0, True),
                     lambda c: c['archives'][0]['programs'][0]['decoded_words'].pop(),
                     lambda c: c['archives'][0]['programs'][0]['packets'][0].__setitem__('kind', 'unknown'),
                     lambda c: c['archives'][0].__setitem__('source_offset', -1)]
        for mutate in mutations:
            value = copy.deepcopy(catalog()); mutate(value)
            with self.subTest(mutation=mutate), self.assertRaises(ValueError):
                project_indexed_graphics(value)

    def test_scoped_page_and_invalid_catalog(self):
        for version in (1, 2):
            with self.subTest(version=version), tempfile.TemporaryDirectory(prefix='viewer-indexed-scope-') as directory:
                index, page = fixture(Path(directory), version=version); parts = index.parent / (index.name + '.parts')
                home, world, scoped = payload(index), payload(parts/'world.html'), payload(page)
                self.assertNotIn('indexed_archives', home); self.assertNotIn('indexed_archives', world)
                self.assertEqual(len(scoped['indexed_archives']['families']), 3)
                self.assertEqual(next(item for item in home['resources'] if item['id']=='graphics.indexed-archives')['viewer_scope'], 'indexed-graphics')
        with tempfile.TemporaryDirectory(prefix='viewer-indexed-invalid-') as directory:
            bad = catalog(); bad['archives'][0]['programs'][0]['decoded_words'].pop()
            _, page = fixture(Path(directory), bad); data = payload(page)
            self.assertIsNone(data['indexed_archives'])
            self.assertTrue(data['resources'][0]['status'].startswith('Invalid indexed graphics'))


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class IndexedGraphicsBrowserTest(unittest.TestCase):
    def test_tiles_palettes_program_playback_and_focus(self):
        with tempfile.TemporaryDirectory(prefix='viewer-indexed-browser-') as directory:
            work = Path(directory); _, page = fixture(work); browser = Chrome(work, page)
            try:
                self.assertEqual(browser.evaluate('''(async()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change')),
                    input=e=>e.dispatchEvent(new Event('input'));
                  const pixel=()=>[...q('#indexed-sheet').getContext('2d').getImageData(0,0,1,1).data].join(',');
                  const row=q('[data-resource-id="graphics.indexed-archives"]');
                  if(q('#indexed-graphics').hidden||q('#indexed-graphics>details').open||q('#indexed-detail').textContent)throw Error('lazy page');
                  row.querySelector('a[href="#indexed-graphics"]').click();await new Promise(r=>setTimeout(r,0));
                  if(!q('#indexed-graphics>details').open||q('#indexed-family').options.length!==3||
                     q('#indexed-archive').options.length!==2||q('#indexed-program').options.length!==2||
                     q('#indexed-page').options.length!==2||pixel()!=='17,17,17,255')throw Error('first sheet');
                  q('#indexed-palette').value='0';change(q('#indexed-palette'));if(pixel()!=='255,0,0,255')throw Error('palette');
                  q('#indexed-depth').value='8';change(q('#indexed-depth'));
                  if(pixel()!=='0,192,255,255'||!q('#indexed-palette-bank').disabled||!q('#indexed-detail').textContent.includes('lack palette colors'))throw Error('8-bit missing palette');
                  q('#indexed-palette').value='gray';change(q('#indexed-palette'));if(pixel()!=='17,17,17,255')throw Error('8-bit grayscale');
                  q('#indexed-depth').value='4';change(q('#indexed-depth'));q('#indexed-page').value='1';change(q('#indexed-page'));
                  if(pixel()!=='17,17,17,255'||!q('#indexed-detail').textContent.includes('129 complete tiles'))throw Error('second page');
                  const canvas=q('#indexed-sheet'),rect=canvas.getBoundingClientRect();
                  canvas.dispatchEvent(new MouseEvent('click',{clientX:rect.left+rect.width/32,clientY:rect.top+rect.height/16}));
                  if(!q('#indexed-tile-detail').textContent.includes('Tile 128')||!q('#indexed-tile-detail').textContent.includes('0x1000'))throw Error('tile position');
                  window.indexedTimers=new Map();let serial=0;
                  window.setTimeout=(fn,delay)=>{const id=++serial;indexedTimers.set(id,{fn,delay});return id};
                  window.clearTimeout=id=>indexedTimers.delete(id);
                  window.fireIndexed=()=>{const [id,item]=indexedTimers.entries().next().value;indexedTimers.delete(id);item.fn();return item.delay};
                  q('#indexed-rate').value='10';q('#indexed-play').click();
                  if(fireIndexed()!==100||q('#indexed-program').value!=='1'||pixel()!=='34,34,34,255'||q('#indexed-progress').value!==2)throw Error('advance');
                  fireIndexed();if(!q('#indexed-stop').disabled||!q('#indexed-status').textContent.includes('finished'))throw Error('finish');
                  q('#indexed-loop').checked=true;q('#indexed-play').click();fireIndexed();
                  if(q('#indexed-program').value!=='0'||pixel()!=='17,17,17,255')throw Error('loop');
                  q('#indexed-family').value='1';change(q('#indexed-family'));
                  if(q('#indexed-stop').disabled||!q('#indexed-loop').checked||q('#indexed-rate').value!=='10'||indexedTimers.size!==1)throw Error('preserved settings');
                  q('#indexed-family').value='0';change(q('#indexed-family'));q('#indexed-archive').value='1';change(q('#indexed-archive'));
                  if(!q('#indexed-play').disabled||!q('#indexed-stop').disabled||indexedTimers.size||!q('#indexed-loop').checked)throw Error('single program');
                  q('#indexed-archive').value='0';change(q('#indexed-archive'));q('#indexed-rate').value='0';q('#indexed-play').click();
                  if(!q('#indexed-stop').disabled||!q('#indexed-status').textContent.includes('Choose a rate'))throw Error('rate guard');
                  q('#indexed-rate').value='10';q('#indexed-play').click();q('#indexed-graphics>details').open=false;
                  q('#indexed-graphics>details').dispatchEvent(new Event('toggle'));
                  if(!q('#indexed-stop').disabled||indexedTimers.size)throw Error('closed stop');
                  q('#indexed-graphics>details').open=true;q('#indexed-program').focus();return 'PASS';
                })()'''), 'PASS')
                browser.click('#indexed-play')
                self.assertEqual(browser.evaluate('document.activeElement.id'), 'indexed-program')
            finally:
                browser.close()

    def test_standalone_grayscale_and_absent_catalog(self):
        with tempfile.TemporaryDirectory(prefix='viewer-indexed-standalone-') as directory:
            work = Path(directory); _, page = fixture(work, palettes=False); browser = Chrome(work, page)
            try:
                self.assertTrue(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s);q('#indexed-graphics>details').open=true;
                  return q('#indexed-palette').value==='gray'&&q('#indexed-palette-bank').disabled&&q('#indexed-family').options.length===3;
                })()'''))
            finally:
                browser.close()
