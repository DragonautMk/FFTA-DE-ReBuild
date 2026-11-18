"""A warm export must match the tested inputs and both published files."""
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[2]


@unittest.skipUnless(os.name == 'posix' and shutil.which('bash') and shutil.which('sha256sum'),
                     'requires the Linux build tools')
class VerifiedExportTests(unittest.TestCase):
    def test_restore_invalidation_and_failed_build(self):
        with tempfile.TemporaryDirectory() as temporary:
            base = Path(temporary)
            project, artifacts, inputs = [base / name for name in ('project', 'artifacts', 'inputs')]
            (project / 'target').mkdir(parents=True)
            inputs.mkdir()
            script = inputs / 'docker_cached_export.sh'
            shutil.copy2(ROOT / 'scripts' / script.name, script)
            recipe = inputs / 'Dockerfile.linux'
            recipe.write_text('recipe')
            source = project / 'main.rs'
            source.write_text('source')
            key = 'sha256:' + '1' * 64
            archive = 'FFTARecomp-linux-portable.tar.gz'
            def run(command, identity=key):
                return subprocess.run(['bash', str(script), 'linux', str(artifacts), 'bash', '-c', command],
                                      cwd=project, env=dict(os.environ, FFTA_TOOLCHAIN_ID=identity),
                                      capture_output=True, text=True)
            produce = 'mkdir -p ../artifacts && cp main.rs ../artifacts/' + archive + ' && echo tested > ../artifacts/BUILD-INFO.txt'
            self.assertEqual(run(produce).returncode, 0)
            shutil.rmtree(artifacts)
            os.utime(source, (1, 1))
            self.assertEqual(run('exit 9').returncode, 0)
            self.assertEqual((artifacts / archive).read_text(), 'source')
            source.write_text('changed')
            self.assertEqual(run('exit 9').returncode, 9)
            source.write_text('source')
            self.assertEqual(run('exit 9').returncode, 0)
            self.assertEqual(run('exit 9', 'sha256:' + '2' * 64).returncode, 9)
            recipe.write_text('updated recipe')
            self.assertEqual(run('exit 9').returncode, 9)
            recipe.write_text('recipe')
            (project / 'target/.ffta-verified-export-linux' / archive).write_text('damaged')
            self.assertEqual(run('exit 9').returncode, 9)
            self.assertEqual(run(produce).returncode, 0)
            self.assertEqual(run('exit 9', '').returncode, 9)
