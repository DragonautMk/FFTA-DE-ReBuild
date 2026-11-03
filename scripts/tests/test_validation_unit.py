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

    def args(self, code=None, **changes):
        code = code or "from pathlib import Path; Path('result.json').write_text('{\"status\":\"PASS\"}'); Path('dataset').mkdir(); Path('dataset/manifest.toml').write_text('revision=1'); Path('dataset/resource.json').write_text('generated payload')"
        args = argparse.Namespace(name='synthetic', result=['result.json'], evidence=[],
                                  protect=[str(self.source)], env=[], wine=False, wineserver=None,
                                  timeout=30, keep_work=None, command=[sys.executable, '-c', code])
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
