"""Verify bounded binary tilemap streams and their independent byte-grid view."""
import copy
import hashlib
import json
import shutil
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from view_rom_data import generate, project_binary_segments, project_byte_streams, project_halfword_streams
from viewer_browser_harness import Chrome


def source(offset, role, data):
    return {'offset': offset, 'length': len(data), 'role': role, 'sha256': hashlib.sha256(data).hexdigest()}


def banks():
    # 3 ascending, 2 repeated, 3 literal halfwords; the ascending run wraps.
    half = bytes([4, 2, 5, 6, 1, 2]) + bytes.fromhex('0380feff02402301030034127856bc9a')
    # Two byte tokens with unowned alignment bytes between and after them.
    first = bytes([3, 2, 7, 8, 4, 2]) + bytes.fromhex('0380fe')
    second = bytes.fromhex('0300010203')
    parts = [(0x1000, 'halfword-stream', half), (0x2000, 'byte-stream', first), (0x200a, 'byte-stream', second)]
    descriptor = (0x08002000).to_bytes(4, 'little') + bytes([0x34, 0x12])
    row = bytes([5, 0, 0, 0]) + (0x08003000).to_bytes(4, 'little') + bytes(4)
    controls = [(0x3000, 'type5-transfer-descriptor', descriptor), (0x4000, 'type5-transfer-row', row),
                (0x4010, 'type5-transfer-row', row)]
    return parts, controls


def fixture(work, *, include_direct=True, invalid_bank=False, invalid_direct=False, bad_byte=False, byte_only=False):
    dataset = work / 'dataset'; dataset.mkdir(); parts, controls = banks()
    if byte_only:
        parts = parts[1:]
    if bad_byte:
        parts[1 if not byte_only else 0] = (0x2000, 'byte-stream', bytes([3, 2, 7, 8, 9, 2]) + bytes.fromhex('0380fe'))
    manifest = 'profile="europe"\nextractor_revision=1\ndecoder_revision=1\n'
    items = [('tilemap-stream-bank', parts, invalid_bank)]
    if include_direct:
        items.append(('tilemap-direct-transfers', controls, invalid_direct))
    for name, segments, invalid in items:
        content = b''.join(chunk for _, _, chunk in segments); (dataset / (name + '.bin')).write_bytes(content)
        manifest += (f'[[resources]]\nid="graphics.{name}"\npath="{name}.bin"\n'
                     f'format="{name}-v1-bin"\nsha256="{hashlib.sha256(content).hexdigest()}"\n')
        for index, (offset, role, chunk) in enumerate(segments):
            sha = '0' * 64 if invalid and index == 0 else hashlib.sha256(chunk).hexdigest()
            manifest += (f'[[resources.sources]]\noffset={offset}\nlength={len(chunk)}\n'
                         f'role="{role}"\nsha256="{sha}"\n')
    (dataset / 'manifest.toml').write_text(manifest)
    return generate(dataset, work / 'view', split=True)


def payload(path):
    return json.JSONDecoder().raw_decode(path.read_text().split('const data = ', 1)[1])[0]


