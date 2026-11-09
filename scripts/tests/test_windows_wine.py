"""Exercise container Wine setup with real synthetic foreground commands."""
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]


@unittest.skipUnless(os.name == 'posix', 'Linux container helper')
class WineContainerTests(unittest.TestCase):
    def setUp(self):
        temp = tempfile.TemporaryDirectory(prefix='ffta wine gate ')
        self.addCleanup(temp.cleanup)
        self.root = Path(temp.name)
        self.driver = self.root / 'test-command.py'
        self.driver.write_text('''
import json, os, sys
from pathlib import Path
assert os.environ['LANG'] == os.environ['LC_ALL'] == 'C.UTF-8'
value = os.environ['FFTA_DEBUG_CONTENT_LINK_FIXTURE']
assert value.startswith('Z:')
inputs = Path(value[2:].replace('\\\\', '/'))
assert (inputs / 'linked.py').is_symlink()
assert (inputs / 'linked-dir').is_symlink()
source = inputs.parent / 'external/outside.py'
assert os.path.samefile(source, inputs / 'shared.py')
assert source.stat().st_nlink == 2
assert inputs.parent.parent == Path(os.environ['TMPDIR'])
Path(os.environ['OBSERVATION']).write_text(json.dumps({'fixture': str(inputs.parent), 'args': sys.argv[1:]}))
sys.exit(int(os.environ.get('TEST_EXIT', '0')))
''')
        self.server = self.root / 'fake-wineserver'
        self.server.write_text(f'#!{sys.executable}\n' + '''
import json, os, sys
from pathlib import Path
assert sys.argv[1:] == ['-w']
observation = Path(os.environ['OBSERVATION'])
fixture = Path(json.loads(observation.read_text())['fixture'])
assert (fixture / 'inputs/linked.py').is_symlink()
observation.with_suffix('.drained').write_text('server still had the fixture')
sys.exit(int(os.environ.get('SERVER_EXIT', '0')))
''')
        self.server.chmod(0o755)
        self.observation = self.root / 'observation.json'
        self.env = dict(os.environ, TMPDIR=str(self.root),
                        WINEPREFIX=str(self.root / 'owned-container-prefix'),
                        WINESERVER=str(self.server), OBSERVATION=str(self.observation),
                        LANG='C', LC_ALL='C', FFTA_DEBUG_CONTENT_LINK_FIXTURE='unrelated-inherited-value')

    def run_gate(self, **env):
        return subprocess.run([sys.executable, str(ROOT / 'scripts/windows/test_wine.py'),
                               sys.executable, str(self.driver), 'two words', '--test-threads=1'],
                              env=dict(self.env, **env), capture_output=True, text=True, timeout=10)

    def test_success_uses_real_links_utf8_arguments_and_drains_before_retirement(self):
        result = self.run_gate()
        self.assertEqual(result.returncode, 0, result.stderr)
        value = json.loads(self.observation.read_text())
        self.assertEqual(value['args'], ['two words', '--test-threads=1'])
        self.assertTrue(self.observation.with_suffix('.drained').exists())
        self.assertFalse(Path(value['fixture']).exists())

    def test_failed_tests_keep_exit_code_fixture_and_complete_server_drain(self):
        result = self.run_gate(TEST_EXIT='101')
        self.assertEqual(result.returncode, 101, result.stderr)
        self.assertIn('exit 101', result.stderr)
        self.assertTrue(Path(json.loads(self.observation.read_text())['fixture']).is_dir())
        self.assertTrue(self.observation.with_suffix('.drained').exists())

    def test_failed_drain_keeps_fixture_and_fails_the_gate(self):
        result = self.run_gate(SERVER_EXIT='8')
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('execution failed', result.stderr)
        self.assertTrue(Path(json.loads(self.observation.read_text())['fixture']).is_dir())

    def test_missing_owned_prefix_refuses_before_command(self):
        result = self.run_gate(WINEPREFIX='')
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('owned WINEPREFIX', result.stderr)
        self.assertFalse(self.observation.exists())


if __name__ == '__main__':
    unittest.main()
