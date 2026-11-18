"""Exercise the PowerShell entry point with a fixture Docker CLI on Linux."""
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[2]


@unittest.skipUnless(os.name == "posix" and shutil.which("pwsh"), "requires Linux PowerShell")
class PowerShellDockerBuildTests(unittest.TestCase):
    def setUp(self):
        temp = tempfile.TemporaryDirectory()
        self.addCleanup(temp.cleanup)
        self.work = Path(temp.name)
        self.repo = self.work / "checkout with spaces"
        self.launcher = self.repo / "scripts/windows/build_docker.ps1"
        self.launcher.parent.mkdir(parents=True)
        for name in ("build_docker.ps1", "docker_toolchain.ps1"):
            shutil.copyfile(ROOT / "scripts/windows" / name, self.launcher.parent / name)
        self.calls = self.work / "calls.jsonl"
        stub = self.work / "docker"
        stub.write_text("""#!/usr/bin/env python3
from pathlib import Path
import json, os, sys
args = sys.argv[1:]
if args == ['buildx', 'inspect']:
    print('Driver: docker-container')
elif args[:2] == ['buildx', 'build']:
    target = Path(args[args.index('--file') + 1]).suffix[1:]
    dest = args[args.index('--output') + 1].split('type=local,dest=', 1)[1]
    with open(os.environ['CALLS'], 'a') as out:
        print(json.dumps([target, dest]), file=out)
    if target == os.environ.get('FAIL_TARGET'):
        sys.exit(19)
else:
    sys.exit(20)
""")
        stub.chmod(0o755)
        driver = self.repo / "scripts/android/build_android_docker.py"
        driver.parent.mkdir(parents=True)
        driver.write_text("""import json, os, sys
assert len(sys.argv) == 3 and sys.argv[1] == '--output'
with open(os.environ['CALLS'], 'a') as out:
    print(json.dumps(['android', sys.argv[2]]), file=out)
sys.exit(23 if os.environ.get('FAIL_TARGET') == 'android' else 0)
""")

    def build(self, *arguments, fail=""):
        self.calls.write_text("")
        env = dict(os.environ, PATH=str(self.work) + os.pathsep + os.environ["PATH"],
                   CALLS=str(self.calls), FAIL_TARGET=fail)
        result = subprocess.run(["pwsh", "-NoProfile", "-NonInteractive", "-File",
                                 str(self.launcher), *arguments], cwd=self.work, env=env,
                                stdin=subprocess.DEVNULL, capture_output=True, text=True, timeout=30)
        calls = [json.loads(line) for line in self.calls.read_text().splitlines()]
        return result, calls

    def test_defaults_and_single_android_destination(self):
        for args, platform in [((), "windows"), (("-Platform", "ANDROID"), "android")]:
            result, calls = self.build(*args)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(calls, [[platform, str(self.repo / "dist" / f"docker-{platform}")]])
        result, calls = self.build("-Platform", "android", "-OutputDir", "my apk")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(calls, [["android", str(self.work / "my apk")]])

    def test_all_separates_outputs_and_stops_on_first_failure(self):
        args = ("-Platform", "all", "-OutputDir", "combined builds")
        targets = ["windows", "linux", "android"]
        expected = [[t, str(self.work / "combined builds" / f"docker-{t}")] for t in targets]
        result, calls = self.build(*args)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(calls, expected)
        for target, code in [("linux", 19), ("android", 23)]:
            result, calls = self.build(*args, fail=target)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn(f"exit {code}", result.stderr)
            self.assertEqual(calls, expected[:targets.index(target) + 1])

    def test_unsafe_output_is_rejected_before_builds(self):
        result, calls = self.build("-Platform", "all", "-OutputDir", "bad,path")
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("output directory", result.stderr)
        self.assertEqual(calls, [])
