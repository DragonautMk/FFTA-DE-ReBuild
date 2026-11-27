"""Check the optional physical audio extension and preserve existing IDs."""
import copy
import hashlib
import json
import struct
import tempfile
import unittest
from pathlib import Path

from test_viewer_scoped_browser import parse_page
from view_rom_data import generate, project_audio_instruments
from viewer_physical_audio import BANK_STARTS, CGB_START, ROM_BASE, merge_physical_audio


def fixture_data():
    offsets = [position for start, end in zip(BANK_STARTS, BANK_STARTS[1:])
               for position in range(start, end, 12)]
    old_pcm = 0x160000
    tones = [{'id': i, 'source_offset': offset, 'ctype': 0,
              'word04': ROM_BASE + old_pcm, 'word08': 0xffffffff}
             for i, offset in enumerate(offsets[:429])]
    banks = []
    for start, end in zip(BANK_STARTS, BANK_STARTS[1:]):
        programs = [{'selector': i, 'tone': offsets.index(start + i * 12)}
                    for i in range(3)]
        banks.append({'source_address': ROM_BASE + start, 'programs': programs})
    waves = [{'id': 0, 'source_offset': old_pcm, 'tone_ids': list(range(429)),
              'frequency_word': 22050 * 256, 'declared_length': 4, 'loop_start': 0,
              'flags': 0x4000, 'samples': [0, 32, -32, 0]}]
    cgb = [{'id': i, 'source_offset': CGB_START + i * 16,
            'tone_ids': [], 'amplitudes': [i] * 32} for i in range(2)]
    catalog = {'tones': tones, 'banks': banks, 'map_links': [], 'pcm_waves': waves, 'cgb_waves': cgb}
    audio = {'path': 'tones.json', 'waves': waves, 'cgb_waves': cgb,
             'instruments': project_audio_instruments(catalog)}
    new_pcm = [0x170000 + i * 0x1000 for i in range(8)]
    records = bytearray()
    for i in range(70):
        kind, source = 0, ROM_BASE + old_pcm
        if i < 8:
            source = ROM_BASE + new_pcm[i]
        elif i < 14:
            kind, source = 3, ROM_BASE + CGB_START + (i - 6) * 16
        elif i in (68, 69):
            kind, source = (0x40 if i == 68 else 0x80), 0
        records.extend(struct.pack('<BBBBII', kind, 60, 0, 0, source, 0xffffffff))
    segments = [(offsets[429], 'physical-tone-records', bytes(records))]
    for offset in new_pcm:
        segments.append((offset, 'direct-pcm-waves',
                         struct.pack('<HHIII', 0, 0x4000, 22050 * 256, 0, 4) + bytes([1, 254, 3, 252, 42])))
    segments.append((CGB_START + 32, 'direct-cgb-waves', bytes([0x1e] * 96)))
    return audio, catalog, segments


def fixture(work, *, corrupt_source=False):
    audio, catalog, segments = fixture_data()
    dataset = work / 'dataset'
    dataset.mkdir()
    source = json.dumps(catalog).encode()
    physical = b''.join(chunk for _, _, chunk in segments)
    (dataset / 'tones.json').write_bytes(source)
    (dataset / 'physical.bin').write_bytes(physical)
    manifest = 'profile="europe"\nextractor_revision=359\ndecoder_revision=358\n'
    for identity, path, format_name, content in (
        ('audio.m4a-tones', 'tones.json', 'audio-tones-v1-json', source),
        ('audio.physical-tone-banks', 'physical.bin', 'physical-tone-banks-v1-bin', physical),
    ):
        manifest += (f'[[resources]]\nid="{identity}"\npath="{path}"\nformat="{format_name}"\n'
                     f'sha256="{hashlib.sha256(content).hexdigest()}"\n')
        if identity.endswith('physical-tone-banks'):
            for i, (offset, role, chunk) in enumerate(segments):
                digest = '0' * 64 if corrupt_source and i == 0 else hashlib.sha256(chunk).hexdigest()
                manifest += (f'[[resources.sources]]\noffset={offset}\nlength={len(chunk)}\n'
                             f'role="{role}"\nsha256="{digest}"\n')
    (dataset / 'manifest.toml').write_text(manifest)
    return generate(dataset, work / 'view', split=True), audio, segments


class PhysicalAudioTest(unittest.TestCase):
    def test_merge_preserves_existing_ids_and_decodes_new_sources(self):
        audio, catalog, segments = fixture_data()
        before = copy.deepcopy(audio)
        merged = merge_physical_audio(segments, audio, catalog)
        self.assertEqual(audio, before)
        self.assertEqual(merged['physical_counts'], {'new_tones': 70, 'new_pcm': 8, 'new_cgb': 6})
        self.assertEqual(merged['instruments']['tones'][:429], before['instruments']['tones'])
        self.assertEqual(sum(len(b['programs']) for b in merged['instruments']['banks']), 499)
        for original, after in zip(before['instruments']['banks'], merged['instruments']['banks']):
            self.assertEqual(after['programs'][:3], original['programs'])
        self.assertEqual(merged['waves'][1]['samples'], [1, -2, 3, -4])
        self.assertEqual(merged['waves'][1]['interpolation_endpoint'], 42)
        self.assertEqual(merged['waves'][1]['tone_ids'], [429])
        self.assertEqual(merged['cgb_waves'][2]['amplitudes'], [1, 14] * 16)
        self.assertEqual(merged['cgb_waves'][2]['tone_ids'], [437])
        self.assertEqual(merged['instruments']['map_links'], [])

    def test_rejects_incomplete_or_untrusted_inputs_without_mutation(self):
        audio, catalog, segments = fixture_data()
        before = copy.deepcopy(audio)
        cases = [segments[:-1], segments + [segments[1]],
                 [(segments[0][0], 'unknown', segments[0][2]), *segments[1:]],
                 [segments[0], (segments[1][0], segments[1][1], segments[1][2][:-1]), *segments[2:]]]
        for broken in cases:
            with self.subTest(segments=len(broken)), self.assertRaises(ValueError):
                merge_physical_audio(broken, audio, catalog)
            self.assertEqual(audio, before)

    def test_split_generation_authenticates_ranges_and_keeps_audio_lazy(self):
        for corrupt in (False, True):
            with self.subTest(corrupt=corrupt), tempfile.TemporaryDirectory(prefix='viewer-physical-audio-') as directory:
                index, old, _ = fixture(Path(directory), corrupt_source=corrupt)
                parts = index.parent / (index.name + '.parts')
                home, samples = parse_page(index), parse_page(parts / 'audio-samples.html')
                self.assertNotIn('audio', home)
                resource = next(r for r in home['resources'] if r['id'] == 'audio.physical-tone-banks')
                if corrupt:
                    self.assertIn('Preview unavailable', resource['status'])
                    self.assertEqual(len(samples['audio']['waves']), len(old['waves']))
                else:
                    self.assertIn('Available', resource['status'])
                    self.assertEqual(samples['audio']['physical_path'], 'physical.bin')
                    self.assertEqual(len(samples['audio']['waves']), 9)
                    wave = samples['audio']['waves'][1]
                    self.assertNotIn('samples', wave)
                    self.assertTrue((parts / wave['samples_asset']['href']).is_file())


if __name__ == '__main__':
    unittest.main()
