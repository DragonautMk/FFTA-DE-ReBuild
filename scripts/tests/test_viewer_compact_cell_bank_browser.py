"""Exercise compact-cell frame decoding, animation and manual graphics."""
import copy
import hashlib
import json
import shutil
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from view_rom_data import decode_compact_cell_record, generate, project_compact_cell_bank
from viewer_browser_harness import Chrome


def record(flags, frames, sequence):
    prefix = len(frames) * 2; offsets = []; position = prefix
    for value in frames:
        offsets.append(position); position += len(value)
    directory = b''.join(value.to_bytes(2, 'little') for value in offsets) + b''.join(frames)
    split = 4 + len(directory)
    assert len(sequence) % 2 == 0
    return flags.to_bytes(2, 'little') + split.to_bytes(2, 'little') + directory + (len(sequence) // 2).to_bytes(2, 'little') + bytes(sequence)


def segments():
    half = record(12, [bytes.fromhex('020101020280ffff'), bytes.fromhex('02010304020000000004')],
                  [0x40, 7, 0x81, 8, 0xc0, 9])
    byte = record(14, [bytes.fromhex('030105060380fe00'), bytes.fromhex('03010708030001020300')],
                  [0, 11, 1, 12])
    return [(0x1000, 'compact-record', half), (0x2000, 'compact-record', byte),
            (0x3000, 'compact-record', bytes([12, 0])), (0x4000, 'constructor-literal', bytes(4))]


def fixture(work, *, bad_hash=False, missing=False):
    dataset = work / 'dataset'; dataset.mkdir(); parts = segments(); content = b''.join(chunk for _, _, chunk in parts)
    if not missing: (dataset / 'compact-cell-bank.bin').write_bytes(content)
    manifest = ('profile="europe"\nextractor_revision=1\ndecoder_revision=1\n'
                '[[resources]]\nid="graphics.compact-cell-bank"\npath="compact-cell-bank.bin"\n'
                f'format="compact-cell-bank-v1-bin"\nsha256="{hashlib.sha256(content).hexdigest()}"\n')
    for index, (offset, role, chunk) in enumerate(parts):
        digest = '0' * 64 if bad_hash and index == 0 else hashlib.sha256(chunk).hexdigest()
        manifest += (f'[[resources.sources]]\noffset={offset}\nlength={len(chunk)}\n'
                     f'role="{role}"\nsha256="{digest}"\n')
    resources = [('obj-custom-banks', {'banks': [{'source_offset': 0x5000, 'words': [0x11111111] * 8}]}),
                 ('ui-raw-palettes', {'banks': [{'source_offset': 0x6000, 'colors': [0, 31] + [0] * 14}]})]
    for name, value in resources:
        data = json.dumps(value).encode(); (dataset / (name + '.json')).write_bytes(data)
        manifest += (f'[[resources]]\nid="graphics.{name}"\npath="{name}.json"\n'
                     f'format="{name}-v1-json"\nsha256="{hashlib.sha256(data).hexdigest()}"\n')
    (dataset / 'manifest.toml').write_text(manifest)
    return generate(dataset, work / 'view', split=True)


def payload(path):
    return json.JSONDecoder().raw_decode(path.read_text().split('const data = ', 1)[1])[0]


class CompactCellProjectionTest(unittest.TestCase):
    def test_halfword_and_byte_runs_and_sequence_fields(self):
        half, byte = segments()[:2]
        a = decode_compact_cell_record(half[0], half[2]); b = decode_compact_cell_record(byte[0], byte[2])
        self.assertEqual(a['frames'][0]['cells'], [65535, 0]); self.assertEqual(a['frames'][1]['cells'], [0, 1024])
        self.assertEqual(b['frames'][0]['cells'], [254, 255, 0]); self.assertEqual(b['frames'][1]['cells'], [1, 2, 3])
        self.assertEqual(a['sequence'][1], {'frame_index': 1, 'high_flags': 2, 'second_byte': 8})
        self.assertEqual(b['frames'][0]['placement'], [5, 6])
        for bad in (half[2][:-1], bytes([13, 0]) + half[2][2:], half[2][:4] + bytes([3, 0]) + half[2][6:]):
            with self.assertRaises(ValueError): decode_compact_cell_record(0x1000, bad)
        parts = segments(); data = b''.join(chunk for _, _, chunk in parts)
        sources = [{'offset': off, 'role': role, 'length': len(chunk), 'sha256': hashlib.sha256(chunk).hexdigest()}
                   for off, role, chunk in parts]
        result = project_compact_cell_bank(data, sources)
        self.assertEqual(len(result['records']), 2); self.assertEqual(result['errors'][0]['source_offset'], 0x3000)
        bad = copy.deepcopy(sources); bad[0]['sha256'] = '0' * 64
        with self.assertRaises(ValueError): project_compact_cell_bank(data, bad)

    def test_scoped_and_missing_or_corrupt_sources(self):
        for options in ({}, {'bad_hash': True}, {'missing': True}):
            with self.subTest(options=options), tempfile.TemporaryDirectory(prefix='viewer-compact-bank-') as directory:
                index = fixture(Path(directory), **options); parts = index.parent / 'index.html.parts'
                home, world, audio = payload(index), payload(parts / 'interface-maps.html'), payload(parts / 'audio.html')
                self.assertNotIn('compact_cell_bank', home); self.assertNotIn('compact_cell_bank', audio)
                self.assertEqual(bool(world['compact_cell_bank']), not options)
                if not options: self.assertIn('1 invalid records', world['resources'][0]['status'])


@unittest.skipUnless(shutil.which('google-chrome'), 'Google Chrome is unavailable')
class CompactCellBrowserTest(unittest.TestCase):
    def test_playback_focus_palette_and_persistent_controls(self):
        with tempfile.TemporaryDirectory(prefix='viewer-compact-bank-browser-') as directory:
            work = Path(directory); index = fixture(work); browser = Chrome(work, index.parent / 'index.html.parts/interface-maps.html')
            try:
                self.assertEqual(browser.evaluate('''(()=>{
                  const q=s=>document.querySelector(s),change=e=>e.dispatchEvent(new Event('change')),
                    pixel=()=>[...q('#compact-cell-canvas').getContext('2d').getImageData(0,0,1,1).data];
                  q('[data-resource-id="graphics.compact-cell-bank"] a[href="#compact-cell-bank"]').click();
                  if(!q('#compact-cell-bank>details').open||q('#compact-cell-frame').options.length!==2||
                     pixel().join(',')!=='255,255,255,255'||!q('#compact-cell-errors').textContent.includes('0x3000'))throw Error('initial');
                  q('#compact-cell-sequence').value='1';change(q('#compact-cell-sequence'));
                  if(q('#compact-cell-frame').value!=='1'||!q('#compact-cell-sequence-detail').textContent.includes('high flags 0x2'))throw Error('sequence');
                  q('#compact-cell-graphics').value='0';change(q('#compact-cell-graphics'));
                  q('#compact-cell-palette').value='0';change(q('#compact-cell-palette'));
                  if(pixel().join(',')!=='255,0,0,255')throw Error('manual palette');
                  window.compactTimers=new Map();let id=0;
                  window.setTimeout=(fn,delay)=>{const n=++id;compactTimers.set(n,{fn,delay});return n};
                  window.clearTimeout=n=>compactTimers.delete(n);
                  const fire=()=>{const [n,t]=compactTimers.entries().next().value;compactTimers.delete(n);t.fn();return t.delay};
                  q('#compact-cell-rate').value='20';q('#compact-cell-loop').checked=true;
                  q('#compact-cell-sequence').focus();const event=new PointerEvent('pointerdown',{isPrimary:true,button:0,cancelable:true,bubbles:true});
                  q('#compact-cell-play').dispatchEvent(event);if(!event.defaultPrevented||document.activeElement!==q('#compact-cell-sequence'))throw Error('focus');
                  q('#compact-cell-play').click();if(fire()!==50||q('#compact-cell-frame').value!=='0')throw Error('advance');
                  q('#compact-cell-record').value='1';change(q('#compact-cell-record'));
                  if(!q('#compact-cell-loop').checked||q('#compact-cell-rate').value!=='20'||q('#compact-cell-stop').disabled||
                     q('#compact-cell-graphics').value!=='0'||q('#compact-cell-palette').value!=='0')throw Error('persistent controls');
                  fire();if(q('#compact-cell-frame').value!=='0'||!q('#compact-cell-source').textContent.includes('byte cells'))throw Error('record loop');
                  q('#compact-cell-stop').click();if(compactTimers.size)throw Error('stop');
                  q('#compact-cell-graphics').value='none';change(q('#compact-cell-graphics'));
                  if(pixel().join(',')!=='254,254,254,255')throw Error('byte grid');
                  return 'PASS';
                })()'''), 'PASS')
            finally: browser.close()

    def test_invalid_or_missing_bank_is_independent(self):
        for options in ({'bad_hash': True}, {'missing': True}):
            with self.subTest(options=options), tempfile.TemporaryDirectory(prefix='viewer-compact-bank-partial-') as directory:
                work = Path(directory); index = fixture(work, **options); browser = Chrome(work, index.parent / 'index.html.parts/interface-maps.html')
                try: self.assertTrue(browser.evaluate("document.querySelector('#compact-cell-bank').hidden && !document.querySelector('#obj-tiles').hidden"))
                finally: browser.close()
