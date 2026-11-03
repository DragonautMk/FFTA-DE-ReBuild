import contextlib
import importlib.util
import io
import os
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch


spec = importlib.util.spec_from_file_location(
    'extract_rom_data', Path(__file__).parents[1] / 'extract_rom_data.py')
extract = importlib.util.module_from_spec(spec)
spec.loader.exec_module(extract)


class ExtractRomDataTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix='ffta extraction ')
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name).resolve()
        self.rom = self.root / 'European game.gba'
        self.rom.write_bytes(b'argument routing fixture, not a ROM')
        self.executable = self.root / 'recomp candidate.exe'
        self.executable.touch()
        self.stdout = io.StringIO()
        self.stderr = io.StringIO()
        stack = contextlib.ExitStack()
        self.addCleanup(stack.close)
        stack.enter_context(contextlib.redirect_stdout(self.stdout))
        stack.enter_context(contextlib.redirect_stderr(self.stderr))
        stack.enter_context(patch.object(extract, 'ROOT', self.root))
        previous = Path.cwd()
        self.addCleanup(os.chdir, previous)

    def publish(self, command, **kwargs):
        output = Path(command[-1])
        output.mkdir(parents=True)
        (output / 'manifest.toml').write_text('')
        (output / 'bg-main-graphics.json').write_text('{}')
        return subprocess.CompletedProcess(command, 0)

    def test_default_cargo_build_and_summary(self):
        with patch.object(extract.shutil, 'which', return_value='/tools/cargo'), \
                patch.object(extract.subprocess, 'run', side_effect=self.publish) as run:
            self.assertEqual(extract.main([str(self.rom)]), 0)
        command = run.call_args.args[0]
        self.assertEqual(command[:10], ['/tools/cargo', 'run', '--locked', '--release',
                                       '--package', 'recomp', '--bin', 'recomp', '--', 'rom-data'])
        self.assertEqual(command[-3:], ['extract', str(self.rom),
                         str(self.root / 'target/rom-data/European game')])
        self.assertEqual(run.call_args.kwargs['cwd'], self.root)
        self.assertIn('2 readable files', self.stdout.getvalue())
        self.assertIn('bg-main-graphics.json', self.stdout.getvalue())

    def test_existing_binary_wine_and_relative_paths_from_other_directory(self):
        # Path.resolve must use the caller's cwd, not the repository cwd.
        relative_rom = Path('European game.gba')
        os.chdir(self.root)
        with patch.object(extract.shutil, 'which', return_value='/tools/wine'), \
                patch.object(extract.subprocess, 'run', side_effect=self.publish) as run:
            self.assertEqual(extract.main([str(relative_rom), '--recomp',
                             self.executable.name, '--wine', '--output', 'decoded files']), 0)
        self.assertEqual(run.call_args.args[0], ['/tools/wine', str(self.executable),
                         'rom-data', 'extract', str(self.rom), str(self.root / 'decoded files')])

    def test_failure_status_is_preserved_without_success_summary(self):
        with patch.object(extract.subprocess, 'run', return_value=
                          subprocess.CompletedProcess([], 7)):
            self.assertEqual(extract.main([str(self.rom), '--recomp', str(self.executable)]), 7)
        self.assertNotIn('Dataset ready', self.stdout.getvalue())
        self.assertFalse((self.root / 'target').exists())

    def test_missing_inputs_and_tools_do_not_launch_a_process(self):
        cases = [[str(self.root / 'missing.gba')], [str(self.rom), '--wine'],
                 [str(self.rom), '--recomp', str(self.root / 'missing.exe')],
                 [str(self.rom)],
                 [str(self.rom), '--recomp', str(self.executable), '--wine']]
        with patch.object(extract.shutil, 'which', return_value=None), \
                patch.object(extract.subprocess, 'run') as run:
            for args in cases:
                with self.subTest(args=args), self.assertRaises(SystemExit):
                    extract.main(args)
            run.assert_not_called()

    def test_success_without_output_is_an_error(self):
        with patch.object(extract.subprocess, 'run', return_value=
                          subprocess.CompletedProcess([], 0)):
            self.assertEqual(extract.main([str(self.rom), '--recomp', str(self.executable)]), 1)
        self.assertNotIn('Dataset ready', self.stdout.getvalue())


if __name__ == '__main__':
    unittest.main()
