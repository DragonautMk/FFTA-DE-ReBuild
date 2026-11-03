"""Docker launcher checks; no ROM or running Docker daemon is needed."""
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import tarfile
import unittest


ROOT = Path(__file__).resolve().parents[2]


@unittest.skipUnless(os.name == 'posix' and shutil.which('bash'), 'requires Bash')
class DockerCoverageTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix='ffta coverage docker ')
        self.addCleanup(self.temp.cleanup)
        self.directory = Path(self.temp.name).resolve()
        self.repo = self.directory / 'project checkout'
        self.script = self.repo / 'scripts/linux/rom_coverage_docker.sh'
        self.script.parent.mkdir(parents=True)
        shutil.copy2(ROOT / 'scripts/linux/rom_coverage_docker.sh', self.script)
        self.defaults = {
            'rom': 'rom/1194 - Final Fantasy Tactics Advance (E)(Surplus).gba',
            'dataset': 'target/rom-data-rank-gates/windows-managed-europe/offline dataset',
            'translation': 'out/1194 - Final Fantasy Tactics Advance (E)(Surplus).dll',
            'evidence': 'docs/development/rom-coverage/evidence.json',
        }
        for key, name in self.defaults.items():
            path = self.repo / name
            if key == 'dataset':
                path = path / 'manifest.toml'
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text('argument routing fixture, not game data')
        self.bin = self.directory / 'bin'
        self.bin.mkdir()
        # The child PATH deliberately contains no Python, Cargo or nm.
        for name in ('dirname', 'basename', 'mkdir', 'id', 'mktemp', 'date', 'tee', 'git', 'cp', 'rm'):
            (self.bin / name).symlink_to(shutil.which(name))
        docker = self.bin / 'docker'
        docker.write_text('''#!/bin/bash
printf '%s\\0' "$@" > "$DOCKER_TEST_LOG/$1.args"
if [[ "$1" == buildx ]]; then
    cp "${!#}/runtime.tar" "$DOCKER_TEST_LOG/runtime.tar"
    cp "${!#}/scripts/rom_coverage.py" "$DOCKER_TEST_LOG/current-tool.py"
    exit "${DOCKER_TEST_BUILD_STATUS:-0}"
fi
if [[ "${DOCKER_TEST_NO_OUTPUT:-0}" == 0 && "${DOCKER_TEST_RUN_STATUS:-0}" == 0 ]]; then
    for arg in "$@"; do
        if [[ "$arg" == type=bind,source=*,target=/output ]]; then
            path=${arg#type=bind,source=}; path=${path%,target=/output}
            printf 'report fixture' > "$path/index.html"
            printf '{"generation":"fixture"}' > "$path/coverage.json"
        fi
    done
fi
exit "${DOCKER_TEST_RUN_STATUS:-0}"
''')
        docker.chmod(0o755)
        self.environment = dict(os.environ, PATH=str(self.bin), DOCKER_TEST_LOG=str(self.directory))
        for name in ('Cargo.toml','Cargo.lock','crates/model.rs','assets/public','TinyCC/public',
                     'Licenses/public','gamedb.sqlite','docs/changes/proof.json',
                     'scripts/rom_coverage.py','scripts/rom_coverage.html','scripts/local_artifacts.py',
                     'scripts/rom_coverage_audit.py','scripts/rom_coverage_ranges.py',
                     'scripts/report_home.py','scripts/report_home.html',
                     'scripts/debug_campaign.py','scripts/debug_campaign.html',
                     'scripts/debug_artifacts.py','scripts/debug_artifacts.html',
                     'scripts/artifact_coordinator.py',
                     'docs/development/rom-coverage/code-seeds.json'):
            p = self.repo/name; p.parent.mkdir(parents=True,exist_ok=True); p.write_text('committed')
        for args in (['init','-q'],['add','Cargo.toml','Cargo.lock','crates','assets','TinyCC','Licenses','gamedb.sqlite','scripts'],
                     ['-c','user.name=Coverage test','-c','user.email=test@example.invalid','commit','-qm','fixture']):
            subprocess.run(['git','-C',str(self.repo),*args],check=True,capture_output=True)
        (self.repo/'crates/model.rs').write_text('unfinished extraction')
        (self.repo/'scripts/report_home.py').write_text('unfinished embedded tool')
        (self.repo/'scripts/rom_coverage.py').write_text('current tool')

    def run_script(self, *args, **environment):
        return subprocess.run([shutil.which('bash'), str(self.script), *map(str, args)],
                              cwd=self.directory, env=dict(self.environment, **environment),
                              text=True, capture_output=True)

    def call(self, name):
        path = self.directory / (name + '.args')
        return path.read_bytes().decode().rstrip('\0').split('\0') if path.exists() else None

    def test_defaults_from_unrelated_cwd_without_host_toolchain(self):
        result = self.run_script()
        self.assertEqual(result.returncode, 0, result.stderr)
        build, run = self.call('buildx'), self.call('run')
        self.assertEqual(build[build.index('--file') + 1], str(self.repo / 'Dockerfile.coverage'))
        self.assertTrue(build[-1].startswith(str(self.repo/'target/rom-coverage-context-')))
        self.assertFalse(Path(build[-1]).exists())
        with tarfile.open(self.directory/'runtime.tar') as archive:
            self.assertEqual(archive.extractfile('crates/model.rs').read(), b'committed')
            for name in ('report_home.py', 'report_home.html', 'debug_campaign.py',
                         'debug_campaign.html', 'debug_artifacts.py', 'debug_artifacts.html',
                         'artifact_coordinator.py', 'local_artifacts.py'):
                self.assertEqual(archive.extractfile('scripts/' + name).read(), b'committed')
            self.assertEqual(archive.extractfile('scripts/rom_coverage.py').read(), b'committed')
            self.assertFalse(any(n.startswith(('rom/','out/','target/')) for n in archive.getnames()))
        self.assertEqual((self.directory/'current-tool.py').read_text(),'current tool')
        for key, target in [('rom', '/input/source.gba'), ('evidence', '/input/evidence.json')]:
            self.assertIn(f'type=bind,source={self.repo / self.defaults[key]},target={target},readonly', run)
        self.assertIn(f'type=bind,source={self.repo / "target/rom-data/coverage"},target=/cache', run)
        self.assertIn('--dataset-cache', run)
        self.assertEqual(run[run.index('--cache-keep')+1], '2')
        self.assertNotIn('--translation', run)
        self.assertNotIn('--dataset', run)
        output = self.repo / 'docs/development/rom-coverage'
        self.assertIn(f'type=bind,source={output},target=/output', run)
        self.assertEqual(run[run.index('--network') + 1], 'none')
        self.assertEqual(run[run.index('--user') + 1], f'{os.getuid()}:{os.getgid()}')
        self.assertNotIn('/var/run/docker.sock', ' '.join(run))
        self.assertIn(str(output / 'index.html'), result.stdout)

    def test_relative_overrides_and_absolute_output_preserve_spaces(self):
        args = []
        for key, old in self.defaults.items():
            path = self.repo / 'custom inputs' / Path(old).name
            source = self.repo / old
            if key == 'dataset':
                shutil.copytree(source, path)
            else:
                path.parent.mkdir(parents=True, exist_ok=True)
                shutil.copy2(source, path)
            args += ['--' + key, str(path.relative_to(self.repo))]
        output = self.directory / 'coverage output'
        result = self.run_script(*args, '--output', output)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn(f'type=bind,source={output},target=/output', self.call('run'))
        self.assertIn(str(self.repo / 'custom inputs'), ' '.join(self.call('run')))

    def test_repeatable_audit_and_comparison_mounts_are_readonly(self):
        files = [self.repo / name for name in ('private/audit one.json', 'private/audit two.json', 'old report.json')]
        for p in files:
            p.parent.mkdir(exist_ok=True); p.write_text('{}')
        result = self.run_script('--audit', 'private/audit one.json', '--audit', files[1], '--compare', files[2])
        self.assertEqual(result.returncode, 0, result.stderr)
        run = self.call('run')
        for p, target in zip(files, ('/input/audit-0.json', '/input/audit-1.json', '/input/previous.json')):
            self.assertIn(f'type=bind,source={p},target={target},readonly', run)
        self.assertEqual(run.count('--audit'), 2)
        self.assertEqual(run[run.index('--compare')+1], '/input/previous.json')
        with tarfile.open(self.directory/'runtime.tar') as archive:
            self.assertFalse(any('private' in n for n in archive.getnames()))

    def test_missing_audit_or_comparison_fails_before_build(self):
        for flag in ('--audit', '--compare'):
            result = self.run_script(flag, 'absent.json')
            self.assertEqual(result.returncode, 2)
            self.assertIsNone(self.call('buildx'))

    def test_missing_dataset_or_input_stops_before_docker(self):
        for key, name in self.defaults.items():
            path = self.repo / name
            if key == 'dataset':
                path /= 'manifest.toml'
            data = path.read_bytes()
            path.unlink()
            with self.subTest(key=key):
                args = ['--' + key, self.defaults[key]] if key in ('dataset', 'translation') else []
                result = self.run_script(*args)
                self.assertEqual(result.returncode, 2)
                self.assertIn(str(path), result.stdout + result.stderr)
                self.assertIsNone(self.call('buildx'))
            path.write_bytes(data)

    def test_failures_are_propagated_without_success_message(self):
        result = self.run_script(DOCKER_TEST_BUILD_STATUS='17')
        self.assertEqual(result.returncode, 17)
        self.assertIsNone(self.call('run'))
        self.assertNotIn('ROM coverage on host:', result.stdout)
        result = self.run_script(DOCKER_TEST_RUN_STATUS='23')
        self.assertEqual(result.returncode, 23)
        self.assertIsNotNone(self.call('run'))
        self.assertNotIn('ROM coverage on host:', result.stdout)
        logs = list((self.repo / 'target/build-logs').glob('rom-coverage-*.log'))
        self.assertTrue(any('exit 23' in p.read_text() for p in logs))

    def test_success_without_report_is_an_error_with_persistent_log(self):
        result = self.run_script(DOCKER_TEST_NO_OUTPUT='1')
        self.assertEqual(result.returncode, 1)
        self.assertIn('No report produced:', result.stdout)
        self.assertNotIn('ROM coverage on host:', result.stdout)
        logs = list((self.repo / 'target/build-logs').glob('rom-coverage-*.log'))
        self.assertTrue(any('No report produced:' in p.read_text() for p in logs))

    def test_bad_arguments_and_csv_paths_stop_before_docker(self):
        for args in [('--unknown',), ('--output',), ('--rom', ''),
                     *[('--output', 'bad' + char + 'path') for char in (',', '"', '\n', '\r')]]:
            with self.subTest(args=args):
                self.assertEqual(self.run_script(*args).returncode, 2)
        self.assertIsNone(self.call('buildx'))


if __name__ == '__main__':
    unittest.main()
