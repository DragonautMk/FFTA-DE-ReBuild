import json
import os
from pathlib import Path
import select
import shutil
import subprocess
import tempfile
import time
import unittest


ROOT = Path(__file__).resolve().parents[2]


@unittest.skipUnless(os.name == 'posix' and shutil.which('bash'), 'requires Bash')
class LinuxBuilderTests(unittest.TestCase):
    platform = 'linux'

    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix='ffta linux builder ')
        self.addCleanup(self.temp.cleanup)
        self.directory = Path(self.temp.name)
        self.repo = self.directory / 'project checkout'
        self.script = self.repo / f'scripts/linux/build_{self.platform}_docker.sh'
        self.script.parent.mkdir(parents=True)
        shutil.copy2(ROOT / 'scripts/linux' / self.script.name, self.script)
        shutil.copy2(ROOT / 'scripts/linux/docker_toolchain.sh', self.script.parent)
        self.recipe = self.repo / f'Dockerfile.{self.platform}'
        shutil.copy2(ROOT / self.recipe.name, self.recipe)
        self.arguments = self.directory / 'docker-arguments.json'
        docker = self.directory / 'docker'
        docker.write_text('''#!/usr/bin/env python3
import json, os, sys
from pathlib import Path
args = sys.argv[1:]
state = Path(os.environ['DOCKER_TEST_ARGUMENTS']).parent
if args == ['buildx', 'inspect']:
    print('Driver: docker')
    sys.exit(0)
if args[:2] == ['image', 'inspect']:
    if '--format' in args:
        print('sha256:' + '1' * 64)
    sys.exit(0 if (state / args[-1].replace(':', '_')).exists() else 1)
if '--target' in args and args[args.index('--target') + 1] == 'toolchain':
    (state / args[args.index('--tag') + 1].replace(':', '_')).touch()
    with (state / 'preparations').open('a') as log:
        log.write('prepared\\n')
    sys.exit(0)
Path(os.environ['DOCKER_TEST_ARGUMENTS']).write_text(json.dumps(sys.argv[1:]))
print('builder stdout', flush=True)
print('builder stderr', file=sys.stderr, flush=True)
sys.exit(int(os.environ.get('DOCKER_TEST_STATUS', '0')))
''')
        docker.chmod(0o755)
        self.environment = dict(os.environ,
                                PATH=str(self.directory) + os.pathsep + os.environ['PATH'],
                                DOCKER_TEST_ARGUMENTS=str(self.arguments))

    def log(self):
        logs = list((self.repo / 'target/build-logs').glob(f'{self.platform}-*.log'))
        self.assertEqual(len(logs), 1)
        return logs[0].read_text()

    def test_success_preserves_output_path_from_unrelated_working_directory(self):
        output = 'export with spaces'
        result = subprocess.run(['bash', str(self.script), output], cwd=self.directory,
                                env=self.environment, capture_output=True, text=True, timeout=10)
        self.assertEqual(result.returncode, 0, result.stderr)
        arguments = json.loads(self.arguments.read_text())
        self.assertEqual(arguments[arguments.index('--file') + 1], str(self.repo / f'Dockerfile.{self.platform}'))
        self.assertEqual(arguments[arguments.index('--output') + 1], f'type=local,dest={output}')
        self.assertIn('FFTA_TOOLCHAIN_ID=sha256:' + '1' * 64, arguments)
        self.assertEqual(arguments[-1], str(self.repo))
        self.assertIn('Portable archive and BUILD-INFO.txt exported', self.log())
        self.assertIn('builder stdout', self.log())
        self.assertIn('builder stderr', self.log())

    def test_reuse_tracks_only_toolchain_inputs_and_bounds_success_logs(self):
        def build():
            subprocess.run(['bash', str(self.script)], cwd=self.directory, env=self.environment,
                           capture_output=True, check=True)
            args = json.loads(self.arguments.read_text())
            return args[args.index('--build-arg') + 1]
        first = build()
        self.recipe.write_text(self.recipe.read_text() + '\n# Package-only change\n')
        self.assertEqual(build(), first)
        self.recipe.write_text(self.recipe.read_text().replace('# End of cached toolchain',
                               'ENV EXTRA_TOOL=2\n# End of cached toolchain'))
        self.assertNotEqual(build(), first)
        self.assertEqual((self.directory / 'preparations').read_text().splitlines(), ['prepared'] * 2)
        self.assertEqual([p.name for p in (self.repo / 'target/build-logs').iterdir()],
                         [f'{self.platform}-latest.log'])

    def test_failure_keeps_diagnostics_and_exit_code_without_blocking_automation(self):
        result = subprocess.run(['bash', str(self.script)], cwd=self.directory,
                                env=dict(self.environment, DOCKER_TEST_STATUS='17'),
                                capture_output=True, text=True, timeout=10)
        self.assertEqual(result.returncode, 17)
        self.assertIn('exit 17', result.stderr)
        self.assertIn('builder stdout', self.log())
        self.assertIn('builder stderr', self.log())
        self.assertNotIn('Portable archive', result.stdout)
        self.assertNotIn('Press Enter', result.stdout + result.stderr)

    def test_csv_unsafe_output_is_rejected_before_docker(self):
        for character in [',', '"', '\n', '\r']:
            with self.subTest(character=character):
                result = subprocess.run(['bash', str(self.script), 'bad'+character+'path'],
                                        cwd=self.directory, env=self.environment,
                                        capture_output=True, text=True, timeout=10)
                self.assertEqual(result.returncode, 2)
                self.assertFalse(self.arguments.exists())

    def test_terminal_failure_stays_open_until_enter(self):
        import pty
        master, slave = pty.openpty()
        process = None
        try:
            process = subprocess.Popen(['bash', str(self.script)], cwd=self.directory,
                                       env=dict(self.environment, DOCKER_TEST_STATUS='17'),
                                       stdin=slave, stdout=slave, stderr=slave)
            os.close(slave)
            slave = None
            output = b''
            deadline = time.monotonic() + 10
            while b'Press Enter' not in output:
                remaining = deadline - time.monotonic()
                self.assertGreater(remaining, 0, output.decode(errors='replace'))
                ready, _, _ = select.select([master], [], [], remaining)
                self.assertTrue(ready, output.decode(errors='replace'))
                output += os.read(master, 4096)
            self.assertIsNone(process.poll(), 'the terminal must remain open after failure')
            os.write(master, b'\n')
            self.assertEqual(process.wait(timeout=5), 17)
            self.assertIn('builder stderr', self.log())
        finally:
            if process is not None and process.poll() is None:
                process.kill()
                process.wait()
            os.close(master)
            if slave is not None:
                os.close(slave)


class WindowsBuilderTests(LinuxBuilderTests):
    platform = 'windows'


if __name__ == '__main__':
    unittest.main()
