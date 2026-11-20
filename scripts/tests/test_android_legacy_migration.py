import importlib.util
import contextlib
import io
import sys
import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest import mock

ROOT = Path(__file__).resolve().parents[2]
SPEC = importlib.util.spec_from_file_location(
    'android_legacy_migration', ROOT / 'AndroidPort/scripts/migrate_legacy_saves.py')
M = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(M)


class FakeDevice:
    def __init__(self):
        self.rom = b'synthetic ROM identity only'
        self.rom_hash = M.digest(self.rom)
        self.state = f'files/SaveStates/{self.rom_hash}/slot-1.ffstate'
        self.backup = (f'files/SaveBackups/{self.rom_hash}/ffta.sav/'
                       f'backup-00000000000000000001-{M.digest(b"old backup")}.sav')
        self.data = {
            M.OLD: {'files/ffta.gba': self.rom, 'files/ffta.sav': b'old progress',
                    self.state: b'old state', self.backup: b'old backup'},
            M.NEW: {'files/ffta.gba': self.rom, 'files/ffta.sav': b'new progress',
                    self.state: b'new state'},
        }
        self.dirs = set()
        self.active = set()
        self.writes = 0
        self.fail_replace = None
        self.mutate_on_stage = False

    def running(self, package): return package in self.active
    def files(self, package): return list(self.data[package])
    def exists(self, package, path): return path in self.data[package] or (package, path) in self.dirs
    def hash(self, package, path):
        data = self.data[package].get(path)
        return M.digest(data) if data is not None else None
    def read(self, package, path): return self.data[package][path]
    def mkdir_exclusive(self, package, path):
        if self.exists(package, path): raise M.MigrationError('lock already exists')
        self.dirs.add((package, path)); self.writes += 1
    def write_new(self, package, path, data):
        if self.exists(package, path): raise M.MigrationError('file already exists')
        self.data[package][path] = data; self.writes += 1
        if self.mutate_on_stage and '/item-' in path:
            self.data[M.NEW]['files/ffta.sav'] = b'concurrent progress'
            self.mutate_on_stage = False
    def replace(self, package, source, destination):
        if self.fail_replace == destination:
            self.fail_replace = None
            raise M.MigrationError('injected publish failure')
        self.data[package][destination] = self.data[package].pop(source); self.writes += 1
    def remove(self, package, path):
        del self.data[package][path]; self.writes += 1
    def remove_tree(self, package, path):
        if path != M.LOCK: raise AssertionError(path)
        self.dirs.remove((package, path))
        for name in list(self.data[package]):
            if name.startswith(path + '/'): del self.data[package][name]
        self.writes += 1


class LegacyMigrationTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix='ffta-migration-test-')
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.device = FakeDevice()

    def test_copies_all_saves_preserves_both_preimages_and_is_one_shot(self):
        old_before = dict(self.device.data[M.OLD])
        new_before = dict(self.device.data[M.NEW])
        result = M.migrate(self.device, self.root)
        self.assertEqual(result['status'], 'migrated')
        self.assertEqual(result['copied'], 3)
        self.assertEqual(self.device.data[M.OLD], old_before)
        for path in ('files/ffta.sav', self.device.state, self.device.backup):
            self.assertEqual(self.device.data[M.NEW][path], old_before[path])
        backup = Path(result['backup'])
        self.assertEqual((backup / 'new-before/files/ffta.sav').read_bytes(), new_before['files/ffta.sav'])
        self.assertEqual((backup / 'new-before' / self.device.state).read_bytes(), new_before[self.device.state])
        self.assertEqual((backup / 'old' / self.device.state).read_bytes(), old_before[self.device.state])
        self.assertIn(M.MARKER, self.device.data[M.NEW])
        self.assertNotIn((M.NEW, M.LOCK), self.device.dirs)
        # Playing after migration must not be overwritten by a second invocation.
        self.device.data[M.NEW]['files/ffta.sav'] = b'later gameplay'
        writes = self.device.writes
        self.assertEqual(M.migrate(self.device, self.root)['status'], 'already_migrated')
        self.assertEqual(self.device.writes, writes)
        self.assertEqual(self.device.data[M.NEW]['files/ffta.sav'], b'later gameplay')

    def test_rom_mismatch_and_running_app_leave_saves_untouched(self):
        before = dict(self.device.data[M.NEW])
        self.device.data[M.NEW]['files/ffta.gba'] = b'another ROM'
        with self.assertRaisesRegex(M.MigrationError, 'same ROM'):
            M.migrate(self.device, self.root)
        self.assertEqual(self.device.data[M.NEW]['files/ffta.sav'], before['files/ffta.sav'])
        self.assertEqual(self.device.writes, 0)
        self.device.data[M.NEW]['files/ffta.gba'] = self.device.rom
        self.device.active.add(M.NEW)
        with self.assertRaisesRegex(M.MigrationError, 'Close both'):
            M.migrate(self.device, self.root)
        self.assertEqual(self.device.writes, 0)

    def test_failure_rolls_back_new_and_replaced_files(self):
        old_before = dict(self.device.data[M.OLD])
        new_before = dict(self.device.data[M.NEW])
        self.device.fail_replace = 'files/ffta.sav'
        with self.assertRaisesRegex(M.MigrationError, 'rolled back its changes'):
            M.migrate(self.device, self.root)
        self.assertEqual(self.device.data[M.OLD], old_before)
        self.assertEqual(self.device.data[M.NEW], new_before)
        self.assertNotIn((M.NEW, M.LOCK), self.device.dirs)
        self.assertTrue(list(self.root.glob('*/manifest.json')))

    def test_concurrent_change_aborts_without_overwriting_it(self):
        self.device.mutate_on_stage = True
        with self.assertRaisesRegex(M.MigrationError, 'changed before import'):
            M.migrate(self.device, self.root)
        self.assertEqual(self.device.data[M.NEW]['files/ffta.sav'], b'concurrent progress')
        self.assertEqual(self.device.data[M.NEW][self.device.state], b'new state')
        self.assertNotIn(M.MARKER, self.device.data[M.NEW])

    def test_existing_lock_and_unknown_save_are_never_removed(self):
        self.device.dirs.add((M.NEW, M.LOCK))
        with self.assertRaises(M.MigrationError): M.migrate(self.device, self.root)
        self.assertIn((M.NEW, M.LOCK), self.device.dirs)
        self.device.dirs.clear()
        self.device.data[M.OLD]['files/SaveStates/unknown/file.ffstate'] = b'data'
        with self.assertRaisesRegex(M.MigrationError, 'unsupported save path'):
            M.migrate(self.device, self.root)
        self.assertEqual(self.device.data[M.NEW]['files/ffta.sav'], b'new progress')

    def test_damaged_legacy_backup_is_not_published(self):
        self.device.data[M.OLD][self.device.backup] = b'corrupt backup'
        before = dict(self.device.data[M.NEW])
        with self.assertRaisesRegex(M.MigrationError, 'recorded checksum'):
            M.migrate(self.device, self.root)
        self.assertEqual(self.device.data[M.NEW], before)
        self.assertNotIn((M.NEW, M.LOCK), self.device.dirs)

    def test_adb_refuses_symlinked_destination_directories(self):
        bridge = M.Adb('synthetic-device')
        with mock.patch.object(bridge, 'shell', side_effect=[
                SimpleNamespace(returncode=1), SimpleNamespace(returncode=0)]) as call:
            with self.assertRaisesRegex(M.MigrationError, 'symbolic link'):
                bridge.replace(M.NEW, M.LOCK + '/item-0', self.device.state)
        self.assertEqual(call.call_count, 2)

    def test_backup_must_be_outside_git_but_standalone_script_is_supported(self):
        checkout = self.root / 'checkout'
        (checkout / '.git').mkdir(parents=True)
        with mock.patch.object(sys, 'argv', ['migrate', '--backup-root', str(checkout / 'backup')]), \
             contextlib.redirect_stderr(io.StringIO()):
            with self.assertRaises(SystemExit) as error: M.main()
        self.assertEqual(error.exception.code, 2)
        bridge = mock.MagicMock()
        bridge.call.return_value = SimpleNamespace(stdout=b'device')
        with mock.patch.object(M, '__file__', str(self.root / 'downloaded.py')), \
             mock.patch.object(sys, 'argv', ['migrate', '--backup-root', str(self.root / 'private')]), \
             mock.patch.object(M, 'Adb', return_value=bridge), \
             mock.patch.object(M, 'inside_git', return_value=False), \
             mock.patch.object(M, 'migrate', return_value={'status':'already_migrated','copied':0}), \
             contextlib.redirect_stdout(io.StringIO()):
            self.assertEqual(M.main(), 0)


if __name__ == '__main__':
    unittest.main()
