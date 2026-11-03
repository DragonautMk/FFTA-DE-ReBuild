"""Native build routing with synthetic Cargo artifacts; no Rust/game execution."""
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]


@unittest.skipUnless(os.name == 'posix', 'uses a synthetic executable Cargo tool')
class NativeBuildTests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory(prefix="ffta build ' à ")
        self.addCleanup(temporary.cleanup)
        self.directory = Path(temporary.name)
        self.root = self.directory / 'checkout'
        (self.root / 'scripts/linux').mkdir(parents=True)
        (self.root / 'TinyCC').mkdir()
        for name in ['scripts/build_desktop.py', 'scripts/linux/package_ffta_linux.sh']:
            shutil.copy2(ROOT / name, self.root / name)
        tools = self.directory / 'tools'; tools.mkdir()
        cargo = tools / 'cargo'
        cargo.write_text('''#!/usr/bin/env python3
import json,os,sys
from pathlib import Path
Path('cargo-args.json').write_text(json.dumps(sys.argv[1:]))
if os.environ.get('CARGO_FAILURE'):sys.exit(int(os.environ['CARGO_FAILURE']))
windows=os.environ['TEST_PLATFORM']=='windows';suffix='.exe' if windows else ''
directory=Path(os.environ['CARGO_TARGET_DIR'])/'configured-target'/'release'
directory.mkdir(parents=True,exist_ok=True)
for name in ['recomp','ffta-play']:
 if name==os.environ.get('MISSING_ARTIFACT'):continue
 target=directory if name!='ffta-play' or not os.environ.get('SPLIT_ARTIFACTS') else directory/'other'
 target.mkdir(exist_ok=True);path=target/(name+suffix)
 header=bytearray(64)
 if windows:
  header[:2]=b'MZ';header[60:64]=(2048).to_bytes(4,'little')
  header.extend(bytes(2048-len(header))+b'PE'+bytes(2)+(0x8664).to_bytes(2,'little'))
 else:
  header[:6]=b'\\x7fELF\\x02\\x01';header[18:20]=int(os.environ.get('ELF_MACHINE','62')).to_bytes(2,'little')
 path.write_bytes(header)
 print(json.dumps(dict(reason='compiler-artifact',target=dict(name=name,kind=['bin']),executable=str(path))))
print(json.dumps(dict(reason='build-finished',success=True)))
''')
        cargo.chmod(0o755)
        (self.root / 'TinyCC/build.py').write_text('''import os,sys,json
from pathlib import Path
Path('tinycc-args.json').write_text(json.dumps(sys.argv[1:]))
if os.environ.get('TINYCC_FAILURE'):sys.exit(int(os.environ['TINYCC_FAILURE']))
p=Path(sys.argv[sys.argv.index('--output')+1]);p.parent.mkdir(parents=True,exist_ok=True);p.write_text('SYNTHETIC compiler')
''')
        (self.root / 'scripts/package_ffta_portable.py').write_text('''import json,sys
from pathlib import Path
Path('package-args.json').write_text(json.dumps(sys.argv[1:]))
p=Path(sys.argv[sys.argv.index('--runtime-dir')+1]);assert p.is_dir()
Path('package-result.txt').write_text(str(p))
''')
        (self.root / 'target/release').mkdir(parents=True)
        (self.root / 'target/release/recomp').write_text('OLD output must not be selected')
        (self.root / 'package-result.txt').write_text('PREVIOUS accepted package')
        self.runtime = self.directory / 'custom target cache'
        self.environment = dict(os.environ, PATH=str(tools)+os.pathsep+os.environ['PATH'],
                                CARGO_TARGET_DIR=str(self.runtime), TEST_PLATFORM='linux')

    def run_build(self, platform='linux', wrapper=False, **environment):
        command = (['bash', str(self.root / 'scripts/linux/package_ffta_linux.sh')] if wrapper
                   else [sys.executable, str(self.root / 'scripts/build_desktop.py'), '--platform', platform])
        return subprocess.run(command, cwd=self.directory,
                              env=dict(self.environment, TEST_PLATFORM=platform, **environment),
                              capture_output=True, text=True, timeout=15)

    def test_native_wrapper_uses_reported_custom_target_paths_from_unrelated_cwd(self):
        result = self.run_build(wrapper=True)
        self.assertEqual(result.returncode, 0, result.stderr)
        expected = self.runtime / 'configured-target/release'
        self.assertEqual((self.root/'package-result.txt').read_text(), str(expected))
        self.assertTrue((expected/'tcc/libtcc.so').is_file())
        self.assertIn('--locked', json.loads((self.root/'cargo-args.json').read_text()))

    def test_windows_artifacts_with_large_pe_header_use_same_selected_directory(self):
        result = self.run_build('windows')
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertTrue((self.runtime/'configured-target/release/tcc/libtcc.dll').is_file())
        self.assertEqual(json.loads((self.root/'package-args.json').read_text())[:2],
                         ['--platform', 'windows'])

    def test_cargo_failure_cannot_package_old_outputs(self):
        result = self.run_build(CARGO_FAILURE='17')
        self.assertEqual(result.returncode, 17, result.stderr)
        self.assertFalse((self.root/'tinycc-args.json').exists())
        self.assertEqual((self.root/'package-result.txt').read_text(), 'PREVIOUS accepted package')

    def test_missing_split_or_wrong_architecture_artifacts_refuse_before_tinycc(self):
        for environment in [dict(MISSING_ARTIFACT='ffta-play'), dict(SPLIT_ARTIFACTS='1'),
                            dict(ELF_MACHINE='183')]:
            with self.subTest(environment=environment):
                result = self.run_build(**environment)
                self.assertNotEqual(result.returncode, 0)
                self.assertFalse((self.root/'tinycc-args.json').exists())
                self.assertEqual((self.root/'package-result.txt').read_text(), 'PREVIOUS accepted package')

    def test_tinycc_failure_does_not_replace_the_previous_package(self):
        result = self.run_build(TINYCC_FAILURE='19')
        self.assertEqual(result.returncode, 19, result.stderr)
        self.assertFalse((self.root/'package-args.json').exists())
        self.assertEqual((self.root/'package-result.txt').read_text(), 'PREVIOUS accepted package')


if __name__ == '__main__':
    unittest.main()
