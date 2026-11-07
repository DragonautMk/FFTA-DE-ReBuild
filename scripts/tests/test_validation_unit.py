"""Bounded validation lifecycle using synthetic data and foreground children."""
import argparse
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import validation_unit as validation
from artifact_coordinator import Lease, NAME


class ValidationUnitTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix='ffta validation è ')
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name).resolve()
        self.source = self.root / 'original.sav'
        self.source.write_bytes(b'private original')
        # Windows Python under Wine has no native git.exe. The real host-side
        # Git guard is exercised by the separate Wine integration invocation.
        patch = mock.patch.object(validation, 'ensure_untracked')
        self.git = patch.start()
        self.addCleanup(patch.stop)
        space_patch = mock.patch.object(validation.shutil, 'disk_usage',
                                       return_value=argparse.Namespace(free=20 * validation.GIB))
        self.space = space_patch.start()
        self.addCleanup(space_patch.stop)

    def args(self, code=None, **changes):
        code = code or "from pathlib import Path; Path('result.json').write_text('{\"status\":\"PASS\"}'); Path('dataset').mkdir(); Path('dataset/manifest.toml').write_text('revision=1'); Path('dataset/resource.json').write_text('generated payload')"
        args = argparse.Namespace(name='synthetic', result=['result.json'], evidence=[],
                                  protect=[str(self.source)], env=[], wine=False, wineserver=None,
                                  timeout=30, keep_work=None, keep_path=[], keep_reason=None,
                                  keep_hot='Synthetic lifecycle fixture; archive behavior has separate tests',
                                  min_free_gib=5, reserve_gib=1, space_exception=None,
                                  command=[sys.executable, '-c', code])
        for key, value in changes.items():
            setattr(args, key, value)
        return args

    def units(self):
        return list((self.root / 'target/validation/units').iterdir())

    def receipt(self):
        return json.loads((self.units()[0] / 'unit-result.json').read_text())

    def test_success_retires_payload_keeps_evidence_and_private_input(self):
        before = validation.snapshot(self.source, hashes=True)
        unit = validation.run_unit(self.args(), root=self.root)
        self.assertFalse((unit / 'work').exists())
        self.assertTrue((unit / 'evidence/result.json').is_file())
        self.assertTrue((unit / 'evidence/dataset/manifest.toml').is_file())
        self.assertFalse((unit / 'evidence/dataset/resource.json').exists())
        self.assertTrue((unit / 'command.log').is_file())
        self.assertTrue((unit / 'work-inventory.json').is_file())
        self.assertEqual(validation.snapshot(self.source, hashes=True), before)
        self.assertEqual(self.receipt()['cleanup'], 'removed')
        self.assertEqual(self.receipt()['status'], 'PASS')
        self.assertFalse((unit / NAME).exists())
        self.git.assert_called_once_with(unit / 'work', self.root)

    def test_nonzero_missing_or_negative_result_retains_work(self):
        for code in ["raise SystemExit(7)", 'pass',
                     "from pathlib import Path; Path('result.json').write_text('{\"status\":\"FAIL\"}')"]:
            with self.subTest(code=code):
                previous = set(self.units()) if (self.root / 'target/validation/units').exists() else set()
                with self.assertRaises((ValueError, OSError)):
                    validation.run_unit(self.args(code), root=self.root)
                unit, = set(self.units()) - previous
                self.assertTrue((unit / 'work').is_dir())
                self.assertEqual(json.loads((unit / 'unit-result.json').read_text())['status'], 'FAIL')
                self.assertFalse((unit / NAME).exists())

    def test_changed_protected_input_prevents_cleanup(self):
        args = self.args()
        args.command[-1] += f'; Path({str(self.source)!r}).write_bytes(b"changed")'
        with self.assertRaisesRegex(ValueError, 'Protected input changed'):
            validation.run_unit(args, root=self.root)
        self.assertTrue((self.units()[0] / 'work/dataset/resource.json').exists())

    def test_private_file_in_scratch_is_preserved(self):
        args = self.args()
        args.command[-1] += '; Path("unexpected.sav").write_bytes(b"save")'
        with self.assertRaisesRegex(ValueError, 'Private input in scratch'):
            validation.run_unit(args, root=self.root)
        self.assertEqual((self.units()[0] / 'work/unexpected.sav').read_bytes(), b'save')

    def test_canonical_hardlinked_inputs_are_guarded_but_shared_scratch_is_refused(self):
        canonical = self.root / 'canonical.exe'
        canonical.write_bytes(b'canonical build')
        alias = self.root / 'cargo-alias.exe'
        os.link(canonical, alias)
        self.assertEqual(canonical.stat().st_nlink, 2)
        unit = validation.run_unit(self.args(protect=[str(canonical)]), root=self.root)
        self.assertFalse((unit / 'work').exists())
        self.assertEqual(alias.read_bytes(), b'canonical build')
        with self.assertRaisesRegex(ValueError, 'Nonordinary/shared'):
            validation.snapshot(canonical)
        args = self.args(protect=[str(canonical)])
        args.command[-1] += f'; import os; os.link({str(canonical)!r}, "shared.bin")'
        with self.assertRaisesRegex(ValueError, 'Nonordinary/shared'):
            validation.run_unit(args, root=self.root)

    def test_explicit_keep_reason_and_extra_evidence(self):
        args = self.args(keep_work='current comparison dataset', evidence=['dataset/resource.json'])
        unit = validation.run_unit(args, root=self.root)
        self.assertTrue((unit / 'work/dataset/resource.json').exists())
        self.assertTrue((unit / 'evidence/dataset/resource.json').exists())
        self.assertEqual(self.receipt()['cleanup'], 'kept')
        self.assertEqual(self.receipt()['keep_reason'], 'current comparison dataset')
        self.assertEqual(self.receipt()['retention_mode'], 'all')

    def test_internal_cargo_hardlinks_do_not_turn_success_into_retained_failure(self):
        args = self.args()
        args.command[-1] += '; import os; Path("build").mkdir(); Path("build/script.exe").write_bytes(b"compiler output"); os.link("build/script.exe", "build/script-alias.exe")'
        unit = validation.run_unit(args, root=self.root)
        self.assertFalse((unit / 'work').exists())
        inventory = json.loads((unit / 'work-inventory.json').read_text())
        self.assertEqual(inventory['build/script.exe']['links'], 2)
        expected = sum(row.get('size', 0) for row in inventory.values()) - len(b'compiler output')
        self.assertEqual(self.receipt()['work_bytes_before'], expected)
        self.assertEqual(self.receipt()['work_bytes_retained'], 0)

    def test_selected_retention_preserves_paths_and_removes_unselected_siblings(self):
        args = self.args(keep_path=['dataset', 'states/checkpoint.ffstate', 'build/test.exe'],
                         keep_reason='current dataset, private checkpoint and pinned test; next comparison')
        args.command[-1] += '; import os; Path("states").mkdir(); Path("states/checkpoint.ffstate").write_bytes(b"checkpoint"); Path("states/cache.bin").write_bytes(b"scratch"); Path("empty").mkdir(); Path("build").mkdir(); Path("build/test.exe").write_bytes(b"test binary"); os.link("build/test.exe", "build/build-script.exe"); os.link("result.json", "alias-result.json")'
        unit = validation.run_unit(args, root=self.root)
        work = unit / 'work'
        self.assertEqual((work / 'states/checkpoint.ffstate').read_bytes(), b'checkpoint')
        self.assertEqual((work / 'dataset/resource.json').read_text(), 'generated payload')
        self.assertEqual((work / 'build/test.exe').stat().st_nlink, 1)
        for rel in ['result.json', 'alias-result.json', 'states/cache.bin', 'empty', 'build/build-script.exe']:
            self.assertFalse((work / rel).exists(), rel)
        self.assertTrue((unit / 'evidence/alias-result.json').is_file())
        self.assertEqual(self.receipt()['cleanup'], 'kept')
        self.assertEqual(self.receipt()['retention_mode'], 'selected')
        self.assertEqual(self.receipt()['kept_paths'], args.keep_path)
        self.assertEqual(self.receipt()['work_bytes_retained'],
                         sum(p.stat().st_size for p in work.rglob('*') if p.is_file()))

    def test_selective_retention_still_refuses_unselected_private_files(self):
        args = self.args(keep_path=['dataset'], keep_reason='current dataset')
        args.command[-1] += '; Path("unexpected.sav").write_bytes(b"save")'
        with self.assertRaisesRegex(ValueError, 'Private input in scratch'):
            validation.run_unit(args, root=self.root)
        self.assertEqual((self.units()[0] / 'work/unexpected.sav').read_bytes(), b'save')
        self.assertTrue((self.units()[0] / 'work/result.json').exists())

    def test_missing_retained_path_preserves_whole_unit(self):
        with self.assertRaisesRegex(ValueError, 'Selected retention path is missing'):
            validation.run_unit(self.args(keep_path=['missing.exe'], keep_reason='selected binary'),
                                root=self.root)
        self.assertTrue((self.units()[0] / 'work/dataset/resource.json').exists())

    def test_retention_options_require_unambiguous_reason(self):
        for changes in [{'keep_path': ['dataset']}, {'keep_reason': 'unused'}, {'keep_reason': '  '},
                        {'keep_path': ['dataset'], 'keep_reason': '  '}, {'keep_work': '  '},
                        {'keep_work': 'all', 'keep_path': ['dataset'], 'keep_reason': 'some'}]:
            with self.subTest(changes=changes), self.assertRaises(ValueError):
                validation.run_unit(self.args(**changes), root=self.root)
        self.assertFalse((self.root / 'target').exists())

    def test_new_external_hardlink_before_cleanup_prevents_removal(self):
        def add_external_link(work, root):
            os.link(work / 'dataset/resource.json', root / 'external-link.json')
        self.git.side_effect = add_external_link
        with self.assertRaisesRegex(ValueError, 'external hard link'):
            validation.run_unit(self.args(keep_path=['dataset'], keep_reason='current dataset'),
                                root=self.root)
        self.assertTrue((self.units()[0] / 'work/result.json').exists())
        self.assertEqual((self.root / 'external-link.json').read_text(), 'generated payload')

    def test_low_space_refuses_before_input_hashing_unit_creation_and_launch(self):
        self.space.return_value.free = 6 * validation.GIB - 1
        with mock.patch.object(validation, 'snapshot') as snapshot, \
                mock.patch.object(validation.subprocess, 'run') as launch:
            with self.assertRaisesRegex(ValueError, 'Insufficient free space'):
                validation.run_unit(self.args(), root=self.root)
            snapshot.assert_not_called()
            launch.assert_not_called()
        self.assertFalse((self.root / 'target').exists())

    def test_space_is_rechecked_immediately_before_launch(self):
        self.space.side_effect = [argparse.Namespace(free=20 * validation.GIB),
                                  argparse.Namespace(free=0)]
        with mock.patch.object(validation.subprocess, 'run') as launch:
            with self.assertRaisesRegex(ValueError, 'Insufficient free space'):
                validation.run_unit(self.args(), root=self.root)
            launch.assert_not_called()
        self.assertEqual(self.receipt()['status'], 'FAIL')
        self.assertFalse((self.units()[0] / NAME).exists())

    def test_space_budgets_and_recorded_small_check_exception(self):
        for changes in [{'min_free_gib': -1}, {'reserve_gib': float('nan')},
                        {'reserve_gib': float('inf')}, {'reserve_gib': 1e308}, {'reserve_gib': 0.1},
                        {'min_free_gib': 0, 'space_exception': '  '}]:
            with self.subTest(changes=changes), self.assertRaises(ValueError):
                validation.run_unit(self.args(**changes), root=self.root)
        self.space.return_value.free = validation.GIB
        unit = validation.run_unit(self.args(min_free_gib=0.5, reserve_gib=0.1,
                                            space_exception='synthetic files below 1 MiB; no build'),
                                   root=self.root)
        self.assertFalse((unit / 'work').exists())
        self.assertEqual(self.receipt()['space_check']['exception_reason'],
                         'synthetic files below 1 MiB; no build')
        self.assertEqual(self.receipt()['space_check']['required_bytes'],
                         validation.math.ceil(0.6 * validation.GIB))

    def test_tracked_scratch_and_nested_leases_are_refused(self):
        self.git.side_effect = ValueError('tracked content refused')
        with self.assertRaisesRegex(ValueError, 'tracked content'):
            validation.run_unit(self.args(), root=self.root)
        self.assertTrue((self.units()[0] / 'work').exists())
        self.git.side_effect = None
        args = self.args()
        args.command[-1] += f'; Path({NAME!r}).mkdir()'
        with self.assertRaisesRegex(ValueError, 'Activity marker'):
            validation.run_unit(args, root=self.root)

    def test_timeout_retains_work_and_protective_lease(self):
        with self.assertRaises(subprocess.TimeoutExpired):
            validation.run_unit(self.args('import time; time.sleep(5)', timeout=0.3), root=self.root)
        self.assertTrue((self.units()[0] / 'work').is_dir())
        self.assertTrue((self.units()[0] / NAME).is_dir())
        self.assertTrue(self.receipt()['leases_retained'])

    def test_prefix_is_reused_but_cannot_be_adopted_or_used_concurrently(self):
        base = self.root / 'prefix-pool'
        base.mkdir()
        prefix = validation.managed_prefix(base)
        (prefix / 'marker.txt').write_text('same environment')
        self.assertEqual(validation.managed_prefix(base), prefix)
        with Lease(prefix, 'synthetic-holder'):
            with self.assertRaisesRegex(ValueError, 'Active or interrupted'):
                with Lease(validation.managed_prefix(base), 'second'):
                    self.fail('Must not share a live prefix')
        self.assertEqual((prefix / 'marker.txt').read_text(), 'same environment')
        (prefix / validation.PREFIX_MARKER).unlink()
        with self.assertRaises((OSError, ValueError)):
            validation.managed_prefix(base)

    def test_unsafe_paths_and_environment_override_are_refused(self):
        for p in ['../result.json', '/result.json', 'C:/result.json', 'dir/../../result.json', '']:
            with self.subTest(path=p), self.assertRaises(ValueError):
                validation.relative_path(p)
        with self.assertRaisesRegex(ValueError, 'WINEPREFIX is managed'):
            validation.run_unit(self.args(env=['WINEPREFIX=unowned']), root=self.root)

    @unittest.skipUnless(os.name == 'posix', 'Native Linux link check pending; Wine integration covers links')
    def test_linked_scratch_never_follows_external_destination(self):
        args = self.args()
        args.command[-1] += f'; Path("link").symlink_to({str(self.source)!r})'
        with self.assertRaises(ValueError):
            validation.run_unit(args, root=self.root)
        self.assertEqual(self.source.read_bytes(), b'private original')


if __name__ == '__main__':
    unittest.main()