class TilemapStreamProjectionTest(unittest.TestCase):
    def test_decoding_runs_alignment_and_shared_roots(self):
        parts, controls = banks()
        content = b''.join(chunk for _, _, chunk in parts)
        sources = [source(offset, role, chunk) for offset, role, chunk in parts]
        segments = project_binary_segments(content, sources)
        half = project_halfword_streams(segments)
        control_content = b''.join(chunk for _, _, chunk in controls)
        control_sources = [source(offset, role, chunk) for offset, role, chunk in controls]
        byte = project_byte_streams(segments, project_binary_segments(control_content, control_sources))
        self.assertEqual(half[0]['cells'], [0xfffe, 0xffff, 0, 0x123, 0x123, 0x1234, 0x5678, 0x9abc])
        self.assertEqual(byte[0]['cells'], [254, 255, 0, 1, 2, 3])
        self.assertEqual(byte[0]['metadata']['encoded_extent'], 16)
        self.assertEqual(len(byte[0]['references']), 2)
        self.assertEqual((byte[0]['metadata']['origin_x'], byte[0]['metadata']['origin_y']), (7, 8))

    def test_source_and_stream_guards(self):
        parts, controls = banks(); content = b''.join(chunk for _, _, chunk in parts)
        sources = [source(offset, role, chunk) for offset, role, chunk in parts]
        for mutate in [lambda s: s[0].update(offset=True), lambda s: s[0].update(length=0),
                       lambda s: s[0].update(sha256='0' * 64), lambda s: s[1].update(offset=0x1001)]:
            bad = copy.deepcopy(sources); mutate(bad)
            with self.subTest(bad=bad), self.assertRaises(ValueError): project_binary_segments(content, bad)
        with self.assertRaises(ValueError): project_binary_segments(content + b'\x00', sources)
        with self.assertRaises(ValueError): project_halfword_streams([(0x1000, 'halfword-stream', parts[0][2][:-1])])
        invalid = bytearray(parts[0][2]); invalid[6:8] = b'\x00\x00'
        with self.assertRaises(ValueError): project_halfword_streams([(0x1000, 'halfword-stream', bytes(invalid))])
        bank = project_binary_segments(content, sources)
        missing = [entry for entry in bank if entry[0] != 0x200a]
        with self.assertRaisesRegex(ValueError, 'Incomplete byte'):
            project_byte_streams(missing, controls)
        bad_row = [(offset, role, bytes([4]) + chunk[1:] if role == 'type5-transfer-row' else chunk)
                   for offset, role, chunk in controls]
        with self.assertRaisesRegex(ValueError, 'type-5 row'): project_byte_streams(bank, bad_row)

    def test_scopes_and_partial_catalogs(self):
        for options in ({}, {'include_direct': False}, {'invalid_bank': True}, {'invalid_direct': True},
                        {'bad_byte': True}, {'byte_only': True}):
            with self.subTest(options=options), tempfile.TemporaryDirectory(prefix='viewer-stream-projection-') as directory:
                work = Path(directory); index = fixture(work, **options); parts = index.parent / 'index.html.parts'
                world = payload(parts / 'interface-maps.html')
                self.assertNotIn('byte_tilemaps', payload(index)); self.assertNotIn('byte_tilemaps', payload(parts / 'audio.html'))
                half = not (options.get('invalid_bank') or options.get('byte_only'))
                byte = not any(options.get(key) for key in ('invalid_bank', 'invalid_direct', 'bad_byte')) and options.get('include_direct', True)
                self.assertEqual('stream_bank' in world['extra_tilemaps'], half)
                self.assertEqual(bool(world['byte_tilemaps']), byte)


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class TilemapStreamBrowserTest(unittest.TestCase):
    def test_halfword_and_byte_views_and_source_links(self):
        with tempfile.TemporaryDirectory(prefix='viewer-stream-browser-') as directory:
            work = Path(directory); index = fixture(work); browser = Chrome(work, index.parent / 'index.html.parts/interface-maps.html')
            try:
                self.assertEqual(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s),row=q('[data-resource-id="graphics.tilemap-stream-bank"]');
                  const half=row.querySelector('a[href="#tilemaps"]'), byte=row.querySelector('a[href="#byte-tilemaps"]');
                  if(!half||!byte)throw Error('links');half.click();
                  if(!q('#tilemaps>details').open || !q('#tilemap-id').selectedOptions[0].textContent.includes('0x1000') ||
                     q('#tilemap-cells').querySelectorAll('button').length!==8 ||
                     !q('#tilemap-source-fields').textContent.includes('origin_x'))throw Error('halfword map');
                  byte.click();
                  if(!q('#byte-tilemaps>details').open || q('#byte-tilemap-id').options.length!==1 ||
                     q('#byte-tilemap-cells').querySelectorAll('button').length!==6 || q('#byte-tilemap-references').rows.length!==2 ||
                     !q('#byte-tilemap-source').textContent.includes('tilemap-stream-bank.bin'))throw Error('byte map');
                  const pixel=[...q('#byte-tilemap-canvas').getContext('2d').getImageData(0,0,1,1).data].join(',');
                  if(pixel!=='254,254,254,255')throw Error('grayscale');
                  q('#byte-tilemap-cells').querySelectorAll('button')[2].click();
                  if(!q('#byte-tilemap-cell-detail').textContent.includes('Cell (2, 0) · value 0 · 0x00'))throw Error('cell');
                  const direct=q('[data-resource-id="graphics.tilemap-direct-transfers"] a[href="#byte-tilemaps"]');
                  if(!direct)throw Error('direct reference');return 'PASS';
                })()'''), 'PASS')
            finally: browser.close()

    def test_partial_views_are_independent(self):
        for options in ({'include_direct': False}, {'invalid_bank': True}, {'invalid_direct': True},
                        {'bad_byte': True}, {'byte_only': True}):
            with self.subTest(options=options), tempfile.TemporaryDirectory(prefix='viewer-stream-partial-') as directory:
                work = Path(directory); index = fixture(work, **options); browser = Chrome(work, index.parent / 'index.html.parts/interface-maps.html')
                try:
                    actual = browser.evaluate('''(()=>{
                      const q=s=>document.querySelector(s),row=q('[data-resource-id="graphics.tilemap-stream-bank"]');
                      return {half:!q('#tilemaps').hidden,byte:!q('#byte-tilemaps').hidden,
                        halfLink:!!row.querySelector('a[href="#tilemaps"]'),byteLink:!!row.querySelector('a[href="#byte-tilemaps"]')};
                    })()''')
                    half = not (options.get('invalid_bank') or options.get('byte_only'))
                    byte = not any(options.get(key) for key in ('invalid_bank', 'invalid_direct', 'bad_byte')) and options.get('include_direct', True)
                    self.assertEqual(actual, {'half': half, 'byte': byte, 'halfLink': half, 'byteLink': byte})
                finally: browser.close()
