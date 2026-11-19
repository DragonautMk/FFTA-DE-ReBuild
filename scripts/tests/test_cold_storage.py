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

    def external_fixture(self, store):
        self.pack(store)
        entry, data = store.entries()[0]
        result = self.source.parent / 'result.json'
        result.write_text('{"status":"PASS"}')
        data['proof'].update(scope='target/completed', completion=[{
            'path': 'target/completed/result.json', 'sha256': cold.digest(result)}])
        cold.save(entry / 'entry.json', data)
        mount = self.root / 'external'; (mount / 'archives').mkdir(parents=True)
        return entry, data, mount

    def matching_file(self, name):
        copy = self.source.parent / name
        shutil.copy2(self.source / 'data.bin', copy)
        return copy

    @unittest.skipUnless(os.name == 'posix' and hasattr(os, 'listxattr'), 'POSIX archive sharing')
    def test_compact_legacy_entries_offline_and_preserve_proof_and_independent_restores(self):
        one, two = self.matching_file('one'), self.matching_file('two')
        expected = one.read_bytes()
        with cold.Store(self.root) as store:
            with mock.patch.object(cold, 'compact_local_payloads', return_value={'shared': 0}):
                self.pack(store, one)
                store.pack(two, reason='Keep failed diagnostics locally', proof={'status': 'FAIL'})
            entries = store.entries()
            payloads = [e / 'payload.tar.zst' for e, _ in entries]
            before = [(e / 'entry.json').read_bytes() for e, _ in entries]
            preview = cold.compact_local_payloads(store, entries)
            self.assertEqual(preview['candidates'], 1)
            self.assertFalse(payloads[0].samefile(payloads[1]))
            with mock.patch.object(cold, 'external_mount', side_effect=AssertionError('USB accessed')):
                result = cold.compact_local_payloads(store, entries, apply=True)
                self.assertEqual(result['released_bytes'], preview['candidate_bytes'])
                self.assertTrue(payloads[0].samefile(payloads[1]))
                self.assertFalse(payloads[0].stat().st_mode & 0o222)
                self.assertEqual(before, [(e / 'entry.json').read_bytes() for e, _ in entries])
                self.assertEqual(cold.compact_local_payloads(store, entries, apply=True)['shared'], 0)
                for e, d in entries:
                    self.assertTrue(store.restore(e, d))
                one.write_bytes(b'new working copy')
                self.assertEqual(two.read_bytes(), expected)

    @unittest.skipUnless(os.name == 'posix' and hasattr(os, 'listxattr'), 'POSIX archive sharing')
    def test_new_publication_shares_automatically_and_offload_preserves_its_local_peer(self):
        # A single-file archive with the same tar member and timestamps.
        peer = self.matching_file('peer')
        other = self.matching_file('other')
        with cold.Store(self.root) as store:
            self.pack(store, peer)
            result = self.pack(store, other)
            self.assertEqual(result['archive_added_bytes'], 0)
            entry, data = store.matching([other])[0]
            result_path = other.parent / 'result.json'
            result_path.write_text('{"status":"PASS"}')
            data['proof']['scope'] = 'target/completed'
            cold.save(entry / 'entry.json', data)
            mount = self.root / 'external'
            (mount / 'archives').mkdir(parents=True)
            with mock.patch.object(cold, 'external_mount', return_value=(mount, 'test-uuid')), \
                    mock.patch.object(cold, 'host_activity', return_value={}):
                store.offload(entry, data, mount, 'archives', completion_paths=[result_path])
            with mock.patch.object(cold, 'external_mount', side_effect=AssertionError('USB accessed')):
                e, d = store.matching([peer])[0]
                self.assertTrue(store.restore(e, d))
                self.assertEqual(peer.read_bytes(), (self.source / 'data.bin').read_bytes())

    @unittest.skipUnless(os.name == 'posix' and hasattr(os, 'listxattr'), 'POSIX archive permissions')
    def test_sharing_excludes_different_access_and_links_outside_catalog(self):
        one, two = self.matching_file('one'), self.matching_file('two')
        with cold.Store(self.root) as store:
            with mock.patch.object(cold, 'compact_local_payloads', return_value={'shared': 0}):
                self.pack(store, one)
                self.pack(store, two)
            entries = store.entries()
            first, second = [e / 'payload.tar.zst' for e, _ in entries]
            first.chmod(0o400)
            second.chmod(0o444)
            self.assertEqual(cold.compact_local_payloads(store, entries, apply=True)['candidates'], 0)
            second.chmod(0o400)
            os.link(first, self.root / 'outside.zst')
            self.assertEqual(cold.compact_local_payloads(store, entries, apply=True)['candidates'], 0)
            self.assertFalse(first.samefile(second))

    @unittest.skipUnless(os.name == 'posix' and hasattr(os, 'listxattr'), 'POSIX archive sharing')
    def test_corrupt_peer_keeps_new_source_and_link_failure_keeps_separate_archives(self):
        peer, copy = self.matching_file('peer'), self.matching_file('copy')
        with cold.Store(self.root) as store:
            self.pack(store, peer)
            with mock.patch.object(cold.os, 'link', side_effect=PermissionError(cold.errno.EPERM, 'links disabled')):
                self.pack(store, copy)
            entries = store.entries()
            payloads = [e / 'payload.tar.zst' for e, _ in entries]
            self.assertFalse(payloads[0].samefile(payloads[1]))
            candidate = self.matching_file('candidate')
            with payloads[0].open('rb') as f:
                corrupt = bytearray(f.read())
            sealed_mode = payloads[0].stat().st_mode & 0o555
            corrupt[-1] ^= 1
            payloads[0].chmod(0o600)
            payloads[0].write_bytes(corrupt)
            payloads[0].chmod(sealed_mode)
            with self.assertRaisesRegex(ValueError, 'checksum mismatch'):
                self.pack(store, candidate)
            self.assertEqual(candidate.read_bytes(), (self.source / 'data.bin').read_bytes())

    def test_reviewed_local_diagnostic_preserves_negative_outcome_and_rejects_offload(self):
        scope = self.source.parent
        first, last = scope / 'first.json', scope / 'final.json'
        first.write_text('{"closed": true, "verify_exit": 1}')
        last.write_text('{"closed": true, "status": "PASS"}')
        with cold.Store(self.root) as store, mock.patch.object(cold, 'host_activity', return_value={'checked': True}):
            review = cold.prepare_review(store, self.source, [first, last], scope, local_diagnostic='MIXED')
            cold.apply_review(store, review, 'Keep both diagnostic runs offline')
            entry, data = store.entries()[0]
            self.assertEqual(data['proof']['status'], 'MIXED')
            self.assertEqual(data['proof']['original_outcome'], 'MIXED')
            self.assertNotIn('completion', data['proof'])
            with self.assertRaisesRegex(ValueError, 'internal disk'):
                store.offload(entry, data, self.root / 'missing-volume', 'archives', completion_paths=[last])
            store.restore(entry, data)
            self.assertTrue((self.source / 'data.bin').is_file())

    def test_diagnostic_review_checks_scope_lease_and_outcomes_again_before_retirement(self):
        scope = self.source.parent
        result = scope / 'failure.json'
        result.write_text('{"status": "running", "closed": true}')
        with cold.Store(self.root) as store, mock.patch.object(cold, 'host_activity', return_value={'checked': True}):
            with self.assertRaisesRegex(ValueError, 'Closed diagnostic receipt'):
                cold.prepare_review(store, self.source, [result], scope, local_diagnostic='FAIL')
            result.write_text('{"status": "FAIL"}')
            guard = scope / 'sibling' / NAME
            guard.mkdir(parents=True)
            with self.assertRaisesRegex(ValueError, 'Existing activity lease'):
                cold.prepare_review(store, self.source, [result], scope, local_diagnostic='FAIL')
            guard.rmdir()
            review = cold.prepare_review(store, self.source, [result], scope, local_diagnostic='FAIL')
            original = cold.read_archive
            def change_evidence(*args):
                original(*args)
                result.write_text('{"status": "FAIL", "changed": true}')
            with mock.patch.object(cold, 'read_archive', side_effect=change_evidence):
                with self.assertRaisesRegex(ValueError, 'evidence changed'):
                    cold.apply_review(store, review, 'Preserve changed diagnostic')
            self.assertTrue((self.source / 'data.bin').is_file())

    def test_external_offload_roundtrip_and_unavailable_or_corrupt_volume(self):
        expected = cold.logical(cold.scan(self.source))
        with cold.Store(self.root) as store:
            entry, data, mount = self.external_fixture(store)
            def volume(path, expected_uuid=None):
                self.assertEqual(Path(path), mount)
                self.assertIn(expected_uuid, (None, 'test-uuid'))
                return mount, 'test-uuid'
            with mock.patch.object(cold, 'external_mount', side_effect=volume), \
                 mock.patch.object(cold, 'host_activity', return_value={}):
                data = store.offload(entry, data, mount, 'archives')
                payload = mount / data['external_payload']['path']
                self.assertFalse((entry / 'payload.tar.zst').exists())
                self.assertTrue(payload.exists())
                self.assertEqual(store.entries()[0][1]['external_payload'], data['external_payload'])
                store.restore(entry, data)
                self.assertEqual(cold.logical(cold.scan(self.source)), expected)
                store.retire(entry, data)
                with payload.open('ab') as f: f.write(b'corrupt')
                with self.assertRaisesRegex(ValueError, 'checksum mismatch'):
                    store.restore(entry, data)
            with mock.patch.object(cold, 'external_mount', side_effect=ValueError('volume unavailable')):
                with self.assertRaisesRegex(ValueError, 'volume unavailable'):
                    store.restore(entry, data)
            self.assertFalse(self.source.exists())
            self.assertTrue(payload.exists())

    def test_external_offload_failure_preserves_original_and_never_overwrites(self):
        with cold.Store(self.root) as store:
            entry, data, mount = self.external_fixture(store)
            destination = mount / 'archives' / (entry.name + '.tar.zst')
            with mock.patch.object(cold, 'external_mount', return_value=(mount, 'test-uuid')), \
                 mock.patch.object(cold, 'host_activity', return_value={}):
                destination.write_bytes(b'existing file')
                with self.assertRaises(FileExistsError):
                    store.offload(entry, data, mount, 'archives')
                self.assertEqual(destination.read_bytes(), b'existing file')
                self.assertTrue((entry / 'payload.tar.zst').exists())
                destination.unlink()
                with mock.patch.object(cold.shutil, 'copyfileobj', side_effect=OSError('interrupted')):
                    with self.assertRaisesRegex(OSError, 'interrupted'):
                        store.offload(entry, data, mount, 'archives')
                self.assertTrue((entry / 'payload.tar.zst').exists())
                self.assertNotIn('external_payload', json.loads((entry / 'entry.json').read_text()))

    def test_external_offload_accepts_existing_zero_exit_list(self):
        with cold.Store(self.root) as store:
            entry, data, mount = self.external_fixture(store)
            result = self.source.parent / 'result.json'
            result.write_text(json.dumps([{'exit': 0, 'filter': 'first'}, {'exit': 0, 'filter': 'second'}]))
            data['proof']['completion'][0]['sha256'] = cold.digest(result)
            original = json.loads(json.dumps(data['proof']))
            cold.save(entry / 'entry.json', data)
            with mock.patch.object(cold, 'external_mount', return_value=(mount, 'test-uuid')), \
                 mock.patch.object(cold, 'host_activity', return_value={}):
                published = store.offload(entry, data, mount, 'archives')
                self.assertEqual(published['proof'], original)
                self.assertEqual(published['offload_review']['basis'], 'archive proof')
                store.restore(entry, published)
                self.assertTrue((self.source / 'data.bin').exists())

    def test_external_offload_can_add_reviewed_evidence_without_rewriting_history(self):
        with cold.Store(self.root) as store:
            entry, data, mount = self.external_fixture(store)
            result = self.source.parent / 'result.json'
            result.write_text(json.dumps({'exit': 0, 'protected_unchanged': True, 'test_result': True}))
            data['proof'].update(basis='idle-history', completion=[], outcome_guards=[{
                'name': 'result.json', 'sha256': cold.digest(result)}])
            original = json.loads(json.dumps(data['proof']))
            cold.save(entry / 'entry.json', data)
            with mock.patch.object(cold, 'external_mount', return_value=(mount, 'test-uuid')), \
                 mock.patch.object(cold, 'host_activity', return_value={}):
                published = store.offload(entry, data, mount, 'archives',
                                          completion_paths=['target/completed/result.json'])
                self.assertEqual(published['proof'], original)
                self.assertEqual(published['offload_review']['basis'], 'explicit')
                self.assertEqual(published['offload_review']['completion'][0]['sha256'], cold.digest(result))
                store.restore(entry, published)
                self.assertTrue((self.source / 'data.bin').exists())

    def test_external_offload_refuses_failed_outside_or_changed_evidence(self):
        with cold.Store(self.root) as store:
            entry, data, mount = self.external_fixture(store)
            result = self.source.parent / 'result.json'
            data['proof'].update(basis='idle-history', completion=[], outcome_guards=[{
                'name': 'result.json', 'sha256': cold.digest(result)}])
            cold.save(entry / 'entry.json', data)
            outside = self.root / 'outside.json'; outside.write_text('{"status":"PASS"}')
            with mock.patch.object(cold, 'external_mount', return_value=(mount, 'test-uuid')), \
                 mock.patch.object(cold, 'host_activity', return_value={}):
                with self.assertRaisesRegex(ValueError, 'within the scope'):
                    store.offload(entry, data, mount, 'archives', completion_paths=[outside])
                result.write_text('{"exit":0,"test_result":false}')
                with self.assertRaisesRegex(ValueError, 'completed/PASS'):
                    store.offload(entry, data, mount, 'archives', completion_paths=[result])
                result.write_text('{"exit":0,"test_result":true}')
                with self.assertRaisesRegex(ValueError, 'Historical outcome evidence changed'):
                    store.offload(entry, data, mount, 'archives', completion_paths=[result])
            self.assertTrue((entry / 'payload.tar.zst').exists())
            self.assertNotIn('external_payload', json.loads((entry / 'entry.json').read_text()))

    def test_external_mount_refuses_wrong_or_missing_uuid(self):
        result = mock.Mock(stdout=json.dumps({'filesystems': [{'target': str(self.root), 'uuid': 'actual'}]}))
        with mock.patch.object(cold.subprocess, 'run', return_value=result):
            with self.assertRaisesRegex(ValueError, 'UUID'):
                cold.external_mount(self.root, 'different')
            result.stdout = json.dumps({'filesystems': [{'target': str(self.root), 'uuid': None}]})
            with self.assertRaisesRegex(ValueError, 'UUID'):
                cold.external_mount(self.root)

    def test_external_offload_changed_completion_retains_local_copy(self):
        with cold.Store(self.root) as store:
            entry, data, mount = self.external_fixture(store)
            copy = cold.shutil.copyfileobj
            def changed_completion(src, dst, length):
                copy(src, dst, length)
                (self.source.parent / 'result.json').write_text('{"status":"FAIL"}')
            with mock.patch.object(cold, 'external_mount', return_value=(mount, 'test-uuid')), \
                 mock.patch.object(cold, 'host_activity', return_value={}), \
                 mock.patch.object(cold.shutil, 'copyfileobj', side_effect=changed_completion):
                with self.assertRaisesRegex(ValueError, 'completed/PASS'):
                    store.offload(entry, data, mount, 'archives')
            self.assertTrue((entry / 'payload.tar.zst').exists())
            self.assertNotIn('external_payload', json.loads((entry / 'entry.json').read_text()))

    def test_external_offload_disconnect_after_publication_keeps_local_copy(self):
        with cold.Store(self.root) as store:
            entry, data, mount = self.external_fixture(store)
            calls = 0
            def volume(path, expected_uuid=None):
                nonlocal calls
                calls += 1
                if calls == 3:
                    raise ValueError('volume unavailable')
                return mount, 'test-uuid'
            with mock.patch.object(cold, 'external_mount', side_effect=volume), \
                 mock.patch.object(cold, 'host_activity', return_value={}):
                with self.assertRaisesRegex(ValueError, 'volume unavailable'):
                    store.offload(entry, data, mount, 'archives')
            self.assertTrue((entry / 'payload.tar.zst').exists())
            self.assertIn('external_payload', json.loads((entry / 'entry.json').read_text()))

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
