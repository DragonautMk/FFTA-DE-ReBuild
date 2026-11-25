import hashlib
import json
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from view_rom_data import generate, project_audio_instruments, project_audio_pcm_pitch, project_audio_sequences, project_equipment_sprites, project_item_links, project_observed_tilemaps, project_script_sprites


class ViewerTest(unittest.TestCase):
    def test_item_links_preserve_inactive_slots_and_reject_invalid_records(self):
        catalog = {'source_offset': 100, 'records': [{'count': 1, 'opaque': 9,
                   'slots': [{'selector': index, 'value': 255 - index} for index in range(9)]}]}
        self.assertEqual(project_item_links(catalog), catalog)
        catalog['records'][0]['count'] = 10
        with self.assertRaisesRegex(ValueError, 'record'):
            project_item_links(catalog)
        catalog['records'][0]['count'] = 1
        catalog['records'][0]['slots'][-1]['selector'] = True
        with self.assertRaisesRegex(ValueError, 'slot'):
            project_item_links(catalog)
        catalog['records'][0]['slots'][-1]['selector'] = 8
        catalog['source_offset'] = 0xfffffff0
        with self.assertRaisesRegex(ValueError, 'source'):
            project_item_links(catalog)

    def test_script_sprites_retain_conditional_witnesses_and_validate_sources(self):
        producer = {'mode': 'normal', 'sprite_id': 3, 'base': 4, 'selector': 2,
                    'unit_kind': 1, 'job_id': 7, 'metadata_source': 300,
                    'script_sources': [1000, 1004]}
        catalog = {
            'pixels': [{'source_offset': 0x898a40, 'tiles': [{'indices': [[0] * 8 for _ in range(8)]}]}],
            'objects': [{'source_offset': 0xa50020, 'objects': []}],
            'sequences': [{'source_offset': 500, 'opaque_header': 0, 'frames': [
                {'word00': 0, 'word04': 0, 'duration': 1, 'operation': 1, 'bytes_0a_13': [0] * 10}]}],
            'bindings': [{'sprite_id': 3, 'animation': 4, 'part': 0, 'index_source': 100,
                          'target': {'kind': 'shared-index-root', 'source_offset': 600, 'sprite_id': 8}}],
            'producers': [producer],
        }
        result = project_script_sprites(catalog, alternate=True)
        self.assertEqual(result['producers'], [producer])
        self.assertEqual(result['bindings'][0]['target'], catalog['bindings'][0]['target'])
        self.assertIsNone(result['bindings'][0]['sequence'])
        producer['script_sources'] = [True]
        with self.assertRaisesRegex(ValueError, 'producer'):
            project_script_sprites(catalog, alternate=True)
        producer['script_sources'] = [1000]
        producer['base'] = 65536
        with self.assertRaisesRegex(ValueError, 'producer'):
            project_script_sprites(catalog, alternate=True)
        producer['base'] = 4
        producer['mode'] = 'unknown'
        with self.assertRaisesRegex(ValueError, 'producer'):
            project_script_sprites(catalog, alternate=True)

    def test_equipment_graphics_links_and_commands(self):
        catalog = {
            'pixels': [{'source_offset': 0x898a40, 'tiles': [{'indices': [[0] * 8 for _ in range(8)]}]}],
            'objects': [{'source_offset': 0xa50020, 'objects': []}],
            'sequences': [{'source_offset': 500, 'opaque_header': 4, 'frames': [
                {'word00': 0, 'word04': 0, 'duration': 1, 'operation': 1, 'bytes_0a_13': [0] * 10},
                {'word00': 999, 'word04': 888, 'duration': 2, 'operation': 8, 'bytes_0a_13': [7] + [0] * 9},
            ]}],
            'bindings': [{'sprite_id': 3, 'animation': 4, 'part': 1, 'index_source': 100, 'sequence_source': 500},
                         {'sprite_id': 3, 'animation': 4, 'part': 0, 'index_source': 104, 'sequence_source': None}],
        }
        projected = project_equipment_sprites(catalog)
        self.assertEqual(projected['bindings'][0]['sequence'], 0)
        self.assertIsNone(projected['bindings'][1]['sequence'])
        self.assertEqual(projected['sequences'][0]['frames'][0]['block'], 0)
        self.assertEqual(projected['sequences'][0]['frames'][0]['objects'], 0)
        self.assertNotIn('block', projected['sequences'][0]['frames'][1])
        self.assertEqual(projected['sequences'][0]['frames'][1]['word00'], 999)
        catalog['sequences'][0]['frames'][0]['word00'] = 1
        with self.assertRaisesRegex(ValueError, 'graphics link'):
            project_equipment_sprites(catalog)
        catalog['sequences'][0]['frames'][0]['word00'] = 0
        catalog['bindings'][1]['sequence_source'] = 999
        with self.assertRaisesRegex(ValueError, 'binding'):
            project_equipment_sprites(catalog)

    def test_alternate_sprite_targets_keep_shared_roots_distinct(self):
        catalog = {
            'pixels': [{'source_offset': 0x898a40, 'tiles': [{'indices': [[0] * 8 for _ in range(8)]}]}],
            'objects': [{'source_offset': 0xa50020, 'objects': []}],
            'sequences': [{'source_offset': 500, 'opaque_header': 0, 'frames': [
                {'word00': 0, 'word04': 0, 'duration': 1, 'operation': 1, 'bytes_0a_13': [0] * 10},
                {'word00': 99, 'word04': 88, 'duration': 1, 'operation': 2, 'bytes_0a_13': [0] * 10}]}],
            'bindings': [
                {'sprite_id': 3, 'animation': 0, 'part': 0, 'index_source': 100,
                 'target': {'kind': 'sequence', 'source_offset': 500}},
                {'sprite_id': 3, 'animation': 4, 'part': 1, 'index_source': 104, 'target': {'kind': 'null'}},
                {'sprite_id': 4, 'animation': 0, 'part': 0, 'index_source': 108,
                 'target': {'kind': 'shared-index-root', 'source_offset': 600, 'sprite_id': 8}},
            ],
        }
        projected = project_equipment_sprites(catalog, alternate=True)
        self.assertEqual([item['sequence'] for item in projected['bindings']], [0, None, None])
        self.assertEqual(projected['bindings'][2]['target'], catalog['bindings'][2]['target'])
        self.assertNotIn('block', projected['sequences'][0]['frames'][1])
        catalog['bindings'][2]['target']['sprite_id'] = -1
        with self.assertRaisesRegex(ValueError, 'shared sprite index'):
            project_equipment_sprites(catalog, alternate=True)
        catalog['bindings'][2]['target']['sprite_id'] = 8
        catalog['bindings'][0]['target']['source_offset'] = None
        with self.assertRaisesRegex(ValueError, 'sequence target'):
            project_equipment_sprites(catalog, alternate=True)

    def test_observed_tilemaps_decode_and_reject_bad_tokens(self):
        catalog = {
            'source_offset': 100, 'width': 2, 'height': 2, 'mode': 4,
            'runs': [{'control': 0x8003, 'value': 0xfffe}, {'control': 0x4001, 'value': 12}],
            'additional_streams': [{'source_offset': 200, 'width': 2, 'height': 2, 'mode': 4,
                                    'tokens': [{'control': 2, 'values': [3, 4]},
                                               {'control': 0x4002, 'values': [5]}]}],
            'rectangles': [{'source_offset': 300, 'width': 2, 'height': 1, 'cells': [9, 10]}],
        }
        maps = project_observed_tilemaps(catalog)
        self.assertEqual(maps[0]['cells'], [0xfffe, 0xffff, 0, 12])
        self.assertEqual(maps[1]['cells'], [3, 4, 5, 5])
        self.assertEqual(maps[2]['cells'], [9, 10])
        catalog['additional_streams'][0]['tokens'][0]['values'].pop()
        with self.assertRaises(ValueError):
            project_observed_tilemaps(catalog)
        catalog['additional_streams'][0]['tokens'][0]['values'].append(4)
        catalog['rectangles'][0]['cells'].append(11)
        with self.assertRaises(ValueError):
            project_observed_tilemaps(catalog)

    def test_obj_bank_projection_and_invalid_words(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            dataset = root / 'dataset'
            dataset.mkdir()
            obj = {'banks': [{'source_offset': 100, 'words': [0x21] + [0] * 7}],
                   'relative_archive': {'banks': [{'source_offset': 200, 'words': [0] * 9}]},
                   'pointer_tables': [{'source_offset': 50, 'bank_offsets': [100, 999]}]}
            def write_obj():
                content = json.dumps(obj).encode()
                (dataset / 'obj-graphics.json').write_bytes(content)
                (dataset / 'manifest.toml').write_text(
                    'profile="test"\nextractor_revision=1\ndecoder_revision=1\n'
                    '[[resources]]\nid="graphics.obj-custom-banks"\npath="obj-graphics.json"\n'
                    'format="obj-custom-banks-v1-json"\nsha256="' + hashlib.sha256(content).hexdigest() + '"\n')
                page = generate(dataset, root / 'output').read_text()
                return json.JSONDecoder().raw_decode(page.split('const data = ', 1)[1])[0]
            payload = write_obj()
            self.assertEqual(payload['obj_graphics']['banks'], obj['banks'])
            self.assertEqual(payload['obj_graphics']['archive_banks'], obj['relative_archive']['banks'])
            self.assertEqual(payload['obj_graphics']['selectors'], obj['pointer_tables'])
            obj['relative_archive']['banks'][0]['words'][0] = -1
            payload = write_obj()
            self.assertIsNone(payload['obj_graphics'])
            self.assertIn('Invalid OBJ graphics bank', payload['resources'][0]['status'])

    def test_name_keyboard_pages_and_invalid_widths(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            dataset = root / 'dataset'
            dataset.mkdir()
            cells = [[0x8007, 0x4073] + [0] * 8 for _ in range(6)]
            keyboard = {'row_widths': [[2] * 6 for _ in range(3)],
                        'cells': [[cell for row in cells for cell in row] for _ in range(3)],
                        'programs': [{'source_offset': 100 + page} for page in range(3)]}
            def write_keyboard():
                content = json.dumps(keyboard).encode()
                (dataset / 'name-keyboard.json').write_bytes(content)
                (dataset / 'manifest.toml').write_text(
                    'profile="test"\nextractor_revision=1\ndecoder_revision=1\n'
                    '[[resources]]\nid="text.name-keyboard"\npath="name-keyboard.json"\n'
                    'format="name-keyboard-v1-json"\nsha256="' + hashlib.sha256(content).hexdigest() + '"\n')
                page = generate(dataset, root / 'output').read_text()
                return json.JSONDecoder().raw_decode(page.split('const data = ', 1)[1])[0]
            payload = write_keyboard()
            self.assertEqual(payload['name_keyboard']['pages'][2],
                             {'row_widths': [2] * 6, 'cells': keyboard['cells'][2], 'source_offset': 102})
            keyboard['row_widths'][2][0] = 1
            payload = write_keyboard()
            self.assertIsNone(payload['name_keyboard'])
            self.assertIn('Keyboard row and width mismatch', payload['resources'][0]['status'])

    def test_ui_graphics_and_palette_projection(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            dataset = root / 'dataset'
            dataset.mkdir()
            graphics = {'banks': [{'source_offset': 100, 'words': [0x21] + [0] * 7 + [42]}],
                        'selectors': [{'id': 'test', 'pointers': [{'bank_offset': 100}, {'bank_offset': 999}]}]}
            palettes = {'banks': [{'source_offset': 200, 'colors': [0, 31, 992] + [0] * 13}]}
            manifest = 'profile="test"\nextractor_revision=1\ndecoder_revision=1\n'
            for resource_id, name, fmt, value in [
                ('graphics.ui-custom-banks', 'ui-graphics.json', 'ui-custom-banks-v1-json', graphics),
                ('graphics.ui-raw-palettes', 'ui-palettes.json', 'ui-raw-palettes-v1-json', palettes),
            ]:
                content = json.dumps(value).encode()
                (dataset / name).write_bytes(content)
                manifest += (f'[[resources]]\nid="{resource_id}"\npath="{name}"\n'
                             f'format="{fmt}"\nsha256="{hashlib.sha256(content).hexdigest()}"\n')
            (dataset / 'manifest.toml').write_text(manifest)
            page = generate(dataset, root / 'output').read_text()
            payload = json.JSONDecoder().raw_decode(page.split('const data = ', 1)[1])[0]
            self.assertEqual(payload['ui_graphics']['banks'][0]['words'], graphics['banks'][0]['words'])
            self.assertEqual(payload['ui_graphics']['selectors'][0]['bank_offsets'], [100, 999])
            self.assertEqual(payload['ui_palettes']['banks'], palettes['banks'])

    def test_instrument_links_and_missing_tones(self):
        catalog = {
            'banks': [{'source_address': 0x08001000, 'programs': [{'selector': 7, 'tone': 0}, {'selector': 8, 'tone': 1}]}],
            'tones': [{'id': 0, 'ctype': 0, 'word04': 0x08002000},
                      {'id': 1, 'ctype': 0x40, 'word04': 0x08003000},
                      {'id': 2, 'ctype': 0, 'word04': 0x08004000}],
            'map_links': [{'parent_tone': 1, 'key': 64, 'tone': 2}],
        }
        projected = project_audio_instruments(catalog)
        self.assertEqual(projected['banks'][0]['programs'][0], {'selector': 7, 'tone': 0})
        self.assertEqual(projected['map_links'][0], {'parent_tone': 1, 'key': 64, 'tone': 2})
        catalog['tones'][0]['word08'] = 0x80408080
        self.assertEqual(project_audio_instruments(catalog)['tones'][0]['word08'], 0x80408080)
        catalog['tones'][0]['word08'] = -1
        with self.assertRaisesRegex(ValueError, 'envelope word'):
            project_audio_instruments(catalog)
        catalog['tones'][0]['word08'] = 0x80408080
        self.assertIsNone(project_audio_instruments({}))
        catalog['map_links'][0]['tone'] = 99
        with self.assertRaises(ValueError):
            project_audio_instruments(catalog)

    def test_pcm_pitch_table_projection_and_guards(self):
        catalog = {'pcm_keys': {'entries': [{'base': i % 12, 'shift': i % 16} for i in range(180)]},
                   'pcm_bases': {'entries': [i * 100 for i in range(12)]}}
        projected = project_audio_pcm_pitch(catalog)
        self.assertEqual(projected['keys'], catalog['pcm_keys']['entries'])
        self.assertEqual(projected['bases'], catalog['pcm_bases']['entries'])
        catalog['pcm_keys']['entries'][179]['base'] = 12
        with self.assertRaisesRegex(ValueError, 'pitch key'):
            project_audio_pcm_pitch(catalog)
        catalog['pcm_keys']['entries'][179]['base'] = 11
        catalog['pcm_bases']['entries'][0] = 0x100000000
        with self.assertRaisesRegex(ValueError, 'pitch tables'):
            project_audio_pcm_pitch(catalog)

    def test_world_road_samples_and_invalid_count(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            dataset = root / 'dataset'
            dataset.mkdir()
            roads = {'paths': [{'index': 0, 'point_count': 2, 'encoded_pointer': 123,
                                'points': [{'x': -1, 'y': 2}, {'x': 15, 'y': 7}]}]}
            def write_roads():
                content = json.dumps(roads).encode()
                (dataset / 'world-roads.json').write_bytes(content)
                (dataset / 'manifest.toml').write_text(
                    'profile="test"\nextractor_revision=1\ndecoder_revision=1\n'
                    '[[resources]]\nid="gameplay.world-roads"\npath="world-roads.json"\n'
                    'format="world-roads-v1-json"\nsha256="' + hashlib.sha256(content).hexdigest() + '"\n')
            write_roads()
            page = generate(dataset, root / 'output').read_text()
            payload = json.JSONDecoder().raw_decode(page.split('const data = ', 1)[1])[0]
            self.assertEqual(payload['world_roads']['paths'][0]['points'], roads['paths'][0]['points'])
            roads['paths'][0]['point_count'] = 3
            write_roads()
            page = generate(dataset, root / 'output').read_text()
            payload = json.JSONDecoder().raw_decode(page.split('const data = ', 1)[1])[0]
            self.assertIsNone(payload['world_roads'])
            self.assertIn('Invalid world road', payload['resources'][0]['status'])

    def test_background_tile_links_and_invalid_words(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            dataset = root / 'dataset'
            dataset.mkdir()
            graphic = {'descriptor_graphics': [0, 0], 'graphics': [
                {'relative_source': 1234, 'destination_offset': 64, 'words': [0x21] + [0] * 7}]}
            def write_graphic():
                content = json.dumps(graphic).encode()
                (dataset / 'bg-main-graphics.json').write_bytes(content)
                (dataset / 'manifest.toml').write_text(
                    'profile="test"\nextractor_revision=1\ndecoder_revision=1\n'
                    '[[resources]]\nid="graphics.bg-main-graphics"\npath="bg-main-graphics.json"\n'
                    'format="bg-main-graphics-v1-json"\nsha256="' + hashlib.sha256(content).hexdigest() + '"\n')
            write_graphic()
            page = generate(dataset, root / 'output').read_text()
            payload = json.JSONDecoder().raw_decode(page.split('const data = ', 1)[1])[0]
            self.assertEqual(payload['bg_graphics']['descriptor_graphics'], [0, 0])
            self.assertEqual(payload['bg_graphics']['graphics'][0], graphic['graphics'][0])
            graphic['graphics'][0]['words'].pop()
            write_graphic()
            page = generate(dataset, root / 'output').read_text()
            payload = json.JSONDecoder().raw_decode(page.split('const data = ', 1)[1])[0]
            self.assertIsNone(payload['bg_graphics'])
            self.assertIn('Invalid background tile words', payload['resources'][0]['status'])

    def test_sequence_successors_and_invalid_branch(self):
        codes = [
            (100, {'kind': 'parameter', 'command': 187, 'value': 60}),
            (102, {'kind': 'note', 'clock': 2, 'key': 60, 'velocity': 100, 'extension': None}),
            (105, {'kind': 'wait', 'clock': 1}),
            (106, {'kind': 'pattern', 'target': 5}),
            (111, {'kind': 'end'}),
            (112, {'kind': 'note', 'clock': 2, 'key': 64, 'velocity': 90, 'extension': None}),
            (115, {'kind': 'return'}),
        ]
        catalog = {'driver': {'clocks': list(range(49))},
                   'instructions': [{'id': i, 'source_offset': at, 'explicit': True, 'code': code}
                                    for i, (at, code) in enumerate(codes)],
                   'tracks': [{'id': 0, 'directory_header': 0, 'track_index': 0, 'entry': 0}]}
        projected = project_audio_sequences(catalog)
        self.assertEqual(projected['instructions'][3]['next'], 4)
        self.assertEqual(projected['instructions'][3]['code'], ['pattern', 5])
        self.assertEqual(projected['instructions'][5]['next'], 6)
        self.assertEqual(projected['instructions'][1]['code'], ['note', 2, 60, 100, None])
        catalog['instructions'][3]['code']['target'] = 99
        with self.assertRaises(ValueError):
            project_audio_sequences(catalog)
        catalog['instructions'][3]['code']['target'] = 5
        catalog['tracks'].append({'id': 1, 'directory_header': 0, 'track_index': 0, 'entry': 0})
        with self.assertRaises(ValueError):
            project_audio_sequences(catalog)
        catalog['tracks'].pop()
        sprite = {'pixels': [{'source_offset': 0x898a40, 'tiles': [{'indices': [[0] * 8 for _ in range(8)]}]}],
                  'sequences': [{'source_offset': 500, 'frames': [{'pixel_offset': 0, 'object_offset': 0, 'duration': 1}]}],
                  'objects': [{'source_offset': 0xa50020, 'objects': []}],
                  'bindings': [{'sprite_id': 0, 'sequence_source': 500}]}
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            dataset = root / 'dataset'
            dataset.mkdir()
            manifest = 'profile="test"\nextractor_revision=1\ndecoder_revision=1\n'
            for resource_id, name, fmt, value in [
                ('audio.m4a-sequences', 'audio-sequences.json', 'audio-sequences-v1-json', catalog),
                ('graphics.unit-obj-initial-pose', 'unit-obj-initial-pose.json', 'unit-obj-initial-pose-v1-json', sprite),
            ]:
                content = json.dumps(value).encode()
                (dataset / name).write_bytes(content)
                manifest += (f'[[resources]]\nid="{resource_id}"\npath="{name}"\n'
                             f'format="{fmt}"\nsha256="{hashlib.sha256(content).hexdigest()}"\n')
            (dataset / 'manifest.toml').write_text(manifest)
            page = generate(dataset, root / 'output').read_text()
            payload = json.JSONDecoder().raw_decode(page.split('const data = ', 1)[1])[0]
            self.assertEqual(payload['sequences']['instructions'][3]['code'], ['pattern', 5])
            self.assertEqual(payload['sprite_pixels']['sprites'][0]['frames'][0]['duration'], 1)

    def test_manifest_errors_and_palette(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            dataset = root / 'dataset'
            dataset.mkdir()
            palette = json.dumps({'entries': [{'banks': [[31, 992, 31744, 32799] + [0] * 12]}]}).encode()
            (dataset / 'palette.json').write_bytes(palette)
            (dataset / 'bad.json').write_bytes(b'bad')
            dimensions = ('[[entries]]\nwidth=8\nheight=16\n' * 12).encode()
            (dataset / 'obj-dimensions.toml').write_bytes(dimensions)
            checksum = hashlib.sha256(palette).hexdigest()
            dimensions_checksum = hashlib.sha256(dimensions).hexdigest()
            (dataset / 'manifest.toml').write_text(
                'profile="test"\nextractor_revision=2\ndecoder_revision=1\n'
                '[[resources]]\nid="graphics.bg-base-palettes"\npath="palette.json"\n'
                'format="bg-base-palettes-v2-json"\nsha256="' + checksum + '"\n'
                '[[resources]]\nid="missing"\npath="missing.json"\nformat="future"\nsha256="x"\n'
                '[[resources]]\nid="bad"\npath="bad.json"\nformat="future"\nsha256="x"\n'
                '[[resources]]\nid="unknown"\npath="palette.json"\nformat="future"\nsha256="' + checksum + '"\n'
                '[[resources]]\nid="graphics.obj-dimensions"\npath="obj-dimensions.toml"\n'
                'format="obj-dimensions-v1"\nsha256="' + dimensions_checksum + '"\n'
            )
            page = generate(dataset, root / 'output').read_text()
            payload = json.JSONDecoder().raw_decode(page.split('const data = ', 1)[1])[0]
            self.assertEqual([row['status'] for row in payload['resources']],
                             ['Available', 'File missing', 'SHA-256 mismatch', 'Preview unavailable', 'Available'])
            self.assertEqual(payload['palettes']['entries'][0]['banks'][0][:4], [31, 992, 31744, 32799])
            self.assertEqual(payload['dimensions']['entries'][11], {'width': 8, 'height': 16})
            self.assertEqual(hashlib.sha256((dataset / 'palette.json').read_bytes()).hexdigest(), checksum)
            with self.assertRaises(ValueError):
                generate(dataset, dataset / 'output')

    def test_script_text_preserves_glyphs_and_command_bytes(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            dataset = root / 'dataset'
            dataset.mkdir()
            archive = {'languages': [{'id': 2, 'banks': [{'source_offset': 100, 'programs': [
                {'source_offset': 104, 'header': 3, 'units': [
                    {'kind': 'glyph', 'index': 7}, {'kind': 'command', 'opcode': 83, 'operands': [0, 9]},
                    {'kind': 'glyph', 'index': 42}]}]}]}]}
            glyph = {'index': 7, 'width': 8, 'tile_words': [1] + [0] * 31}
            unused = {'index': 8, 'width': 3, 'tile_words': [0] * 32}
            files = [('text.script-archives', 'script-text.json', 'script-text-v1-json', archive),
                     ('font.complete-bank', 'font-bank.json', 'font-bank-v1-json', {'glyphs': [glyph, unused]})]
            manifest = 'profile="test"\nextractor_revision=1\ndecoder_revision=1\n'
            for resource_id, name, fmt, value in files:
                content = json.dumps(value).encode()
                (dataset / name).write_bytes(content)
                manifest += (f'[[resources]]\nid="{resource_id}"\npath="{name}"\n'
                             f'format="{fmt}"\nsha256="{hashlib.sha256(content).hexdigest()}"\n')
            (dataset / 'manifest.toml').write_text(manifest)
            page = generate(dataset, root / 'output').read_text()
            payload = json.JSONDecoder().raw_decode(page.split('const data = ', 1)[1])[0]
            preview = payload['script_texts']
            self.assertEqual(preview['languages'][0]['banks'][0]['programs'][0],
                             {'source_offset': 104, 'header': 3, 'units': [7, [83, 0, 9], 42]})
            self.assertEqual(preview['glyphs'], [glyph])
            self.assertEqual(payload['font'], {'glyphs': [glyph, unused], 'path': 'font-bank.json'})
            self.assertEqual([row['status'] for row in payload['resources']],
                             ['Available', 'Available for glyph atlas and script text'])
            self.assertEqual(json.loads((dataset / 'script-text.json').read_text()), archive)

    def test_pointer_catalog_preserves_aliases_and_rejects_bad_indices(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            dataset = root / 'dataset'
            dataset.mkdir()
            glyph = {'index': 7, 'width': 8, 'tile_words': [1] + [0] * 31}
            catalog = {
                'glyphs': [glyph],
                'strings': [{'source_offset': 104, 'encoding': 'extended', 'units': [
                    {'kind': 'glyph', 'index': 7}, {'kind': 'control_byte', 'value': 0},
                    {'kind': 'glyph', 'index': 42}]}],
                'pointer_catalogs': [{'reader': 123, 'languages': [{'source_offset': 100, 'entries': [0, 0]}]}],
                'archive_catalogs': [{'reader': 456, 'languages': [{'source_offset': 200, 'offsets': [4, 65535],
                    'programs': [{'source_offset': 204, 'header': 3, 'units': [
                        {'kind': 'glyph', 'index': 7}, {'kind': 'command', 'opcode': 83, 'operands': [0, 9]}]}]}]}],
            }
            def write_catalog():
                content = json.dumps(catalog).encode()
                (dataset / 'text-catalogs.json').write_bytes(content)
                (dataset / 'manifest.toml').write_text(
                    'profile="test"\nextractor_revision=1\ndecoder_revision=1\n'
                    '[[resources]]\nid="text.catalogs"\npath="text-catalogs.json"\n'
                    'format="text-catalogs-v1-json"\nsha256="' + hashlib.sha256(content).hexdigest() + '"\n')
            write_catalog()
            page = generate(dataset, root / 'output').read_text()
            payload = json.JSONDecoder().raw_decode(page.split('const data = ', 1)[1])[0]
            pointer = payload['pointer_texts']
            self.assertEqual(pointer['catalogs'][0]['languages'][0]['entries'], [0, 0])
            self.assertEqual(pointer['strings'][0]['units'], [7, [0], 42])
            self.assertEqual(pointer['glyphs'], [glyph])
            relative = payload['relative_texts']
            self.assertEqual(relative['catalogs'][0]['languages'][0]['programs'][0]['units'], [7, [83, 0, 9]])
            self.assertEqual(relative['glyphs'], [glyph])
            catalog['archive_catalogs'][0]['languages'][0]['offsets'][-1] = 0
            write_catalog()
            page = generate(dataset, root / 'output').read_text()
            payload = json.JSONDecoder().raw_decode(page.split('const data = ', 1)[1])[0]
            self.assertIsNotNone(payload['pointer_texts'])
            self.assertIsNone(payload['relative_texts'])
            self.assertIn('invalid relative archives', payload['resources'][0]['status'])
            catalog['pointer_catalogs'][0]['languages'][0]['entries'][1] = 1
            write_catalog()
            page = generate(dataset, root / 'output').read_text()
            payload = json.JSONDecoder().raw_decode(page.split('const data = ', 1)[1])[0]
            self.assertIsNone(payload['pointer_texts'])
            self.assertIn('Invalid pointer text index', payload['resources'][0]['status'])


if __name__ == '__main__':
    unittest.main()
