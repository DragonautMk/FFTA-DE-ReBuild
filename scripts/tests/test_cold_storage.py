"""Content-preserving cold archives and declared-input lifecycle regressions."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import cold_storage as cold
import validation_unit as validation
from artifact_coordinator import NAME


@unittest.skipUnless(shutil.which('zstd'), 'zstd CLI is required for archive tests')
class ColdStorageTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix='cold storage è ')
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name).resolve()
        self.source = self.root / 'target/completed/payload'
        self.source.mkdir(parents=True)
        (self.source / 'data.bin').write_bytes(b'preserved artifact\n' * 2000)
        (self.source / 'empty').mkdir()
        for module in [cold, validation]:
            patch = mock.patch.object(module, 'ensure_untracked')
            patch.start(); self.addCleanup(patch.stop)
        patch = mock.patch.object(cold.shutil, 'disk_usage', return_value=argparse.Namespace(free=100 * cold.GIB))
        self.space = patch.start(); self.addCleanup(patch.stop)

    def pack(self, store, source=None):
        return store.pack(source or self.source, reason='Preserved completed synthetic fixture',
                          proof={'kind': 'foreground-validation', 'status': 'PASS'})

    def test_directory_file_metadata_and_internal_hardlinks_roundtrip(self):
        (self.source / 'manifest.toml').write_text('revision=1\n')
        binary = self.source / 'test.exe'
        binary.write_bytes(b'fake executable')
        binary.chmod(0o751)
        os.link(binary, self.source / 'alias.exe')
        os.utime(binary, ns=(1660000000123456789, 1660000000123456789))
        expected = cold.logical(cold.scan(self.source))
        with cold.Store(self.root) as store:
            data = self.pack(store)
            entry, data = store.entries()[0]
            self.assertFalse(self.source.exists())
            self.assertEqual((entry / 'evidence/manifest.toml').read_text(), 'revision=1\n')
            self.assertLess(data['archive_bytes'], data['logical_bytes'])
            self.assertTrue(store.restore(entry, data))
            self.assertEqual(cold.logical(cold.scan(self.source)), expected)
            self.assertEqual(binary.stat().st_ino, (self.source / 'alias.exe').stat().st_ino)
            self.assertFalse(store.restore(entry, data))
            store.retire(entry, data)
            self.assertFalse(self.source.exists())
            self.assertEqual(data['state'], 'archived')
        self.assertFalse((store.base / NAME).exists())

    def test_single_file_roundtrip(self):
        source = self.source / 'data.bin'; expected = source.read_bytes()
        with cold.Store(self.root) as store:
            self.pack(store, source)
            entry, data = store.entries()[0]
            store.restore(entry, data)
            self.assertEqual(source.read_bytes(), expected)
            store.retire(entry, data)
            self.assertFalse(source.exists())

    @unittest.skipUnless(hasattr(os, 'setxattr'), 'User extended attributes need a supported host')
    def test_wine_dos_attributes_are_preserved(self):
        os.setxattr(self.source / 'data.bin', 'user.DOSATTRIB', b'0x2')
        os.setxattr(self.source, 'user.validation', b'preserved\x00value')
        before = cold.logical(cold.scan(self.source))
        with cold.Store(self.root) as store:
            self.pack(store)
            entry, data = store.entries()[0]
            store.restore(entry, data)
            self.assertEqual(cold.logical(cold.scan(self.source)), before)
            self.assertEqual(os.getxattr(self.source / 'data.bin', 'user.DOSATTRIB'), b'0x2')

    @unittest.skipUnless(os.name == 'posix', 'POSIX Wine link names')
    def test_wine_links_never_follow_or_modify_external_destinations(self):
        external = self.root / 'personal'; external.mkdir()
        (external / 'sentinel').write_bytes(b'personal data')
        (self.source / 'dosdevices').mkdir()
        (self.source / 'dosdevices/z:').symlink_to('/')
        (self.source / 'users').symlink_to(external)
        before = cold.logical(cold.scan(self.source))
        with cold.Store(self.root) as store:
            self.pack(store)
            entry, data = store.entries()[0]
            store.restore(entry, data)
            self.assertEqual(cold.logical(cold.scan(self.source)), before)
            store.retire(entry, data)
        self.assertEqual((external / 'sentinel').read_bytes(), b'personal data')

    def test_private_shared_active_tracked_and_canonical_paths_are_refused(self):
        with cold.Store(self.root) as store:
            (self.source / 'original.sav').write_bytes(b'private')
            with self.assertRaisesRegex(ValueError, 'Private input'):
                self.pack(store)
            (self.source / 'original.sav').unlink()
            os.link(self.source / 'data.bin', self.root / 'outside.bin')
            with self.assertRaisesRegex(ValueError, 'External hard link'):
                self.pack(store)
            (self.root / 'outside.bin').unlink()
            (self.source / NAME).mkdir()
            with self.assertRaisesRegex(ValueError, 'artifact lease'):
                self.pack(store)
            (self.source / NAME).rmdir()
            with mock.patch.object(cold, 'ensure_untracked', side_effect=ValueError('tracked')):
                with self.assertRaisesRegex(ValueError, 'tracked'):
                    self.pack(store)
            for p in ['target', 'target/debug/x', 'target/x86_64-pc-windows-gnu/release/x',
                      'target/validation/wine-prefix/drive_c', 'target/cold-storage/x']:
                with self.subTest(path=p), self.assertRaises(ValueError):
                    store.source(p)
        self.assertTrue((self.source / 'data.bin').exists())

    def test_corrupt_archive_refuses_restore_and_keeps_original_archive(self):
        with cold.Store(self.root) as store:
            self.pack(store)
            entry, data = store.entries()[0]
            with (entry / 'payload.tar.zst').open('ab') as f: f.write(b'corrupt')
            with self.assertRaisesRegex(ValueError, 'checksum mismatch'):
                store.restore(entry, data)
            self.assertFalse(self.source.exists())
            self.assertTrue((entry / 'payload.tar.zst').exists())

    def test_destination_conflict_and_modified_restored_input_are_preserved(self):
        with cold.Store(self.root) as store:
            self.pack(store)
            entry, data = store.entries()[0]
            self.source.mkdir(); (self.source / 'new-data').write_bytes(b'new')
            with self.assertRaisesRegex(ValueError, 'refusing overwrite'):
                store.restore(entry, data)
            self.assertEqual((self.source / 'new-data').read_bytes(), b'new')
            shutil.rmtree(self.source)
            store.restore(entry, data)
            (self.source / 'data.bin').write_bytes(b'changed')
            with self.assertRaisesRegex(ValueError, 'input changed'):
                store.retire(entry, data)
            self.assertEqual((self.source / 'data.bin').read_bytes(), b'changed')
            cold.read_archive(entry / 'payload.tar.zst', store.load(entry, data))

    def test_source_change_during_pack_keeps_original_and_incomplete_diagnostics(self):
        original = cold.read_archive
        def change_after_verification(*args, **kwargs):
            original(*args, **kwargs)
            (self.source / 'data.bin').write_bytes(b'new source')
        with cold.Store(self.root) as store, mock.patch.object(cold, 'read_archive', side_effect=change_after_verification):
            with self.assertRaisesRegex(ValueError, 'Source changed'):
                self.pack(store)
            entry, data = store.entries()[0]
            self.assertEqual(data['state'], 'preparing')
            self.assertEqual((self.source / 'data.bin').read_bytes(), b'new source')
            self.assertTrue((entry / 'payload.pending.tar.zst').exists())

    def test_interrupted_retirement_recovery_preserves_archive_and_restores_content(self):
        before = cold.logical(cold.scan(self.source))
        with cold.Store(self.root) as store:
            with mock.patch.object(cold, 'remove_owned', side_effect=OSError('simulated interruption')):
                with self.assertRaises(OSError): self.pack(store)
            entry, data = store.entries()[0]
            self.assertEqual(data['state'], 'retiring')
            self.assertTrue((entry / 'retiring/data.bin').exists())
            with self.assertRaisesRegex(ValueError, 'explicit recovery'):
                store.restore(entry, data)
            store.recover(entry, data)
            store.restore(entry, data)
            self.assertEqual(cold.logical(cold.scan(self.source)), before)

    def test_partial_restore_recovery_never_publishes_incomplete_files(self):
        with cold.Store(self.root) as store:
            self.pack(store)
            entry, data = store.entries()[0]
            def interrupted(payload, rows, dest):
                dest.mkdir(); (dest / 'data.bin').write_bytes(b'partial')
                raise OSError('interrupted extraction')
            with mock.patch.object(cold, 'read_archive', side_effect=interrupted):
                with self.assertRaises(OSError): store.restore(entry, data)
            self.assertFalse(self.source.exists())
            self.assertEqual(data['state'], 'restoring')
            store.recover(entry, data)
            store.restore(entry, data)
            self.assertEqual((self.source / 'data.bin').read_bytes(), b'preserved artifact\n' * 2000)

    def test_low_space_and_concurrent_lease_refuse_without_retirement(self):
        with cold.Store(self.root) as store:
            with self.assertRaisesRegex(ValueError, 'Active or interrupted'):
                with cold.Store(self.root): self.fail('Store must be exclusive')
            self.space.return_value.free = 0
            with self.assertRaisesRegex(ValueError, 'Insufficient space'): self.pack(store)
            self.assertTrue(self.source.exists())
            self.assertEqual(store.entries(), [])
            self.space.return_value.free = 100 * cold.GIB
            self.pack(store)
            entry, data = store.entries()[0]
            self.space.return_value.free = 0
            with self.assertRaisesRegex(ValueError, 'Insufficient space'): store.restore(entry, data)
            self.assertFalse(self.source.exists())
            self.assertEqual(data['state'], 'archived')

    def test_manifest_rejects_traversal_and_link_ancestors(self):
        row = {'type':'dir', 'mode':0o755}
        for bad in [{'..':row}, {'.':row, '../outside':row}, {'.':row, '/outside':row},
                    {'.':row, 'link':{'type':'symlink','mode':0o777,'target':'/'}, 'link/file':row}]:
            with self.subTest(rows=bad), self.assertRaises(ValueError): cold.check_manifest(bad)

    def test_review_binds_inventory_completion_and_rechecks_activity(self):
        evidence = self.source.parent / 'result.json'
        evidence.write_text('{"status":"PASS"}')
        with cold.Store(self.root) as store, mock.patch.object(cold, 'host_activity', return_value={'host': True}) as activity:
            review = cold.prepare_review(store, self.source, [evidence], self.source.parent)
            data = cold.apply_review(store, review, 'Reviewed historical fixture')
            self.assertEqual(activity.call_count, 3)
            self.assertEqual(data['state'], 'archived')
            self.assertFalse((self.source.parent / NAME).exists())

    def test_stale_review_and_changed_completion_never_retire_source(self):
        evidence = self.source.parent / 'result.json'
        evidence.write_text('{"status":"PASS"}')
        with cold.Store(self.root) as store, mock.patch.object(cold, 'host_activity', return_value={'host': True}):
            review = cold.prepare_review(store, self.source, [evidence], self.source.parent)
            (self.source / 'data.bin').write_bytes(b'changed since preview')
            with self.assertRaisesRegex(ValueError, 'Reviewed source inventory changed'):
                cold.apply_review(store, review, 'Old review')
            evidence.write_text('{"status":"FAIL"}')
            with self.assertRaisesRegex(ValueError, 'completed/PASS'):
                cold.apply_review(store, review, 'Changed result')
            self.assertEqual((self.source / 'data.bin').read_bytes(), b'changed since preview')

    def test_changed_retirement_residue_is_not_discarded_by_recovery(self):
        with cold.Store(self.root) as store:
            with mock.patch.object(cold, 'remove_owned', side_effect=OSError('interruption')):
                with self.assertRaises(OSError): self.pack(store)
            entry, data = store.entries()[0]
            (entry / 'retiring/data.bin').write_bytes(b'new diagnostic data')
            with self.assertRaisesRegex(ValueError, 'Changed retirement residue'):
                store.recover(entry, data)
            self.assertEqual((entry / 'retiring/data.bin').read_bytes(), b'new diagnostic data')

    def test_cli_run_resolves_original_paths_and_retires_successful_copy(self):
        subprocess.run(['git', 'init', '-q', str(self.root)], check=True)
        with cold.Store(self.root) as store: self.pack(store)
        code = f'from pathlib import Path; assert Path({str(self.source / "data.bin")!r}).read_bytes().startswith(b"preserved")'
        result = subprocess.run([sys.executable, str(Path(cold.__file__)), '--root', str(self.root),
                                 'run', '--input', str(self.source), '--', sys.executable, '-c', code],
                                capture_output=True, text=True)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertFalse(self.source.exists())
        self.assertFalse((self.root / 'target/cold-storage' / NAME).exists())

    def args(self, code, **changes):
        args = argparse.Namespace(name='cold-reader', result=['result.json'], evidence=[],
                                  protect=[str(self.source / 'data.bin')], env=[], wine=False,
                                  wineserver=None, timeout=30, keep_work=None, keep_path=[],
                                  keep_reason=None, keep_hot=None, min_free_gib=5, reserve_gib=1,
                                  space_exception=None, command=[sys.executable, '-c', code])
        for key, value in changes.items(): setattr(args, key, value)
        return args

    def test_runner_restores_declared_input_and_retires_it_after_pass(self):
        with cold.Store(self.root) as store: self.pack(store)
        code = f'from pathlib import Path; assert Path({str(self.source / "data.bin")!r}).read_bytes().startswith(b"preserved"); Path("result.json").write_text(\'{{"status":"PASS"}}\')'
        unit = validation.run_unit(self.args(code), root=self.root)
        receipt = json.loads((unit / 'unit-result.json').read_text())
        self.assertEqual(receipt['cold_inputs_retired'], ['target/completed/payload'])
        self.assertFalse(self.source.exists())
        self.assertFalse((unit / 'work').exists())
        self.assertFalse((self.root / 'target/cold-storage' / NAME).exists())

    def test_runner_archives_successful_retained_outputs_by_default(self):
        code='from pathlib import Path; Path("result.json").write_text(\'{"status":"PASS"}\'); Path("dataset").mkdir(); Path("dataset/data.bin").write_bytes(b"retained output" * 1000)'
        unit = validation.run_unit(self.args(code, keep_path=['dataset'], keep_reason='next comparison'), root=self.root)
        receipt = json.loads((unit / 'unit-result.json').read_text())
        self.assertEqual(receipt['retention_storage'], 'cold')
        self.assertEqual(receipt['cleanup'], 'kept')
        self.assertFalse((unit / 'work').exists())
        with cold.Store(self.root) as store:
            entry, data = store.entries()[0]
            store.restore(entry, data)
        self.assertEqual((unit / 'work/dataset/data.bin').read_bytes(), b'retained output' * 1000)

    def test_small_recorded_space_exception_also_applies_to_output_archival(self):
        self.space.return_value.free = cold.GIB
        code='from pathlib import Path; Path("result.json").write_text(\'{"status":"PASS"}\'); Path("payload.bin").write_bytes(b"small synthetic payload")'
        unit = validation.run_unit(self.args(code, keep_path=['payload.bin'], keep_reason='next synthetic check',
                                            min_free_gib=0.5, reserve_gib=0.1,
                                            space_exception='Synthetic payload below 1 MiB; no build'), root=self.root)
        self.assertFalse((unit / 'work').exists())
        self.assertEqual(json.loads((unit / 'unit-result.json').read_text())['retention_storage'], 'cold')

    def test_failed_consumer_keeps_restored_diagnostics_and_timeout_keeps_lease(self):
        with cold.Store(self.root) as store: self.pack(store)
        with self.assertRaisesRegex(ValueError, 'exited 4'):
            validation.run_unit(self.args('raise SystemExit(4)'), root=self.root)
        self.assertTrue(self.source.exists())
        self.assertFalse((self.root / 'target/cold-storage' / NAME).exists())
        with cold.Store(self.root) as store:
            entry, data = store.entries()[0]; store.retire(entry, data)
        with self.assertRaises(subprocess.TimeoutExpired):
            validation.run_unit(self.args('import time; time.sleep(5)', timeout=0.2), root=self.root)
        self.assertTrue(self.source.exists())
        self.assertTrue((self.root / 'target/cold-storage' / NAME).exists())


if __name__ == '__main__':
    unittest.main()
