"""ROM-free checks for atlas accounting and classification boundaries."""
import random
import sqlite3
import contextlib
import hashlib
import io
import json
from pathlib import Path
import tempfile
from types import SimpleNamespace
import unittest
from unittest import mock

import rom_coverage as coverage


def points(ranges):
    return {i for a, b in ranges for i in range(a, b)}


class CoverageAccountingTests(unittest.TestCase):
    def test_interval_accounting_matches_independent_byte_sets(self):
        rng = random.Random(932)
        for _ in range(250):
            def sample():
                return [[a, rng.randrange(a + 1, 65)]
                        for a in (rng.randrange(64) for _ in range(rng.randrange(20)))]
            a, b = sample(), sample()
            self.assertEqual(points(coverage.union(a)), points(a))
            self.assertEqual(coverage.length(a), len(points(a)))
            self.assertEqual(points(coverage.intersection(a, b)), points(a) & points(b))
            unknown = coverage.complement(a + b, 64)
            self.assertEqual(points(unknown), set(range(64)) - points(a + b))
            self.assertEqual(coverage.length(a + b) + coverage.length(unknown), 64)

    def test_distinct_resource_owners_not_duplicate_ranges(self):
        owners = {'font': [[0, 20], [0, 20], [5, 15]],
                  'text': [[10, 30]], 'ui': [[15, 25], [25, 32]]}
        shared = {i for i in range(40) if sum(i in points(s) for s in owners.values()) >= 2}
        self.assertEqual(points(coverage.multiple_owners(owners)), shared)
        self.assertEqual(coverage.multiple_owners({'one': [[0, 20], [0, 20]]}), [])

    def test_endpoints_are_exclusive_and_adjacent_ranges_do_not_overlap(self):
        self.assertEqual(coverage.intersection([[0, 4]], [[4, 8]]), [])
        self.assertEqual(coverage.union([[0, 4], [4, 8]]), [[0, 8]])
        self.assertEqual(coverage.complement([[0, 4], [8, 12]], 12), [[4, 8]])
        self.assertEqual(coverage.multiple_owners({'a': [[0, 4]], 'b': [[4, 8]]}), [])

    def test_bad_boundaries_are_rejected_not_clipped_or_inflated(self):
        for span in ((-1, 4), (4, 4), (5, 4), (0, 65)):
            with self.assertRaises(ValueError):
                coverage.checked_span(*span, 64)
        with self.assertRaises(ValueError):
            coverage.complement([[60, 66]], 64)


