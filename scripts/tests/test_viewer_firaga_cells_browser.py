"""Exercise compact Firaga cell decoding, navigation and preview timing."""
import copy
import functools
import hashlib
import json
import shutil
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from view_rom_data import decode_tilemap_tokens, generate, project_firaga_compact_cells
from viewer_browser_harness import Chrome


SHAPES = [(3, 2, 8), (3, 4, 16), (6, 6, 52), (7, 8, 52), (7, 8, 44),
          (10, 9, 88), (9, 9, 82), (7, 8, 72), (7, 8, 88), (6, 8, 100)]
OFFSETS = [20, 28, 44, 96, 148, 192, 280, 362, 434, 522]


@functools.lru_cache(None)
def runs(cells, words):
    if cells == words == 0:
        return []
    if cells <= 0 or words < 2:
        return None
    for count in range(1, min(cells, words - 1) + 1):
        rest = runs(cells - count, words - count - 1)
        if rest is not None:
            return [(0, count)] + rest
    for count in range(cells, 0, -1):
        rest = runs(cells - count, words - 2)
        if rest is not None:
            return [(0x8000 if count % 2 == 0 else 0x4000, count)] + rest
    return None


def catalog():
    frames = []
    for index, ((width, height, length), offset) in enumerate(zip(SHAPES, OFFSETS)):
        plan = runs(width * height, (length - 4) // 2)
        assert plan is not None
        tokens = [{'control': mode | count, 'values': ([0xfffe] if mode else list(range(count)))}
                  for mode, count in plan]
        frames.append({'source_offset': 0xae9c34 + offset, 'width': width, 'height': height,
                       'opaque_header': [index, 255 - index], 'tokens': tokens})
    return {'caller_literal': 0x0839cc98, 'table_pointer': 0x08ae9c30, 'flags': 12, 'split_offset': 628,
            'directory_offsets': OFFSETS, 'frames': frames,
            'sequence': [{'frame_index': i, 'high_flags': i % 4, 'second_byte': i + 60} for i in range(10)],
            'indexed_fields': {'byte_cc96': 1, 'halfword_cca8': 2, 'byte_ccba': 3, 'byte_ccd2': 4, 'word_ccfc': 5}}


def fixture(work, *, observed=True, missing_frame=False, invalid_firaga=False, invalid_observed=False):
    dataset = work / 'dataset'; dataset.mkdir(); value = catalog()
    if missing_frame:
        value['sequence'][2]['frame_index'] = 63
    if invalid_firaga:
        value['flags'] = 0
    items = [('graphics.firaga-compact-cells', 'firaga-compact-cells', 1, value)]
    if observed:
        items.append(('graphics.observed-tilemap', 'observed-tilemap', 1,
                      {'source_offset': 100, 'width': 2, 'height': 2, 'mode': 0 if invalid_observed else 4,
                       'runs': [{'control': 0x4004, 'value': 1}]}))
    manifest = 'profile="test"\nextractor_revision=1\ndecoder_revision=1\n'
    for rid, name, version, value in items:
        content = json.dumps(value).encode(); (dataset / (name + '.json')).write_bytes(content)
        manifest += (f'[[resources]]\nid="{rid}"\npath="{name}.json"\nformat="{name}-v{version}-json"\n'
                     f'sha256="{hashlib.sha256(content).hexdigest()}"\n')
    (dataset / 'manifest.toml').write_text(manifest)
    return generate(dataset, work / 'view', split=True)


def payload(path):
    return json.JSONDecoder().raw_decode(path.read_text().split('const data = ', 1)[1])[0]


class FiragaCellsProjectionTest(unittest.TestCase):
    def test_tokens_decode_and_preserve_raw_fields(self):
        self.assertEqual(decode_tilemap_tokens([{'control': 3, 'values': [1, 2, 3]},
          {'control': 0x4002, 'values': [7]}, {'control': 0xc003, 'values': [65534]}], 8),
          [1, 2, 3, 7, 7, 65534, 65535, 0])
        raw = catalog(); projected = project_firaga_compact_cells(raw)
        self.assertEqual(projected['sequence'], raw['sequence'])
        self.assertEqual(projected['metadata']['indexed_fields'], raw['indexed_fields'])
        for frame, original in zip(projected['maps'], raw['frames']):
            self.assertEqual(frame['tokens'], original['tokens'])
            self.assertEqual(frame['opaque_header'], original['opaque_header'])
            self.assertEqual(len(frame['cells']), frame['width'] * frame['height'])

    def test_rejects_bad_streams_geometry_and_fields(self):
        mutations = [lambda c: c['frames'][0]['tokens'][0].__setitem__('control', 0),
                     lambda c: c['frames'][0].__setitem__('width', 4),
                     lambda c: c['frames'][0].__setitem__('source_offset', 0),
                     lambda c: c['sequence'][0].__setitem__('high_flags', 4),
                     lambda c: c['indexed_fields'].__setitem__('word_ccfc', -1)]
        for mutate in mutations:
            value = copy.deepcopy(catalog()); mutate(value)
            with self.subTest(mutation=mutate), self.assertRaises(ValueError):
                project_firaga_compact_cells(value)

    def test_scoped_resource_and_standalone_support(self):
        for observed in (True, False):
            with self.subTest(observed=observed), tempfile.TemporaryDirectory(prefix='viewer-firaga-cells-scope-') as directory:
                index = fixture(Path(directory), observed=observed); parts = index.parent / (index.name + '.parts')
                self.assertNotIn('firaga_compact_cells', payload(index))
                self.assertEqual(len(payload(parts / 'interface.html')['firaga_compact_cells']['maps']), 10)
                self.assertNotIn('firaga_compact_cells', payload(parts / 'objects.html'))


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class FiragaCellsBrowserTest(unittest.TestCase):
    def test_invalid_resource_never_links_to_the_other_catalog(self):
        for invalid in ('firaga', 'observed'):
            with self.subTest(invalid=invalid), tempfile.TemporaryDirectory(prefix='viewer-firaga-cells-invalid-') as directory:
                work = Path(directory)
                index = fixture(work, invalid_firaga=invalid == 'firaga', invalid_observed=invalid == 'observed')
                browser = Chrome(work, index.parent / (index.name + '.parts/interface.html'))
                try:
                    actual = browser.evaluate('''(()=>{
                      const row=id=>[...document.querySelector('#rows').rows].find(r=>r.dataset.resourceId===id);
                      return ['graphics.firaga-compact-cells','graphics.observed-tilemap']
                        .map(id=>!!row(id).querySelector('a[href="#tilemaps"]'));
                    })()''')
                    self.assertEqual(actual, [invalid != 'firaga', invalid != 'observed'])
                finally: browser.close()

    def test_frame_order_timing_loop_and_raw_fields(self):
        with tempfile.TemporaryDirectory(prefix='viewer-firaga-cells-') as directory:
            work = Path(directory); index = fixture(work); browser = Chrome(work, index.parent / (index.name + '.parts/interface.html'))
            try:
                self.assertEqual(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change',{bubbles:true}));
                  window.pending=new Map();let serial=0;
                  window.setTimeout=(fn,delay)=>{const id=++serial;pending.set(id,{fn,delay});return id};
                  window.clearTimeout=id=>pending.delete(id);
                  const fire=()=>{const [id,item]=pending.entries().next().value;pending.delete(id);item.fn();return item.delay};
                  const row=[...q('#rows').rows].find(r=>r.dataset.resourceId==='graphics.firaga-compact-cells');
                  if(!row||!row.querySelector('a'))throw Error('resource link');row.querySelector('a').click();
                  if(!q('#tilemaps>details').open||q('#firaga-cell-controls').hidden||q('#firaga-cell-sequence').options.length!==10||
                     q('#firaga-cell-play').disabled||q('#tilemap-canvas').width!==24||q('#tilemap-canvas').height!==16)throw Error('first frame');
                  if(!q('#firaga-cell-sequence-detail').textContent.includes('second byte 0x3C')||q('#firaga-cell-encoding').hidden||
                     !q('#firaga-cell-runs').rows.length||!q('#firaga-cell-metadata').textContent.includes('indexed_fields'))throw Error('raw fields');
                  q('#firaga-cell-sequence').value='3';change(q('#firaga-cell-sequence'));
                  if(q('#tilemap-id').value!=='4'||q('#tilemap-canvas').width!==56||q('#tilemap-canvas').height!==64||
                     !q('#firaga-cell-sequence-detail').textContent.includes('high flags 0x3'))throw Error('selected frame');
                  q('#firaga-cell-rate').value='5';q('#firaga-cell-loop').checked=true;q('#firaga-cell-play').click();
                  if(fire()!==200||q('#firaga-cell-sequence').value!=='4')throw Error('play');
                  q('#firaga-cell-sequence').value='9';change(q('#firaga-cell-sequence'));
                  if(fire()!==200||q('#firaga-cell-sequence').value!=='0')throw Error('loop');
                  q('#tilemap-id').value='0';change(q('#tilemap-id'));
                  if(!q('#firaga-cell-controls').hidden||!q('#firaga-cell-stop').disabled||pending.size)throw Error('stop on leaving');
                  return 'PASS';
                })()'''), 'PASS')
                browser.evaluate("document.querySelector('#rows tr[data-resource-id=\"graphics.firaga-compact-cells\"] a').click()")
                self.assertTrue(browser.evaluate("document.querySelector('#firaga-cell-sequence').focus();document.activeElement.id==='firaga-cell-sequence'"))
                browser.click('#firaga-cell-play')
                self.assertEqual(browser.evaluate('document.activeElement.id'), 'firaga-cell-sequence')
            finally: browser.close()

    def test_missing_sequence_frame_stops_without_guessing(self):
        with tempfile.TemporaryDirectory(prefix='viewer-firaga-cells-missing-') as directory:
            work = Path(directory); index = fixture(work, observed=False, missing_frame=True)
            browser = Chrome(work, index.parent / (index.name + '.parts/interface.html'))
            try:
                self.assertTrue(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change'));
                  if(q('#tilemaps').hidden||q('#firaga-cell-controls').hidden)return false;
                  q('#firaga-cell-sequence').value='2';change(q('#firaga-cell-sequence'));
                  return q('#firaga-cell-play').disabled&&q('#firaga-cell-sequence-detail').textContent.includes('frame is unavailable')&&
                    q('#firaga-cell-status').textContent.includes('unavailable');
                })()'''))
            finally: browser.close()
