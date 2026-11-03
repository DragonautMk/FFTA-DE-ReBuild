import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[2]
SCRIPT = ROOT / 'scripts/linux/extract_rom_data_docker.sh'


@unittest.skipUnless(os.name == 'posix' and shutil.which('bash'), 'requires Bash')
class DockerExtractionTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix='ffta docker extraction ')
        self.addCleanup(self.temp.cleanup)
        self.directory = Path(self.temp.name).resolve()
        self.repo = self.directory / 'project checkout'
        self.script = self.repo / 'scripts/linux/extract_rom_data_docker.sh'
        self.script.parent.mkdir(parents=True)
        shutil.copy2(SCRIPT, self.script)
        self.default_rom = self.repo / 'rom/1194 - Final Fantasy Tactics Advance (E)(Surplus).gba'
        self.default_rom.parent.mkdir()
        self.default_rom.write_bytes(b'default argument routing fixture, not a ROM')
        self.rom = self.repo / 'rom/European game.gba'
        self.rom.write_bytes(b'argument routing fixture, not a ROM')
        self.output = self.directory / 'decoded files' / 'dataset'
        self.log = self.directory / 'docker-args.jsonl'
        binary = self.directory / 'docker'
        binary.write_text('''#!/usr/bin/env python3
import json, os, sys
with open(os.environ['DOCKER_TEST_LOG'], 'a') as log:
    log.write(json.dumps(sys.argv[1:]) + '\\n')
key = 'DOCKER_TEST_BUILD_STATUS' if sys.argv[1] == 'buildx' else 'DOCKER_TEST_RUN_STATUS'
sys.exit(int(os.environ.get(key, '0')))
''')
        binary.chmod(0o755)
        self.environment = dict(os.environ, PATH=str(self.directory) + os.pathsep + os.environ['PATH'],
                                DOCKER_TEST_LOG=str(self.log))

    def run_script(self, *args, **environment):
        return subprocess.run(['bash', str(self.script), *map(str, args)], cwd=self.directory,
                              env=dict(self.environment, **environment), text=True, capture_output=True)

    def calls(self):
        return [json.loads(row) for row in self.log.read_text().splitlines()] if self.log.exists() else []

    def test_relative_paths_spaces_and_atomic_output_mount(self):
        result = self.run_script(Path('rom/European game.gba'), Path('decoded files/dataset'))
        self.assertEqual(result.returncode, 0, result.stderr)
        build, run = self.calls()
        self.assertEqual(build[:2], ['buildx', 'build'])
        self.assertEqual(build[build.index('--file') + 1], str(self.repo / 'Dockerfile.linux'))
        self.assertEqual(build[build.index('--target') + 1], 'extract')
        self.assertIn('--load', build)
        self.assertEqual(build[-1], str(self.repo))
        self.assertIn(f'type=bind,source={self.rom},target=/input/source.gba,readonly', run)
        self.assertIn(f'type=bind,source={self.output.parent},target=/output', run)
        self.assertEqual(run[-3:], ['/input/source.gba', '--output', '/output/dataset'])
        self.assertEqual(run[run.index('--user') + 1], f'{os.getuid()}:{os.getgid()}')
        self.assertFalse(self.output.exists(), 'the host must not precreate the atomic publication target')
        self.assertIn(str(self.output), result.stdout)

    def test_default_output_uses_repository_target(self):
        result = self.run_script(self.rom)
        self.assertEqual(result.returncode, 0, result.stderr)
        run = self.calls()[1]
        self.assertIn(f'type=bind,source={self.repo / "target/rom-data"},target=/output', run)
        self.assertEqual(run[-1], '/output/European game')

    def test_no_arguments_use_default_rom_from_project_root(self):
        result = self.run_script()
        self.assertEqual(result.returncode, 0, result.stderr)
        run = self.calls()[1]
        self.assertIn(f'type=bind,source={self.default_rom},target=/input/source.gba,readonly', run)
        self.assertEqual(run[-1], '/output/' + self.default_rom.stem)

    def test_missing_default_rom_reports_resolved_path_without_docker(self):
        self.default_rom.unlink()
        result = self.run_script()
        self.assertEqual(result.returncode, 2)
        self.assertIn(str(self.default_rom), result.stderr)
        self.assertIn('first argument', result.stderr)
        self.assertEqual(self.calls(), [])

    def test_build_failure_stops_before_extraction(self):
        result = self.run_script(self.rom, self.output, DOCKER_TEST_BUILD_STATUS='17')
        self.assertEqual(result.returncode, 17)
        self.assertEqual(len(self.calls()), 1)
        self.assertNotIn('Extracted dataset on host', result.stdout)

    def test_container_failure_is_propagated(self):
        result = self.run_script(self.rom, self.output, DOCKER_TEST_RUN_STATUS='23')
        self.assertEqual(result.returncode, 23)
        self.assertEqual(len(self.calls()), 2)
        self.assertNotIn('Extracted dataset on host', result.stdout)

    def test_bad_input_and_mount_paths_do_not_invoke_docker(self):
        comma_rom = self.directory / 'game,copy.gba'
        comma_rom.touch()
        for args in [(self.directory / 'missing.gba',), (comma_rom,),
                     (self.rom, self.directory / 'comma,path'),
                     (self.rom, self.directory / 'newline\npath')]:
            with self.subTest(args=args):
                self.assertEqual(self.run_script(*args).returncode, 2)
        self.assertEqual(self.calls(), [])


if __name__ == '__main__':
    unittest.main()