class CoverageRefreshTests(unittest.TestCase):
    def test_evidence_revision_does_not_freeze_dataset_but_rom_must_match(self):
        e = {'baseline': ['rom', 61, 60]}
        coverage.check_baseline({'extractor_revision': 61, 'decoder_revision': 60}, e, 'rom')
        coverage.check_baseline({'extractor_revision': 65, 'decoder_revision': 64}, e, 'rom')
        coverage.check_baseline({'extractor_revision': 37, 'decoder_revision': 36}, e, 'rom')
        with self.assertRaisesRegex(ValueError, 'ROM differs'):
            coverage.check_baseline({'extractor_revision': 61, 'decoder_revision': 60}, e, 'other')

    def test_source_only_ranges_require_real_hashes_and_exact_extents(self):
        rom = bytes(range(32))
        extra = dict(id='test', ranges=[[4, 12]], sources=[dict(
            offset=4, length=8, sha256=hashlib.sha256(rom[4:12]).hexdigest())])
        self.assertEqual(coverage.source_only_spans(extra, rom), [[4, 12]])
        with self.assertRaisesRegex(ValueError, 'authentication'):
            coverage.source_only_spans(extra, bytes(32))
        extra['ranges'] = [[4, 13]]
        with self.assertRaisesRegex(ValueError, 'ranges differ'):
            coverage.source_only_spans(extra, rom)

    def test_portable_seeds_bind_rom_list_and_translation_without_nm(self):
        rom = bytes(32)
        with tempfile.TemporaryDirectory() as tmp:
            p = Path(tmp) / 'seeds.json'
            data = dict(schema=1, rom_sha256=hashlib.sha256(rom).hexdigest(),
                        translation_sha256='retained', entries=['08000000', '08000005'],
                        seeds_sha256=hashlib.sha256(b'08000000\n08000005\n').hexdigest(),
                        excluded_non_rom_entries=3)
            def publish():
                p.write_text(json.dumps(data))
                return dict(code_seeds_sha256=coverage.sha(p), translation_sha256='retained')
            e = publish()
            self.assertEqual(coverage.retained_seeds(p, e, rom)[0], [0x08000000, 0x08000005])
            with self.assertRaisesRegex(ValueError, 'ROM identity'):
                coverage.retained_seeds(p, e, bytes([1]) * 32)
            data['entries'] = ['08000000', '08000000']
            with self.assertRaisesRegex(ValueError, 'invalid retained'):
                coverage.retained_seeds(p, publish(), rom)
            p.write_text('altered')
            with self.assertRaisesRegex(ValueError, 'differs from reviewed'):
                coverage.retained_seeds(p, e, rom)

    def test_default_cache_prepares_once_and_reuses_without_extractor(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            rom = root / 'synthetic.bin'; rom.write_bytes(b'synthetic fixture')
            e = root / 'evidence.json'
            e.write_text(json.dumps(dict(baseline=[coverage.sha(rom), 61, 60])))
            args = SimpleNamespace(dataset=None, dataset_cache=str(root / 'cache'),
                                   rom=str(rom), evidence=str(e), extractor='recomp')
            def extract(command, check):
                self.assertTrue(check)
                self.assertEqual(command[:3], ['recomp', 'rom-data', 'extract'])
                dest = Path(command[-1]); dest.mkdir()
                (dest / 'manifest.toml').write_text('extractor_revision=65\ndecoder_revision=64\n')
            with mock.patch.object(coverage, 'extractor_generation', return_value=(65,64)), \
                 mock.patch.object(coverage.subprocess, 'run', side_effect=extract) as run:
                coverage.prepare_dataset(args)
                self.assertEqual(Path(args.dataset).name, 'e65-d64')
                self.assertEqual(run.call_count, 1)
                args.dataset = None
                coverage.prepare_dataset(args)
                self.assertEqual(run.call_count, 1)
                (Path(args.dataset) / 'manifest.toml').write_text('extractor_revision=61\ndecoder_revision=60\n')
                args.dataset = None
                with self.assertRaisesRegex(ValueError, 'differs from built extractor'):
                    coverage.prepare_dataset(args)

    def test_failed_validation_preserves_existing_report_and_repeat_is_explicit(self):
        with tempfile.TemporaryDirectory() as tmp:
            args = SimpleNamespace(dataset='already provided', output=tmp)
            report = dict(generation='e65-d64', generated_at='first', reviewed_generation='e61-d60',
                          evidence_date='2026-10-25', pending_review=[], historical_dataset=False,
                          totals=dict(runtime_families=60), code=dict(analyzer_executable_sha256='first'))
            with mock.patch.object(coverage, 'build', return_value=report), contextlib.redirect_stdout(io.StringIO()):
                self.assertEqual(coverage.generate(args), 0)
            before = {p.name: p.read_bytes() for p in Path(tmp).iterdir()}
            with mock.patch.object(coverage, 'build', side_effect=ValueError('bad data')):
                with self.assertRaises(ValueError):
                    coverage.generate(args)
            self.assertEqual(before, {p.name: p.read_bytes() for p in Path(tmp).iterdir()})
            output = io.StringIO()
            report['generated_at'] = 'second'
            report['code']['analyzer_executable_sha256'] = 'rebuilt'
            with mock.patch.object(coverage, 'build', return_value=report), contextlib.redirect_stdout(output):
                coverage.generate(args)
            self.assertIn('Coverage unchanged', output.getvalue())
            self.assertNotEqual(before['index.html'], (Path(tmp) / 'index.html').read_bytes())

    def test_review_requires_same_decoded_file_and_source_map(self):
        evidence = json.loads((coverage.ROOT / 'docs/development/rom-coverage/evidence.json').read_text())
        rid, note = next(iter(evidence['resources'].items()))
        resource = dict(id=rid, sha256=note['decoded_sha256'], sources=[])
        evidence['resources'][rid]['sources_sha256'] = coverage.sources_sha256(resource)
        self.assertEqual(coverage.resource_evidence(resource, evidence)['evidence_status'], 'reviewed')
        for changed in ({**resource, 'id':'new'}, {**resource, 'sha256':'changed'},
                        {**resource, 'sources':[dict(offset=4,length=8)]}):
            actual = coverage.resource_evidence(changed, evidence)
            self.assertFalse(actual['ordinary_positive'])
            self.assertEqual(actual['evidence_status'], 'unreviewed')
        evidence['resources'][rid]['references'] = ['missing-proof.json']
        with self.assertRaisesRegex(ValueError, 'missing evidence'):
            coverage.resource_evidence(resource, evidence)

    def test_new_resources_migrate_source_only_without_inheriting_ordinary_use(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp); dataset = root / 'dataset'; dataset.mkdir()
            rom = bytes(range(256)); (root / 'rom').write_bytes(rom)
            (root / 'proof.json').write_text('{}')
            with sqlite3.connect(root / 'gamedb.sqlite') as db:
                db.execute('CREATE TABLE functions (sha256 TEXT)')
            def source(a,b):
                return dict(offset=a,length=b-a,sha256=hashlib.sha256(rom[a:b]).hexdigest(),role='fixture')
            resources = []
            for name,a,b in [('reviewed',192,208),('new',208,224)]:
                (dataset / name).write_text(name)
                resources.append(dict(id=name,path=name,sha256=coverage.sha(dataset/name),sources=[source(a,b)]))
            evidence = dict(baseline=[coverage.sha(root/'rom'),61,60],date='2026-10-25',limits=[],
                            resources={'reviewed':dict(decoded_sha256=resources[0]['sha256'],
                            sources_sha256=coverage.sources_sha256(resources[0]),ordinary_positive=True,
                            references=['proof.json'],note='accepted')},source_only=[dict(id='new',
                            sources=[source(208,240)],ranges=[[208,240]],references=['proof.json'],note='model')])
            (root / 'evidence.json').write_text(json.dumps(evidence))
            manifest = dict(source_rom_sha256=coverage.sha(root/'rom'),source_rom_size=256,
                            extractor_revision=65,decoder_revision=64,profile='synthetic',resources=resources)
            args = SimpleNamespace(rom=root/'rom',dataset=dataset,evidence=root/'evidence.json',
                                   translation=None,analyzer='unused',code_seeds='unused')
            with mock.patch.object(coverage, 'ROOT', root), \
                 mock.patch.object(coverage.tomllib, 'loads', return_value=manifest), \
                 mock.patch.object(coverage, 'extractor_generation', return_value=(65,64)), \
                 mock.patch.object(coverage, 'collect_sharing', return_value=[]), \
                 mock.patch.object(coverage, 'code_ranges', return_value=([],{})):
                (dataset/'manifest.toml').write_text('mocked parser')
                report = coverage.build(args)
                self.assertEqual(report['pending_review'], ['new'])
                self.assertEqual(report['layers']['ordinary_resource_extents'], [[192,208]])
                self.assertEqual(report['resources'][-1]['ranges'], [[224,240]])
                self.assertEqual(report['resources'][-1]['id'], 'new/source-only')
                resources[1]['sources'] = [source(208,240)]
                report = coverage.build(args)
                self.assertEqual(report['totals']['source_only_data'], 0)
                self.assertEqual(report['totals']['identified_families'], 2)
                manifest['extractor_revision'] = 37
                self.assertTrue(coverage.build(args)['historical_dataset'])
                (dataset/'new').write_text('tampered')
                with self.assertRaisesRegex(ValueError, 'invalid decoded'):
                    coverage.build(args)

    def test_audio_sharing_counts_same_address_references_not_equal_content(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp); resources = []
            def resource(rid, contents, spans):
                name = str(len(resources)) + '.json'
                (root / name).write_text(json.dumps(contents))
                resources.append(dict(id=rid, path=name, sources=[dict(offset=a, length=n, role='test') for a,n in spans]))
            resource('graphics.bg-descriptors', dict(entries=[]), [(0,4)])
            resource('graphics.bg-packet-programs', {}, [])
            resource('graphics.bg-transfer-programs', dict(descriptor_links=[]), [])
            resource('audio.m4a-directory', dict(headers=[dict(id=0,source_offset=200)],
                     songs=[dict(id=0,header_id=0),dict(id=1,header_id=0)]), [(200,8)])
            resource('audio.m4a-tones', dict(pcm_waves=[dict(source_offset=500)], tones=[
                dict(id=0,word04=0x08000000+500), dict(id=1,word04=0x08000000+500)]), [(500,16),(516,4)])
            records = coverage.collect_sharing(root, dict(resources=resources))
            self.assertEqual(len(records), 3)
            self.assertEqual(coverage.length([r['range'] for r in records]), 28)
            self.assertTrue(all(r['references'] == 2 for r in records))

    def test_cache_retention_runs_only_after_successful_automatic_publication(self):
        with tempfile.TemporaryDirectory() as tmp:
            args = SimpleNamespace(dataset='kept',dataset_cache=str(Path(tmp)/'cache'),cache_keep=2,output=tmp)
            report = dict(generation='e65-d64',generated_at='now',reviewed_generation='e61-d60',
                          evidence_date='2026-10-25',pending_review=[],historical_dataset=False,
                          totals=dict(runtime_families=60))
            with mock.patch.object(coverage, 'prepare_dataset'), \
                 mock.patch.object(coverage, 'prune_coverage_cache') as prune:
                with mock.patch.object(coverage, 'build', side_effect=ValueError('failed validation')):
                    with self.assertRaises(ValueError): coverage.generate(args)
                prune.assert_not_called()
                def cleanup(cache,current,keep,*,lease):
                    self.assertTrue((Path(tmp)/'index.html').is_file())
                    self.assertEqual((cache,current,keep), (str(Path(tmp)/'cache'),'kept',2))
                    lease.check(cache)
                    raise PermissionError('read-only cache')
                prune.side_effect = cleanup
                with mock.patch.object(coverage, 'build', return_value=report), \
                     contextlib.redirect_stdout(io.StringIO()), contextlib.redirect_stderr(io.StringIO()):
                    self.assertEqual(coverage.generate(args),0)
                    self.assertEqual(prune.call_count,1)
                    args.dataset_cache = None  # Explicit datasets have no eviction.
                    coverage.generate(args)
                    self.assertEqual(prune.call_count,1)


if __name__ == '__main__':
    unittest.main()
